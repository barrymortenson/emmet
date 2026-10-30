// SPDX-License-Identifier: AGPL-3.0-or-later
//! The agent replay rig (plan section 16): each test gets a trusted copy of tests/fixtures/projects/agent, a fake-agent
//! folder naming its transcript, and a SessionConfig whose environment is the test home's, so neither the fakes nor
//! the MCP servers they start see anything of this machine's. The bus on the shared data dir answers `approve`,
//! `ask`, and `notes` through one Hub for every session in the test binary, as the app's does.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use emmet_agent::{AgentAdapter, AgentEvent, AgentKind, EventStream, Hub, Session, SessionConfig};
use emmet_core::data::DataDir;
use serde_json::Value;

use crate::{emmet, emmet_bin, repo, shared_data, target_dir, temp_dir, test_home};

/// How long a test waits for one event.
const WAIT: Duration = Duration::from_secs(60);

pub fn fake_bin(cli: &str) -> PathBuf {
    let bin = target_dir().join(format!("fake-{cli}{}", std::env::consts::EXE_SUFFIX));
    assert!(
        bin.exists(),
        "{} is missing; run `cargo build --workspace` first",
        bin.display()
    );
    bin
}

/// The hub behind this test binary's bus (started once, on the shared data dir).
pub fn hub() -> Hub {
    static HUB: OnceLock<(Hub, emmet_bus::Server)> = OnceLock::new();
    HUB.get_or_init(|| {
        let hub = Hub::default();
        let h = hub.clone();
        let server = emmet_bus::Server::start(&DataDir::at(shared_data()), move |msg| route(&h, msg))
            .expect("the test bus starts");
        (hub, server)
    })
    .0
    .clone()
}

/// What the app's bus handler does for the chat's messages (apps/desktop/src-tauri/src/chat.rs).
pub fn route(hub: &Hub, msg: emmet_bus::Message) -> emmet_bus::Reply {
    use emmet_bus::{Message, Reply};
    let answer = match msg {
        Message::Approve { session, request } => hub.approve(&session, request),
        Message::Ask { session, question } => hub.ask(&session, question),
        Message::Notes { session } => hub.take_notes(&session),
        _ => return Reply::undelivered("no window in the agent tests"),
    };
    match answer {
        Some(a) => Reply {
            delivered: true,
            answer: Some(a),
            ..Default::default()
        },
        None => Reply::undelivered("no such chat session"),
    }
}

pub struct Rig {
    pub project: PathBuf,
    /// EMMET_FAKE_DIR: the scenario files and what each fake start recorded.
    pub fake: PathBuf,
    pub home: PathBuf,
    pub cfg: SessionConfig,
    /// The app data dir the adapter reads the ChatGPT accounts from (the shared one, or a [`crate::chatgpt::Plan`]'s);
    /// the bus stays on the shared one.
    pub data: PathBuf,
}

impl Rig {
    /// A rig whose fake plays `tests/replay/<cli>/<scenario>.jsonl`.
    pub fn new(name: &str, cli: &str, scenario: &str) -> Self {
        Self::in_home(name, cli, scenario, test_home())
    }

    pub fn in_home(name: &str, cli: &str, scenario: &str, home: PathBuf) -> Self {
        let dir = temp_dir(&format!("agent-{name}"));
        let project = dir.join("project");
        copy(&repo().join("tests/fixtures/projects/agent"), &project);
        let out = emmet()
            .arg("--data-dir")
            .arg(shared_data())
            .arg("--project")
            .arg(&project)
            .arg("trust")
            .output()
            .expect("emmet trust");
        assert!(out.status.success(), "trust: {}", crate::text(&out));
        let project = emmet_core::fsx::canonical(&project).expect("canonical project");
        let fake = dir.join("fake");
        std::fs::create_dir_all(&fake).expect("fake dir");
        let rig = Self {
            cfg: SessionConfig {
                id: format!("t{}", emmet_core::fsx::random_hex(1)),
                project: project.clone(),
                data: shared_data().to_path_buf(),
                emmet: emmet_bin(),
                resume: None,
                model: None,
                effort: None,
                env: vec![],
                hub: hub(),
                native_steer: true,
                openai_key: None,
            },
            project,
            fake,
            home,
            data: shared_data().to_path_buf(),
        };
        rig.scenario(cli, "", scenario);
        let mut rig = rig;
        rig.cfg.env = rig.env();
        rig
    }

    /// Points the fake at another transcript (`variant` "-usage" for Claude's usage probe).
    pub fn scenario(&self, cli: &str, variant: &str, scenario: &str) {
        let path = repo().join("tests/replay").join(cli).join(format!("{scenario}.jsonl"));
        assert!(path.exists(), "{} is missing", path.display());
        std::fs::write(
            self.fake.join(format!("{cli}{variant}.scenario")),
            path.display().to_string(),
        )
        .expect("scenario");
    }

    /// The environment the CLI gets: the test home, the fakes, a file keychain, and what a process needs to start.
    fn env(&self) -> Vec<(String, String)> {
        let home = self.home.display().to_string();
        let mut env = vec![
            ("HOME".to_string(), home.clone()),
            ("USERPROFILE".into(), home.clone()),
            (
                "APPDATA".into(),
                self.home.join("AppData/Roaming").display().to_string(),
            ),
            (
                "LOCALAPPDATA".into(),
                self.home.join("AppData/Local").display().to_string(),
            ),
            ("EMMET_FAKE_DIR".into(), self.fake.display().to_string()),
            ("EMMET_CLAUDE_BIN".into(), fake_bin("claude").display().to_string()),
            ("EMMET_CODEX_BIN".into(), fake_bin("codex").display().to_string()),
            (
                "EMMET_FAKE_KEYCHAIN".into(),
                self.fake.join("keychain").display().to_string(),
            ),
        ];
        // Windows names are case-insensitive (SystemRoot is SYSTEMROOT), so each is listed once.
        for key in [
            "PATH",
            "TMPDIR",
            "TEMP",
            "TMP",
            "SystemRoot",
            "windir",
            "PATHEXT",
            "ComSpec",
        ] {
            if let Ok(v) = std::env::var(key) {
                env.push((key.to_string(), v));
            }
        }
        env
    }

    pub fn adapter(&self, kind: AgentKind) -> Box<dyn AgentAdapter> {
        emmet_agent::adapter(kind, &self.cfg.env, &self.data)
    }

    /// Runs on `plan`'s ChatGPT accounts, keychain, fake issuer, and fake browser.
    pub fn with_plan(mut self, plan: &crate::chatgpt::Plan) -> Self {
        self.cfg.env.retain(|(k, _)| !plan.env.iter().any(|(p, _)| p == k));
        self.cfg.env.extend(plan.env.iter().cloned());
        self.data = plan.data.clone();
        self
    }

    /// Saves a Codex key in the rig's file keychain.
    pub fn key(&self, key: &str) {
        emmet_agent::keychain::set(&self.cfg.env, key).expect("fake keychain");
    }

    pub fn start(&self, kind: AgentKind) -> (Box<dyn Session>, Events) {
        self.start_with(kind, self.cfg.clone())
    }

    pub fn start_with(&self, kind: AgentKind, cfg: SessionConfig) -> (Box<dyn Session>, Events) {
        let mut session = self.adapter(kind).start(cfg).expect("the session starts");
        let events = Events::new(session.events());
        (session, events)
    }

    /// What the fake's `n`th start recorded: `{args, env, cwd}`.
    pub fn record(&self, cli: &str, n: usize) -> Value {
        let text = std::fs::read_to_string(self.fake.join(format!("{cli}-{n}.json"))).expect("the fake's record");
        serde_json::from_str(&text).expect("record JSON")
    }

    /// Every line Emmet wrote to the CLI, from the session's transcript.
    pub fn written(&self) -> Vec<Value> {
        let path = self
            .project
            .join(".emmet/transcripts")
            .join(format!("{}.jsonl", self.cfg.id));
        std::fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter(|l| l["dir"] == "in")
            .map(|l| l["msg"].clone())
            .collect()
    }

    /// [`outline`], with this rig's paths and run times replaced, for a snapshot that's the same on every machine.
    pub fn outline(&self, events: &[AgentEvent]) -> Outline {
        let clean = |lines: Vec<String>| -> Vec<String> {
            let v = crate::redact(serde_json::json!(lines), &[(&self.project, "<project>")]);
            v.as_array()
                .expect("a list")
                .iter()
                .map(|l| without_seconds(l.as_str().unwrap_or("")).chars().take(90).collect())
                .collect()
        };
        let (mut stream, mut cards, mut files) = (vec![], vec![], vec![]);
        for e in events {
            match e {
                AgentEvent::FileChange { .. } => files.push(e.clone()),
                e if from_hub(e) => cards.push(e.clone()),
                e => stream.push(e.clone()),
            }
        }
        let mut files = outline(&files);
        files.sort();
        files.dedup();
        Outline {
            stream: clean(outline(&stream)),
            cards: clean(outline(&cards)),
            files,
        }
    }

    pub fn model_py(&self) -> String {
        std::fs::read_to_string(self.project.join("model.py")).expect("model.py")
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("copy dir");
    for e in std::fs::read_dir(from).expect("read fixture") {
        let e = e.expect("entry");
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            copy(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), dest).expect("copy file");
        }
    }
}

/// The session's events as they arrive, kept for the failure message.
pub struct Events {
    rx: EventStream,
    pub all: Vec<AgentEvent>,
}

impl Events {
    pub fn new(rx: EventStream) -> Self {
        Self { rx, all: vec![] }
    }

    /// Waits for the first event `pred` accepts, returning the events up to and including it.
    pub fn until(&mut self, what: &str, pred: impl Fn(&AgentEvent) -> bool) -> Vec<AgentEvent> {
        let mut got = vec![];
        loop {
            match self.rx.recv_timeout(WAIT) {
                Ok(e) => {
                    self.all.push(e.clone());
                    let done = pred(&e);
                    got.push(e);
                    if done {
                        return got;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    panic!("no {what} in {} s; so far:\n{:#?}", WAIT.as_secs(), outline(&self.all))
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("the session ended before {what}; so far:\n{:#?}", outline(&self.all))
                }
            }
        }
    }

    /// The events through the next TurnDone.
    pub fn turn(&mut self) -> Vec<AgentEvent> {
        self.until("the end of the turn", |e| matches!(e, AgentEvent::TurnDone { .. }))
    }

    /// The events until the CLI exits.
    pub fn exit(&mut self) -> Vec<AgentEvent> {
        self.until("the exit", |e| matches!(e, AgentEvent::Exited { .. }))
    }
}

/// A session's events in the three orders the design promises: the CLI's own stream, in order; the cards and notes the
/// hub raises over the bus, in order among themselves (they race the stream: a card can show before its tool call);
/// and the files changed by the end of the turn (found by scanning the project, so a fast CLI's next edit can show a
/// step early).
#[derive(Debug, serde::Serialize)]
pub struct Outline {
    pub stream: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cards: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

/// Whether the event came from the hub (emmet_host or `emmet mcp --session` over the bus), not the CLI's stdout:
/// Claude's approval cards (ids `a…`), every question, and notes taken by an Emmet tool.
fn from_hub(e: &AgentEvent) -> bool {
    match e {
        AgentEvent::Approval { id, .. } | AgentEvent::ApprovalClosed { id } => id.starts_with('a'),
        AgentEvent::Question { .. } | AgentEvent::QuestionClosed { .. } => true,
        AgentEvent::NoteDelivered { path, .. } => *path == emmet_agent::SteerPath::Queued,
        _ => false,
    }
}

/// "(0.17 s)" → "(… s)": a run's time isn't part of what a test pins.
fn without_seconds(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('(') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let digits = after
            .find(" s)")
            .filter(|&j| j > 0 && after[..j].chars().all(|c| c.is_ascii_digit() || c == '.'));
        match digits {
            Some(j) => {
                out.push_str("(… s)");
                rest = &after[j + 3..];
            }
            None => {
                out.push('(');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// A compact, OS-independent outline of an event sequence for snapshots: one line per event, with each message's
/// text deltas joined into one line.
pub fn outline(events: &[AgentEvent]) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let mut text: Option<(String, String, String)> = None;
    let flush = |out: &mut Vec<String>, text: &mut Option<(String, String, String)>| {
        if let Some((kind, _, t)) = text.take() {
            out.push(format!("{kind}: {}", t.trim().replace('\n', " ⏎ ")));
        }
    };
    for e in events {
        match e {
            AgentEvent::Text { message, delta } | AgentEvent::Reasoning { message, delta } => {
                let kind = if matches!(e, AgentEvent::Text { .. }) {
                    "text"
                } else {
                    "reasoning"
                };
                match &mut text {
                    Some((k, m, t)) if k == kind && m == message => t.push_str(delta),
                    _ => {
                        flush(&mut out, &mut text);
                        text = Some((kind.into(), message.clone(), delta.clone()));
                    }
                }
                continue;
            }
            _ => flush(&mut out, &mut text),
        }
        out.push(match e {
            AgentEvent::Started { model, billing, .. } => {
                format!(
                    "started {} ({})",
                    model.as_deref().unwrap_or("?"),
                    billing.as_ref().map_or("?".into(), |b| b.label())
                )
            }
            AgentEvent::ToolCall { name, .. } => format!("tool_call {name}"),
            AgentEvent::ToolResult { ok, summary, .. } => {
                let first = summary.lines().next().unwrap_or("");
                format!("tool_result {} {first}", if *ok { "ok" } else { "error" })
            }
            AgentEvent::FileChange { path, kind } => format!("file_change {path} {kind:?}"),
            AgentEvent::Approval { kind, tool, .. } => format!("approval {kind:?} {tool}"),
            AgentEvent::ApprovalClosed { .. } => "approval_closed".into(),
            AgentEvent::Question { question, .. } => format!("question {}", question.question),
            AgentEvent::QuestionClosed { reason, .. } => {
                format!("question_closed {}", reason.as_deref().unwrap_or("answered"))
            }
            AgentEvent::NoteDelivered { path, .. } => format!("note_delivered {path:?}"),
            AgentEvent::Usage { .. } => "usage".into(),
            AgentEvent::Models { models } => format!("models {}", models.len()),
            AgentEvent::TurnDone { reason } => format!("turn_done {reason:?}"),
            AgentEvent::Limits { snapshot } => format!(
                "limits {}",
                snapshot
                    .windows
                    .iter()
                    .map(|w| format!("{} {}%", w.label, w.percent.unwrap_or(-1.0)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            AgentEvent::Error { code, .. } => format!("error {code:?}"),
            AgentEvent::Exited { code } => format!("exited {}", code.map_or("killed".into(), |c| c.to_string())),
            AgentEvent::Text { .. } | AgentEvent::Reasoning { .. } => unreachable!(),
        });
    }
    flush(&mut out, &mut text);
    out
}
