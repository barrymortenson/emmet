// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet mcp-host` (plan sections 09 and 10), through an rmcp client: what `approve` allows on its own (reads, and
//! edits to model files inside the project) and what becomes a card (anything else, including agent config inside
//! the project and paths that leave it through `..` or a link). With no app to show the card, it's denied with the
//! reason. And `ask_user`'s contract.

use emmet_it::agents::Rig;
use emmet_it::{emmet_bin, isolate, temp_dir};
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use serde_json::{Value, json};

async fn host(project: &std::path::Path) -> RunningService<rmcp::RoleClient, ()> {
    let mut cmd = tokio::process::Command::new(emmet_bin());
    let mut std_cmd = std::process::Command::new(emmet_bin());
    isolate(&mut std_cmd);
    for (k, v) in std_cmd.get_envs() {
        match v {
            Some(v) => cmd.env(k, v),
            None => cmd.env_remove(k),
        };
    }
    // A data dir with no app: every card is refused with the reason.
    cmd.arg("--data-dir")
        .arg(temp_dir("host-data"))
        .arg("--project")
        .arg(project)
        .args(["mcp-host", "--session", "s1"]);
    ().serve(TokioChildProcess::new(cmd).expect("spawn"))
        .await
        .expect("initialize")
}

async fn call(h: &RunningService<rmcp::RoleClient, ()>, tool: &str, args: Value) -> rmcp::model::CallToolResult {
    let params = CallToolRequestParams::new(tool.to_string()).with_arguments(args.as_object().unwrap().clone());
    h.call_tool(params).await.expect("tools/call")
}

async fn decide(h: &RunningService<rmcp::RoleClient, ()>, tool: &str, input: Value) -> String {
    let r = call(
        h,
        "approve",
        json!({ "tool_name": tool, "input": input, "tool_use_id": "t1" }),
    )
    .await;
    let text = r.content[0].as_text().unwrap().text.clone();
    let v: Value = serde_json::from_str(&text).unwrap();
    if v["behavior"] == "allow" {
        assert_eq!(v["updatedInput"], input, "allow passes the input back unchanged");
        "allow".into()
    } else {
        assert!(v["message"].as_str().unwrap().contains("couldn't ask the user"), "{v}");
        "card".into()
    }
}

#[tokio::test]
async fn approve_allows_reads_and_model_edits_and_asks_for_the_rest() {
    let rig = Rig::new("host-approve", "claude", "plain-turn");
    let p = rig.project.clone();
    let h = host(&p).await;
    let f = |rel: &str| p.join(rel).display().to_string();
    let cases: Vec<(&str, Value, &str)> = vec![
        ("Read", json!({ "file_path": "/etc/hosts" }), "allow"),
        ("Grep", json!({ "pattern": "x" }), "allow"),
        (
            "Edit",
            json!({ "file_path": f("model.py"), "old_string": "a", "new_string": "b" }),
            "allow",
        ),
        (
            "Write",
            json!({ "file_path": f("notes.txt"), "content": "hi" }),
            "allow",
        ),
        (
            "Write",
            json!({ "file_path": f("models/roof.py"), "content": "" }),
            "allow",
        ),
        (
            "Write",
            json!({ "file_path": "notes.txt", "content": "relative to the project" }),
            "allow",
        ),
        ("mcp__emmet__run_model", json!({}), "allow"),
        ("Bash", json!({ "command": "ls" }), "card"),
        ("WebFetch", json!({ "url": "https://example.com" }), "card"),
        (
            "Write",
            json!({ "file_path": f("../outside.txt"), "content": "x" }),
            "card",
        ),
        (
            "Write",
            json!({ "file_path": f("new/../../outside.txt"), "content": "x" }),
            "card",
        ),
        (
            "Write",
            json!({ "file_path": f(".claude/settings.json"), "content": "{}" }),
            "card",
        ),
        ("Write", json!({ "file_path": f(".mcp.json"), "content": "{}" }), "card"),
        (
            "Write",
            json!({ "file_path": f(".git/hooks/pre-commit"), "content": "" }),
            "card",
        ),
        (
            "Edit",
            json!({ "file_path": f("AGENTS.md"), "old_string": "a", "new_string": "b" }),
            "card",
        ),
        ("Write", json!({ "file_path": f("claude.md"), "content": "" }), "card"),
        ("Edit", json!({}), "card"),
    ];
    for (tool, input, want) in cases {
        assert_eq!(decide(&h, tool, input.clone()).await, want, "{tool} {input}");
    }
    #[cfg(unix)]
    {
        let outside = temp_dir("host-outside");
        std::os::unix::fs::symlink(&outside, p.join("link")).unwrap();
        let input = json!({ "file_path": f("link/escape.txt"), "content": "x" });
        assert_eq!(decide(&h, "Write", input).await, "card", "a link out of the project");
    }
    h.cancel().await.ok();
}

#[tokio::test]
async fn ask_user_needs_two_to_four_options_and_says_why_there_is_no_answer() {
    let rig = Rig::new("host-ask", "claude", "plain-turn");
    let h = host(&rig.project).await;
    let one = call(
        &h,
        "ask_user",
        json!({ "question": "Which?", "options": [{ "label": "A" }] }),
    )
    .await;
    assert_eq!(one.is_error, Some(true));
    assert!(one.content[0].as_text().unwrap().text.contains("2 to 4 options"));
    // No app: the chat is gone, so there's no answer and the agent is told to end its turn.
    let r = call(
        &h,
        "ask_user",
        json!({ "question": "Which?", "header": "A very long header", "options": [{ "label": "A" }, { "label": "B" }] }),
    )
    .await;
    assert_eq!(
        r.structured_content,
        Some(json!({ "answered": false, "reason": "stopped" }))
    );
    assert!(r.content[0].as_text().unwrap().text.contains("End your turn"));
    h.cancel().await.ok();
}
