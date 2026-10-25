// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet mcp-host --session <id>`: the app-internal MCP server `emmet_host` (plan sections 09 and 10). `approve` is
//! Claude Code's `--permission-prompt-tool`; `ask_user` is the question card (D-32). Both reach the running app over
//! the bus, and the app injects this server only into its own chat sessions; a terminal agent asks in its terminal.

use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use emmet_core::data::DataDir;
use emmet_core::envelope::{Error, Result};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// How long a card waits for the user: just under the raised tool timeout of both CLIs (an hour; ADR 0001), so the
/// agent gets `reason: "timeout"` instead of a tool error.
const WAIT: Duration = Duration::from_secs(3540);

/// Tools that only read, which both CLIs' sandboxes let through anyway (plan section 10).
const READS: &[&str] = &["Read", "Glob", "Grep", "LS", "NotebookRead"];
/// Tools that write one file, named by `file_path` (or `notebook_path`).
const EDITS: &[&str] = &["Edit", "Write", "MultiEdit", "NotebookEdit"];

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApproveArgs {
    /// The tool Claude Code wants to run.
    pub tool_name: String,
    /// Its input.
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub tool_use_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AskOption {
    /// What the user clicks: a few words.
    pub label: String,
    /// One line on what picking it means.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AskArgs {
    /// The question, in one sentence.
    pub question: String,
    /// A short label for the card, 12 characters at most ("Scale", "Roof style").
    #[serde(default)]
    pub header: Option<String>,
    /// 2 to 4 options, your recommendation first. The user can always answer in their own words instead.
    pub options: Vec<AskOption>,
    /// Let the user pick more than one option.
    #[serde(default)]
    pub multi: Option<bool>,
}

/// The user's answer, or why there isn't one.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Answer {
    pub answered: bool,
    /// The labels the user picked.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
    /// What the user wrote instead of, or as well as, picking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other: Option<String>,
    /// "stopped" or "timeout", when there is no answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone)]
pub struct EmmetHost {
    data: DataDir,
    root: PathBuf,
    session: String,
    tool_router: ToolRouter<Self>,
}

fn text(t: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(t)])
}

#[tool_router(router = tool_router)]
impl EmmetHost {
    pub fn new(data: DataDir, root: PathBuf, session: String) -> Self {
        Self {
            data,
            root,
            session,
            tool_router: Self::tool_router(),
        }
    }

    /// Claude Code's permission prompt. Reads, and edits inside the project, are allowed; anything else becomes an
    /// approval card in Emmet.
    #[tool]
    async fn approve(&self, Parameters(args): Parameters<ApproveArgs>) -> CallToolResult {
        let me = self.clone();
        let decision = tokio::task::spawn_blocking(move || me.decide(&args))
            .await
            .unwrap_or_else(
                |e| json!({ "behavior": "deny", "message": format!("Emmet's approval check failed: {e}") }),
            );
        text(decision.to_string())
    }

    /// Ask the user one question, with 2 to 4 options (recommended first). Use it only when the choice is theirs and a
    /// sensible default won't do. The answer comes back as this tool's result, so carry on in the same turn.
    #[tool(output_schema = rmcp::handler::server::tool::schema_for_output::<Answer>())]
    async fn ask_user(&self, Parameters(args): Parameters<AskArgs>) -> CallToolResult {
        if !(2..=4).contains(&args.options.len()) {
            let mut out = text(format!(
                "ask_user needs 2 to 4 options; this question has {}. Ask again with 2 to 4.",
                args.options.len()
            ));
            out.is_error = Some(true);
            return out;
        }
        let header = args.header.map(|h| h.chars().take(12).collect::<String>());
        let question = json!({
            "question": args.question,
            "header": header,
            "options": args.options,
            "multi": args.multi.unwrap_or(false),
        });
        let (data, session) = (self.data.clone(), self.session.clone());
        let reply = tokio::task::spawn_blocking(move || {
            emmet_bus::send(&data, emmet_bus::Message::Ask { session, question }, WAIT)
        })
        .await
        .unwrap_or_else(|e| emmet_bus::Reply::undelivered(e.to_string()));
        let answer = match (reply.delivered, reply.answer) {
            (true, Some(a)) => serde_json::from_value(a).unwrap_or_else(|_| Answer {
                reason: Some("stopped".into()),
                ..Default::default()
            }),
            // No answer before the wait ran out (the bus says so), or the chat closed.
            (_, _) if reply.reason.as_deref().is_some_and(|r| r.contains("didn't answer")) => Answer {
                reason: Some("timeout".into()),
                ..Default::default()
            },
            _ => Answer {
                reason: Some("stopped".into()),
                ..Default::default()
            },
        };
        let summary = if answer.answered {
            match (answer.choices.join(", "), &answer.other) {
                (chose, None) => format!("The user chose: {chose}"),
                (chose, Some(o)) if chose.is_empty() => format!("The user answered in their own words: \"{o}\""),
                (chose, Some(o)) => format!("The user chose: {chose}, and added: \"{o}\""),
            }
        } else if answer.reason.as_deref() == Some("timeout") {
            "No answer yet. End your turn now: the user's answer will arrive as their next message.".into()
        } else {
            "The user stopped the turn, so there is no answer. End your turn now.".into()
        };
        let mut out = CallToolResult::structured(serde_json::to_value(&answer).expect("answer serializes"));
        out.content = vec![ContentBlock::text(summary)];
        out
    }
}

impl EmmetHost {
    /// The permission decision for Claude Code: `{behavior: "allow", updatedInput}` or `{behavior: "deny", message}`.
    fn decide(&self, args: &ApproveArgs) -> Value {
        let allow = || json!({ "behavior": "allow", "updatedInput": args.input });
        if auto_allowed(&self.root, &args.tool_name, &args.input) {
            return allow();
        }
        let request = json!({ "tool_name": args.tool_name, "input": args.input, "tool_use_id": args.tool_use_id });
        let reply = emmet_bus::send(
            &self.data,
            emmet_bus::Message::Approve {
                session: self.session.clone(),
                request,
            },
            WAIT,
        );
        match reply.answer {
            Some(a) if reply.delivered && a["behavior"] == "allow" => allow(),
            Some(a) if reply.delivered => json!({
                "behavior": "deny",
                "message": a["message"].as_str().unwrap_or("The user denied this in Emmet."),
            }),
            _ => json!({
                "behavior": "deny",
                "message": format!(
                    "Emmet couldn't ask the user ({}), so this wasn't allowed. Nothing ran.",
                    reply.reason.as_deref().unwrap_or("no answer")
                ),
            }),
        }
    }
}

/// Files inside the project that configure an agent: an edit to one is a card, since it changes what the next session
/// may do. Dot folders and files (`.claude/`, `.codex/`, `.mcp.json`, `.git/hooks/`, `.emmet/`) are cards too.
const AGENT_FILES: &[&str] = &["AGENTS.md", "CLAUDE.md", "CLAUDE.local.md", "AGENTS.override.md"];

/// What `approve` allows without a card (plan section 10): reads, our own tools, and an edit to a model file inside the
/// project. The path must stay inside after following links, so a link out of the project, or `..`, is a card, and so
/// is an edit to agent config ([`AGENT_FILES`], anything under a dot folder).
pub fn auto_allowed(root: &Path, tool: &str, input: &Value) -> bool {
    if READS.contains(&tool) || tool.starts_with("mcp__emmet__") || tool == "mcp__emmet_host__ask_user" {
        return true;
    }
    if !EDITS.contains(&tool) {
        return false;
    }
    // The field the tool writes: a NotebookEdit's notebook_path, everyone else's file_path.
    let field = if tool == "NotebookEdit" {
        "notebook_path"
    } else {
        "file_path"
    };
    let Some(path) = input[field].as_str() else {
        return false;
    };
    let Some(rel) = inside(root, Path::new(path)) else {
        return false;
    };
    !rel.components().any(|c| is_config(&c.as_os_str().to_string_lossy()))
}

/// A path component that's agent config: a dot name, or an agent's instruction file. Windows ignores a name's
/// trailing dots and spaces, and `:` names a file's alternate stream, so `CLAUDE.md.` and `CLAUDE.md:x` count too.
fn is_config(name: &str) -> bool {
    let name = name.trim_end_matches(['.', ' ']);
    name.starts_with('.') || name.contains(':') || AGENT_FILES.iter().any(|f| name.eq_ignore_ascii_case(f))
}

/// Where `path` (absolute, or relative to `root`) lands inside `root` once links are followed, relative to it; None if
/// outside. A file that doesn't exist yet is judged by its nearest existing folder.
fn inside(root: &Path, path: &Path) -> Option<PathBuf> {
    let Ok(root) = emmet_core::fsx::canonical(root) else {
        return None;
    };
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let mut existing = full.as_path();
    let mut rest = Vec::new();
    while !existing.exists() {
        // There, but not followable: a dangling link, which a write would follow out of the project.
        if existing.symlink_metadata().is_ok() {
            return None;
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name.to_owned());
                existing = parent;
            }
            _ => return None,
        }
    }
    let mut real = emmet_core::fsx::canonical(existing).ok()?;
    for name in rest.iter().rev() {
        // A name that's `..` (or `.`) after a missing folder can't be judged without the folder; refuse it.
        if Path::new(name).components().any(|c| !matches!(c, Component::Normal(_))) {
            return None;
        }
        real.push(name);
    }
    real.strip_prefix(&root).ok().map(Path::to_path_buf)
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for EmmetHost {
    fn get_info(&self) -> ServerConfig {
        let mut info = ServerConfig::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("emmet_host", emmet_core::version());
        info.instructions = Some(
            "Emmet's own chat tools. ask_user asks the user one question with 2 to 4 options and returns the answer."
                .into(),
        );
        info
    }
}

/// Serves `emmet_host` on stdin and stdout until the client closes them.
pub fn serve(data: DataDir, root: PathBuf, session: String) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::internal(format!("starting the MCP runtime: {e}")))?;
    rt.block_on(async move {
        let service = EmmetHost::new(data, root, session)
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| Error::internal(format!("starting the emmet_host server: {e}")))?;
        service
            .waiting()
            .await
            .map_err(|e| Error::internal(format!("the emmet_host server stopped: {e}")))?;
        Ok(())
    })
}
