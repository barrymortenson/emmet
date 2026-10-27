// SPDX-License-Identifier: AGPL-3.0-or-later
//! fake-claude: replays a Claude Code stream-JSON transcript (see the crate docs). Beyond the replay it does what
//! Claude Code does around a tool call: asks the permission prompt tool (the real `emmet_host approve`) wherever the
//! recording shows Claude Code asking 0B's stub, runs MCP tools on the real servers, and makes Edit and Write changes.
//! It answers `--version` and `auth status --json` too, for detection, and `auth login` as setup::sign_in runs it:
//! `claude-login.mode` in `EMMET_FAKE_DIR` says how the browser sign-in goes (`ok`, `fail`, `unconfirmed`, or `wait`), and
//! `claude-login.json` records the arguments and whether stdin was closed.

use std::collections::HashMap;
use std::path::PathBuf;

use emmet_fake_agents::{
    Fake, Servers, Subst, edit_file, exit_after_eof, fail, mcp_name, read, text_blocks, write, write_raw,
};
use serde_json::{Value, json};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        println!("2.1.310 (Claude Code)");
        return;
    }
    // Detection's status check and the sign-in don't count as starts.
    if args.first().map(String::as_str) == Some("auth") {
        let dir = PathBuf::from(std::env::var_os("EMMET_FAKE_DIR").unwrap_or_default());
        if args.get(1).map(String::as_str) == Some("login") {
            login(&dir, &args);
        }
        let auth = std::fs::read_to_string(dir.join("claude-auth.json")).unwrap_or_else(|_| {
            r#"{"loggedIn":true,"authMethod":"claude.ai","apiKeySource":"none","subscriptionType":"max"}"#.into()
        });
        println!("{}", auth.trim());
        return;
    }
    let fake = Fake::setup("claude");
    if args.iter().any(|a| a == "--bare" || a == "--setting-sources") {
        fail("Emmet must never pass --bare or --setting-sources (D-37)");
    }
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let usage = args.iter().any(|a| a == "--no-session-persistence");
    let project = std::env::current_dir().unwrap_or_else(|e| fail(e));
    let mcp = flag("--mcp-config").map(PathBuf::from);
    let subst = Subst::new(&project, mcp.as_ref().and_then(|p| p.parent()));
    let servers = Servers::start(mcp.map(|p| servers(&p)).unwrap_or_default());
    let script = fake.script(if usage { "-usage" } else { "" });

    // Where Claude Code asked the permission prompt tool, by tool use id (0B's stub logged each).
    let mut asked: HashMap<String, Value> = HashMap::new();
    for l in script.iter().filter(|l| l.dir == "stub" && l.msg["tool"] == "approve") {
        let args = subst.apply(&l.msg["args"]);
        if let Some(id) = args["tool_use_id"].as_str() {
            asked.insert(id.to_string(), args);
        }
    }
    let mut tool_uses: HashMap<String, (String, Value)> = HashMap::new();
    let mut requests: HashMap<String, String> = HashMap::new();
    // A user message the recorder wrote right after an interrupt (before the turn ended): Emmet sends it once the turn
    // is over, so it's read when the next turn starts.
    let mut deferred: Option<Value> = None;
    let mut last_in: Option<&str> = None;
    let expect = |want: &Value, requests: &mut HashMap<String, String>| {
        let Some(got) = read() else { std::process::exit(0) };
        let kind = want["type"].as_str().unwrap_or("");
        if got["type"] != kind {
            fail(format!("expected a {kind} message from the adapter, got {got}"));
        }
        if let (Some(r), Some(a)) = (want["request_id"].as_str(), got["request_id"].as_str()) {
            requests.insert(r.to_string(), a.to_string());
        }
    };
    for line in &script {
        if line.dir != "in" && line.dir != "stub" && line.dir != "note" {
            last_in = None;
        }
        match line.dir.as_str() {
            "in" => {
                if line.msg["type"] == "user" && last_in == Some("control_request") {
                    deferred = Some(line.msg.clone());
                    continue;
                }
                last_in = line.msg["type"].as_str();
                expect(&line.msg, &mut requests);
            }
            "out" => {
                let mut msg = subst.apply(&line.msg);
                if msg["type"] == "system"
                    && msg["subtype"] == "init"
                    && let Some(want) = deferred.take()
                {
                    expect(&want, &mut requests);
                }
                match msg["type"].as_str() {
                    Some("control_response") => {
                        let r = msg["response"]["request_id"].as_str().unwrap_or("").to_string();
                        if let Some(actual) = requests.get(&r) {
                            msg["response"]["request_id"] = json!(actual);
                        }
                    }
                    Some("assistant") => {
                        for b in msg["message"]["content"].as_array().into_iter().flatten() {
                            if b["type"] == "tool_use" {
                                let (id, name) = (b["id"].as_str().unwrap_or(""), b["name"].as_str().unwrap_or(""));
                                tool_uses.insert(id.to_string(), (name.to_string(), b["input"].clone()));
                            }
                        }
                    }
                    Some("user") => {
                        if let Some(blocks) = msg["message"]["content"].as_array_mut() {
                            for b in blocks.iter_mut().filter(|b| b["type"] == "tool_result") {
                                let id = b["tool_use_id"].as_str().unwrap_or("").to_string();
                                if let Some((name, input)) = tool_uses.get(&id) {
                                    run_tool(&servers, &asked, &id, name, input, b);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                write(&msg);
            }
            "raw" => write_raw(line.msg.as_str().unwrap_or("")),
            "err" => eprintln!("{}", line.msg.as_str().unwrap_or("")),
            "note" => {
                if let Some((code, crash)) = line.exit() {
                    if crash {
                        std::process::exit(code);
                    }
                    exit_after_eof(code);
                }
            }
            _ => {}
        }
    }
    exit_after_eof(0)
}

/// The servers in Claude Code's `--mcp-config` file.
fn servers(path: &std::path::Path) -> Vec<(String, String, Vec<String>)> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| fail(format!("{}: {e}", path.display())));
    let config: Value = serde_json::from_str(&text).unwrap_or_else(|e| fail(e));
    config["mcpServers"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(name, s)| {
            let args = s["args"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|a| a.as_str().map(String::from))
                .collect();
            (name.clone(), s["command"].as_str().unwrap_or("").to_string(), args)
        })
        .collect()
}

/// What Claude Code does before it writes a tool result: the permission prompt where it asked one, then the tool.
/// `block` is the recorded `tool_result`, rewritten in place.
fn run_tool(servers: &Servers, asked: &HashMap<String, Value>, id: &str, name: &str, input: &Value, block: &mut Value) {
    if let Some(request) = asked.get(id) {
        let answer = servers.call("emmet_host", "approve", request);
        let text = answer["content"][0]["text"].as_str().unwrap_or("{}");
        let decision: Value = serde_json::from_str(text).unwrap_or_default();
        if decision["behavior"] != "allow" {
            block["content"] = json!(decision["message"].as_str().unwrap_or("denied"));
            block["is_error"] = json!(true);
            return;
        }
        if block["is_error"] == true {
            // 0B's stub denied what Emmet allows (a read outside the project): show what the tool would return.
            block["is_error"] = json!(false);
            block["content"] = json!(match name {
                "Read" => std::fs::read_to_string(input["file_path"].as_str().unwrap_or("")).unwrap_or_default(),
                _ => "ok".into(),
            });
        }
    }
    if let Some((server, tool)) = mcp_name(name)
        && servers.has(server, tool)
    {
        let result = servers.call(server, tool, input);
        block["content"] = Value::Array(text_blocks(&result));
        block["is_error"] = json!(result["isError"] == true);
        return;
    }
    if block["is_error"] != true {
        edit_file(name, input);
    }
}

/// `claude auth login`: Claude Code's own words, then the outcome the test chose. Signing in writes the status
/// `auth status` reports, in the fake's folder.
fn login(dir: &std::path::Path, args: &[String]) -> ! {
    use std::io::Write;
    // Emmet must never hand Claude Code anything at its paste prompt: its stdin is closed.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = [0u8; 1];
        let _ = tx.send(matches!(std::io::Read::read(&mut std::io::stdin(), &mut buf), Ok(0)));
    });
    let closed = rx.recv_timeout(std::time::Duration::from_millis(500)).unwrap_or(false);
    let _ = std::fs::write(
        dir.join("claude-login.json"),
        json!({"args": args, "stdin_closed": closed}).to_string(),
    );
    let mode = std::fs::read_to_string(dir.join("claude-login.mode")).unwrap_or_else(|_| "ok".into());
    println!("Opening browser to sign in…");
    println!("If the browser didn't open, visit: https://claude.example.invalid/oauth/authorize?state=fake-state");
    print!("Paste code here if prompted > ");
    let _ = std::io::stdout().flush();
    match mode.trim() {
        "ok" => {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let _ = std::fs::write(
                dir.join("claude-auth.json"),
                r#"{"loggedIn":true,"authMethod":"claude.ai","apiKeySource":"none","subscriptionType":"pro"}"#,
            );
            println!("Login successful.");
            std::process::exit(0);
        }
        // Claude Code says it's done, but `auth status` still says signed out.
        "unconfirmed" => {
            println!("Login successful.");
            std::process::exit(0);
        }
        "wait" => {
            std::thread::sleep(std::time::Duration::from_secs(600));
            std::process::exit(1);
        }
        _ => {
            println!("Login failed: the authorization was denied");
            std::process::exit(1);
        }
    }
}
