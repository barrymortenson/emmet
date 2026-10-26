// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Codex adapter (plan section 10, ADR 0001): `codex app-server` over stdio JSON-RPC (no `jsonrpc` field), pinned
//! through Emmet's provider block to the user's ChatGPT plan (Sign in with ChatGPT, 1E, ADR 0011) or to their API key
//! from the keychain, and never on the CLI's own login (D-35). Approvals are the app-server's own server requests;
//! Emmet accepts the MCP tool-call elicitations for its two servers. This module is the arguments, the encoding, the
//! mapping, and the plan session's token renewal; the types it sends live in `types`.

pub mod types;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use serde_json::{Value, json};

use crate::chatgpt::{AGENT_NAME, ChatGpt, CodexUse};
use crate::claude::shorten;
use crate::session::{Engine, Io, Protocol, session_dir, set_env};
use crate::{
    AgentAdapter, AgentError, AgentErrorCode, AgentEvent, AgentKind, Answer, ApprovalKind, Billing, Decision,
    Detection, DoneReason, EventStream, Result, Session, SessionConfig, SteerPath, UsageSnapshot, UserNote, UserTurn,
    detect, keychain,
};
use types::*;

/// Emmet's two MCP servers, whose tool-call approvals Emmet accepts itself.
const OURS: &[&str] = &["emmet", "emmet_host"];
/// The variable the API-key provider reads the key from.
pub const KEY_VAR: &str = "OPENAI_API_KEY";
/// The variable OpenAI's ChatGPT plan provider reads the access token from.
pub const TOKEN_VAR: &str = "ACCESS_TOKEN";
/// A plan session renews its token before a turn that would start with less than this left, so a turn always has at
/// least this long before the token expires (access tokens last an hour).
pub const RENEW_BEFORE: Duration = Duration::from_secs(30 * 60);

pub struct Codex {
    env: Vec<(String, String)>,
    data: PathBuf,
}

impl Codex {
    /// `data` is the app data dir, where the ChatGPT accounts are.
    pub fn new(env: Vec<(String, String)>, data: &Path) -> Self {
        Self {
            env,
            data: data.to_path_buf(),
        }
    }

    fn chatgpt(&self) -> ChatGpt {
        ChatGpt::new(&self.data, &self.env)
    }
}

fn key_billing() -> Billing {
    Billing::ApiKey {
        source: Some("keychain".into()),
    }
}

/// Codex on a ChatGPT plan: "Codex · ChatGPT plan".
pub fn plan_billing() -> Billing {
    Billing::Plan {
        name: Some("ChatGPT".into()),
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

/// The app-server's arguments (plan section 10): the provider (OpenAI's ChatGPT plan block, word for word, or the API
/// key's), Emmet's two MCP servers for this session, `emmet_host`'s raised tool timeout, and the sandbox.
/// `--strict-config` rejects any key Codex doesn't know.
pub fn args(cfg: &SessionConfig, plan: bool) -> Vec<String> {
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
    let provider: Vec<String> = if plan {
        vec![
            "model_provider=\"openai_chatgpt_plan\"".into(),
            "model_providers.openai_chatgpt_plan.name=\"ChatGPT plan\"".into(),
            "model_providers.openai_chatgpt_plan.base_url=\"https://api.openai.com/v1\"".into(),
            format!("model_providers.openai_chatgpt_plan.env_key=\"{TOKEN_VAR}\""),
            "model_providers.openai_chatgpt_plan.wire_api=\"responses\"".into(),
            "model_providers.openai_chatgpt_plan.requires_openai_auth=false".into(),
            "model_providers.openai_chatgpt_plan.supports_websockets=false".into(),
        ]
    } else {
        vec![
            "model_provider=\"emmet_api_key\"".into(),
            "model_providers.emmet_api_key.name=\"OpenAI API key\"".into(),
            "model_providers.emmet_api_key.base_url=\"https://api.openai.com/v1\"".into(),
            format!("model_providers.emmet_api_key.env_key=\"{KEY_VAR}\""),
            "model_providers.emmet_api_key.wire_api=\"responses\"".into(),
            "model_providers.emmet_api_key.requires_openai_auth=false".into(),
        ]
    };
    let overrides = [
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
    for o in provider.into_iter().chain(overrides) {
        a.push("-c".into());
        a.push(o);
    }
    a
}

/// Starts one app-server process on `secret` (the API key, or the plan's access token).
fn spawn(cfg: &SessionConfig, bin: &Path, plan: bool, secret: &str) -> Result<Engine> {
    session_dir(cfg)?;
    let args = args(cfg, plan);
    let mut cmd = detect::command(bin);
    cmd.args(&args).current_dir(&cfg.project);
    set_env(&mut cmd, &cfg.env);
    cmd.env(if plan { TOKEN_VAR } else { KEY_VAR }, secret);
    let meta = json!({ "cli": "codex", "args": args, "date": crate::time::rfc3339(crate::time::now_ms()) });
    let proto = CodexProto {
        cwd: cfg.project.display().to_string(),
        resume: cfg.resume.clone(),
        model: cfg.model.clone(),
        plan,
        ..Default::default()
    };
    Engine::spawn(
        AgentKind::Codex,
        cfg,
        cmd,
        Box::new(proto),
        meta,
        vec![secret.to_string()],
    )
}

impl AgentAdapter for Codex {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn detect(&self) -> Detection {
        let has_key = keychain::has(&self.env).unwrap_or(false);
        let mut d = detect::detect(AgentKind::Codex, &self.env, has_key);
        if d.path.is_none() {
            return d;
        }
        match self.chatgpt().codex_use() {
            CodexUse::Plan { .. } => {
                d.signed_in = true;
                d.billing = plan_billing();
                d.fix = None;
            }
            CodexUse::PlanOff { fix, .. } => {
                d.signed_in = false;
                d.billing = plan_billing();
                d.fix = Some(fix);
            }
            CodexUse::ApiKey => {}
        }
        d
    }

    fn start(&self, cfg: SessionConfig) -> Result<Box<dyn Session>> {
        cfg.check()?;
        let bin = detect::find(AgentKind::Codex, &self.env)
            .ok_or_else(|| AgentError::new(AgentErrorCode::NotInstalled, "Codex isn't installed."))?;
        let plan = match &cfg.openai_key {
            Some(_) => None,
            None => match self.chatgpt().codex_use() {
                CodexUse::Plan { id, .. } => Some(id),
                CodexUse::PlanOff { fix, .. } => return Err(AgentError::new(AgentErrorCode::NotSignedIn, fix)),
                CodexUse::ApiKey => None,
            },
        };
        if let Some(id) = plan {
            let chatgpt = self.chatgpt();
            let token = chatgpt.access_token(&id, None, RENEW_BEFORE)?;
            return Ok(Box::new(PlanSession::start(cfg, bin, chatgpt, id, token)?));
        }
        let key = match cfg.openai_key.clone() {
            Some(k) => Some(k),
            None => keychain::get(&self.env)?,
        };
        let key = key.ok_or_else(|| {
            AgentError::new(
                AgentErrorCode::NotSignedIn,
                "Continue with ChatGPT, or add your OpenAI API key, in Settings → Agents.",
            )
        })?;
        Ok(Box::new(spawn(&cfg, &bin, false, &key)?))
    }

    /// OpenAI documents no usage source for an API key or a ChatGPT plan, so Codex reports its billing state only
    /// (D-36); on a plan the app shows "Using ChatGPT plan" with Manage usage.
    fn usage(&self) -> Result<UsageSnapshot> {
        let billing = match self.chatgpt().codex_use() {
            CodexUse::ApiKey => key_billing(),
            _ => plan_billing(),
        };
        Ok(UsageSnapshot::without_meters(AgentKind::Codex, billing, None))
    }
}

/// A Codex session on the user's ChatGPT plan. OpenAI's rule for the app-server: when the token renews, restart it
/// with the new `ACCESS_TOKEN`, initialize, and `thread/resume` the saved thread. Emmet renews only between turns:
/// before one that would start with less than [`RENEW_BEFORE`] left, or after the API refused the token. The chat sees
/// one session throughout: the old process's exit is never passed on.
struct PlanSession {
    cfg: SessionConfig,
    bin: PathBuf,
    chatgpt: ChatGpt,
    id: String,
    token: String,
    engine: Engine,
    /// Which engine is current: a retired engine's events stop at its exit.
    generation: Arc<AtomicU64>,
    /// The API refused the token (a 401): renew before the next turn.
    refused: Arc<AtomicBool>,
    tx: Sender<crate::AgentEvent>,
    events: Option<EventStream>,
    /// Signaled when the current engine's events end.
    ended: Option<Receiver<()>>,
}

impl PlanSession {
    fn start(cfg: SessionConfig, bin: PathBuf, chatgpt: ChatGpt, id: String, token: String) -> Result<Self> {
        let engine = spawn(&cfg, &bin, true, &token)?;
        let (tx, rx) = channel();
        let mut s = Self {
            cfg,
            bin,
            chatgpt,
            id,
            token,
            engine,
            generation: Arc::new(AtomicU64::new(0)),
            refused: Arc::new(AtomicBool::new(false)),
            tx,
            events: Some(rx),
            ended: None,
        };
        s.forward();
        Ok(s)
    }

    /// Passes the current engine's events on until it exits, or until it's retired.
    fn forward(&mut self) {
        let rx = self.engine.events();
        let (tx, generation, refused) = (self.tx.clone(), self.generation.clone(), self.refused.clone());
        let mine = generation.load(Ordering::SeqCst);
        let (done, ended) = channel();
        self.ended = Some(ended);
        std::thread::spawn(move || {
            for ev in rx {
                if generation.load(Ordering::SeqCst) != mine {
                    if matches!(ev, AgentEvent::Exited { .. }) {
                        break;
                    }
                    continue;
                }
                if matches!(
                    ev,
                    AgentEvent::Error {
                        code: AgentErrorCode::AuthFailed,
                        ..
                    }
                ) {
                    refused.store(true, Ordering::SeqCst);
                }
                if tx.send(ev).is_err() {
                    break;
                }
            }
            let _ = done.send(());
        });
    }

    /// Before a turn: a fresh enough token, and if it changed, a new app-server on it, resuming the thread.
    fn renew(&mut self) -> Result<()> {
        let refused = self.refused.swap(false, Ordering::SeqCst);
        let token = self
            .chatgpt
            .access_token(&self.id, refused.then_some(self.token.as_str()), RENEW_BEFORE)?;
        if token == self.token {
            return Ok(());
        }
        let thread = self.engine.resume_token();
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.engine.close();
        if let Some(ended) = self.ended.take() {
            let _ = ended.recv_timeout(Duration::from_secs(5));
        }
        let mut cfg = self.cfg.clone();
        cfg.resume = thread.or(cfg.resume);
        self.engine = spawn(&cfg, &self.bin, true, &token)?;
        self.cfg = cfg;
        self.token = token;
        self.forward();
        Ok(())
    }
}

impl Session for PlanSession {
    fn send(&mut self, turn: UserTurn) -> Result<()> {
        if !self.engine.running() {
            self.renew()?;
        }
        self.engine.send(turn)
    }

    fn respond(&mut self, approval: &str, decision: Decision) -> Result<()> {
        self.engine.respond(approval, decision)
    }

    fn answer(&mut self, question: &str, answer: Answer) -> Result<()> {
        self.engine.answer(question, answer)
    }

    fn interrupt(&mut self) -> Result<()> {
        self.engine.interrupt()
    }

    fn steer(&mut self, text: &str) -> Result<UserNote> {
        // With no turn running, a redirect starts one.
        if !self.engine.running() {
            self.renew()?;
        }
        self.engine.steer(text)
    }

    fn cancel_note(&mut self, id: &str) -> bool {
        self.engine.cancel_note(id)
    }

    fn edit_note(&mut self, id: &str, text: &str) -> bool {
        self.engine.edit_note(id, text)
    }

    fn events(&mut self) -> EventStream {
        self.events.take().unwrap_or_else(|| channel().1)
    }

    fn resume_token(&self) -> Option<String> {
        self.engine.resume_token()
    }

    fn close(&mut self) {
        self.engine.close();
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
    /// On the ChatGPT plan provider: its errors get OpenAI's recovery, and its billing says so.
    plan: bool,
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
                io.emit(agent_error(e, "Codex refused the turn.", self.plan));
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
                    billing: Some(if self.plan { plan_billing() } else { key_billing() }),
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
                io.emit(agent_error(&p["error"], "Codex reported an error.", self.plan));
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
                            io.emit(agent_error(
                                &turn["error"],
                                "Codex ended the turn with an error.",
                                self.plan,
                            ));
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

/// OpenAI's ChatGPT plan errors (its errors-and-recovery page), with the recovery each one names. The codes reach
/// Codex's error text from the Responses API; a direct-admission failure has only its HTTP status.
const PLAN_ERRORS: &[(&str, AgentErrorCode, &str)] = &[
    (
        "subscription_sharing_usage_limit_exceeded",
        AgentErrorCode::QuotaExceeded,
        LIMIT,
    ),
    (
        "subscription_sharing_user_not_eligible",
        AgentErrorCode::Agent,
        "ChatGPT plan use isn't available for this account, workspace, or policy. To keep going, use an OpenAI API \
         key in Settings → Agents.",
    ),
    (
        "subscription_sharing_usage_unavailable",
        AgentErrorCode::Agent,
        "OpenAI couldn't check your ChatGPT plan's usage just now. You're still signed in; try again in a few minutes.",
    ),
    (
        "subscription_sharing_user_unavailable",
        AgentErrorCode::Agent,
        "OpenAI couldn't load your ChatGPT account just now. You're still signed in; try again in a few minutes.",
    ),
    (
        "subscription_sharing_unsupported_capability",
        AgentErrorCode::Agent,
        "ChatGPT plan use doesn't support something this request used; OpenAI names it below.",
    ),
    (
        "subscription_sharing_route_not_supported",
        AgentErrorCode::Agent,
        "ChatGPT plan use doesn't support the request Codex made.",
    ),
    (
        "subscription_sharing_invalid_user",
        AgentErrorCode::AuthFailed,
        "OpenAI couldn't validate your ChatGPT account. Sign in again in Settings → Agents.",
    ),
    (
        "chatpass_v2_scope_not_authorized",
        AgentErrorCode::AuthFailed,
        PERMISSION,
    ),
    (
        "chatpass_v2_invalid_authorization_context",
        AgentErrorCode::AuthFailed,
        PERMISSION,
    ),
];

/// The usage-limit card's text. It names no reset time: OpenAI's code doesn't say whose limit it is, or when it ends.
pub const LIMIT: &str = "You've reached a ChatGPT usage limit: your plan's, or one set for Emmet. Review it in ChatGPT \
                         settings.";
const PERMISSION: &str = "This ChatGPT sign-in doesn't permit the request. Check the account in Settings → Agents.";

/// The HTTP status in a `codexErrorInfo` variant (`{"httpConnectionFailed": {"httpStatusCode": 401}}`).
fn http_status(info: &Value) -> Option<u64> {
    info.as_object()?.values().find_map(|v| v["httpStatusCode"].as_u64())
}

/// An error as an event, with its code: on the ChatGPT plan, OpenAI's recovery first and Codex's own words (with the
/// status, code, and request id) after; otherwise a refused key, no credit, or anything else.
fn agent_error(e: &Value, fallback: &str, plan: bool) -> AgentEvent {
    let message = e["message"].as_str().unwrap_or(fallback);
    let info = &e["codexErrorInfo"];
    if plan {
        let text = format!("{message} {}", e["additionalDetails"].as_str().unwrap_or(""));
        let status = http_status(info);
        let known = PLAN_ERRORS
            .iter()
            .find(|(code, ..)| text.contains(code))
            .map(|(_, c, t)| (*c, *t))
            .or(match (info.as_str(), status) {
                (Some("usageLimitExceeded"), _) => Some((AgentErrorCode::QuotaExceeded, LIMIT)),
                (_, Some(401)) => Some((
                    AgentErrorCode::AuthFailed,
                    "ChatGPT didn't accept the sign-in or its plan permission. Check the account in Settings → Agents.",
                )),
                (_, Some(403)) => Some((
                    AgentErrorCode::Agent,
                    "OpenAI didn't admit the request (for example, from outside a permitted region).",
                )),
                (_, Some(503)) => Some((
                    AgentErrorCode::Agent,
                    "ChatGPT plan use is unavailable just now. You're still signed in; try again later.",
                )),
                _ => None,
            });
        if let Some((code, why)) = known {
            return AgentEvent::Error {
                code,
                message: format!("{why}\n\nOpenAI said: {}", text.trim()),
            };
        }
    }
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
            // OpenAI: the name should match the registration's agent_name_hint.
            client_info: ClientInfo {
                name: AGENT_NAME.into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn code_of(e: Value, plan: bool) -> (AgentErrorCode, String) {
        match agent_error(&e, "fallback", plan) {
            AgentEvent::Error { code, message } => (code, message),
            other => panic!("{other:?}"),
        }
    }

    /// OpenAI's errors-and-recovery table: each documented code, in the error text Codex passes on, gets its recovery
    /// first and Codex's words after; admission failures go by their HTTP status; an API key's errors are unchanged.
    #[test]
    fn plan_errors_get_openai_recovery() {
        let responses = |code: &str, status: u16| {
            json!({
                "message": format!("unexpected status {status}: {{\"error\":{{\"code\":\"{code}\"}}}}, request id: req_1"),
                "codexErrorInfo": { "responseStreamConnectionFailed": { "httpStatusCode": status } },
            })
        };
        for (code, status, want) in [
            (
                "subscription_sharing_usage_limit_exceeded",
                429,
                AgentErrorCode::QuotaExceeded,
            ),
            ("subscription_sharing_user_not_eligible", 403, AgentErrorCode::Agent),
            ("subscription_sharing_usage_unavailable", 503, AgentErrorCode::Agent),
            (
                "subscription_sharing_unsupported_capability",
                400,
                AgentErrorCode::Agent,
            ),
            ("subscription_sharing_route_not_supported", 403, AgentErrorCode::Agent),
            ("subscription_sharing_invalid_user", 401, AgentErrorCode::AuthFailed),
            ("chatpass_v2_scope_not_authorized", 403, AgentErrorCode::AuthFailed),
            (
                "chatpass_v2_invalid_authorization_context",
                403,
                AgentErrorCode::AuthFailed,
            ),
            ("subscription_sharing_user_unavailable", 503, AgentErrorCode::Agent),
        ] {
            let (got, message) = code_of(responses(code, status), true);
            let why = PLAN_ERRORS.iter().find(|(c, ..)| *c == code).unwrap().2;
            assert_eq!(got, want, "{code}");
            assert!(
                message.starts_with(why) && message.contains("req_1"),
                "{code}: {message}"
            );
        }
        let admission = |status: u16| json!({ "message": "{\"detail\":\"denied\"}", "codexErrorInfo": { "httpConnectionFailed": { "httpStatusCode": status } } });
        assert_eq!(code_of(admission(401), true).0, AgentErrorCode::AuthFailed);
        assert!(code_of(admission(403), true).1.contains("permitted region"));
        assert!(code_of(admission(503), true).1.contains("still signed in"));
        assert_eq!(
            code_of(json!({ "message": "x", "codexErrorInfo": "usageLimitExceeded" }), true).0,
            AgentErrorCode::QuotaExceeded
        );
        // The API-key provider: the 1C mapping, Codex's message as it is.
        let (got, message) = code_of(responses("subscription_sharing_invalid_user", 401), false);
        assert_eq!(got, AgentErrorCode::AuthFailed);
        assert!(message.starts_with("unexpected status"));
        assert_eq!(
            code_of(json!({ "message": "Something else" }), true).1,
            "Something else"
        );
    }
}
