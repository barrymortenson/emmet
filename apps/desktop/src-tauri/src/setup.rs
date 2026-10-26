// SPDX-License-Identifier: AGPL-3.0-or-later
//! First run (plan section 14, until 4A designs it): when the LDraw library or Python isn't set up, the window offers
//! one Set up button, which runs the emmet sidecar's `library install` and `doctor --fix`, so a fresh install works
//! without a terminal.

use std::process::{Command, Stdio};

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::app::{AppState, Problem};

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
