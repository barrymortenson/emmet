// SPDX-License-Identifier: AGPL-3.0-or-later
//! First run (plan section 14, session 4A): the LDraw library downloads in the host, with its progress in the window
//! and a resume on Try again, then the emmet sidecar's `doctor --fix` sets Python up, so a fresh install works without
//! a terminal. "Update the library" (an old library's banner) runs the same download. Then the agents (D-41, ADR 0012): Install runs the provider's own installer and Sign in runs
//! Claude Code's own `claude auth login`; the window asks for the provider notice (D-37) first, until the user accepts
//! it.

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Mutex;

use emmet_agent::AgentKind;
use emmet_agent::setup::{Cancel, Failure, SetupError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::app::{AppState, Problem};

/// The install or sign-in running for each agent, to cancel it. One at a time per agent: the start screen and the
/// picker each have a card, and a second click mustn't end the first job or take its Cancel.
#[derive(Default)]
pub struct Jobs(Mutex<HashMap<AgentKind, Cancel>>);

impl Jobs {
    fn start(&self, kind: AgentKind) -> Result<Cancel, SetupError> {
        let mut jobs = self.0.lock().expect("jobs");
        if jobs.contains_key(&kind) {
            return Err(SetupError {
                failure: Failure::Busy,
                message: format!(
                    "Emmet is already installing or signing in to {}. Wait for it to finish, or cancel it.",
                    kind.label()
                ),
                detail: None,
            });
        }
        let cancel = Cancel::default();
        jobs.insert(kind, cancel.clone());
        Ok(cancel)
    }

    /// Forgets `cancel`'s job, and only that one.
    fn end(&self, kind: AgentKind, cancel: &Cancel) {
        let mut jobs = self.0.lock().expect("jobs");
        if jobs.get(&kind).is_some_and(|c| c.same(cancel)) {
            jobs.remove(&kind);
        }
    }
}

/// A line an installer printed, for the window's progress.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct SetupEvent {
    pub kind: AgentKind,
    pub line: String,
}

/// Runs `job` for `kind` on a blocking thread, cancelable with `agent_cancel`. Afterwards, even if it crashed, the
/// cached agent list is dropped so the window's next `agents` call detects again.
async fn job<T: Send + 'static>(
    app: &AppHandle,
    kind: AgentKind,
    job: impl FnOnce(&AppState, &Cancel) -> Result<T, SetupError> + Send + 'static,
) -> Result<(), SetupError> {
    let cancel = app.state::<AppState>().jobs.start(kind)?;
    let (handle, c) = (app.clone(), cancel.clone());
    let result = tauri::async_runtime::spawn_blocking(move || job(&handle.state::<AppState>(), &c).map(|_| ())).await;
    let state = app.state::<AppState>();
    state.jobs.end(kind, &cancel);
    state.chat.forget_agents();
    result.unwrap_or_else(|e| {
        Err(SetupError {
            failure: Failure::Failed,
            message: "Something went wrong inside Emmet while setting up the agent. Try again.".into(),
            detail: Some(e.to_string()),
        })
    })
}

/// Installs Claude Code or Codex with the provider's official installer (D-41), streaming its lines as SetupEvents.
#[tauri::command]
#[specta::specta]
pub async fn agent_install(app: AppHandle, kind: AgentKind) -> Result<(), SetupError> {
    let events = app.clone();
    job(&app, kind, move |state, cancel| {
        let env = state.chat.env();
        let has_key = emmet_agent::keychain::has(env).unwrap_or(false);
        emmet_agent::setup::install(kind, env, has_key, cancel, &mut |line| {
            let _ = SetupEvent {
                kind,
                line: line.to_string(),
            }
            .emit(&events);
        })
    })
    .await
}

/// Signs in to Claude Code with its own `claude auth login`: with a Claude plan, or `console` for a Claude Console
/// account. Waits for the browser; `agent_cancel` stops it.
#[tauri::command]
#[specta::specta]
pub async fn claude_sign_in(app: AppHandle, console: bool) -> Result<(), SetupError> {
    job(&app, AgentKind::Claude, move |state, cancel| {
        emmet_agent::setup::sign_in(state.chat.env(), console, cancel)
    })
    .await
}

/// Stops `kind`'s install or sign-in.
#[tauri::command]
#[specta::specta]
pub fn agent_cancel(app: AppHandle, kind: AgentKind) {
    if let Some(c) = app.state::<AppState>().jobs.0.lock().expect("jobs").get(&kind) {
        c.cancel();
    }
}

/// Whether the user has accepted the provider notice (D-37) with Continue.
#[tauri::command]
#[specta::specta]
pub fn provider_notice(app: AppHandle) -> bool {
    emmet_agent::setup::notice_seen(app.state::<AppState>().data.root())
}

#[tauri::command]
#[specta::specta]
pub fn provider_notice_seen(app: AppHandle) -> Result<(), Box<Problem>> {
    emmet_agent::setup::see_notice(app.state::<AppState>().data.root()).map_err(|e| Problem::internal(e.message))
}

/// A terminal: the fallback when Claude Code's sign-in can't finish in the browser (the user runs `claude`, then
/// `/login`). On Windows, cmd.exe and powershell.exe go by full path, so PATH can't put another one first.
#[tauri::command]
#[specta::specta]
pub async fn open_terminal() -> Result<(), Box<Problem>> {
    let mut cmd = if cfg!(windows) {
        let system = std::path::Path::new(&std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()))
            .join("System32");
        let mut c = Command::new(system.join("cmd.exe"));
        c.args(["/c", "start", ""])
            .arg(system.join("WindowsPowerShell\\v1.0\\powershell.exe"));
        c
    } else {
        let mut c = Command::new("/usr/bin/open");
        c.args(["-a", "Terminal"]);
        c
    };
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    crate::runs::no_window(&mut cmd);
    // Both launchers return as soon as the terminal opens, so their status says whether it did.
    let status = tauri::async_runtime::spawn_blocking(move || cmd.status())
        .await
        .map_err(|e| Problem::internal(e.to_string()))?;
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(Problem::internal(format!("Emmet couldn't open a terminal ({s})."))),
        Err(e) => Err(Problem::internal(format!("Emmet couldn't open a terminal: {e}"))),
    }
}

/// The provider's own page saying which plans include its CLI (D-41), in the browser.
#[tauri::command]
#[specta::specta]
pub fn open_plans(app: AppHandle, kind: AgentKind) -> Result<(), Box<Problem>> {
    let url = match kind {
        AgentKind::Claude => emmet_agent::setup::CLAUDE_PLANS,
        AgentKind::Codex => emmet_agent::setup::CODEX_PLANS,
    };
    emmet_agent::chatgpt::open_url(app.state::<AppState>().chat.env(), url).map_err(|e| Problem::internal(e.message))
}

/// What first run is doing: the library download (`done` of `total` bytes), then Python (no amounts).
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct PartsProgress {
    /// library or python.
    pub step: String,
    pub done: u64,
    pub total: Option<u64>,
}

/// The library this Emmet installs: its release, sha256, and URL. A debug build takes E2E's local fixture server from
/// EMMET_LIBRARY_URL, EMMET_LIBRARY_SHA256, and EMMET_LIBRARY_RELEASE, so a test never downloads the real library; a
/// release build ignores them.
pub fn library_pin() -> (String, String, String) {
    if cfg!(debug_assertions)
        && let (Ok(url), Ok(sha), Ok(release)) = (
            std::env::var("EMMET_LIBRARY_URL"),
            std::env::var("EMMET_LIBRARY_SHA256"),
            std::env::var("EMMET_LIBRARY_RELEASE"),
        )
    {
        return (release, sha, url);
    }
    use emmet_core::library::{RELEASE, SHA256, URL};
    (RELEASE.into(), SHA256.into(), URL.into())
}

/// Whether the installed library is an older release than the pinned one (releases are YYYY-MM, so they sort as
/// text; a test library's name isn't a date and is never old).
pub fn library_old(installed: &str) -> bool {
    let (pinned, _, _) = library_pin();
    let dated =
        |r: &str| r.len() == 7 && r.as_bytes()[4] == b'-' && r.chars().filter(char::is_ascii_digit).count() == 6;
    dated(installed) && dated(&pinned) && installed < pinned.as_str()
}

/// One setup at a time: the first-run screen and the old library's banner share the download.
static SETTING_UP: Mutex<()> = Mutex::new(());

/// Installs the library when it's missing or older than the pinned one, with its progress as [`PartsProgress`]
/// events (a stopped download picks up where it stopped next time), sets Python up, then rebuilds the open project.
#[tauri::command]
#[specta::specta]
pub async fn setup(app: AppHandle) -> Result<(), Box<Problem>> {
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Ok(_one) = SETTING_UP.try_lock() else {
            return Err(Problem::internal("Emmet is already setting up the parts library."));
        };
        let state = handle.state::<AppState>();
        let installed = emmet_core::library::require(&state.data).ok();
        if installed.as_ref().is_none_or(|i| library_old(&i.release)) {
            let (release, sha, url) = library_pin();
            let mut last = std::time::Instant::now() - std::time::Duration::from_secs(1);
            let mut report = |done: u64, total: Option<u64>| {
                // A few times a second is enough for a progress bar, and the window isn't flooded.
                if last.elapsed().as_millis() >= 100 || total == Some(done) {
                    last = std::time::Instant::now();
                    let _ = PartsProgress {
                        step: "library".into(),
                        done,
                        total,
                    }
                    .emit(&handle);
                }
            };
            emmet_core::library::install(
                &state.data,
                &release,
                &sha,
                emmet_core::library::Source::Url(&url),
                &mut report,
            )?;
        }
        let _ = PartsProgress {
            step: "python".into(),
            done: 0,
            total: None,
        }
        .emit(&handle);
        sidecar(&state, &["doctor", "--fix"])?;
        if let Some(root) = state.open_root() {
            state.request_run(&root);
        }
        Ok(())
    })
    .await
    .map_err(|e| Problem::internal(e.to_string()))?
}

/// Runs `emmet --json --data-dir <data> <args>` and turns a failed envelope into the UI's problem.
fn sidecar(state: &AppState, args: &[&str]) -> Result<(), Box<Problem>> {
    let mut cmd = Command::new(&state.emmet);
    cmd.arg("--json")
        .arg("--data-dir")
        .arg(state.data.root())
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    crate::runs::no_window(&mut cmd);
    let out = cmd
        .output()
        .map_err(|e| Problem::internal(format!("Can't start {}: {e}", state.emmet.display())))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let env: Value = text
        .lines()
        .rev()
        .find_map(|l| serde_json::from_str(l).ok())
        .unwrap_or_default();
    // doctor --fix answers ok with a list of checks; it's set up when it says ready.
    let ready = env["ok"] == true && (env["data"]["ready"].is_null() || env["data"]["ready"] == true);
    if ready {
        return Ok(());
    }
    if env["ok"] == false {
        return Err(Box::new(Problem::from_envelope(&env["error"])));
    }
    let failed: Vec<String> = env["data"]["checks"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c["ok"] == false)
        .map(|c| {
            format!(
                "{}: {}",
                c["name"].as_str().unwrap_or(""),
                c["detail"].as_str().unwrap_or("")
            )
        })
        .collect();
    Err(Problem::internal(format!(
        "Setup didn't finish: {}.",
        failed.join("; ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The start screen and the picker each have a card: a second start while one runs is refused, and only the job
    /// that's ending can take its own entry away.
    #[test]
    fn one_job_per_agent_and_each_ends_only_itself() {
        let jobs = Jobs::default();
        let first = jobs.start(AgentKind::Claude).expect("starts");
        let Err(busy) = jobs.start(AgentKind::Claude) else {
            panic!("a second start while one runs")
        };
        assert_eq!(busy.failure, Failure::Busy);
        assert!(!first.canceled(), "a second click doesn't end the first job");
        assert!(jobs.start(AgentKind::Codex).is_ok(), "the other agent is its own");

        jobs.end(AgentKind::Claude, &Cancel::default());
        assert!(
            jobs.start(AgentKind::Claude).is_err(),
            "another job's end left this one alone"
        );
        jobs.end(AgentKind::Claude, &first);
        let next = jobs.start(AgentKind::Claude).expect("free again");
        jobs.end(AgentKind::Claude, &first);
        assert!(
            jobs.0.lock().unwrap()[&AgentKind::Claude].same(&next),
            "a late end of the old job keeps the new"
        );
    }
}
