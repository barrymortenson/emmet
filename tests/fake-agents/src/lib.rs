// SPDX-License-Identifier: AGPL-3.0-or-later
//! The replay harness's shared half (plan section 16): what fake-claude and fake-codex both do. Each fake replays one
//! recorded transcript (tests/replay/, 0B's `{t, dir, msg}` lines) as the CLI would, so the adapters can be tested
//! end to end without a model:
//!
//! - It waits for each `in` line from the adapter (checking its kind) and writes each `out` line, with the recording's
//!   paths (`<WS>/project`, …) replaced by this run's. There's no sleeping: the adapter's own input paces it.
//! - It starts the MCP servers the adapter configured (the real `emmet mcp` and `emmet mcp-host`) and calls them for
//!   every tool they have, so results, approvals, questions, and redirect notes are real. Tools only 0B's stub had
//!   (`ping`, `block`) keep their recorded results.
//! - It makes the file changes the CLI would (Claude's Edit and Write, Codex's fileChange items), so tests can check
//!   the project.
//!
//! Settings come from `EMMET_FAKE_DIR`: `<cli>.scenario` names the transcript (`<cli>-usage.scenario` for Claude's
//! usage probe), each start is counted (a transcript with two processes, like resume, plays one per start), and each
//! start's arguments, environment, and folder are recorded as `<cli>-<n>.json` for the tests to read.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use serde_json::{Value, json};

pub struct Fake {
    pub dir: PathBuf,
    pub cli: &'static str,
    /// This start's number, from 0.
    pub start: usize,
}

/// Stops the fake with a message the adapter's crash report will carry.
pub fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("fake agent: {msg}");
    std::process::exit(3)
}

impl Fake {
    /// Reads `EMMET_FAKE_DIR`, counts this start, and records its arguments, environment, and folder.
    pub fn setup(cli: &'static str) -> Self {
        let dir = std::env::var_os("EMMET_FAKE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| fail("EMMET_FAKE_DIR isn't set"));
        let count = dir.join(format!("{cli}.count"));
        let start: usize = std::fs::read_to_string(&count)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        let _ = std::fs::write(&count, (start + 1).to_string());
        let record = json!({
            "args": std::env::args().skip(1).collect::<Vec<_>>(),
            "env": std::env::vars().collect::<HashMap<_, _>>(),
            "cwd": std::env::current_dir().ok(),
        });
        let _ = std::fs::write(dir.join(format!("{cli}-{start}.json")), record.to_string());
        Self { dir, cli, start }
    }

    /// The transcript lines for this start: the scenario's `start`-th process (or its last).
    pub fn script(&self, variant: &str) -> Vec<Line> {
        let name = format!("{}{variant}.scenario", self.cli);
        let path = std::fs::read_to_string(self.dir.join(&name))
            .unwrap_or_else(|e| fail(format!("{name} in EMMET_FAKE_DIR: {e}")));
        let text = std::fs::read_to_string(path.trim())
            .unwrap_or_else(|e| fail(format!("the transcript {}: {e}", path.trim())));
        let mut segments: Vec<Vec<Line>> = vec![];
        for l in text.lines().filter(|l| !l.trim().is_empty()) {
            let v: Value = serde_json::from_str(l).unwrap_or_else(|e| fail(format!("a transcript line: {e}")));
            let line = Line {
                dir: v["dir"].as_str().unwrap_or("").to_string(),
                msg: v["msg"].clone(),
            };
            if line.dir == "note" && line.msg.get("meta").is_some() || segments.is_empty() {
                segments.push(vec![]);
            }
            segments.last_mut().expect("a segment").push(line);
        }
        let n = self.start.min(segments.len().saturating_sub(1));
        segments.into_iter().nth(n).unwrap_or_default()
    }
}

pub struct Line {
    pub dir: String,
    pub msg: Value,
}

impl Line {
    /// The exit a note records, if it's one: (code, whether it's a crash that doesn't wait for stdin to close).
    pub fn exit(&self) -> Option<(i32, bool)> {
        let e = self.msg.get("exit")?;
        let crash = e["crash"] == true;
        let code = e["code"].as_i64().map(|c| c as i32).unwrap_or(137);
        Some((code, crash))
    }
}

/// The recording's placeholders and this run's paths, longest first.
pub struct Subst(Vec<(String, String)>);

impl Subst {
    pub fn new(project: &Path, session: Option<&Path>) -> Self {
        let p = |p: &Path| p.display().to_string();
        let mut pairs = vec![("<WS>/project".to_string(), p(project))];
        if let Some(s) = session {
            pairs.push(("<WS>/session".into(), p(s)));
        }
        if let Some(h) = std::env::var_os("HOME").or(std::env::var_os("USERPROFILE")) {
            pairs.push(("<WS>/home".into(), p(Path::new(&h))));
        }
        pairs.push(("<WS>".into(), p(project.parent().unwrap_or(project))));
        pairs.push(("<TMP>".into(), p(&std::env::temp_dir())));
        Self(pairs)
    }

    pub fn apply(&self, v: &Value) -> Value {
        match v {
            Value::String(s) => {
                let mut s = s.clone();
                for (from, to) in &self.0 {
                    s = s.replace(from, to);
                }
                Value::String(s)
            }
            Value::Array(a) => Value::Array(a.iter().map(|v| self.apply(v)).collect()),
            Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), self.apply(v))).collect()),
            v => v.clone(),
        }
    }
}

/// One line from the adapter, or None when it closed stdin.
pub fn read() -> Option<Value> {
    let mut line = String::new();
    loop {
        line.clear();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => return None,
            Ok(_) if line.trim().is_empty() => continue,
            Ok(_) => {
                return Some(
                    serde_json::from_str(line.trim())
                        .unwrap_or_else(|e| fail(format!("the adapter wrote non-JSON: {e}"))),
                );
            }
        }
    }
}

pub fn write(v: &Value) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{v}");
    let _ = out.flush();
}

/// A line that isn't JSON (the malformed-line case).
pub fn write_raw(s: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{s}");
    let _ = out.flush();
}

/// Waits for the adapter to close stdin, then exits with `code`. Anything else it writes is a protocol error.
pub fn exit_after_eof(code: i32) -> ! {
    if let Some(v) = read() {
        fail(format!("the adapter wrote {v} after the recording ended"));
    }
    std::process::exit(code)
}

/// The MCP servers the adapter configured, connected as the CLI would connect them.
pub struct Servers {
    rt: tokio::runtime::Runtime,
    clients: HashMap<String, (RunningService<rmcp::RoleClient, ()>, HashSet<String>)>,
}

impl Servers {
    /// Starts each `(name, command, args)` and lists its tools.
    pub fn start(specs: Vec<(String, String, Vec<String>)>) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|e| fail(e));
        let mut clients = HashMap::new();
        for (name, command, args) in specs {
            let mut cmd = tokio::process::Command::new(&command);
            cmd.args(&args);
            let client = rt
                .block_on(async {
                    let service = ().serve(TokioChildProcess::new(cmd)?).await?;
                    let tools = service.list_all_tools().await?;
                    Ok::<_, Box<dyn std::error::Error>>((service, tools))
                })
                .unwrap_or_else(|e| fail(format!("the MCP server {name} ({command}) didn't start: {e}")));
            let tools = client.1.iter().map(|t| t.name.to_string()).collect();
            clients.insert(name, (client.0, tools));
        }
        Self { rt, clients }
    }

    pub fn has(&self, server: &str, tool: &str) -> bool {
        self.clients.get(server).is_some_and(|(_, t)| t.contains(tool))
    }

    /// Calls a tool and returns the MCP result as JSON (`content`, `structuredContent`, `isError`).
    pub fn call(&self, server: &str, tool: &str, args: &Value) -> Value {
        let (client, _) = &self.clients[server];
        let params =
            CallToolRequestParams::new(tool.to_string()).with_arguments(args.as_object().cloned().unwrap_or_default());
        let r = self
            .rt
            .block_on(client.call_tool(params))
            .unwrap_or_else(|e| fail(format!("{server}.{tool}: {e}")));
        serde_json::to_value(r).unwrap_or_default()
    }
}

/// `mcp__<server>__<tool>`, split.
pub fn mcp_name(name: &str) -> Option<(&str, &str)> {
    name.strip_prefix("mcp__")?.split_once("__")
}

/// A tool result's text blocks, as a CLI would pass them on (an image becomes a note that there was one).
pub fn text_blocks(result: &Value) -> Vec<Value> {
    result["content"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|b| match b["type"].as_str() {
            Some("text") => json!({ "type": "text", "text": b["text"] }),
            _ => json!({ "type": "text", "text": "[image]" }),
        })
        .collect()
}

/// Claude Code's Edit, MultiEdit, and Write, on the real file.
pub fn edit_file(tool: &str, input: &Value) {
    let Some(path) = input["file_path"].as_str() else {
        return;
    };
    match tool {
        "Write" => {
            let _ = std::fs::write(path, input["content"].as_str().unwrap_or(""));
        }
        "Edit" | "MultiEdit" => {
            let Ok(mut text) = std::fs::read_to_string(path) else {
                return;
            };
            let edits = if tool == "Edit" {
                vec![input.clone()]
            } else {
                input["edits"].as_array().cloned().unwrap_or_default()
            };
            for e in edits {
                let (old, new) = (
                    e["old_string"].as_str().unwrap_or(""),
                    e["new_string"].as_str().unwrap_or(""),
                );
                text = if e["replace_all"] == true {
                    text.replace(old, new)
                } else {
                    text.replacen(old, new, 1)
                };
            }
            let _ = std::fs::write(path, text);
        }
        _ => {}
    }
}

/// A Codex fileChange: `add` writes the diff as the file, `update` applies a unified diff, `delete` removes it.
pub fn apply_change(change: &Value) {
    let Some(path) = change["path"].as_str() else { return };
    let diff = change["diff"].as_str().unwrap_or("");
    match change["kind"]["type"].as_str() {
        Some("add") => {
            let _ = std::fs::write(path, diff);
        }
        Some("delete") => {
            let _ = std::fs::remove_file(path);
        }
        Some("update") => {
            let Ok(text) = std::fs::read_to_string(path) else {
                return;
            };
            let _ = std::fs::write(path, patch(&text, diff));
        }
        _ => {}
    }
}

/// Applies a unified diff's hunks by their context, not their line numbers (enough for recorded diffs).
fn patch(text: &str, diff: &str) -> String {
    let mut out = text.to_string();
    let mut old = String::new();
    let mut new = String::new();
    let flush = |out: &mut String, old: &mut String, new: &mut String| {
        if !old.is_empty() || !new.is_empty() {
            *out = out.replacen(old.as_str(), new, 1);
        }
        old.clear();
        new.clear();
    };
    for line in diff.lines() {
        if line.starts_with("@@") {
            flush(&mut out, &mut old, &mut new);
        } else if let Some(l) = line.strip_prefix('-') {
            old.push_str(l);
            old.push('\n');
        } else if let Some(l) = line.strip_prefix('+') {
            new.push_str(l);
            new.push('\n');
        } else if let Some(l) = line.strip_prefix(' ') {
            old.push_str(l);
            old.push('\n');
            new.push_str(l);
            new.push('\n');
        }
    }
    flush(&mut out, &mut old, &mut new);
    out
}
