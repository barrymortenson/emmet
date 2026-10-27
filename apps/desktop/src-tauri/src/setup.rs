// SPDX-License-Identifier: AGPL-3.0-or-later
//! First run (plan section 14, until 4A designs it): when the LDraw library or Python isn't set up, the window offers
//! one Set up button, which runs the emmet sidecar's `library install` and `doctor --fix`, so a fresh install works
//! without a terminal. Then the agents (D-41, ADR 0012): Install runs the provider's own installer, Sign in runs
//! Claude Code's own `claude auth login`, both after the provider notice (D-37) has been shown once; and Antigravity's
//! terminal mode gets Emmet's MCP server in the project's `.agents/mcp_config.json` (D-42).

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Mutex;

use emmet_agent::AgentKind;
use emmet_agent::setup::{Cancel, SetupError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::app::{AppState, Problem};

/// The install or sign-in running for each agent, to cancel it.
#[derive(Default)]
pub struct Jobs(Mutex<HashMap<AgentKind, Cancel>>);

impl Jobs {
    fn start(&self, kind: AgentKind) -> Cancel {
        let cancel = Cancel::default();
        if let Some(old) = self.0.lock().expect("jobs").insert(kind, cancel.clone()) {
            old.cancel();
        }
        cancel
    }

    fn end(&self, kind: AgentKind) {
        self.0.lock().expect("jobs").remove(&kind);
    }
}

/// A line an installer printed, for the window's progress.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct SetupEvent {
    pub kind: AgentKind,
    pub line: String,
}

/// Runs `job` for `kind` on a blocking thread, cancelable with `agent_cancel`; detection runs again after it.
async fn job<T: Send + 'static>(
    app: &AppHandle,
    kind: AgentKind,
    job: impl FnOnce(&AppState, &Cancel) -> Result<T, SetupError> + Send + 'static,
) -> Result<(), SetupError> {
    let handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        let cancel = state.jobs.start(kind);
        let r = job(&state, &cancel);
        state.jobs.end(kind);
        state.chat.forget_agents();
        r.map(|_| ())
    })
    .await;
    result.unwrap_or_else(|e| {
        Err(SetupError {
            failure: emmet_agent::setup::Failure::Failed,
            message: e.to_string(),
            detail: None,
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

/// The provider notice (D-37) was shown: true once the user has seen it.
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
/// `/login`), and where Antigravity's terminal mode runs. With `in_project`, it opens in the open project's folder,
/// never one the window names.
#[tauri::command]
#[specta::specta]
pub fn open_terminal(app: AppHandle, in_project: bool) -> Result<(), Box<Problem>> {
    let dir = if in_project {
        Some(
            app.state::<AppState>()
                .open_root()
                .ok_or_else(|| Problem::internal("Open a project first."))?,
        )
    } else {
        None
    };
    // The folder is the working directory, never an argument: cmd.exe would parse a name with & or ^ in it. Both
    // programs go by full path, since `start` would look for powershell.exe in that folder first.
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
        if let Some(d) = &dir {
            c.arg(d);
        }
        c
    };
    if let Some(d) = &dir {
        cmd.current_dir(d);
    }
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    crate::runs::no_window(&mut cmd);
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| Problem::internal(format!("Emmet couldn't open a terminal: {e}")))
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

/// What the Antigravity card shows (D-42): where agy is, and whether the open project has Emmet's MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AntigravityView {
    pub found: emmet_agent::setup::Antigravity,
    pub configured: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn antigravity(app: AppHandle) -> AntigravityView {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let configured = state.open_root().is_some_and(|root| {
            std::fs::read(emmet_agent::setup::antigravity_config(&root))
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .is_some_and(|d| d["mcpServers"]["emmet"].is_object())
        });
        AntigravityView {
            found: emmet_agent::setup::antigravity(state.chat.env()),
            configured,
        }
    })
    .await
    .unwrap_or(AntigravityView {
        found: emmet_agent::setup::Antigravity {
            path: None,
            version: None,
        },
        configured: false,
    })
}

/// Adds Emmet's MCP server to the open project's `.agents/mcp_config.json`, for agy run in that folder. The project
/// is the open one, never a path from the window.
#[tauri::command]
#[specta::specta]
pub fn antigravity_set_up(app: AppHandle) -> Result<(), Box<Problem>> {
    let state = app.state::<AppState>();
    let root = state
        .open_root()
        .ok_or_else(|| Problem::internal("Open a project first."))?;
    emmet_agent::setup::write_antigravity_config(&root, &state.emmet)
        .map(|_| ())
        .map_err(|e| Problem::internal(e.message))
}

/// Installs the library (when it's missing) and sets Python up, then rebuilds the open project.
#[tauri::command]
#[specta::specta]
pub async fn setup(app: AppHandle) -> Result<(), Box<Problem>> {
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        if emmet_core::library::require(&state.data).is_err() {
            sidecar(&state, &["library", "install"])?;
        }
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
