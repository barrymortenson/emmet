// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Claude Code adapter (plan section 10, ADR 0001): `claude -p` with stream-JSON both ways. Approvals reach the
//! user through `emmet_host approve` (the permission prompt tool) and the hub, so this module is only the arguments,
//! the encoding, and the mapping of stream-JSON to events.
//!
//! The process gets the user's environment exactly (D-37): never `--bare`, never `--setting-sources`, and no
//! credential variable added, removed, or changed. `MCP_TOOL_TIMEOUT` is the one variable Emmet adds.

use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use serde_json::{Value, json};

use crate::session::{Engine, Io, Protocol, no_window, session_dir, set_env};
use crate::{
    AgentAdapter, AgentError, AgentErrorCode, AgentEvent, AgentKind, Billing, Decision, Detection, DoneReason, Result,
    Session, SessionConfig, UsageSnapshot, UserNote, detect,
};

/// One hour, so `ask_user` can wait for a slow answer (ADR 0001: no cap showed up).
pub const MCP_TOOL_TIMEOUT: &str = "3600000";

/// The session settings (D-40): Claude Code's sandbox on macOS (failing rather than running unsandboxed), and on
/// native Windows, where it has no sandbox, a card for every shell command and web request.
pub fn settings() -> &'static str {
    if cfg!(windows) {
        r#"{"permissions":{"ask":["Bash","PowerShell","WebFetch","WebSearch"]}}"#
    } else {
        r#"{"sandbox":{"enabled":true,"failIfUnavailable":true}}"#
    }
}

pub struct Claude {
    env: Vec<(String, String)>,
    data: Option<PathBuf>,
}

impl Claude {
    pub fn new(env: Vec<(String, String)>) -> Self {
        Self { env, data: None }
    }

    /// Where the usage probe runs: an empty folder in app data.
    pub fn with_data(mut self, data: PathBuf) -> Self {
        self.data = Some(data);
        self
    }

    fn bin(&self) -> Result<PathBuf> {
        detect::find(AgentKind::Claude, &self.env)
            .ok_or_else(|| AgentError::new(AgentErrorCode::NotInstalled, "Claude Code isn't installed."))
    }
}

/// The arguments for a chat session (plan section 10), with `mcp` the session's mcp.json.
pub fn args(mcp: &std::path::Path, resume: Option<&str>, model: Option<&str>) -> Vec<String> {
    let mut a: Vec<String> = [
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--include-partial-messages",
        "--mcp-config",
    ]
    .map(String::from)
    .to_vec();
    a.push(mcp.display().to_string());
    a.extend(
        [
            "--strict-mcp-config",
            "--permission-prompt-tool",
            "mcp__emmet_host__approve",
            "--allowedTools",
            "mcp__emmet__*",
            "mcp__emmet_host__ask_user",
            "Read(./**)",
            "Edit(model.py)",
            "Edit(models/**)",
            "Edit(CARD.md)",
            "--disallowedTools",
            "AskUserQuestion",
            "--settings",
            settings(),
            "--append-system-prompt",
            crate::APP_POINTER,
        ]
        .map(String::from),
    );
    if let Some(r) = resume {
        // One argument, so a token could never read as a flag (and cfg.check() allows only plain ids).
        a.push(format!("--resume={r}"));
    }
    if let Some(m) = model {
        a.extend(["--model".into(), m.into()]);
    }
    a
}

/// The session's MCP servers: `emmet` and `emmet_host`, both the sidecar, both for this session.
pub fn mcp_config(cfg: &SessionConfig) -> Value {
    let (emmet, data, project) = (
        cfg.emmet.display().to_string(),
        cfg.data.display().to_string(),
        cfg.project.display().to_string(),
    );
    json!({ "mcpServers": {
        "emmet": { "type": "stdio", "command": emmet,
                   "args": ["--data-dir", data, "--project", project, "mcp", "--session", cfg.id] },
        "emmet_host": { "type": "stdio", "command": emmet,
                        "args": ["--data-dir", data, "--project", project, "mcp-host", "--session", cfg.id] },
    }})
}

impl AgentAdapter for Claude {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn detect(&self) -> Detection {
        detect::detect(AgentKind::Claude, &self.env, false)
    }

    fn start(&self, cfg: SessionConfig) -> Result<Box<dyn Session>> {
        cfg.check()?;
        let bin = self.bin()?;
        let dir = session_dir(&cfg)?;
        let mcp = dir.join("mcp.json");
        std::fs::write(&mcp, serde_json::to_vec_pretty(&mcp_config(&cfg)).expect("json"))
            .map_err(|e| AgentError::new(AgentErrorCode::Io, format!("writing {}: {e}", mcp.display())))?;
        let args = args(&mcp, cfg.resume.as_deref(), cfg.model.as_deref());
        let mut cmd = detect::command(&bin);
        cmd.args(&args).current_dir(&cfg.project);
        set_env(&mut cmd, &cfg.env);
        cmd.env("MCP_TOOL_TIMEOUT", MCP_TOOL_TIMEOUT);
        let meta = json!({ "cli": "claude", "args": args, "date": crate::time::rfc3339(crate::time::now_ms()) });
        let proto = ClaudeProto {
            session_id: cfg.resume.clone(),
            ..Default::default()
        };
        Ok(Box::new(Engine::spawn(
            AgentKind::Claude,
            &cfg,
            cmd,
            Box::new(proto),
            meta,
        )?))
    }

    fn usage(&self) -> Result<UsageSnapshot> {
        let bin = self.bin()?;
        let dir = self
            .data
            .as_ref()
            .map(|d| d.join("usage-probe"))
            .unwrap_or_else(|| std::env::temp_dir().join("emmet-usage-probe"));
        std::fs::create_dir_all(&dir).map_err(|e| AgentError::new(AgentErrorCode::Io, e.to_string()))?;
        Ok(probe(&bin, &dir, &self.env))
    }
}

/// The plan usage probe (D-36, ADR 0001): `/usage` sent to a short-lived `claude -p` in an empty folder, with no MCP
/// servers and no saved session. It spends no model tokens. The report arrives on the assistant message.
fn probe(bin: &std::path::Path, dir: &std::path::Path, env: &[(String, String)]) -> UsageSnapshot {
    let unavailable = |why: String| UsageSnapshot::without_meters(AgentKind::Claude, Billing::Unknown, Some(why));
    let mut cmd = detect::command(bin);
    cmd.args([
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--strict-mcp-config",
        "--no-session-persistence",
    ])
    .current_dir(dir)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null());
    set_env(&mut cmd, env);
    no_window(&mut cmd);
    let Ok(mut child) = cmd.spawn() else {
        return unavailable("Claude Code didn't start".into());
    };
    let mut stdin = child.stdin.take().expect("piped stdin");
    let line = json!({ "type": "user", "message": { "role": "user", "content": "/usage" } });
    let _ = writeln!(stdin, "{line}");
    let stdout = child.stdout.take().expect("piped stdout");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let (mut billing, mut report) = (Billing::Unknown, None);
        for line in BufReader::new(stdout).lines().map_while(std::io::Result::ok) {
            let Ok(m) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if m["type"] == "system" && m["subtype"] == "init" {
                billing = init_billing(&m);
            }
            if m.get("usage_report").is_some() {
                report = Some(m["usage_report"].clone());
            }
            if m["type"] == "result" {
                break;
            }
        }
        let _ = tx.send((billing, report));
    });
    let got = rx.recv_timeout(Duration::from_secs(30));
    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
    match got {
        Ok((billing @ Billing::ApiKey { .. }, _)) => UsageSnapshot::without_meters(AgentKind::Claude, billing, None),
        Ok((billing, Some(report))) => UsageSnapshot::from_usage_report(&report, billing),
        Ok((_, None)) => unavailable("Claude Code sent no usage report".into()),
        Err(_) => unavailable("Claude Code didn't answer /usage in 30 s".into()),
    }
}

/// Who pays, from `system/init`: `apiKeySource` "none" is the plan; otherwise it names the key's variable.
fn init_billing(init: &Value) -> Billing {
    match init["apiKeySource"].as_str() {
        Some("none") => Billing::Plan { name: None },
        Some(source) => Billing::ApiKey {
            source: Some(source.to_string()),
        },
        None => Billing::Unknown,
    }
}

/// stream-JSON (ADR 0001): `system/init`, partial `stream_event`s, `assistant` and `user` messages with tool use and
/// results, `result`, `rate_limit_event`, and `control_response`. Anything else is ignored.
#[derive(Default)]
struct ClaudeProto {
    session_id: Option<String>,
    started: bool,
    /// Messages whose text arrived as partials, so the whole message doesn't repeat it.
    streamed: HashSet<String>,
    interrupts: u64,
    interrupted: bool,
}

fn user_message(text: &str) -> Value {
    json!({ "type": "user", "message": { "role": "user", "content": text } })
}

/// A tool result's text, shortened for a card.
fn summary(content: &Value) -> String {
    let text = match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| b["text"].as_str().or(b["tool_name"].as_str()))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    };
    shorten(&text, 400)
}

pub(crate) fn shorten(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    format!("{cut}…")
}

impl Protocol for ClaudeProto {
    fn send(&mut self, text: &str, io: &mut Io) {
        self.interrupted = false;
        io.write(user_message(text));
    }

    /// 0B: a user message written mid-turn reaches the model at its next step, in the same turn.
    fn steer(&mut self, note: &UserNote, io: &mut Io) -> bool {
        io.write(user_message(&note.text));
        io.emit(AgentEvent::NoteDelivered {
            id: note.id.clone(),
            path: crate::SteerPath::Native,
        });
        true
    }

    fn interrupt(&mut self, io: &mut Io) {
        self.interrupts += 1;
        self.interrupted = true;
        io.write(json!({
            "type": "control_request",
            "request_id": format!("int-{}", self.interrupts),
            "request": { "subtype": "interrupt" },
        }));
    }

    fn respond(&mut self, _id: &str, _d: Decision, _io: &mut Io) -> bool {
        false
    }

    fn on_message(&mut self, m: Value, io: &mut Io) {
        match m["type"].as_str().unwrap_or("") {
            "system" if m["subtype"] == "init" => {
                if let Some(id) = m["session_id"].as_str() {
                    self.session_id = Some(id.to_string());
                }
                // Claude Code repeats init at every turn; the chat needs it once.
                if !self.started {
                    self.started = true;
                    io.emit(AgentEvent::Started {
                        resume: self.session_id.clone().unwrap_or_default(),
                        model: m["model"].as_str().map(String::from),
                        billing: Some(init_billing(&m)),
                    });
                }
            }
            "stream_event" => {
                let ev = &m["event"];
                if ev["type"] != "content_block_delta" {
                    return;
                }
                let message = m["api_message_id"].as_str().unwrap_or("").to_string();
                let delta = &ev["delta"];
                // Thinking can arrive as a signature with no text; an empty delta shows nothing.
                if delta["text"]
                    .as_str()
                    .or(delta["thinking"].as_str())
                    .is_none_or(str::is_empty)
                {
                    return;
                }
                match delta["type"].as_str() {
                    Some("text_delta") => {
                        self.streamed.insert(message.clone());
                        io.emit(AgentEvent::Text {
                            message,
                            delta: delta["text"].as_str().unwrap_or("").into(),
                        });
                    }
                    Some("thinking_delta") => {
                        self.streamed.insert(message.clone());
                        io.emit(AgentEvent::Reasoning {
                            message,
                            delta: delta["thinking"].as_str().unwrap_or("").into(),
                        });
                    }
                    _ => {}
                }
            }
            "assistant" => {
                let msg = &m["message"];
                let id = msg["id"].as_str().unwrap_or("").to_string();
                for block in msg["content"].as_array().into_iter().flatten() {
                    match block["type"].as_str() {
                        Some("text")
                            if !self.streamed.contains(&id)
                                && block["text"].as_str().is_some_and(|t| !t.is_empty()) =>
                        {
                            io.emit(AgentEvent::Text {
                                message: id.clone(),
                                delta: block["text"].as_str().unwrap_or("").into(),
                            })
                        }
                        Some("tool_use") => io.emit(AgentEvent::ToolCall {
                            id: block["id"].as_str().unwrap_or("").into(),
                            name: block["name"].as_str().unwrap_or("").into(),
                            input: block["input"].clone(),
                        }),
                        _ => {}
                    }
                }
            }
            "user" => {
                for block in m["message"]["content"].as_array().into_iter().flatten() {
                    if block["type"] == "tool_result" {
                        io.emit(AgentEvent::ToolResult {
                            id: block["tool_use_id"].as_str().unwrap_or("").into(),
                            ok: block["is_error"] != true,
                            summary: summary(&block["content"]),
                        });
                    }
                }
            }
            "result" => {
                let u = &m["usage"];
                let n = |k: &str| u[k].as_u64().unwrap_or(0);
                io.emit(AgentEvent::Usage {
                    input: n("input_tokens") + n("cache_creation_input_tokens") + n("cache_read_input_tokens"),
                    output: n("output_tokens"),
                });
                let reason = if self.interrupted {
                    DoneReason::Interrupted
                } else if m["subtype"] == "success" && m["is_error"] != true {
                    DoneReason::Completed
                } else {
                    let said = m["result"]
                        .as_str()
                        .or(m["errors"][0].as_str())
                        .unwrap_or("Claude Code ended the turn with an error.");
                    io.emit(AgentEvent::Error {
                        code: AgentErrorCode::Agent,
                        message: said.to_string(),
                    });
                    DoneReason::Failed
                };
                self.interrupted = false;
                io.emit(AgentEvent::TurnDone { reason });
            }
            "rate_limit_event" => io.emit(AgentEvent::Limits {
                snapshot: UsageSnapshot::from_rate_limit(&m["rate_limit_info"]),
            }),
            // control_response, system/status, tool_progress heartbeats, thinking token counts, and anything new:
            // kept in the transcript, ignored here.
            _ => {}
        }
    }

    fn resume_token(&self) -> Option<String> {
        self.session_id.clone()
    }
}
