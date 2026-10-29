// SPDX-License-Identifier: AGPL-3.0-or-later
//! Settings → Emmet (session 4A): Install command line tool, which puts the bundled `emmet` on PATH (D-08) the way
//! Barry chose (docs/decisions.md, 2026-10-28): on macOS a link at /usr/local/bin/emmet, made after macOS's own
//! administrator prompt; on Windows the app's folder on the user's PATH, no admin. Copy diagnostics: the versions,
//! the system, and what's set up, with the home folder written as ~ and no account, key, or token (6B adds the logs
//! and a bundle). Report a problem opens the repo's new-issue page.

use tauri::{AppHandle, Manager};

use crate::app::{AppState, Problem};

/// Where macOS's link goes.
const MAC_LINK: &str = "/usr/local/bin/emmet";
const ISSUES: &str = "https://github.com/barrymortenson/emmet/issues/new";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct CliStatus {
    /// `emmet` on PATH points at this app's copy.
    pub installed: bool,
    /// Where it goes, in words: "/usr/local/bin" or "your PATH".
    pub place: String,
}

fn status(emmet: &std::path::Path) -> CliStatus {
    if cfg!(windows) {
        let dir = emmet.parent().map(|d| d.display().to_string()).unwrap_or_default();
        let path = user_path().unwrap_or_default();
        CliStatus {
            installed: path.split(';').any(|p| {
                p.trim_end_matches('\\')
                    .eq_ignore_ascii_case(dir.trim_end_matches('\\'))
            }),
            place: "your PATH".into(),
        }
    } else {
        CliStatus {
            installed: std::fs::read_link(MAC_LINK).is_ok_and(|l| l == emmet),
            place: "/usr/local/bin".into(),
        }
    }
}

/// The user's own PATH from the registry (the one new terminals get), not this process's.
fn user_path() -> Option<String> {
    let mut cmd = std::process::Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "[Environment]::GetEnvironmentVariable('Path','User')",
    ]);
    crate::runs::no_window(&mut cmd);
    let out = cmd.output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[tauri::command]
#[specta::specta]
pub fn cli_status(state: tauri::State<'_, AppState>) -> CliStatus {
    status(&state.emmet)
}

/// A string as an AppleScript literal: backslashes and double quotes escaped.
fn applescript(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Puts `emmet` on PATH. Null when the user cancels macOS's password prompt.
#[tauri::command]
#[specta::specta]
pub async fn install_cli(app: AppHandle) -> Result<Option<CliStatus>, Box<Problem>> {
    let emmet = app.state::<AppState>().emmet.clone();
    if !emmet.is_file() {
        return Err(Problem::internal(format!(
            "Emmet's command line tool isn't at {}.",
            emmet.display()
        )));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let out = if cfg!(windows) {
            let dir = emmet.parent().map(|d| d.display().to_string()).unwrap_or_default().replace('\'', "''");
            let script = format!(
                "$d = '{dir}'; $p = [Environment]::GetEnvironmentVariable('Path','User'); \
                 if (-not (($p -split ';') -contains $d)) {{ [Environment]::SetEnvironmentVariable('Path', (($p.TrimEnd(';') + ';' + $d).TrimStart(';')), 'User') }}"
            );
            let mut cmd = std::process::Command::new("powershell");
            cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
            crate::runs::no_window(&mut cmd);
            cmd.output()
        } else {
            // macOS asks for the password itself; Emmet never sees it.
            let script = format!(
                "do shell script \"mkdir -p /usr/local/bin && ln -sfn \" & quoted form of {} & \" {MAC_LINK}\" with administrator privileges",
                applescript(&emmet.display().to_string())
            );
            std::process::Command::new("osascript").args(["-e", &script]).output()
        }
        .map_err(|e| Problem::internal(format!("Couldn't run the installer: {e}")))?;
        if !out.status.success() {
            let said = String::from_utf8_lossy(&out.stderr);
            // osascript's "User canceled. (-128)".
            if said.contains("-128") {
                return Ok(None);
            }
            return Err(Problem::internal(format!("The command line tool wasn't installed: {}", said.trim())));
        }
        Ok(Some(status(&emmet)))
    })
    .await
    .map_err(|e| Problem::internal(e.to_string()))?
}

/// The facts a bug report needs, as plain text for the clipboard.
#[tauri::command]
#[specta::specta]
pub async fn diagnostics(app: AppHandle) -> String {
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        let data = &state.data;
        let library = emmet_core::library::require(data).ok().map(|i| i.release);
        let parts = emmet_core::parts_data::version(data).ok().flatten();
        let mut lines = vec![
            format!(
                "Emmet {}{}",
                env!("CARGO_PKG_VERSION"),
                if cfg!(debug_assertions) { " (debug build)" } else { "" }
            ),
            format!("System: {} {}", std::env::consts::OS, std::env::consts::ARCH),
            format!(
                "LDraw library: {} (this Emmet pins {})",
                library.as_deref().unwrap_or("not installed"),
                crate::setup::library_pin().0
            ),
            format!("Parts data: {}", parts.as_deref().unwrap_or("none")),
            format!(
                "Python: {}",
                if emmet_core::python::status(data).ready {
                    "ready"
                } else {
                    "not set up"
                }
            ),
        ];
        for a in crate::chat::agents_now(&handle) {
            let d = &a.detection;
            lines.push(match (&d.path, &d.version) {
                (None, _) => format!("{}: not installed", a.name),
                (Some(_), v) => format!(
                    "{} {}: {}{}",
                    a.name,
                    v.as_deref().unwrap_or("(version unknown)"),
                    if d.signed_in { "signed in" } else { "not signed in" },
                    if d.supported { "" } else { ", untested version" }
                ),
            });
        }
        let home = handle.path().home_dir().ok();
        let words = |p: &std::path::Path| match home.as_deref().and_then(|h| p.strip_prefix(h).ok()) {
            Some(rel) => format!("~/{}", rel.display()),
            None => p.display().to_string(),
        };
        lines.push(format!("App data: {}", words(data.root())));
        lines.push(format!(
            "Projects folder: {}",
            words(&crate::prefs::projects_folder(&handle))
        ));
        lines.join("\n") + "\n"
    })
    .await
    .unwrap_or_default()
}

/// Report a problem…: the repo's new-issue page in the browser.
#[tauri::command]
#[specta::specta]
pub fn report_problem(state: tauri::State<'_, AppState>) -> Result<(), Box<Problem>> {
    emmet_agent::chatgpt::open_url(state.chat.env(), ISSUES).map_err(|e| Problem::internal(e.message))
}
