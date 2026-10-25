// SPDX-License-Identifier: AGPL-3.0-or-later
//! fake-codex: replays a `codex app-server` JSON-RPC transcript (see the crate docs). It maps the recording's request
//! ids to the adapter's, finds the MCP servers in the `-c mcp_servers.*` overrides and runs their tools for real when
//! a tool call completes, and applies fileChange items the adapter approved. It answers `--version` for detection.

use std::collections::HashMap;

use emmet_fake_agents::{Fake, Servers, Subst, apply_change, exit_after_eof, fail, read, write, write_raw};
use serde_json::{Value, json};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        println!("codex-cli 0.160.1");
        return;
    }
    let fake = Fake::setup("codex");
    if args.first().map(String::as_str) != Some("app-server") || !args.iter().any(|a| a == "--strict-config") {
        fail(format!("expected app-server … --strict-config, got {args:?}"));
    }
    let project = std::env::current_dir().unwrap_or_else(|e| fail(e));
    let subst = Subst::new(&project, None);
    let servers = Servers::start(servers(&args));
    let script = fake.script("");

    // The recording's request ids → the adapter's, for the responses to them.
    let mut ids: HashMap<String, Value> = HashMap::new();
    // What the adapter answered each approval request with, by the item it was about.
    let mut decisions: HashMap<String, String> = HashMap::new();
    let mut asked: HashMap<String, String> = HashMap::new();
    for line in &script {
        match line.dir.as_str() {
            "in" => {
                let Some(got) = read() else { std::process::exit(0) };
                let want = &line.msg;
                match want["method"].as_str() {
                    Some(m) if got["method"] != m => fail(format!("expected {m} from the adapter, got {got}")),
                    Some(_) => {
                        if let Some(id) = want.get("id") {
                            ids.insert(id.to_string(), got["id"].clone());
                        }
                    }
                    // A response to one of our requests.
                    None => {
                        if got.get("method").is_some() || got["id"] != want["id"] {
                            fail(format!("expected the response to request {}, got {got}", want["id"]));
                        }
                        if let Some(item) = asked.remove(&want["id"].to_string()) {
                            let d = got["result"]["decision"]
                                .as_str()
                                .or(got["result"]["action"].as_str())
                                .unwrap_or("decline");
                            decisions.insert(item, d.to_string());
                        }
                    }
                }
            }
            "out" => {
                let mut msg = subst.apply(&line.msg);
                let method = msg["method"].as_str().map(String::from);
                match (method.as_deref(), msg.get("id").cloned()) {
                    // A response: to the adapter's own request id.
                    (None, Some(id)) => {
                        if let Some(actual) = ids.get(&id.to_string()) {
                            msg["id"] = actual.clone();
                        }
                    }
                    // A request of ours: remember which item it's about.
                    (Some(_), Some(id)) => {
                        let item = msg["params"]["itemId"].as_str().unwrap_or("").to_string();
                        asked.insert(id.to_string(), item);
                    }
                    (Some("item/completed"), None) => complete(&servers, &decisions, &mut msg["params"]["item"]),
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

/// What Codex does as an item completes: an MCP tool call runs on the real server, and an approved file change is
/// written.
fn complete(servers: &Servers, decisions: &HashMap<String, String>, item: &mut Value) {
    let id = item["id"].as_str().unwrap_or("").to_string();
    match item["type"].as_str() {
        Some("mcpToolCall") => {
            let (server, tool) = (
                item["server"].as_str().unwrap_or(""),
                item["tool"].as_str().unwrap_or(""),
            );
            if !servers.has(server, tool) {
                return;
            }
            let r = servers.call(server, tool, &item["arguments"]);
            let failed = r["isError"] == true;
            item["status"] = json!(if failed { "failed" } else { "completed" });
            item["result"] = json!({ "content": r["content"], "structuredContent": r["structuredContent"] });
            item["error"] = if failed {
                json!({ "message": r["content"][0]["text"] })
            } else {
                Value::Null
            };
        }
        Some("fileChange") if item["status"] == "completed" => {
            if decisions.get(&id).is_some_and(|d| !d.starts_with("accept")) {
                item["status"] = json!("declined");
                return;
            }
            for change in item["changes"].as_array().into_iter().flatten() {
                apply_change(change);
            }
        }
        _ => {}
    }
}

/// `-c mcp_servers.<name>.command="…"` and `.args=[…]`, as Codex reads them (TOML basic strings).
fn servers(args: &[String]) -> Vec<(String, String, Vec<String>)> {
    let mut found: HashMap<String, (String, Vec<String>)> = HashMap::new();
    for kv in args.iter().filter_map(|a| a.strip_prefix("mcp_servers.")) {
        let Some((key, value)) = kv.split_once('=') else {
            continue;
        };
        let Some((name, field)) = key.split_once('.') else {
            continue;
        };
        let entry = found.entry(name.to_string()).or_default();
        match field {
            "command" => entry.0 = strings(value).into_iter().next().unwrap_or_default(),
            "args" => entry.1 = strings(value),
            _ => {}
        }
    }
    let mut out: Vec<_> = found.into_iter().map(|(n, (c, a))| (n, c, a)).collect();
    out.sort();
    out
}

/// The TOML basic strings in a value: one (`"x"`) or a list (`["a","b"]`).
fn strings(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '"' {
            continue;
        }
        let mut s = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => match chars.next() {
                    Some('n') => s.push('\n'),
                    Some(c) => s.push(c),
                    None => {}
                },
                c => s.push(c),
            }
        }
        out.push(s);
    }
    out
}
