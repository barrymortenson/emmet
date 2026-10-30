// SPDX-License-Identifier: AGPL-3.0-or-later
//! The chat's host side (plan sections 04 and 10, session 4B): the project's conversations, kept in its `.emmet/chat/`
//! (each one's agent, its resume token, its model and effort, and its history); one agent session at a time, started
//! when the user sends a message, so a conversation resumes after an app restart; switching agents and Start fresh,
//! which begin a new conversation and keep the old one; the version Emmet saves before each turn and Undo turn;
//! approvals and questions from the bus to the hub; the agents' detection; the plan usage fetch (D-36); and the Codex
//! API key in the keychain (D-35). Sign in with ChatGPT's commands are in `chatgpt.rs`.
//!
//! Every turn starts from something the user did (plan section 10): a message, an answer, or a redirect. Restart
//! after a crash starts the CLI again on the same conversation and sends nothing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use emmet_agent::{
    AgentEvent, AgentKind, Answer, Billing, Decision, Detection, Hub, ModelChoice, Session, SessionConfig, TurnContext,
    UsageSnapshot, UserNote, UserTurn,
};
use emmet_core::project::Project;
use emmet_core::versions;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::app::{AppState, Problem};
use crate::projects::VersionView;

/// Plan usage is fetched at most this often (D-36), whatever the user presses.
const USAGE_EVERY: Duration = Duration::from_secs(60);

/// One agent event for the window. `at` is when the host had it (Unix ms), so the window can measure the time from
/// event to pixels (plan section 10's 50 ms).
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct ChatEvent {
    pub session: String,
    pub seq: u64,
    pub at: f64,
    pub event: AgentEvent,
}

/// A turn ended: the version Undo turn restores, if the turn changed the sources (else none).
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct ChatUndo {
    pub undo: Option<u32>,
}

/// What the picker shows for one agent.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AgentView {
    pub kind: AgentKind,
    /// "Claude Code", "Codex".
    pub name: String,
    pub detection: Detection,
    /// "Claude · Max plan", "Codex · API key".
    pub badge: String,
    /// Said when the account that pays may not be the one the user expects (D-35).
    pub warning: Option<String>,
}

/// Where a message the app wrote for the user came from: the chat shows "Sent from Checks" and the plain words, and
/// the agent gets the details too (law 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// "Ask agent to fix" on a finding.
    Checks,
    /// Rebuild with new build settings.
    Settings,
    /// "Ask agent to fix" on a failed build.
    Build,
    /// "Make it parametric" on an imported file.
    Import,
}

/// One line of a conversation: something the user sent, or an agent event.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ChatRecord {
    pub at: f64,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<Origin>,
    #[serde(default)]
    pub event: Option<AgentEvent>,
}

/// One conversation in the project's history.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SessionInfo {
    pub id: String,
    pub kind: AgentKind,
    /// When it began (Unix ms).
    pub started: f64,
    /// The Claude session id or Codex thread id, once the CLI said it.
    #[serde(default)]
    pub resume: Option<String>,
    /// The model and effort, as the CLI names them; none is its default.
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub effort: Option<String>,
}

/// `.emmet/chat/sessions.json`: the conversations, oldest first (the newest is the current one), and the version Undo
/// turn restores.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Store {
    #[serde(default)]
    sessions: Vec<SessionInfo>,
    #[serde(default)]
    undo: Option<u32>,
}

/// The chat that's running.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LiveSession {
    pub session: String,
    pub kind: AgentKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SessionView {
    pub info: SessionInfo,
    pub records: Vec<ChatRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ChatState {
    /// The session that's running, if one is.
    pub live: Option<LiveSession>,
    /// The project's conversations, oldest first; the newest is the one a message goes to.
    pub sessions: Vec<SessionView>,
    /// The agent a message goes to: the newest conversation's, else the one Settings starts with.
    pub agent: Option<AgentKind>,
    /// The version Undo turn restores.
    pub undo: Option<u32>,
    /// The models and efforts that agent offers, when known (Codex says once a session has started).
    pub models: Vec<ModelChoice>,
}

struct Live {
    project: PathBuf,
    id: String,
    kind: AgentKind,
    /// The model and effort it was started with: a change starts it again on the same conversation.
    model: Option<String>,
    effort: Option<String>,
    /// The ChatGPT account Codex runs on, when it's on the user's plan.
    plan: Option<String>,
    /// The version saved as the turn started, until the turn ends.
    turn: Option<u32>,
    session: Box<dyn Session>,
}

pub struct Chat {
    pub hub: Hub,
    env: OnceLock<Vec<(String, String)>>,
    agents: Mutex<Option<Vec<Detection>>>,
    live: Mutex<Option<Live>>,
    usage: Mutex<Option<(Instant, UsageSnapshot)>>,
    /// What each agent said it offers, from its last session.
    models: Mutex<HashMap<AgentKind, Vec<ModelChoice>>>,
    seq: AtomicU64,
    /// The Sign in with ChatGPT attempt waiting for the browser: set to cancel it.
    pub sign_in: Mutex<Option<Arc<AtomicBool>>>,
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            hub: Hub::default(),
            env: OnceLock::new(),
            agents: Mutex::new(None),
            live: Mutex::new(None),
            usage: Mutex::new(None),
            models: Mutex::new(HashMap::new()),
            seq: AtomicU64::new(0),
            sign_in: Mutex::new(None),
        }
    }
}

fn now() -> f64 {
    emmet_agent::time::now_ms() as f64
}

fn problem(message: impl Into<String>) -> Box<Problem> {
    Problem::internal(message)
}

impl Chat {
    /// The login shell's environment, read once (in the background at launch, so the first chat doesn't wait).
    pub fn env(&self) -> &[(String, String)] {
        self.env.get_or_init(emmet_agent::env::login_env)
    }

    fn detect(&self, data: &Path, refresh: bool) -> Vec<Detection> {
        let mut cached = self.agents.lock().expect("agents");
        if refresh || cached.is_none() {
            let env = self.env().to_vec();
            let found = [AgentKind::Claude, AgentKind::Codex]
                .map(|k| {
                    let (env, data) = (env.clone(), data.to_path_buf());
                    std::thread::spawn(move || emmet_agent::adapter(k, &env, &data).detect())
                })
                .map(|t| t.join().expect("detection"))
                .to_vec();
            *cached = Some(found);
        }
        cached.clone().unwrap_or_default()
    }

    /// The plan's name from detection, for a session whose init only says "plan".
    fn plan_name(&self, kind: AgentKind) -> Option<String> {
        let agents = self.agents.lock().expect("agents");
        agents
            .as_ref()?
            .iter()
            .find(|d| d.kind == kind)
            .and_then(|d| match &d.billing {
                Billing::Plan { name } => name.clone(),
                _ => None,
            })
    }

    /// An agent's sign-in changed: detect again next time.
    pub fn forget_agents(&self) {
        *self.agents.lock().expect("agents") = None;
    }

    /// The ChatGPT account the running chat bills, if it's Codex on a plan.
    pub fn live_plan(&self) -> Option<String> {
        self.live.lock().expect("live").as_ref().and_then(|l| l.plan.clone())
    }

    /// Who the running chat bills, if it's a `kind` chat: what it started with, whatever Settings says since.
    fn live_billing(&self, kind: AgentKind) -> Option<Billing> {
        let live = self.live.lock().expect("live");
        let l = live.as_ref().filter(|l| l.kind == kind && kind == AgentKind::Codex)?;
        Some(match l.plan {
            Some(_) => emmet_agent::codex::plan_billing(),
            None => Billing::ApiKey {
                source: Some("keychain".into()),
            },
        })
    }

    /// What `kind` offers: Claude Code's documented list, or what Codex said in its last session.
    fn models(&self, kind: AgentKind) -> Vec<ModelChoice> {
        match kind {
            AgentKind::Claude => emmet_agent::claude::models(),
            AgentKind::Codex => self
                .models
                .lock()
                .expect("models")
                .get(&kind)
                .cloned()
                .unwrap_or_default(),
        }
    }

    /// Ends the running session, if any.
    pub fn close(&self) {
        if let Some(mut live) = self.live.lock().expect("live").take() {
            live.session.close();
        }
    }
}

/// The chat's messages on the bus: emmet_host's approve and ask_user, and `emmet mcp --session` taking notes.
pub fn route(hub: &Hub, msg: emmet_bus::Message) -> emmet_bus::Reply {
    use emmet_bus::{Message, Reply};
    let answer = match msg {
        Message::Approve { session, request } => hub.approve(&session, request),
        Message::Ask { session, question } => hub.ask(&session, question),
        Message::Notes { session } => hub.take_notes(&session),
        _ => return Reply::undelivered("not a chat message"),
    };
    match answer {
        Some(a) => Reply {
            delivered: true,
            answer: Some(a),
            ..Default::default()
        },
        None => Reply::undelivered("that chat session isn't open in Emmet"),
    }
}

fn badge(kind: AgentKind, billing: &Billing) -> String {
    let who = match kind {
        AgentKind::Claude => "Claude",
        AgentKind::Codex => "Codex",
    };
    format!("{who} · {}", billing.label())
}

fn view(d: Detection) -> AgentView {
    let warning = match (&d.kind, &d.billing) {
        (AgentKind::Claude, Billing::ApiKey { source: Some(var) }) => Some(format!(
            "Claude Code will bill the API key in {var}, not a plan. To use your plan instead, remove {var} from your \
             shell's startup file, then reopen Emmet."
        )),
        _ => None,
    };
    AgentView {
        kind: d.kind,
        name: d.kind.label().into(),
        badge: badge(d.kind, &d.billing),
        warning,
        detection: d,
    }
}

fn chat_dir(project: &Path) -> PathBuf {
    project.join(".emmet").join("chat")
}

fn append(project: &Path, session: &str, record: &ChatRecord) {
    use std::io::Write;
    let dir = chat_dir(project);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(format!("{session}.jsonl")))
    {
        let _ = writeln!(f, "{}", serde_json::to_string(record).expect("record"));
    }
}

/// An id this app made: `c` and 32 hex digits.
fn ours(id: &str) -> bool {
    id.len() == 33 && id.starts_with('c') && id[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

fn new_id() -> String {
    format!("c{}", emmet_core::fsx::random_hex(2))
}

/// The project's conversations. The file comes with the project, which may be someone else's: a session whose id isn't
/// ours, or whose resume token, model, or effort isn't a plain name, is left out. 1C's `.emmet/chat.json` (the one last
/// session) is read when there's no store yet.
fn store(project: &Project) -> Store {
    let read = |p: PathBuf| std::fs::read_to_string(p).ok();
    let mut s: Store = match read(chat_dir(&project.root).join("sessions.json")) {
        Some(text) => serde_json::from_str(&text).unwrap_or_default(),
        None => read(project.root.join(".emmet").join("chat.json"))
            .and_then(|t| serde_json::from_str::<SessionInfo>(&t).ok())
            .map(|old| Store {
                sessions: vec![old],
                undo: None,
            })
            .unwrap_or_default(),
    };
    let plain = |v: &Option<String>| v.as_deref().is_none_or(emmet_agent::model_id);
    s.sessions.retain(|i| {
        ours(&i.id) && i.resume.as_deref().is_none_or(emmet_agent::plain_id) && plain(&i.model) && plain(&i.effort)
    });
    s
}

fn save_store(project: &Project, s: &Store) -> Result<(), Box<Problem>> {
    let dir = project.own_dir(".emmet")?.join("chat");
    std::fs::create_dir_all(&dir).map_err(|e| problem(e.to_string()))?;
    emmet_core::fsx::write_atomic(
        &dir.join("sessions.json"),
        serde_json::to_string_pretty(s).expect("json").as_bytes(),
    )?;
    Ok(())
}

/// Changes the stored session `id`.
fn update_session(project: &Project, id: &str, f: impl FnOnce(&mut SessionInfo)) {
    let mut s = store(project);
    if let Some(i) = s.sessions.iter_mut().find(|i| i.id == id) {
        f(i);
        let _ = save_store(project, &s);
    }
}

/// A conversation's history, with each message's streamed deltas joined into one event, as the window would have
/// them by the end.
fn history(project: &Path, session: &str) -> Vec<ChatRecord> {
    let mut out: Vec<ChatRecord> = Vec::new();
    for line in std::fs::read_to_string(chat_dir(project).join(format!("{session}.jsonl")))
        .unwrap_or_default()
        .lines()
    {
        let Ok(r) = serde_json::from_str::<ChatRecord>(line) else {
            continue;
        };
        if let (Some(prev), Some(next)) = (out.last_mut().and_then(|p| p.event.as_mut()), &r.event)
            && join(prev, next)
        {
            continue;
        }
        out.push(r);
    }
    out
}

/// Adds `next`'s delta to `prev` when both stream the same message.
fn join(prev: &mut AgentEvent, next: &AgentEvent) -> bool {
    match (prev, next) {
        (AgentEvent::Text { message: a, delta }, AgentEvent::Text { message: b, delta: d })
        | (AgentEvent::Reasoning { message: a, delta }, AgentEvent::Reasoning { message: b, delta: d })
            if a == b =>
        {
            delta.push_str(d);
            true
        }
        _ => false,
    }
}

fn open_project(app: &AppHandle) -> Result<Project, Box<Problem>> {
    app.state::<AppState>().project()
}

/// Runs `f` on the live session of the open project.
fn with_live<T>(app: &AppHandle, f: impl FnOnce(&mut Live) -> Result<T, Box<Problem>>) -> Result<T, Box<Problem>> {
    let root = open_project(app)?.root;
    let state = app.state::<AppState>();
    let mut live = state.chat.live.lock().expect("live");
    match live.as_mut() {
        Some(l) if l.project == root => f(l),
        _ => Err(Box::new(Problem {
            hint: "Send a message to start the agent.".into(),
            ..*problem("No chat is running.")
        })),
    }
}

fn agent_problem(e: emmet_agent::AgentError) -> Box<Problem> {
    problem(e.message)
}

/// Detection now, on this thread (the app warms it at launch).
pub fn agents_now(app: &AppHandle) -> Vec<AgentView> {
    let state = app.state::<AppState>();
    state
        .chat
        .detect(state.data.root(), false)
        .into_iter()
        .map(view)
        .collect()
}

/// The agents Emmet found: only a detected, signed-in agent can start a chat; the others carry the fix.
#[tauri::command]
#[specta::specta]
pub async fn agents(app: AppHandle, refresh: bool) -> Vec<AgentView> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.chat.detect(state.data.root(), refresh)
    })
    .await
    .unwrap_or_default()
    .into_iter()
    .map(view)
    .collect()
}

/// The agent a new conversation starts with: Settings' choice, else the first one that's ready.
fn default_agent(app: &AppHandle) -> Option<AgentKind> {
    let state = app.state::<AppState>();
    match crate::prefs::prefs(&state.data).start_with.as_deref() {
        Some("claude") => return Some(AgentKind::Claude),
        Some("codex") => return Some(AgentKind::Codex),
        _ => {}
    }
    let found = state.chat.detect(state.data.root(), false);
    found.iter().find(|d| d.usable()).or(found.first()).map(|d| d.kind)
}

/// A new conversation with `kind`, on Settings' model for Claude Code.
fn fresh(app: &AppHandle, kind: AgentKind) -> SessionInfo {
    let model = match kind {
        AgentKind::Claude => crate::prefs::prefs(&app.state::<AppState>().data).claude_model,
        AgentKind::Codex => None,
    };
    SessionInfo {
        id: new_id(),
        kind,
        started: now(),
        resume: None,
        model,
        effort: None,
    }
}

/// The conversation a message goes to, made if the project has none.
fn current(app: &AppHandle, project: &Project) -> Result<SessionInfo, Box<Problem>> {
    let mut s = store(project);
    if let Some(i) = s.sessions.last() {
        return Ok(i.clone());
    }
    let kind = default_agent(app).ok_or_else(|| problem("Set up Claude Code or Codex first."))?;
    let info = fresh(app, kind);
    s.sessions.push(info.clone());
    save_store(project, &s)?;
    Ok(info)
}

/// Makes sure the current conversation's agent is running, with its model and effort, starting it (on the same
/// conversation, when it has one) if it isn't.
fn ensure_live(app: &AppHandle, project: &Project) -> Result<String, Box<Problem>> {
    let state = app.state::<AppState>();
    let info = current(app, project)?;
    {
        let live = state.chat.live.lock().expect("live");
        if let Some(l) = live.as_ref()
            && l.project == project.root
            && l.id == info.id
            && (l.model.as_ref(), l.effort.as_ref()) == (info.model.as_ref(), info.effort.as_ref())
        {
            return Ok(l.id.clone());
        }
    }
    state.chat.close();
    let cfg = SessionConfig {
        id: info.id.clone(),
        project: project.root.clone(),
        data: state.data.root().to_path_buf(),
        emmet: state.emmet.clone(),
        resume: info.resume.clone(),
        model: info.model.clone(),
        effort: info.effort.clone(),
        env: state.chat.env().to_vec(),
        hub: state.chat.hub.clone(),
        native_steer: true,
        openai_key: None,
    };
    let kind = info.kind;
    let adapter = emmet_agent::adapter(kind, &cfg.env, state.data.root());
    let plan = match emmet_agent::chatgpt::ChatGpt::new(state.data.root(), &cfg.env).codex_use() {
        emmet_agent::chatgpt::CodexUse::Plan { id, .. } if kind == AgentKind::Codex => Some(id),
        _ => None,
    };
    let mut session = adapter.start(cfg).map_err(agent_problem)?;
    let events = session.events();
    *state.chat.live.lock().expect("live") = Some(Live {
        project: project.root.clone(),
        id: info.id.clone(),
        kind,
        model: info.model,
        effort: info.effort,
        plan,
        turn: None,
        session,
    });
    let (handle, project, sid) = (app.clone(), project.clone(), info.id.clone());
    std::thread::spawn(move || pump(&handle, &project, &sid, kind, events));
    Ok(info.id)
}

/// The turn is over: the version saved as it started goes if the turn changed nothing, and otherwise is what Undo
/// turn restores.
fn settle(app: &AppHandle, project: &Project, id: &str) {
    let state = app.state::<AppState>();
    let turn = {
        let mut live = state.chat.live.lock().expect("live");
        live.as_mut().filter(|l| l.id == id).and_then(|l| l.turn.take())
    };
    let Some(n) = turn else { return };
    // A turn that changed nothing keeps no version, and there's no turn to undo.
    let changed = !versions::unchanged_since(project, n).unwrap_or(false);
    if !changed {
        let _ = versions::discard(project, n);
    }
    let mut s = store(project);
    s.undo = changed.then_some(n);
    let _ = save_store(project, &s);
    let _ = ChatUndo { undo: s.undo }.emit(app);
}

/// The session's events, to the window and the history, until the CLI exits.
fn pump(app: &AppHandle, project: &Project, id: &str, kind: AgentKind, events: emmet_agent::EventStream) {
    let state = app.state::<AppState>();
    let root = &project.root;
    for mut event in events {
        let at = now();
        match &mut event {
            AgentEvent::Started { resume, billing, .. } => {
                // The plan's name comes from detection; init says only that it's a plan.
                if let Some(Billing::Plan { name }) = billing
                    && name.is_none()
                {
                    *name = state.chat.plan_name(kind);
                }
                let token = resume.clone();
                update_session(project, id, |i| {
                    i.resume = Some(token).filter(|t| emmet_agent::plain_id(t))
                });
            }
            AgentEvent::Models { models } => {
                state.chat.models.lock().expect("models").insert(kind, models.clone());
            }
            AgentEvent::TurnDone { .. } | AgentEvent::Exited { .. } => settle(app, project, id),
            _ => {}
        }
        // Text arrives in many small deltas; the history keeps them, and reading it joins them again.
        append(
            root,
            id,
            &ChatRecord {
                at,
                user: None,
                from: None,
                event: Some(event.clone()),
            },
        );
        let seq = state.chat.seq.fetch_add(1, Ordering::SeqCst);
        let _ = ChatEvent {
            session: id.to_string(),
            seq,
            at,
            event,
        }
        .emit(app);
    }
    // The session is over; forget it if it's still the live one.
    let mut live = state.chat.live.lock().expect("live");
    if live.as_ref().is_some_and(|l| l.id == id) {
        *live = None;
    }
}

/// Runs `f` off the main thread: starting a CLI, or a turn on the ChatGPT plan that first renews the token and
/// restarts Codex (1E), would freeze the window if a command ran it on the main thread, as Tauri does with a command
/// that isn't async.
async fn blocking<T: Send + 'static>(
    app: AppHandle,
    f: impl FnOnce(&AppHandle) -> Result<T, Box<Problem>> + Send + 'static,
) -> Result<T, Box<Problem>> {
    tauri::async_runtime::spawn_blocking(move || f(&app))
        .await
        .map_err(|e| problem(e.to_string()))?
}

/// A message from the user, which starts a turn: the agent starts first if it isn't running (resuming the
/// conversation). `context` is what the user sees (the selection and the view) and the details behind a message the
/// app wrote (`from`); the history keeps the plain words. Returns the session the events come from.
#[tauri::command]
#[specta::specta]
pub async fn chat_send(
    app: AppHandle,
    text: String,
    context: TurnContext,
    from: Option<Origin>,
) -> Result<String, Box<Problem>> {
    blocking(app, move |app| send(app, text, context, from)).await
}

fn send(app: &AppHandle, text: String, context: TurnContext, from: Option<Origin>) -> Result<String, Box<Problem>> {
    let project = open_project(app)?;
    let id = ensure_live(app, &project)?;
    let state = app.state::<AppState>();
    // Before the agent touches anything: the version Undo turn goes back to. One the project can't keep (read-only)
    // isn't saved, and the turn still goes.
    let who = with_live(app, |l| Ok(l.kind))?;
    let saved = project
        .require_trusted(&state.data)
        .ok()
        .and_then(|_| {
            let name = format!(
                "Before {}'s turn",
                if who == AgentKind::Claude { "Claude" } else { "Codex" }
            );
            versions::save(&state.data, &project, &name, None, true, None).ok()
        })
        .map(|v| v.n);
    with_live(app, |l| {
        l.turn = saved;
        append(
            &l.project,
            &l.id,
            &ChatRecord {
                at: now(),
                user: Some(text.clone()),
                from,
                event: None,
            },
        );
        l.session
            .send(UserTurn {
                text: context.prompt(&text),
            })
            .map_err(agent_problem)
    })?;
    Ok(id)
}

/// A redirect while the agent works (D-33): returns the note, which `NoteDelivered` marks once it's out.
#[tauri::command]
#[specta::specta]
pub async fn chat_steer(app: AppHandle, text: String, context: TurnContext) -> Result<UserNote, Box<Problem>> {
    blocking(app, move |app| {
        with_live(app, |l| {
            let note = l.session.steer(&context.prompt(&text)).map_err(agent_problem)?;
            append(
                &l.project,
                &l.id,
                &ChatRecord {
                    at: now(),
                    user: Some(text),
                    from: None,
                    event: None,
                },
            );
            Ok(note)
        })
    })
    .await
}

/// Changes a redirect that hasn't gone out, or withdraws it (`text` null). False if it's already delivered.
#[tauri::command]
#[specta::specta]
pub fn chat_note(app: AppHandle, id: String, text: Option<String>) -> Result<bool, Box<Problem>> {
    with_live(&app, |l| {
        Ok(match text {
            Some(t) => l.session.edit_note(&id, &t),
            None => l.session.cancel_note(&id),
        })
    })
}

/// Stop (Esc): the turn ends at once; an open question is canceled, and pending redirects go out as the next turn.
#[tauri::command]
#[specta::specta]
pub fn chat_stop(app: AppHandle) -> Result<(), Box<Problem>> {
    with_live(&app, |l| l.session.interrupt().map_err(agent_problem))
}

#[tauri::command]
#[specta::specta]
pub fn chat_respond(app: AppHandle, id: String, decision: Decision) -> Result<(), Box<Problem>> {
    with_live(&app, |l| l.session.respond(&id, decision).map_err(agent_problem))
}

#[tauri::command]
#[specta::specta]
pub fn chat_answer(app: AppHandle, id: String, answer: Answer) -> Result<(), Box<Problem>> {
    with_live(&app, |l| l.session.answer(&id, answer).map_err(agent_problem))
}

/// A new conversation with `kind` (the agent picker), or with the same agent (Start fresh): the agent running now
/// ends, and its conversation is kept. Nothing starts until the next message.
#[tauri::command]
#[specta::specta]
pub fn chat_fresh(app: AppHandle, kind: Option<AgentKind>) -> Result<SessionInfo, Box<Problem>> {
    let project = open_project(&app)?;
    let state = app.state::<AppState>();
    state.chat.close();
    let mut s = store(&project);
    let kind = kind
        .or(s.sessions.last().map(|i| i.kind))
        .or_else(|| default_agent(&app))
        .ok_or_else(|| problem("Set up Claude Code or Codex first."))?;
    // An empty conversation (nothing said yet) is replaced, not kept.
    if let Some(last) = s.sessions.last()
        && history(&project.root, &last.id).is_empty()
    {
        s.sessions.pop();
    }
    let info = fresh(&app, kind);
    s.sessions.push(info.clone());
    save_store(&project, &s)?;
    Ok(info)
}

/// The model and effort for the current conversation (the composer's picker). A running agent takes them at the next
/// message, started again on the same conversation.
#[tauri::command]
#[specta::specta]
pub fn chat_model(app: AppHandle, model: Option<String>, effort: Option<String>) -> Result<SessionInfo, Box<Problem>> {
    let project = open_project(&app)?;
    let info = current(&app, &project)?;
    let state = app.state::<AppState>();
    let offered = state.chat.models(info.kind);
    let model = model.filter(|m| !m.is_empty());
    let known = |m: &str| offered.iter().any(|c| c.id == m) || (offered.is_empty() && emmet_agent::model_id(m));
    if let Some(m) = model.as_deref().filter(|m| !known(m)) {
        return Err(problem(format!("{} doesn't offer the model {m}.", info.kind.label())));
    }
    let efforts: Vec<&String> = offered
        .iter()
        .filter(|c| c.id == model.clone().unwrap_or_default())
        .flat_map(|c| &c.efforts)
        .collect();
    if let Some(e) = effort
        .as_deref()
        .filter(|e| !(efforts.iter().any(|x| x == e) || (efforts.is_empty() && emmet_agent::model_id(e))))
    {
        return Err(problem(format!("That model doesn't take the effort {e}.")));
    }
    let mut s = store(&project);
    let last = s.sessions.last_mut().expect("current made one");
    last.model = model;
    last.effort = effort;
    let out = last.clone();
    save_store(&project, &s)?;
    Ok(out)
}

/// Restart after a crash: the agent starts again on the same conversation. It sends nothing; the user's next message
/// is the next turn.
#[tauri::command]
#[specta::specta]
pub async fn chat_restart(app: AppHandle) -> Result<String, Box<Problem>> {
    blocking(app, |app| ensure_live(app, &open_project(app)?)).await
}

/// Undo turn (⌥⌘Z): restores the version saved before the agent's last turn that changed the sources. The restore
/// saves the state it replaces first, so Undo turn can be undone from Versions. Returns that version.
#[tauri::command]
#[specta::specta]
pub async fn chat_undo(app: AppHandle) -> Result<VersionView, Box<Problem>> {
    blocking(app, |app| {
        let state = app.state::<AppState>();
        let project = open_project(app)?;
        let mut s = store(&project);
        let n = s.undo.ok_or_else(|| problem("There's no agent turn to undo."))?;
        let r = versions::restore(&state.data, &project, n)?;
        s.undo = None;
        save_store(&project, &s)?;
        state.refresh_view(&Project::find(Some(&project.root))?);
        let _ = ChatUndo { undo: None }.emit(app);
        Ok(crate::projects::version_view(&project, &r.saved))
    })
    .await
}

/// The open project's conversations, the live session, and what Undo turn would restore.
#[tauri::command]
#[specta::specta]
pub fn chat_state(app: AppHandle) -> ChatState {
    let state = app.state::<AppState>();
    let Ok(project) = open_project(&app) else {
        return ChatState {
            live: None,
            sessions: vec![],
            agent: None,
            undo: None,
            models: vec![],
        };
    };
    let live = state.chat.live.lock().expect("live");
    let live = live
        .as_ref()
        .filter(|l| l.project == project.root)
        .map(|l| LiveSession {
            session: l.id.clone(),
            kind: l.kind,
        });
    let s = store(&project);
    let agent = s.sessions.last().map(|i| i.kind).or_else(|| default_agent(&app));
    ChatState {
        live,
        models: agent.map(|k| state.chat.models(k)).unwrap_or_default(),
        agent,
        undo: s.undo,
        sessions: s
            .sessions
            .into_iter()
            .map(|info| SessionView {
                records: history(&project.root, &info.id),
                info,
            })
            .collect(),
    }
}

/// Plan usage (D-36): fetched only when the user asks (opening the popover, ⟳, the end of a turn they started), at
/// most once a minute. Codex reports its billing state only.
#[tauri::command]
#[specta::specta]
pub async fn usage(app: AppHandle, kind: AgentKind) -> Option<UsageSnapshot> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let chat = &state.chat;
        if kind == AgentKind::Claude
            && let Some((when, snap)) = chat.usage.lock().expect("usage").as_ref()
            && when.elapsed() < USAGE_EVERY
        {
            return Some(snap.clone());
        }
        // A running Codex chat bills what it started with; the badge says that, not the saved choice.
        if let Some(billing) = chat.live_billing(kind) {
            return Some(UsageSnapshot::without_meters(kind, billing, None));
        }
        let snap = emmet_agent::adapter(kind, chat.env(), state.data.root()).usage().ok()?;
        if kind == AgentKind::Claude {
            *chat.usage.lock().expect("usage") = Some((Instant::now(), snap.clone()));
        }
        Some(snap)
    })
    .await
    .ok()
    .flatten()
}

/// Whether the keychain holds an OpenAI API key for Codex (the key itself never leaves the host).
#[tauri::command]
#[specta::specta]
pub fn openai_key_saved(app: AppHandle) -> bool {
    emmet_agent::keychain::has(app.state::<AppState>().chat.env()).unwrap_or(false)
}

/// Saves the OpenAI API key in the OS keychain (Settings → Agents), or removes it (`key` null).
#[tauri::command]
#[specta::specta]
pub fn openai_key_set(app: AppHandle, key: Option<String>) -> Result<bool, Box<Problem>> {
    let state = app.state::<AppState>();
    let env = state.chat.env();
    match key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty()) {
        Some(k) => emmet_agent::keychain::set(env, &k).map_err(agent_problem)?,
        None => emmet_agent::keychain::clear(env).map_err(agent_problem)?,
    }
    // Codex's sign-in changed: detect it again.
    state.chat.forget_agents();
    Ok(openai_key_saved(app.clone()))
}
