// SPDX-License-Identifier: AGPL-3.0-or-later
//! The chat pane's host side (plan sections 04 and 10): one agent session per project at a time, its events to the
//! window, approvals and questions from the bus to the hub, the chat history in the project's `.emmet/chat/`, the
//! agents' detection, the plan usage fetch (D-36), and the Codex API key in the keychain (D-35). Sign in with ChatGPT's
//! commands are in `chatgpt.rs`.
//!
//! Every turn starts from something the user did (plan section 10): a message, an answer, Resume, or a redirect.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use emmet_agent::{
    AgentEvent, AgentKind, Answer, Billing, Decision, Detection, Hub, Session, SessionConfig, UsageSnapshot, UserNote,
    UserTurn,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::app::{AppState, Problem};

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

/// One line of a chat's history: something the user sent, or an agent event.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ChatRecord {
    pub at: f64,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub event: Option<AgentEvent>,
}

/// The project's last chat session, kept in `.emmet/chat.json` so it can be resumed.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LastSession {
    pub kind: AgentKind,
    pub session: String,
    pub resume: Option<String>,
}

/// The chat that's running.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LiveSession {
    pub session: String,
    pub kind: AgentKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ChatState {
    /// The session that's running, if one is.
    pub live: Option<LiveSession>,
    pub last: Option<LastSession>,
    pub history: Vec<ChatRecord>,
}

struct Live {
    project: PathBuf,
    id: String,
    kind: AgentKind,
    /// The ChatGPT account Codex runs on, when it's on the user's plan.
    plan: Option<String>,
    session: Box<dyn Session>,
}

pub struct Chat {
    pub hub: Hub,
    env: OnceLock<Vec<(String, String)>>,
    agents: Mutex<Option<Vec<Detection>>>,
    live: Mutex<Option<Live>>,
    usage: Mutex<Option<(Instant, UsageSnapshot)>>,
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

/// The project's last session, if `.emmet/chat.json` is one this app wrote. The file comes with the project, which may
/// be someone else's: an id that isn't ours, or a resume token that isn't a plain id, is ignored.
fn last_session(project: &Path) -> Option<LastSession> {
    let last: LastSession =
        serde_json::from_str(&std::fs::read_to_string(project.join(".emmet").join("chat.json")).ok()?).ok()?;
    let ours = last.session.len() == 33
        && last.session.starts_with('c')
        && last.session[1..].bytes().all(|b| b.is_ascii_hexdigit());
    let token_ok = last.resume.as_deref().is_none_or(emmet_agent::plain_id);
    (ours && token_ok).then_some(last)
}

fn save_last(project: &Path, last: &LastSession) {
    let _ = emmet_core::fsx::write_atomic(
        &project.join(".emmet").join("chat.json"),
        serde_json::to_string_pretty(last).expect("json").as_bytes(),
    );
}

fn history(project: &Path, session: &str) -> Vec<ChatRecord> {
    std::fs::read_to_string(chat_dir(project).join(format!("{session}.jsonl")))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn open_project(app: &AppHandle) -> Result<PathBuf, Box<Problem>> {
    app.state::<AppState>()
        .open_root()
        .ok_or_else(|| problem("Open a project first."))
}

/// Runs `f` on the live session of the open project.
fn with_live<T>(app: &AppHandle, f: impl FnOnce(&mut Live) -> Result<T, Box<Problem>>) -> Result<T, Box<Problem>> {
    let root = open_project(app)?;
    let state = app.state::<AppState>();
    let mut live = state.chat.live.lock().expect("live");
    match live.as_mut() {
        Some(l) if l.project == root => f(l),
        _ => Err(Box::new(Problem {
            hint: "Pick an agent to start a chat.".into(),
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

/// Starts a chat with `kind` on the open project, ending any chat that's running. With `resume`, it continues the
/// project's last session for that agent.
#[tauri::command]
#[specta::specta]
pub async fn chat_start(app: AppHandle, kind: AgentKind, resume: bool) -> Result<String, Box<Problem>> {
    let root = open_project(&app)?;
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || start(&handle, root, kind, resume))
        .await
        .map_err(|e| problem(e.to_string()))?
}

fn start(app: &AppHandle, root: PathBuf, kind: AgentKind, resume: bool) -> Result<String, Box<Problem>> {
    let state = app.state::<AppState>();
    state.chat.close();
    let last = last_session(&root).filter(|l| l.kind == kind);
    let id = match (&last, resume) {
        (Some(l), true) => l.session.clone(),
        _ => format!("c{}", emmet_core::fsx::random_hex(2)),
    };
    let cfg = SessionConfig {
        id: id.clone(),
        project: root.clone(),
        data: state.data.root().to_path_buf(),
        emmet: state.emmet.clone(),
        resume: if resume {
            last.as_ref().and_then(|l| l.resume.clone())
        } else {
            None
        },
        // Settings' model for Claude Code (its own alias: opus, sonnet, or haiku); Codex uses its own default until 4B.
        model: match kind {
            AgentKind::Claude => crate::prefs::prefs(&state.data).claude_model,
            AgentKind::Codex => None,
        },
        env: state.chat.env().to_vec(),
        hub: state.chat.hub.clone(),
        native_steer: true,
        openai_key: None,
    };
    let adapter = emmet_agent::adapter(kind, &cfg.env, state.data.root());
    let plan = match emmet_agent::chatgpt::ChatGpt::new(state.data.root(), &cfg.env).codex_use() {
        emmet_agent::chatgpt::CodexUse::Plan { id, .. } if kind == AgentKind::Codex => Some(id),
        _ => None,
    };
    let mut session = adapter.start(cfg).map_err(agent_problem)?;
    let events = session.events();
    save_last(
        &root,
        &LastSession {
            kind,
            session: id.clone(),
            resume: last.filter(|_| resume).and_then(|l| l.resume),
        },
    );
    *state.chat.live.lock().expect("live") = Some(Live {
        project: root.clone(),
        id: id.clone(),
        kind,
        plan,
        session,
    });
    let (handle, sid) = (app.clone(), id.clone());
    std::thread::spawn(move || pump(&handle, &root, &sid, kind, events));
    Ok(id)
}

/// The session's events, to the window and the history, until the CLI exits.
fn pump(app: &AppHandle, root: &Path, id: &str, kind: AgentKind, events: emmet_agent::EventStream) {
    let state = app.state::<AppState>();
    for mut event in events {
        let at = now();
        if let AgentEvent::Started { resume, billing, .. } = &mut event {
            // The plan's name comes from detection; init says only that it's a plan.
            if let Some(Billing::Plan { name }) = billing
                && name.is_none()
            {
                *name = state.chat.plan_name(kind);
            }
            save_last(
                root,
                &LastSession {
                    kind,
                    session: id.to_string(),
                    resume: Some(resume.clone()),
                },
            );
        }
        // Text arrives in many small deltas; the history keeps them, so a reopened chat reads the same.
        append(
            root,
            id,
            &ChatRecord {
                at,
                user: None,
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

/// Runs `f` off the main thread: a turn on the ChatGPT plan may first renew the token and restart Codex (1E), which
/// would freeze the window if a command ran it on the main thread, as Tauri does with a command that isn't async.
async fn blocking<T: Send + 'static>(
    app: AppHandle,
    f: impl FnOnce(&AppHandle) -> Result<T, Box<Problem>> + Send + 'static,
) -> Result<T, Box<Problem>> {
    tauri::async_runtime::spawn_blocking(move || f(&app))
        .await
        .map_err(|e| problem(e.to_string()))?
}

/// A message from the user: a new turn, or (while the agent works) a redirect.
#[tauri::command]
#[specta::specta]
pub async fn chat_send(app: AppHandle, text: String) -> Result<(), Box<Problem>> {
    blocking(app, move |app| send(app, text)).await
}

fn send(app: &AppHandle, text: String) -> Result<(), Box<Problem>> {
    with_live(app, |l| {
        append(
            &l.project,
            &l.id,
            &ChatRecord {
                at: now(),
                user: Some(text.clone()),
                event: None,
            },
        );
        l.session.send(UserTurn { text }).map_err(agent_problem)
    })
}

/// A redirect while the agent works (D-33): returns the note, which `NoteDelivered` marks once it's out.
#[tauri::command]
#[specta::specta]
pub async fn chat_steer(app: AppHandle, text: String) -> Result<UserNote, Box<Problem>> {
    blocking(app, move |app| steer(app, text)).await
}

fn steer(app: &AppHandle, text: String) -> Result<UserNote, Box<Problem>> {
    with_live(app, |l| {
        let note = l.session.steer(&text).map_err(agent_problem)?;
        append(
            &l.project,
            &l.id,
            &ChatRecord {
                at: now(),
                user: Some(text),
                event: None,
            },
        );
        Ok(note)
    })
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

/// Ends the running chat (New chat). Its history stays in the project's .emmet/.
#[tauri::command]
#[specta::specta]
pub fn chat_end(app: AppHandle) {
    app.state::<AppState>().chat.close();
}

/// The open project's chat: the live session, the last one (to resume), and its history.
#[tauri::command]
#[specta::specta]
pub fn chat_state(app: AppHandle) -> ChatState {
    let Some(root) = app.state::<AppState>().open_root() else {
        return ChatState {
            live: None,
            last: None,
            history: vec![],
        };
    };
    let state = app.state::<AppState>();
    let live = state.chat.live.lock().expect("live");
    let live = live.as_ref().filter(|l| l.project == root).map(|l| LiveSession {
        session: l.id.clone(),
        kind: l.kind,
    });
    let last = last_session(&root);
    let history = live
        .as_ref()
        .map(|l| l.session.clone())
        .or(last.as_ref().map(|l| l.session.clone()))
        .map(|id| history(&root, &id))
        .unwrap_or_default();
    ChatState { live, last, history }
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
        // A running Codex chat bills what it started with; the status bar says that, not the saved choice.
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
