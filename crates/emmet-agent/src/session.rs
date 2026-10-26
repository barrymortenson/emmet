// SPDX-License-Identifier: AGPL-3.0-or-later
//! The engine both adapters share (plan section 10): the CLI process, its stdin, the reader and stderr threads, the
//! transcript, turn state, redirect notes and their three paths (D-33), Stop, approvals and questions through the
//! hub, file-change detection, and crash handling. A [`Protocol`] is all a CLI adds: how to encode a turn, a steer,
//! an interrupt, and an approval answer, and how its messages map to [`AgentEvent`]s.

use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use serde_json::Value;

use crate::transcript::{Transcript, redact};
use crate::{
    AgentError, AgentErrorCode, AgentEvent, AgentKind, Answer, ChangeKind, Decision, DoneReason, EventStream, Hub,
    Result, Session, SessionConfig, SteerPath, UserNote, UserTurn,
};

/// The first line of a pending-notes turn after Stop (plan section 10).
const STOPPED: &str = "(You stopped the previous turn.)";
/// How many of the CLI's last stderr lines a crash reports.
const STDERR_LINES: usize = 8;

/// What a protocol asks the engine to do after a call: lines to write to the CLI, events for the chat, and redirect
/// notes the CLI refused natively (they take the queued path).
#[derive(Default)]
pub(crate) struct Io {
    pub writes: Vec<Value>,
    pub events: Vec<AgentEvent>,
    pub refused: Vec<UserNote>,
}

impl Io {
    pub fn write(&mut self, v: Value) {
        self.writes.push(v);
    }
    pub fn emit(&mut self, e: AgentEvent) {
        self.events.push(e);
    }
}

pub(crate) trait Protocol: Send {
    /// The CLI's opening messages (Codex's handshake); Claude Code writes nothing until the first turn.
    fn start(&mut self, _io: &mut Io) {}
    fn send(&mut self, text: &str, io: &mut Io);
    /// Delivers a note mid-turn in the CLI's own protocol. False if it can't now; the engine tries again at the next
    /// step, and the note goes out as the next turn if the turn ends first.
    fn steer(&mut self, note: &UserNote, io: &mut Io) -> bool;
    fn interrupt(&mut self, io: &mut Io);
    /// Answers one of the CLI's own approval requests. False if `id` isn't one.
    fn respond(&mut self, id: &str, d: Decision, io: &mut Io) -> bool;
    fn on_message(&mut self, msg: Value, io: &mut Io);
    fn resume_token(&self) -> Option<String>;
}

struct State {
    proto: Box<dyn Protocol>,
    stdin: Option<ChildStdin>,
    transcript: Transcript,
    tx: Sender<AgentEvent>,
    running: bool,
    /// Tool calls in progress: a note written now reaches the agent at its next step.
    tools: usize,
    stopped: bool,
    /// Notes waiting for the agent's next step, still editable.
    held: Vec<UserNote>,
    native: bool,
    files: Snapshot,
    project: PathBuf,
    closing: bool,
    exited: bool,
    next_note: u64,
    stderr: VecDeque<String>,
    /// Replaced wherever they'd reach the chat (events) or the transcript.
    secrets: Vec<String>,
}

struct Shared {
    state: Mutex<State>,
    child: Mutex<Option<Child>>,
    session: String,
    hub: Hub,
    /// This engine's registration in the hub.
    owner: u64,
    kind: AgentKind,
}

pub(crate) struct Engine {
    shared: Arc<Shared>,
    events: Option<Receiver<AgentEvent>>,
}

impl Engine {
    /// Starts `cmd` (the adapter sets its program, arguments, environment, and folder) and the engine around it.
    /// `secrets` (the credentials in `cmd`'s environment) never reach the transcript or an event.
    pub fn spawn(
        kind: AgentKind,
        cfg: &SessionConfig,
        mut cmd: Command,
        proto: Box<dyn Protocol>,
        meta: Value,
        secrets: Vec<String>,
    ) -> Result<Self> {
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        no_window(&mut cmd);
        let mut child = cmd.spawn().map_err(|e| {
            AgentError::new(
                AgentErrorCode::SpawnFailed,
                format!("{} didn't start: {e}", kind.label()),
            )
        })?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let (tx, rx) = channel();
        let owner = cfg.hub.register(&cfg.id, tx.clone());
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                proto,
                stdin,
                transcript: Transcript::open(&cfg.project, &cfg.id, meta, secrets.clone()),
                tx,
                running: false,
                tools: 0,
                stopped: false,
                held: vec![],
                native: cfg.native_steer,
                files: Snapshot::scan(&cfg.project),
                project: cfg.project.clone(),
                closing: false,
                exited: false,
                next_note: 0,
                stderr: VecDeque::new(),
                secrets,
            }),
            child: Mutex::new(Some(child)),
            session: cfg.id.clone(),
            hub: cfg.hub.clone(),
            owner,
            kind,
        });
        let reader = shared.clone();
        std::thread::spawn(move || reader.read(stdout));
        let errs = shared.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(std::io::Result::ok) {
                let mut s = errs.state.lock().expect("state");
                s.transcript.write("err", Value::String(line.clone()));
                if !line.trim().is_empty() {
                    if s.stderr.len() == STDERR_LINES {
                        s.stderr.pop_front();
                    }
                    s.stderr.push_back(line);
                }
            }
        });
        shared.run(|s, io| s.proto.start(io));
        Ok(Self {
            shared,
            events: Some(rx),
        })
    }
}

impl Shared {
    /// Calls the protocol under the lock and carries out what it asked.
    fn run(&self, f: impl FnOnce(&mut State, &mut Io)) {
        let mut s = self.state.lock().expect("state");
        let mut io = Io::default();
        f(&mut s, &mut io);
        self.apply(&mut s, io);
    }

    fn apply(&self, s: &mut State, io: Io) {
        for v in io.writes {
            s.transcript.write("in", v.clone());
            if let Some(stdin) = s.stdin.as_mut() {
                let line = format!("{v}\n");
                // A failed write means the CLI is gone; the reader reports the exit.
                if stdin.write_all(line.as_bytes()).and_then(|_| stdin.flush()).is_err() {
                    s.stdin = None;
                }
            }
        }
        for note in io.refused {
            self.hub.queue_note(&self.session, note);
        }
        for ev in io.events {
            self.event(s, ev);
        }
    }

    /// Sends one event, after the engine's own bookkeeping for it.
    fn event(&self, s: &mut State, ev: AgentEvent) {
        let ev = s.clean(ev);
        match &ev {
            AgentEvent::ToolCall { .. } => {
                s.tools += 1;
                let _ = s.tx.send(ev);
                self.flush_held(s);
            }
            AgentEvent::ToolResult { .. } => {
                s.tools = s.tools.saturating_sub(1);
                let _ = s.tx.send(ev);
                self.file_changes(s);
            }
            AgentEvent::TurnDone { .. } => {
                s.running = false;
                s.tools = 0;
                self.file_changes(s);
                let _ = s.tx.send(ev);
                self.next_turn(s);
            }
            _ => {
                let _ = s.tx.send(ev);
            }
        }
    }

    /// Writes the held notes now that a step is in progress.
    fn flush_held(&self, s: &mut State) {
        if !s.native || s.held.is_empty() {
            return;
        }
        for note in std::mem::take(&mut s.held) {
            let mut io = Io::default();
            if s.proto.steer(&note, &mut io) {
                self.apply(s, io);
            } else {
                s.held.push(note);
            }
        }
    }

    /// The turn ended with notes still waiting: they go out as the next turn (D-33's third path).
    fn next_turn(&self, s: &mut State) {
        let mut notes = std::mem::take(&mut s.held);
        notes.extend(self.hub.drain_notes(&self.session));
        if notes.is_empty() || s.exited {
            s.stopped = false;
            return;
        }
        let mut text: Vec<&str> = notes.iter().map(|n| n.text.as_str()).collect();
        if s.stopped {
            text.insert(0, STOPPED);
        }
        let text = text.join("\n\n");
        s.stopped = false;
        s.running = true;
        let mut io = Io::default();
        s.proto.send(&text, &mut io);
        for n in notes {
            io.emit(AgentEvent::NoteDelivered {
                id: n.id,
                path: SteerPath::NextTurn,
            });
        }
        self.apply(s, io);
    }

    fn file_changes(&self, s: &mut State) {
        let now = Snapshot::scan(&s.project);
        for (path, kind) in s.files.diff(&now) {
            let ev = s.clean(AgentEvent::FileChange { path, kind });
            let _ = s.tx.send(ev);
        }
        s.files = now;
    }

    /// The reader thread: each line from the CLI, through the protocol, until it closes stdout; then the exit.
    fn read(self: Arc<Self>, stdout: std::process::ChildStdout) {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if line.trim().is_empty() {
                continue;
            }
            let mut s = self.state.lock().expect("state");
            match serde_json::from_str::<Value>(&line) {
                Ok(msg) => {
                    s.transcript.write("out", msg.clone());
                    let mut io = Io::default();
                    s.proto.on_message(msg, &mut io);
                    self.apply(&mut s, io);
                }
                // A line that isn't JSON is kept in the transcript and skipped, never fatal.
                Err(_) => s.transcript.write("bad", Value::String(line)),
            }
        }
        let status = self.child.lock().expect("child").as_mut().and_then(|c| c.wait().ok());
        let code = status.and_then(|s| s.code());
        // stderr may still be draining; give it a moment so a crash message has the CLI's last words.
        std::thread::sleep(Duration::from_millis(50));
        let mut s = self.state.lock().expect("state");
        s.exited = true;
        s.stdin = None;
        s.transcript
            .write("note", serde_json::json!({ "exit": { "code": code } }));
        if !s.closing {
            let said = s.stderr.iter().cloned().collect::<Vec<_>>().join("\n");
            let how = code.map_or("was killed".to_string(), |c| format!("exited with code {c}"));
            let message = if said.is_empty() {
                format!("{} {how}.", self.kind.label())
            } else {
                format!("{} {how}. It said:\n{said}", self.kind.label())
            };
            let ev = s.clean(AgentEvent::Error {
                code: AgentErrorCode::Crashed,
                message,
            });
            let _ = s.tx.send(ev);
            if s.running {
                s.running = false;
                let _ = s.tx.send(AgentEvent::TurnDone {
                    reason: DoneReason::Crashed,
                });
            }
        }
        // Only this engine's registration: a session restarted on the same id (a renewed ChatGPT token) keeps its own.
        self.hub.unregister(&self.session, self.owner);
        let _ = s.tx.send(AgentEvent::Exited { code });
    }

    fn closed(&self) -> AgentError {
        AgentError::new(
            AgentErrorCode::Closed,
            format!("{} isn't running; resume the chat to continue.", self.kind.label()),
        )
    }
}

impl State {
    /// `ev` without the session's secrets.
    fn clean(&self, ev: AgentEvent) -> AgentEvent {
        if self.secrets.is_empty() {
            return ev;
        }
        let text = serde_json::to_string(&ev).expect("an event serializes");
        match redact(&text, &self.secrets) {
            // Fails closed: an event that won't read back once redacted is never passed on as it was.
            Some(t) => serde_json::from_str(&t).unwrap_or(AgentEvent::Error {
                code: AgentErrorCode::Agent,
                message: "The agent said something Emmet hid because it held a credential.".into(),
            }),
            None => ev,
        }
    }
}

impl Engine {
    /// A turn is in progress (or about to start, from pending notes).
    pub fn running(&self) -> bool {
        self.shared.state.lock().expect("state").running
    }
}

impl Session for Engine {
    fn send(&mut self, turn: UserTurn) -> Result<()> {
        let s = self.shared.state.lock().expect("state");
        if s.exited {
            return Err(self.shared.closed());
        }
        // A message sent while the agent works is a redirect.
        if s.running {
            drop(s);
            return self.steer(&turn.text).map(|_| ());
        }
        drop(s);
        self.shared.run(|s, io| {
            s.running = true;
            s.stopped = false;
            s.proto.send(&turn.text, io);
        });
        Ok(())
    }

    fn respond(&mut self, approval: &str, decision: Decision) -> Result<()> {
        if self.shared.hub.decide(&self.shared.session, approval, decision) {
            return Ok(());
        }
        let mut known = false;
        self.shared.run(|s, io| known = s.proto.respond(approval, decision, io));
        if known {
            Ok(())
        } else {
            Err(AgentError::new(
                AgentErrorCode::Closed,
                "That approval is no longer open.",
            ))
        }
    }

    fn answer(&mut self, question: &str, answer: Answer) -> Result<()> {
        if self.shared.hub.answer(&self.shared.session, question, &answer) {
            Ok(())
        } else {
            Err(AgentError::new(
                AgentErrorCode::Closed,
                "That question is no longer open.",
            ))
        }
    }

    fn interrupt(&mut self) -> Result<()> {
        if self.shared.state.lock().expect("state").exited {
            return Err(self.shared.closed());
        }
        // First the cards, so a tool call waiting on the user returns and the interrupt can land.
        self.shared.hub.stop(&self.shared.session);
        self.shared.run(|s, io| {
            if s.running {
                s.stopped = true;
                s.proto.interrupt(io);
            }
        });
        Ok(())
    }

    fn steer(&mut self, text: &str) -> Result<UserNote> {
        let shared = self.shared.clone();
        let mut s = shared.state.lock().expect("state");
        if s.exited {
            return Err(shared.closed());
        }
        s.next_note += 1;
        let note = UserNote {
            id: format!("n{}", s.next_note),
            text: text.to_string(),
            at: crate::time::rfc3339(crate::time::now_ms()),
        };
        let mut io = Io::default();
        if !s.running {
            s.running = true;
            s.stopped = false;
            s.proto.send(text, &mut io);
            io.emit(AgentEvent::NoteDelivered {
                id: note.id.clone(),
                path: SteerPath::NextTurn,
            });
        } else if !s.native {
            shared.hub.queue_note(&shared.session, note.clone());
        } else if s.tools == 0 || !s.proto.steer(&note, &mut io) {
            s.held.push(note.clone());
        }
        shared.apply(&mut s, io);
        Ok(note)
    }

    fn cancel_note(&mut self, id: &str) -> bool {
        let mut s = self.shared.state.lock().expect("state");
        let before = s.held.len();
        s.held.retain(|n| n.id != id);
        s.held.len() != before || self.shared.hub.edit_note(&self.shared.session, id, None)
    }

    fn edit_note(&mut self, id: &str, text: &str) -> bool {
        let mut s = self.shared.state.lock().expect("state");
        if let Some(n) = s.held.iter_mut().find(|n| n.id == id) {
            n.text = text.to_string();
            return true;
        }
        self.shared.hub.edit_note(&self.shared.session, id, Some(text))
    }

    fn events(&mut self) -> EventStream {
        self.events.take().unwrap_or_else(|| channel().1)
    }

    fn resume_token(&self) -> Option<String> {
        self.shared.state.lock().expect("state").proto.resume_token()
    }

    fn close(&mut self) {
        {
            let mut s = self.shared.state.lock().expect("state");
            s.closing = true;
            s.stdin = None;
        }
        self.shared.hub.stop(&self.shared.session);
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            let until = std::time::Instant::now() + Duration::from_secs(3);
            while std::time::Instant::now() < until {
                if shared.state.lock().expect("state").exited {
                    return;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            if let Some(c) = shared.child.lock().expect("child").as_mut() {
                let _ = c.kill();
            }
        });
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.close();
    }
}

/// The project's files by path, size, and modification time: before and after each tool result, the difference is
/// what the agent changed. Generated and tool folders are skipped.
#[derive(Default)]
struct Snapshot(HashMap<String, (u64, Option<SystemTime>)>);

const SKIP: &[&str] = &[
    "build",
    ".emmet",
    ".git",
    "node_modules",
    "__pycache__",
    ".venv",
    "target",
];
const MAX_FILES: usize = 5000;

impl Snapshot {
    fn scan(root: &Path) -> Self {
        let mut files = HashMap::new();
        let mut dirs = vec![(root.to_path_buf(), 0)];
        while let Some((dir, depth)) = dirs.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let Ok(meta) = entry.metadata() else { continue };
                if meta.is_dir() {
                    if depth < 6 && !SKIP.contains(&name.as_str()) {
                        dirs.push((entry.path(), depth + 1));
                    }
                } else if files.len() < MAX_FILES {
                    let rel = entry
                        .path()
                        .strip_prefix(root)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .unwrap_or(name);
                    files.insert(rel, (meta.len(), meta.modified().ok()));
                }
            }
        }
        Self(files)
    }

    fn diff(&self, now: &Snapshot) -> Vec<(String, ChangeKind)> {
        let mut out = Vec::new();
        for (path, sig) in &now.0 {
            match self.0.get(path) {
                None => out.push((path.clone(), ChangeKind::Added)),
                Some(old) if old != sig => out.push((path.clone(), ChangeKind::Modified)),
                _ => {}
            }
        }
        for path in self.0.keys().filter(|p| !now.0.contains_key(*p)) {
            out.push((path.clone(), ChangeKind::Removed));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

/// The CLIs are console programs: on Windows, keep them from opening a console window over the app.
pub(crate) fn no_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}

/// The session's own folder in app data (`sessions/<id>/`), for its per-session config (plan section 10).
pub(crate) fn session_dir(cfg: &SessionConfig) -> Result<PathBuf> {
    let dir = cfg.data.join("sessions").join(&cfg.id);
    std::fs::create_dir_all(&dir).map_err(|e| {
        AgentError::new(
            AgentErrorCode::Io,
            format!("can't make the session folder {}: {e}", dir.display()),
        )
    })?;
    Ok(dir)
}

/// The environment as given, whole: nothing inherited from the app's own process, and nothing filtered (D-37).
pub(crate) fn set_env(cmd: &mut Command, env: &[(String, String)]) {
    cmd.env_clear();
    for (k, v) in env {
        cmd.env(k, v);
    }
}
