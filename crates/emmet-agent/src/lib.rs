// SPDX-License-Identifier: AGPL-3.0-or-later
//! The agent bridge (plan section 10): start the user's own Claude Code or Codex with Emmet's tools and doctrine, turn
//! its protocol into one normalized event stream for the chat pane, and route permission requests and questions to
//! the user, all without touching the user's agent config (D-26).
//!
//! `session` is the shared engine (the process, the transcript, turns, redirect notes, questions, approvals, file
//! changes, and crashes); `claude` and `codex` are each one [`session::Protocol`]: arguments, encoding, and mapping.

pub mod chatgpt;
pub mod claude;
pub mod codex;
pub mod detect;
pub mod env;
pub mod hub;
pub mod keychain;
mod session;
pub mod time;
mod transcript;
pub mod usage;

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use hub::Hub;
pub use usage::UsageSnapshot;

/// The pointer the app gives the agent (plan section 10): through Claude Code's `--append-system-prompt` and Codex's
/// `developerInstructions`. Unlike a project's AGENTS.md (`emmet_core::project::AGENT_POINTER`), it never mentions a
/// terminal: in the app the `emmet` command isn't on the agent's PATH, and 1C's live check found Codex trying it.
pub const APP_POINTER: &str = "You are operating Emmet, which builds brick models from real LDraw parts. Use its two MCP \
servers: before anything else, call the `rules` tool on the `emmet` server and follow it; use the `emmet` tools to search \
parts, run the model, and render it; and to ask the user a question, call `ask_user` on the `emmet_host` server, which \
shows them a card. The `emmet` command isn't on your PATH here, so don't run it in a shell.";

/// This crate's version, which tracks the workspace.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum AgentKind {
    Claude,
    Codex,
}

impl AgentKind {
    /// The CLI's name in plain text (plan section 10's names row).
    pub fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
        }
    }
}

/// Who pays for the session's model requests (D-35): shown in the picker and the chat header.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Billing {
    /// The user's subscription; `name` is its plan ("Max") when the CLI says.
    Plan {
        name: Option<String>,
    },
    /// An API key; `source` names where it came from (`ANTHROPIC_API_KEY`, the keychain).
    ApiKey {
        source: Option<String>,
    },
    Unknown,
}

impl Billing {
    /// "Max plan", "API key", or "account unknown".
    pub fn label(&self) -> String {
        match self {
            Self::Plan { name: Some(n) } => format!("{} plan", capitalize(n)),
            Self::Plan { name: None } => "plan".into(),
            Self::ApiKey { .. } => "API key".into(),
            Self::Unknown => "account unknown".into(),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// What detection found for one CLI (plan section 10: path, version, supported range, signed in).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Detection {
    pub kind: AgentKind,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    /// Inside the version range this Emmet was tested with. Outside it the chat still works, with a quiet note.
    pub supported: bool,
    pub signed_in: bool,
    pub billing: Billing,
    /// The one step that makes the agent usable, when it isn't: a command for the user's terminal, or a setting.
    pub fix: Option<String>,
    /// A quiet note: an untested version.
    pub note: Option<String>,
}

impl Detection {
    pub fn usable(&self) -> bool {
        self.path.is_some() && self.signed_in
    }
}

/// Everything a session needs that isn't the CLI's own: the project, Emmet's paths, and the user's environment.
#[derive(Clone)]
pub struct SessionConfig {
    /// Emmet's id for the chat session (also the `--session` of its MCP servers).
    pub id: String,
    /// The project root, canonical. The CLI runs here.
    pub project: PathBuf,
    /// The app data dir: the per-session config goes in `sessions/<id>/`, and the bus is found from here.
    pub data: PathBuf,
    /// The emmet sidecar, absolute.
    pub emmet: PathBuf,
    /// A Claude session id or Codex thread id to continue.
    pub resume: Option<String>,
    pub model: Option<String>,
    /// The environment the CLI gets, whole (the login shell's: [`env::login_env`]).
    pub env: Vec<(String, String)>,
    pub hub: Hub,
    /// Deliver redirects natively mid-turn where the CLI supports it (ADR 0001); off, they wait for the next Emmet
    /// tool result instead.
    pub native_steer: bool,
    /// An OpenAI API key for Codex in place of the one in the keychain: `emmet eval` passes Barry's eval key (D-37).
    /// The app leaves it None.
    pub openai_key: Option<String>,
}

/// A message the user sends to start a turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct UserTurn {
    pub text: String,
}

/// A redirect: text the user typed while the agent worked (D-33).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct UserNote {
    pub id: String,
    pub text: String,
    /// RFC 3339.
    pub at: String,
}

/// How a redirect reaches the agent (D-33, ADR 0001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum SteerPath {
    /// Mid-turn input in the CLI's own protocol, at its next step.
    Native,
    /// Attached to the next Emmet tool result as `user_notes`.
    Queued,
    /// Sent as the next turn, because the turn ended (or was stopped) first.
    NextTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    AllowOnce,
    AllowSession,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum ApprovalKind {
    /// A shell command.
    Command,
    /// A file change outside what Emmet allows on its own.
    Edit,
    /// Reading outside the sandbox, the network, or more permissions.
    Permission,
    /// Any other tool.
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Modified,
    Removed,
}

/// One option on a question card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct QuestionOption {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// A question from `emmet_host ask_user` (D-32).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Question {
    pub question: String,
    #[serde(default)]
    pub header: Option<String>,
    pub options: Vec<QuestionOption>,
    #[serde(default)]
    pub multi: bool,
}

/// The user's answer to a question card.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Answer {
    pub choices: Vec<String>,
    /// What the user typed in Other.
    #[serde(default)]
    pub other: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum DoneReason {
    Completed,
    /// The user pressed Stop.
    Interrupted,
    /// The CLI ended the turn with an error (a bad key, no credit, a model error).
    Failed,
    /// The CLI exited mid-turn.
    Crashed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum AgentErrorCode {
    NotInstalled,
    NotSignedIn,
    /// The CLI couldn't be started.
    SpawnFailed,
    /// The CLI exited when it shouldn't have.
    Crashed,
    /// The CLI's key was refused.
    AuthFailed,
    /// No credit or plan usage left.
    QuotaExceeded,
    /// The CLI reported an error of its own.
    Agent,
    /// The session already ended.
    Closed,
    Io,
}

/// The normalized event stream (plan section 10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    Started {
        resume: String,
        model: Option<String>,
        billing: Option<Billing>,
    },
    /// Streamed text; deltas with one `message` id belong to one message.
    Text {
        message: String,
        delta: String,
    },
    Reasoning {
        message: String,
        delta: String,
    },
    ToolCall {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        id: String,
        ok: bool,
        summary: String,
    },
    /// A file in the project changed while the agent worked. `path` is relative to the project, with `/`.
    FileChange {
        path: String,
        kind: ChangeKind,
    },
    Approval {
        id: String,
        kind: ApprovalKind,
        /// The tool or request ("Bash", "commandExecution").
        tool: String,
        /// What it would do, for the card: the command, the file.
        detail: String,
    },
    /// The approval card went away without the user (Stop, or the CLI resolved it).
    ApprovalClosed {
        id: String,
    },
    Question {
        id: String,
        question: Question,
    },
    /// The question card closes: answered, or canceled with `reason` ("stopped").
    QuestionClosed {
        id: String,
        reason: Option<String>,
    },
    NoteDelivered {
        id: String,
        path: SteerPath,
    },
    Usage {
        input: u64,
        output: u64,
    },
    TurnDone {
        reason: DoneReason,
    },
    /// Live plan usage from the CLI (Claude's rate_limit_event).
    Limits {
        snapshot: UsageSnapshot,
    },
    Error {
        code: AgentErrorCode,
        message: String,
    },
    /// The CLI process ended. No events follow.
    Exited {
        code: Option<i32>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentError {
    pub code: AgentErrorCode,
    pub message: String,
}

impl AgentError {
    pub fn new(code: AgentErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AgentError {}

pub type Result<T, E = AgentError> = std::result::Result<T, E>;

pub type EventStream = Receiver<AgentEvent>;

pub trait AgentAdapter: Send + Sync {
    fn kind(&self) -> AgentKind;
    fn detect(&self) -> Detection;
    fn start(&self, cfg: SessionConfig) -> Result<Box<dyn Session>>;
    /// The billing state, plus plan meters where the provider documents a source (D-36).
    fn usage(&self) -> Result<UsageSnapshot>;
}

pub trait Session: Send {
    fn send(&mut self, turn: UserTurn) -> Result<()>;
    fn respond(&mut self, approval: &str, decision: Decision) -> Result<()>;
    fn answer(&mut self, question: &str, answer: Answer) -> Result<()>;
    fn interrupt(&mut self) -> Result<()>;
    /// Sends a redirect by the first path that applies, which it returns; `NoteDelivered` says when it arrived.
    fn steer(&mut self, text: &str) -> Result<UserNote>;
    /// Withdraws a redirect that hasn't been delivered. False if it already went out.
    fn cancel_note(&mut self, id: &str) -> bool;
    /// Changes a redirect that hasn't been delivered. False if it already went out.
    fn edit_note(&mut self, id: &str, text: &str) -> bool;
    /// The event stream. It can be taken once.
    fn events(&mut self) -> EventStream;
    /// The Claude session id or Codex thread id, once the CLI has said it.
    fn resume_token(&self) -> Option<String>;
    /// Ends the CLI: closes its input, then kills it if it hasn't exited in a moment.
    fn close(&mut self);
}

/// A plain id (letters, digits, `-`, `_`; at most 128): what a session id and a resume token must be, since they name
/// files and reach the CLI's command line.
pub fn plain_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && !s.starts_with('-')
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

impl SessionConfig {
    /// Refuses an id or resume token that isn't plain (see [`plain_id`]).
    pub(crate) fn check(&self) -> Result<()> {
        if !plain_id(&self.id) || self.resume.as_deref().is_some_and(|r| !plain_id(r)) {
            return Err(AgentError::new(
                AgentErrorCode::SpawnFailed,
                "That chat session's id isn't one Emmet made, so it can't be resumed.",
            ));
        }
        Ok(())
    }
}

/// An adapter for `kind`, running the CLI with `env`; `data` is the app data dir (Claude's usage probe runs there, and
/// the ChatGPT accounts are there).
pub fn adapter(kind: AgentKind, env: &[(String, String)], data: &std::path::Path) -> Box<dyn AgentAdapter> {
    match kind {
        AgentKind::Claude => Box::new(claude::Claude::new(env.to_vec()).with_data(data.to_path_buf())),
        AgentKind::Codex => Box::new(codex::Codex::new(env.to_vec(), data)),
    }
}
