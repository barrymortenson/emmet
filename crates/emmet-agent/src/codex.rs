// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Codex adapter (plan section 10, ADR 0001): `codex app-server` over stdio JSON-RPC (no `jsonrpc` field), pinned
//! to the user's API key from the keychain through Emmet's provider block and never on the CLI's own login (D-35).
//! Approvals are the app-server's own server requests; Emmet accepts the MCP tool-call elicitations for its two
//! servers. This module is the arguments, the encoding, and the mapping; the types it sends live in `types`.

pub mod types;

use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};

use crate::claude::shorten;
use crate::session::{Engine, Io, Protocol, session_dir, set_env};
use crate::{
    AgentAdapter, AgentError, AgentErrorCode, AgentEvent, AgentKind, ApprovalKind, Billing, Decision, Detection,
    DoneReason, Result, Session, SessionConfig, SteerPath, UsageSnapshot, UserNote, detect, keychain,
};
use types::*;

/// Emmet's two MCP servers, whose tool-call approvals Emmet accepts itself.
const OURS: &[&str] = &["emmet", "emmet_host"];
/// The variable the provider block reads the key from.
pub const KEY_VAR: &str = "OPENAI_API_KEY";

pub struct Codex {
    env: Vec<(String, String)>,
}

impl Codex {
    pub fn new(env: Vec<(String, String)>) -> Self {
        Self { env }
    }
}

fn billing() -> Billing {
    Billing::ApiKey {
        source: Some("keychain".into()),
    }
}

/// A TOML basic string, for a `-c key=value` override.
fn toml(s: &str) -> String {
    let mut out = String::from('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn toml_list(items: &[String]) -> String {
    format!("[{}]", items.iter().map(|s| toml(s)).collect::<Vec<_>>().join(","))
}

/// The app-server's arguments (plan section 10): the API-key provider, Emmet's two MCP servers for this session,
/// `emmet_host`'s raised tool timeout, and the sandbox. `--strict-config` rejects any key Codex doesn't know.
pub fn args(cfg: &SessionConfig) -> Vec<String> {
    let (emmet, data, project) = (
        cfg.emmet.display().to_string(),
        cfg.data.display().to_string(),
        cfg.project.display().to_string(),
    );
    let server = |sub: &str| {
        toml_list(&[
            "--data-dir".into(),
            data.clone(),
            "--project".into(),
            project.clone(),
            sub.into(),
            "--session".into(),
            cfg.id.clone(),
        ])
    };
    let overrides = [
        "model_provider=\"emmet_api_key\"".to_string(),
        "model_providers.emmet_api_key.name=\"OpenAI API key\"".into(),
        "model_providers.emmet_api_key.base_url=\"https://api.openai.com/v1\"".into(),
        format!("model_providers.emmet_api_key.env_key=\"{KEY_VAR}\""),
        "model_providers.emmet_api_key.wire_api=\"responses\"".into(),
        "model_providers.emmet_api_key.requires_openai_auth=false".into(),
        format!("mcp_servers.emmet.command={}", toml(&emmet)),
        format!("mcp_servers.emmet.args={}", server("mcp")),
        format!("mcp_servers.emmet_host.command={}", toml(&emmet)),
        format!("mcp_servers.emmet_host.args={}", server("mcp-host")),
        "mcp_servers.emmet_host.tool_timeout_sec=3600".into(),
        "sandbox_mode=\"workspace-write\"".into(),
        "approval_policy=\"on-request\"".into(),
        "sandbox_workspace_write.exclude_slash_tmp=true".into(),
        "sandbox_workspace_write.exclude_tmpdir_env_var=true".into(),
    ];
    let mut a: Vec<String> = ["app-server", "--listen", "stdio://", "--strict-config"]
        .map(String::from)
        .to_vec();
    for o in overrides {
        a.push("-c".into());
        a.push(o);
    }
    a
}

impl AgentAdapter for Codex {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn detect(&self) -> Detection {
        let has_key = keychain::has(&self.env).unwrap_or(false);
        detect::detect(AgentKind::Codex, &self.env, has_key)
    }

    fn start(&self, cfg: SessionConfig) -> Result<Box<dyn Session>> {
        cfg.check()?;
        let bin = detect::find(AgentKind::Codex, &self.env)
            .ok_or_else(|| AgentError::new(AgentErrorCode::NotInstalled, "Codex isn't installed."))?;
        let key = keychain::get(&self.env)?.ok_or_else(|| {
            AgentError::new(
                AgentErrorCode::NotSignedIn,
                "Add your OpenAI API key in Settings → Agents.",
            )
        })?;
        session_dir(&cfg)?;
        let args = args(&cfg);
        let mut cmd = detect::command(&bin);
        cmd.args(&args).current_dir(&cfg.project);
        set_env(&mut cmd, &cfg.env);
        cmd.env(KEY_VAR, key);
        let meta = json!({ "cli": "codex", "args": args, "date": crate::time::rfc3339(crate::time::now_ms()) });
        let proto = CodexProto {
            cwd: cfg.project.display().to_string(),
            resume: cfg.resume.clone(),
            model: cfg.model.clone(),
            ..Default::default()
        };
        Ok(Box::new(Engine::spawn(
            AgentKind::Codex,
            &cfg,
            cmd,
            Box::new(proto),
            meta,
        )?))
    }

    /// OpenAI documents no usage source for an API key, so Codex reports its billing state only (D-36).
    fn usage(&self) -> Result<UsageSnapshot> {
        Ok(UsageSnapshot::without_meters(AgentKind::Codex, billing(), None))
    }
}

/// What a request of ours was, so its response can be read.
enum Sent {
    Initialize,
    Account,
    Thread,
    Turn,
    Steer(UserNote),
    Interrupt,
}

#[derive(Default)]
struct CodexProto {
    cwd: String,
    resume: Option<String>,
    model: Option<String>,
    next: u64,
    sent: HashMap<u64, Sent>,
    thread: Option<String>,
    turn: Option<String>,
    /// Turns sent before the thread exists.
    waiting: Vec<String>,
    /// Stop before turn/start answered: interrupt as soon as the turn has an id.
    interrupt_when_known: bool,
    /// Open approval requests by Emmet's id: (the request's JSON-RPC id, its method, its params).
    approvals: HashMap<String, (Value, String, Value)>,
    /// File changes by item id, to describe a file-change approval.
    changes: HashMap<String, Vec<String>>,
    streamed: HashSet<String>,
    usage: (u64, u64),
    failed: bool,
}

impl CodexProto {
    fn request(&mut self, method: &str, params: Value, what: Sent, io: &mut Io) {
        self.next += 1;
        self.sent.insert(self.next, what);
        io.write(json!({ "id": self.next, "method": method, "params": params }));
    }

    fn start_turn(&mut self, text: &str, io: &mut Io) {
        let Some(thread) = self.thread.clone() else {
            self.waiting.push(text.to_string());
            return;
        };
        self.usage = (0, 0);
        self.failed = false;
        let p = TurnStartParams {
            thread_id: thread,
            input: vec![UserInput::Text { text: text.into() }],
        };
        self.request("turn/start", to(&p), Sent::Turn, io);
    }

    fn interrupt_turn(&mut self, io: &mut Io) {
        if let (Some(thread), Some(turn)) = (self.thread.clone(), self.turn.clone()) {
            let p = TurnInterruptParams {
                thread_id: thread,
                turn_id: turn,
            };
            self.request("turn/interrupt", to(&p), Sent::Interrupt, io);
        }
    }

    fn on_response(&mut self, id: u64, m: &Value, io: &mut Io) {
        let Some(what) = self.sent.remove(&id) else { return };
        let error = m.get("error").filter(|e| !e.is_null());
        let r = &m["result"];
        match (what, error) {
            (Sent::Steer(note), Some(_)) => io.refused.push(note),
            (Sent::Steer(note), None) => io.emit(AgentEvent::NoteDelivered {
                id: note.id,
                path: SteerPath::Native,
            }),
            (Sent::Turn, Some(e)) => {
                io.emit(agent_error(
                    e["message"].as_str().unwrap_or("Codex refused the turn."),
                    &Value::Null,
                ));
                io.emit(AgentEvent::TurnDone {
                    reason: DoneReason::Failed,
                });
            }
            (Sent::Interrupt, Some(_)) => {}
            (_, Some(e)) => io.emit(AgentEvent::Error {
                code: AgentErrorCode::Agent,
                message: format!(
                    "Codex didn't start the session: {}",
                    e["message"].as_str().unwrap_or("no reason given")
                ),
            }),
            (Sent::Initialize, None) => {
                io.write(json!({ "method": "initialized" }));
                self.request("account/read", json!({}), Sent::Account, io);
            }
            (Sent::Account, None) => {
                if r["requiresOpenaiAuth"] == true {
                    io.emit(AgentEvent::Error {
                        code: AgentErrorCode::NotSignedIn,
                        message: "Codex asked for its own sign-in, which Emmet doesn't use; check the API key in \
                                  Settings → Agents."
                            .into(),
                    });
                }
                let instructions = crate::APP_POINTER.to_string();
                match self.resume.clone() {
                    Some(thread_id) => {
                        let p = ThreadResumeParams {
                            thread_id,
                            cwd: self.cwd.clone(),
                            developer_instructions: instructions,
                            model: self.model.clone(),
                        };
                        self.request("thread/resume", to(&p), Sent::Thread, io);
                    }
                    None => {
                        let p = ThreadStartParams {
                            cwd: self.cwd.clone(),
                            developer_instructions: instructions,
                            model: self.model.clone(),
                        };
                        self.request("thread/start", to(&p), Sent::Thread, io);
                    }
                }
            }
            (Sent::Thread, None) => {
                let thread = r["thread"]["id"].as_str().unwrap_or_default().to_string();
                self.thread = Some(thread.clone());
                io.emit(AgentEvent::Started {
                    resume: thread,
                    model: r["model"].as_str().map(String::from),
                    billing: Some(billing()),
                });
                for text in std::mem::take(&mut self.waiting) {
                    self.start_turn(&text, io);
                }
            }
            (Sent::Turn, None) => {
                // 0B: a turn/start mid-turn folds into the running turn and returns its id.
                if let Some(t) = r["turn"]["id"].as_str() {
                    self.turn = Some(t.to_string());
                }
                if std::mem::take(&mut self.interrupt_when_known) {
                    self.interrupt_turn(io);
                }
            }
            (Sent::Interrupt, None) => {}
        }
    }

    fn on_server_request(&mut self, id: Value, method: &str, p: &Value, io: &mut Io) {
        let key = format!("r{}", id.as_u64().map_or_else(|| id.to_string(), |n| n.to_string()));
        let card = |kind, tool: &str, detail: String| AgentEvent::Approval {
            id: key.clone(),
            kind,
            tool: tool.into(),
            detail,
        };
        let reason = p["reason"].as_str().map(|r| format!(" ({r})")).unwrap_or_default();
        match method {
            "mcpServer/elicitation/request" => {
                let ours = p["_meta"]["codex_approval_kind"] == "mcp_tool_call"
                    && p["serverName"].as_str().is_some_and(|s| OURS.contains(&s));
                // Other servers' tools come with 8F (ask the first time per tool); until then they're declined.
                let r = if ours {
                    ElicitationResponse {
                        action: "accept",
                        content: json!({}),
                    }
                } else {
                    ElicitationResponse {
                        action: "decline",
                        content: Value::Null,
                    }
                };
                io.write(json!({ "id": id, "result": to(&r) }));
            }
            "item/commandExecution/requestApproval" => {
                let command = p["command"].as_str().unwrap_or("a command");
                io.emit(card(
                    ApprovalKind::Command,
                    "commandExecution",
                    format!("{command}{reason}"),
                ));
                self.approvals.insert(key.clone(), (id, method.into(), p.clone()));
            }
            "item/fileChange/requestApproval" => {
                let item = p["itemId"].as_str().unwrap_or_default();
                let files = self
                    .changes
                    .get(item)
                    .map(|f| f.join(", "))
                    .unwrap_or_else(|| "files".into());
                io.emit(card(ApprovalKind::Edit, "fileChange", format!("{files}{reason}")));
                self.approvals.insert(key.clone(), (id, method.into(), p.clone()));
            }
            "item/permissions/requestApproval" => {
                let what = serde_json::to_string(&p["permissions"]).unwrap_or_default();
                io.emit(card(ApprovalKind::Permission, "permissions", format!("{what}{reason}")));
                self.approvals.insert(key.clone(), (id, method.into(), p.clone()));
            }
            _ => io.write(
                json!({ "id": id, "error": { "code": -32601, "message": format!("Emmet doesn't handle {method}") } }),
            ),
        }
    }

    fn on_notification(&mut self, method: &str, p: &Value, io: &mut Io) {
        match method {
            "turn/started" => {
                if let Some(t) = p["turn"]["id"].as_str() {
                    self.turn = Some(t.to_string());
                }
                if std::mem::take(&mut self.interrupt_when_known) {
                    self.interrupt_turn(io);
                }
            }
            "item/agentMessage/delta" | "item/reasoning/summaryTextDelta" => {
                let message = p["itemId"].as_str().unwrap_or_default().to_string();
                let delta = p["delta"].as_str().unwrap_or_default().to_string();
                self.streamed.insert(message.clone());
                if delta.is_empty() {
                    return;
                }
                io.emit(if method.starts_with("item/agent") {
                    AgentEvent::Text { message, delta }
                } else {
                    AgentEvent::Reasoning { message, delta }
                });
            }
            "item/started" => {
                let item = &p["item"];
                let id = item["id"].as_str().unwrap_or_default().to_string();
                let call = |name: String, input: Value| AgentEvent::ToolCall {
                    id: id.clone(),
                    name,
                    input,
                };
                match item["type"].as_str().unwrap_or("") {
                    "mcpToolCall" => io.emit(call(
                        format!(
                            "mcp__{}__{}",
                            item["server"].as_str().unwrap_or(""),
                            item["tool"].as_str().unwrap_or("")
                        ),
                        item["arguments"].clone(),
                    )),
                    "commandExecution" => io.emit(call("shell".into(), json!({ "command": item["command"] }))),
                    "fileChange" => {
                        let root = format!("{}/", self.cwd.trim_end_matches(['/', '\\']));
                        let files: Vec<String> = item["changes"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(|c| c["path"].as_str())
                            .map(|p| p.replace('\\', "/"))
                            .map(|p| p.strip_prefix(&root.replace('\\', "/")).map(String::from).unwrap_or(p))
                            .collect();
                        self.changes.insert(id.clone(), files.clone());
                        io.emit(call("apply_patch".into(), json!({ "files": files })));
                    }
                    _ => {}
                }
            }
            "item/completed" => {
                let item = &p["item"];
                let id = item["id"].as_str().unwrap_or_default().to_string();
                let status = item["status"].as_str().unwrap_or("");
                match item["type"].as_str().unwrap_or("") {
                    "mcpToolCall" => {
                        let ok = status == "completed" && item["error"].is_null();
                        let text = if ok {
                            item["result"]["content"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(|b| b["text"].as_str())
                                .collect::<Vec<_>>()
                                .join("\n")
                        } else {
                            item["error"]["message"]
                                .as_str()
                                .unwrap_or("the tool call failed")
                                .to_string()
                        };
                        io.emit(AgentEvent::ToolResult {
                            id,
                            ok,
                            summary: shorten(&text, 400),
                        });
                    }
                    "commandExecution" => io.emit(AgentEvent::ToolResult {
                        id,
                        ok: status == "completed" && item["exitCode"].as_i64() == Some(0),
                        summary: shorten(item["aggregatedOutput"].as_str().unwrap_or(""), 400),
                    }),
                    "fileChange" => {
                        let summary = self.changes.remove(&id).unwrap_or_default().join(", ");
                        io.emit(AgentEvent::ToolResult {
                            id,
                            ok: status == "completed",
                            summary,
                        });
                    }
                    "agentMessage"
                        if !self.streamed.contains(&id) && item["text"].as_str().is_some_and(|t| !t.is_empty()) =>
                    {
                        io.emit(AgentEvent::Text {
                            message: id,
                            delta: item["text"].as_str().unwrap_or("").into(),
                        })
                    }
                    _ => {}
                }
            }
            "thread/tokenUsage/updated" => {
                let last = &p["tokenUsage"]["last"];
                self.usage.0 += last["inputTokens"].as_u64().unwrap_or(0);
                self.usage.1 += last["outputTokens"].as_u64().unwrap_or(0);
            }
            "error" if p["willRetry"] != true => {
                self.failed = true;
                let e = &p["error"];
                io.emit(agent_error(
                    e["message"].as_str().unwrap_or("Codex reported an error."),
                    &e["codexErrorInfo"],
                ));
            }
            "serverRequest/resolved" => {
                let key = format!("r{}", p["requestId"]);
                if self.approvals.remove(&key).is_some() {
                    io.emit(AgentEvent::ApprovalClosed { id: key });
                }
            }
            "turn/completed" => {
                let turn = &p["turn"];
                let reason = match turn["status"].as_str() {
                    Some("completed") => DoneReason::Completed,
                    Some("interrupted") => DoneReason::Interrupted,
                    _ => {
                        if !self.failed {
                            let message = turn["error"]["message"]
                                .as_str()
                                .unwrap_or("Codex ended the turn with an error.");
                            io.emit(agent_error(message, &Value::Null));
                        }
                        DoneReason::Failed
                    }
                };
                self.turn = None;
                self.failed = false;
                io.emit(AgentEvent::Usage {
                    input: self.usage.0,
                    output: self.usage.1,
                });
                io.emit(AgentEvent::TurnDone { reason });
            }
            // Rate limits (all null on a key), thread status, MCP startup, deltas of command output, and anything
            // new: kept in the transcript, ignored here.
            _ => {}
        }
    }
}

/// An error notification as an event, with its code: a refused key, no credit, or anything else.
fn agent_error(message: &str, info: &Value) -> AgentEvent {
    let code = if info == "usageLimitExceeded" || message.contains("Quota exceeded") {
        AgentErrorCode::QuotaExceeded
    } else if info.to_string().contains("401") || message.contains("401 Unauthorized") {
        AgentErrorCode::AuthFailed
    } else {
        AgentErrorCode::Agent
    };
    AgentEvent::Error {
        code,
        message: message.to_string(),
    }
}

fn to(v: &impl serde::Serialize) -> Value {
    serde_json::to_value(v).expect("serializes")
}

impl Protocol for CodexProto {
    fn start(&mut self, io: &mut Io) {
        let p = InitializeParams {
            client_info: ClientInfo {
                name: "emmet".into(),
                title: "Emmet".into(),
                version: emmet_core::version().into(),
            },
        };
        self.request("initialize", to(&p), Sent::Initialize, io);
    }

    fn send(&mut self, text: &str, io: &mut Io) {
        self.start_turn(text, io);
    }

    /// 0B: turn/steer reaches the model at its next step, in the same turn.
    fn steer(&mut self, note: &UserNote, io: &mut Io) -> bool {
        let (Some(thread), Some(turn)) = (self.thread.clone(), self.turn.clone()) else {
            return false;
        };
        let p = TurnSteerParams {
            thread_id: thread,
            expected_turn_id: turn,
            input: vec![UserInput::Text {
                text: note.text.clone(),
            }],
        };
        self.request("turn/steer", to(&p), Sent::Steer(note.clone()), io);
        true
    }

    fn interrupt(&mut self, io: &mut Io) {
        if self.turn.is_some() {
            self.interrupt_turn(io);
        } else {
            self.interrupt_when_known = true;
        }
    }

    fn respond(&mut self, id: &str, d: Decision, io: &mut Io) -> bool {
        let Some((rid, method, params)) = self.approvals.remove(id) else {
            return false;
        };
        let result = if method == "item/permissions/requestApproval" {
            to(&match d {
                Decision::Deny => PermissionsResponse {
                    permissions: json!({}),
                    scope: None,
                },
                Decision::AllowOnce => PermissionsResponse {
                    permissions: params["permissions"].clone(),
                    scope: Some("turn"),
                },
                Decision::AllowSession => PermissionsResponse {
                    permissions: params["permissions"].clone(),
                    scope: Some("session"),
                },
            })
        } else {
            to(&ApprovalResponse {
                decision: match d {
                    Decision::AllowOnce => "accept",
                    Decision::AllowSession => "acceptForSession",
                    Decision::Deny => "decline",
                },
            })
        };
        io.write(json!({ "id": rid, "result": result }));
        true
    }

    fn on_message(&mut self, m: Value, io: &mut Io) {
        let method = m["method"].as_str().map(String::from);
        match (method, m.get("id").cloned()) {
            (Some(method), Some(id)) => self.on_server_request(id, &method, &m["params"], io),
            (Some(method), None) => self.on_notification(&method, &m["params"], io),
            (None, Some(id)) => {
                if let Some(id) = id.as_u64() {
                    self.on_response(id, &m, io);
                }
            }
            (None, None) => {}
        }
    }

    fn resume_token(&self) -> Option<String> {
        self.thread.clone().or_else(|| self.resume.clone())
    }
}
