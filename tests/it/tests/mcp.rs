// SPDX-License-Identifier: AGPL-3.0-or-later
//! The MCP contract (plan sections 09 and 16): an rmcp client against the real `emmet mcp`, one test per tool. Each
//! checks the tool's advertised schemas against docs/spec/mcp/, that its structured output validates against its
//! output schema, its text summary, and an error shape.

use std::path::Path;

use emmet_it::schema::validate;
use emmet_it::{emmet, emmet_bin, isolate, project, redact, repo, shared_data, temp_dir};
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use serde_json::{Value, json};

struct Client {
    service: RunningService<rmcp::RoleClient, ()>,
    tools: Vec<rmcp::model::Tool>,
}

/// What one call returned: the structured envelope, the text summary, and whether it's an error.
struct Call {
    envelope: Value,
    text: String,
    is_error: bool,
    /// Image content blocks: (base64 data, MIME type).
    images: Vec<(String, String)>,
}

async fn connect(project: Option<&Path>) -> Client {
    connect_with(project, shared_data()).await
}

async fn connect_with(project: Option<&Path>, data: &Path) -> Client {
    let mut cmd = tokio::process::Command::new(emmet_bin());
    let mut std_cmd = std::process::Command::new(emmet_bin());
    isolate(&mut std_cmd);
    for (k, v) in std_cmd.get_envs() {
        match v {
            Some(v) => cmd.env(k, v),
            None => cmd.env_remove(k),
        };
    }
    cmd.arg("--data-dir").arg(data).arg("mcp");
    if let Some(p) = project {
        cmd.arg("--project").arg(p);
    }
    let service = ().serve(TokioChildProcess::new(cmd).expect("spawn emmet mcp")).await.expect("initialize");
    let tools = service.list_all_tools().await.expect("tools/list");
    Client { service, tools }
}

impl Client {
    async fn call(&self, name: &str, args: Value) -> Call {
        let params = CallToolRequestParams::new(name.to_string()).with_arguments(args.as_object().unwrap().clone());
        let r: CallToolResult = self.service.call_tool(params).await.expect("tools/call");
        let envelope = r.structured_content.clone().expect("structured content");
        let text = r
            .content
            .iter()
            .filter_map(|c| c.as_text().map(|t| t.text.clone()))
            .collect::<Vec<_>>()
            .join("\n");
        let images = r
            .content
            .iter()
            .filter_map(|c| c.as_image().map(|i| (i.data.clone(), i.mime_type.clone())))
            .collect();
        let tool = self.tools.iter().find(|t| t.name == name).expect("the tool is listed");
        let schema = Value::Object((*tool.output_schema.clone().expect("an output schema")).clone());
        let problems = validate(&schema, &envelope);
        assert!(
            problems.is_empty(),
            "{name}'s output doesn't match its schema:\n{}\n{envelope}",
            problems.join("\n")
        );
        let input = Value::Object((*tool.input_schema).clone());
        assert!(
            validate(&input, &args).is_empty(),
            "{name}: the test's own arguments don't match the input schema"
        );
        Call {
            envelope,
            text,
            is_error: r.is_error == Some(true),
            images,
        }
    }
}

/// The schemas the server advertises are the committed ones.
fn assert_matches_committed(client: &Client, name: &str) {
    let tool = client
        .tools
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("{name} isn't listed"));
    let path = repo().join(format!("docs/spec/mcp/{name}.schema.json"));
    let committed: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("committed schema")).unwrap();
    assert_eq!(
        committed["input"],
        Value::Object((*tool.input_schema).clone()),
        "{name} input schema is stale"
    );
    assert_eq!(
        committed["output"],
        Value::Object((*tool.output_schema.clone().unwrap()).clone()),
        "{name} output schema is stale"
    );
}

#[tokio::test]
async fn lists_exactly_the_tools_built_so_far() {
    let client = connect(None).await;
    let mut names: Vec<String> = client.tools.iter().map(|t| t.name.to_string()).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "get_params",
            "highlight",
            "list_colors",
            "part_info",
            "render",
            "run_model",
            "search_parts",
            "set_params"
        ]
    );
    let info = client.service.peer_info().expect("server info");
    assert_eq!(info.server_info.as_ref().map(|s| s.name.as_str()), Some("emmet"));
}

#[tokio::test]
async fn search_parts() {
    let client = connect(None).await;
    assert_matches_committed(&client, "search_parts");
    let c = client
        .call("search_parts", json!({ "query": "brick 2x4", "limit": 3 }))
        .await;
    assert!(!c.is_error);
    assert_eq!(c.envelope["data"]["parts"][0]["id"], "3001");
    assert_eq!(c.text.lines().next(), Some("3001 Brick 2 x 4: 2 x 4 studs, 3 plates"));
    assert_eq!(c.text.lines().count(), 3, "one line per part");
    let c = client.call("search_parts", json!({ "query": "zzzz" })).await;
    assert_eq!((c.is_error, c.text.as_str()), (false, "No parts match 'zzzz'."));
    // The error shape: no library in this data dir.
    let empty = temp_dir("mcp-no-library");
    let client = connect_with(None, &empty).await;
    let c = client.call("search_parts", json!({ "query": "brick" })).await;
    assert!(c.is_error);
    assert_eq!(c.envelope["error"]["code"], "LIBRARY_MISSING");
    assert!(
        c.text.starts_with("LIBRARY_MISSING: The LDraw library isn't installed"),
        "{}",
        c.text
    );
}

#[tokio::test]
async fn part_info() {
    let client = connect(None).await;
    assert_matches_committed(&client, "part_info");
    let c = client.call("part_info", json!({ "id": "3023" })).await;
    assert_eq!(
        c.text,
        "3023 ~Moved to 3023b (Moved): 1 x 2 studs, 1 plates. Flags: alias. Use 3023b instead"
    );
    let c = client.call("part_info", json!({ "id": "3001x" })).await;
    assert!(c.is_error);
    assert_eq!(c.envelope["error"]["code"], "UNKNOWN_PART");
    assert!(
        c.text.starts_with("UNKNOWN_PART: There's no part '3001x'"),
        "{}",
        c.text
    );
}

#[tokio::test]
async fn run_model() {
    let dir = temp_dir("mcp-run").join("demo");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .arg("new")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let client = connect(Some(&dir)).await;
    assert_matches_committed(&client, "run_model");
    let c = client.call("run_model", json!({})).await;
    assert!(!c.is_error, "{}", c.text);
    assert_eq!(c.envelope["data"]["parts"], 93);
    assert!(c.text.starts_with("Built 93 parts in 12 steps ("), "{}", c.text);
    assert_eq!(c.envelope["meta"]["generation"], 1);
    // The same output the CLI writes.
    assert!(dir.join("build/model.mpd").is_file());

    let c = client.call("run_model", json!({ "params": { "width": 99 } })).await;
    assert!(c.is_error);
    assert_eq!(c.envelope["error"]["code"], "PARAM_INVALID");
    assert!(
        c.text.starts_with("PARAM_INVALID at model.py:"),
        "the file:line leads the summary: {}",
        c.text
    );
}

#[tokio::test]
async fn run_model_error_has_file_and_line() {
    let dir = project("unknown-color");
    let client = connect(Some(&dir)).await;
    let c = client.call("run_model", json!({})).await;
    assert!(c.is_error);
    let v = redact(c.envelope.clone(), &[(&dir, "[project]")]);
    insta::assert_json_snapshot!("mcp_run_model_error", v);
    assert!(
        c.text
            .starts_with("UNKNOWN_COLOR at model.py:10: There's no color 'Light Blush Gray'."),
        "{}",
        c.text
    );
}

#[tokio::test]
async fn get_params() {
    let dir = project("bad-param");
    std::fs::write(dir.join("params.json"), "{}\n").unwrap();
    let client = connect(Some(&dir)).await;
    assert_matches_committed(&client, "get_params");
    let c = client.call("get_params", json!({})).await;
    assert!(!c.is_error, "{}", c.text);
    assert_eq!(c.text, "width = 4, color = \"Tan\"");
    assert_eq!(c.envelope["data"]["params"][0]["max"], 16.0);

    let none = temp_dir("mcp-none");
    let client = connect(Some(&none)).await;
    let c = client.call("get_params", json!({})).await;
    assert!(c.is_error);
    assert_eq!(c.envelope["error"]["code"], "PROJECT_NOT_FOUND");
}

#[tokio::test]
async fn set_params() {
    let dir = project("bad-param");
    std::fs::write(dir.join("params.json"), "{}\n").unwrap();
    let client = connect(Some(&dir)).await;
    assert_matches_committed(&client, "set_params");
    let c = client
        .call("set_params", json!({ "values": { "width": 6, "color": "LBG" } }))
        .await;
    assert!(!c.is_error, "{}", c.text);
    assert_eq!(c.text, "Changed color = \"LBG\", width = 6.");
    let file: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("params.json")).unwrap()).unwrap();
    assert_eq!(file, json!({ "color": "LBG", "width": 6 }));

    let c = client
        .call("set_params", json!({ "values": { "width": "wide" } }))
        .await;
    assert!(c.is_error);
    assert_eq!(c.envelope["error"]["details"]["line"], 9);
    let file: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("params.json")).unwrap()).unwrap();
    assert_eq!(
        file,
        json!({ "color": "LBG", "width": 6 }),
        "a rejected value leaves params.json alone"
    );
}

#[tokio::test]
async fn list_colors() {
    let client = connect(None).await;
    assert_matches_committed(&client, "list_colors");
    let c = client.call("list_colors", json!({})).await;
    assert!(!c.is_error);
    assert!(c.text.contains("71 Light Bluish Gray"), "{}", c.text);
    let c = client.call("list_colors", json!({ "part": "nope" })).await;
    assert!(c.is_error);
    assert_eq!(c.envelope["error"]["code"], "UNKNOWN_PART");
}

/// With no app open, render goes through the app's render mode (a hidden window) and returns a PNG as image content.
#[tokio::test]
async fn render() {
    let dir = project("scene-colors");
    let client = connect(Some(&dir)).await;
    assert_matches_committed(&client, "render");
    let c = client.call("render", json!({ "view": "front", "size": 128 })).await;
    assert!(!c.is_error, "{}", c.envelope);
    assert_eq!(c.envelope["data"]["via"], "render-mode");
    assert_eq!(c.envelope["data"]["view"], "front");
    assert_eq!(c.text, "The front view at 128 px.");
    let [(png, mime)] = c.images.as_slice() else {
        panic!("one image, not {:?}", c.images.len())
    };
    assert_eq!(mime, "image/png");
    emmet_it::check_png(&emmet_core::base64::decode(png).expect("base64"), 128);
    // Several renders at once, each in its own render mode: each gets its own view back.
    let (a, b) = tokio::join!(
        client.call("render", json!({ "view": "top", "size": 128 })),
        client.call("render", json!({ "view": "left", "size": 160 }))
    );
    assert_eq!(
        (a.envelope["data"]["view"].clone(), b.envelope["data"]["view"].clone()),
        (json!("top"), json!("left"))
    );
    for (c, size) in [(&a, 128), (&b, 160)] {
        assert!(!c.is_error, "{}", c.envelope);
        emmet_it::check_png(&emmet_core::base64::decode(&c.images[0].0).unwrap(), size);
    }
    let c = client.call("render", json!({ "view": "sideways" })).await;
    assert!(c.is_error);
    assert_eq!(c.envelope["error"]["code"], "PARAM_INVALID");
}

/// With no app open, highlight says so instead of failing.
#[tokio::test]
async fn highlight() {
    let dir = project("scene-colors");
    let client = connect(Some(&dir)).await;
    assert_matches_committed(&client, "highlight");
    let c = client.call("highlight", json!({ "ids": ["p1"] })).await;
    assert!(!c.is_error, "{}", c.envelope);
    assert_eq!(c.envelope["data"]["delivered"], false);
    assert!(c.text.starts_with("Emmet app not open"), "{}", c.text);
}

#[test]
fn mcp_schemas_are_fresh() {
    // The committed docs/spec/mcp/*.schema.json are what `emmet mcp --write-schemas` writes now.
    let fresh = temp_dir("schemas");
    let out = emmet().arg("mcp").arg("--write-schemas").arg(&fresh).output().unwrap();
    assert!(out.status.success());
    let committed = repo().join("docs/spec/mcp");
    let names = |d: &Path| {
        let mut v: Vec<String> = std::fs::read_dir(d)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    };
    assert_eq!(
        names(&committed),
        names(&fresh),
        "run `emmet mcp --write-schemas docs/spec/mcp`"
    );
    for name in names(&fresh) {
        let a = std::fs::read_to_string(fresh.join(&name)).unwrap();
        let b = std::fs::read_to_string(committed.join(&name))
            .unwrap()
            .replace("\r\n", "\n");
        assert!(
            a == b,
            "docs/spec/mcp/{name} is stale: run `emmet mcp --write-schemas docs/spec/mcp`"
        );
    }
}

#[test]
fn the_schema_validator_catches_mistakes() {
    // The contract tests lean on emmet_it::schema, so it has to fail when it should.
    let schema = json!({
        "type": "object",
        "required": ["ok", "parts"],
        "properties": {
            "ok": { "type": "boolean" },
            "parts": { "type": "array", "items": { "$ref": "#/$defs/Part" }, "maxItems": 2 },
            "size": { "anyOf": [{ "$ref": "#/$defs/Size" }, { "type": "null" }] }
        },
        "$defs": {
            "Part": { "type": "object", "required": ["id"], "properties": { "id": { "type": "string" } } },
            "Size": { "type": "object", "properties": { "x": { "type": "number" } } }
        }
    });
    assert!(
        validate(
            &schema,
            &json!({ "ok": true, "parts": [{ "id": "3001" }], "size": null })
        )
        .is_empty()
    );
    let bad = validate(
        &schema,
        &json!({ "ok": "yes", "parts": [{ "id": 3001 }, {}, {}], "size": { "x": "1" } }),
    );
    let text = bad.join("\n");
    for want in [
        "$.ok: expected boolean",
        "$.parts[0].id: expected string",
        "$.parts[1]: missing required 'id'",
        "3 items is too many",
        "$.size: matches 0 of the anyOf",
    ] {
        assert!(text.contains(want), "missing '{want}' in:\n{text}");
    }
    assert_eq!(
        validate(&schema, &json!({ "ok": true }))[0],
        "$: missing required 'parts'"
    );
}
