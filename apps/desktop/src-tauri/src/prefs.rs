// SPDX-License-Identifier: AGPL-3.0-or-later
//! The app's own settings (Settings, session 4A), kept beside the BOM's color naming in the data dir's settings.json:
//! the projects folder (Documents/Emmet until changed), the appearance, the viewer's quality, the agent to start
//! with, and Claude Code's model. The window reads and changes them by name; the only path in them, the projects
//! folder, is picked in the host's own dialog, never sent by the window.

use std::path::PathBuf;

use emmet_core::data::DataDir;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::app::{AppState, Problem};

/// What the window may set. Each is checked against its choices.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Prefs {
    /// system, light, or dark.
    pub appearance: String,
    /// fast, balanced, or best.
    pub viewer_quality: String,
    /// The agent the chat starts with: claude or codex, or null for the last one used.
    pub start_with: Option<String>,
    /// Claude Code's model: null for its own default, or opus, sonnet, or haiku.
    pub claude_model: Option<String>,
}

/// Settings as the window shows them: the prefs, where new projects go (in words: "Documents/Emmet"), and the BOM's
/// color naming.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PrefsView {
    pub prefs: Prefs,
    pub projects_folder: String,
    pub color_names: String,
}

const APPEARANCE: [&str; 3] = ["system", "light", "dark"];
const QUALITY: [&str; 3] = ["fast", "balanced", "best"];
const AGENTS: [&str; 2] = ["claude", "codex"];
const CLAUDE_MODELS: [&str; 3] = ["opus", "sonnet", "haiku"];

fn read(data: &DataDir) -> Map<String, Value> {
    std::fs::read_to_string(data.settings())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// settings.json to change and write back: empty when there's none yet, and a problem when it can't be read or isn't
/// JSON, so a change never writes over the projects folder and color naming it couldn't see.
pub(crate) fn settings_to_change(data: &DataDir) -> Result<Map<String, Value>, Box<Problem>> {
    let path = data.settings();
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Map::new()),
        Err(e) => return Err(emmet_core::envelope::Error::io(&path, e).into()),
    };
    serde_json::from_str(&text).map_err(|_| {
        Box::new(Problem {
            code: "IO_ERROR".into(),
            message: "Emmet's settings file isn't readable, so Emmet didn't change it.".into(),
            hint: format!("Fix or remove {}, then try again.", path.display()),
            ..*Problem::internal("")
        })
    })
}

pub(crate) fn write(data: &DataDir, settings: &Map<String, Value>) -> Result<(), Box<Problem>> {
    let text = serde_json::to_string_pretty(settings).expect("json") + "\n";
    Ok(emmet_core::fsx::write_atomic(&data.settings(), text.as_bytes())?)
}

fn pick(settings: &Map<String, Value>, key: &str, choices: &[&str]) -> Option<String> {
    settings
        .get(key)
        .and_then(Value::as_str)
        .filter(|v| choices.contains(v))
        .map(String::from)
}

pub fn prefs(data: &DataDir) -> Prefs {
    let s = read(data);
    Prefs {
        appearance: pick(&s, "appearance", &APPEARANCE).unwrap_or_else(|| "system".into()),
        viewer_quality: pick(&s, "viewer_quality", &QUALITY).unwrap_or_else(|| "balanced".into()),
        start_with: pick(&s, "start_with", &AGENTS),
        claude_model: pick(&s, "claude_model", &CLAUDE_MODELS),
    }
}

/// Where new projects go: the folder the user picked, else Documents/Emmet.
pub fn projects_folder(app: &AppHandle) -> PathBuf {
    let data = &app.state::<AppState>().data;
    if let Some(dir) = read(data)
        .get("projects_folder")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        return PathBuf::from(dir);
    }
    // Never the temp folder, which the system empties: Documents in the home folder when the system can't say where
    // Documents is.
    let docs = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir().map(|h| h.join("Documents")));
    docs.unwrap_or_else(|_| PathBuf::from("Documents")).join("Emmet")
}

/// A folder in words, with the home folder left out: "Documents/Emmet".
pub fn in_words(app: &AppHandle, dir: &std::path::Path) -> String {
    let home = app.path().home_dir().ok();
    match home.as_deref().and_then(|h| dir.strip_prefix(h).ok()) {
        Some(rel) => rel.display().to_string(),
        None => dir.display().to_string(),
    }
}

fn view(app: &AppHandle) -> PrefsView {
    let data = &app.state::<AppState>().data;
    let names = serde_json::to_value(emmet_core::bom::ColorNames::preferred(data)).unwrap_or_default();
    PrefsView {
        prefs: prefs(data),
        projects_folder: in_words(app, &projects_folder(app)),
        color_names: names.as_str().unwrap_or("ldraw").to_string(),
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_prefs(app: AppHandle) -> PrefsView {
    view(&app)
}

#[tauri::command]
#[specta::specta]
pub fn set_prefs(app: AppHandle, prefs: Prefs) -> Result<PrefsView, Box<Problem>> {
    let bad = |what: &str, v: &str| {
        Box::new(Problem {
            code: "PARAM_INVALID".into(),
            message: format!("There's no {what} '{v}'."),
            ..*Problem::internal("")
        })
    };
    if !APPEARANCE.contains(&prefs.appearance.as_str()) {
        return Err(bad("appearance", &prefs.appearance));
    }
    if !QUALITY.contains(&prefs.viewer_quality.as_str()) {
        return Err(bad("viewer quality", &prefs.viewer_quality));
    }
    if let Some(a) = prefs.start_with.as_deref().filter(|a| !AGENTS.contains(a)) {
        return Err(bad("agent", a));
    }
    if let Some(m) = prefs.claude_model.as_deref().filter(|m| !CLAUDE_MODELS.contains(m)) {
        return Err(bad("Claude model", m));
    }
    let data = &app.state::<AppState>().data;
    let mut s = settings_to_change(data)?;
    s.insert("appearance".into(), prefs.appearance.into());
    s.insert("viewer_quality".into(), prefs.viewer_quality.into());
    s.insert("start_with".into(), prefs.start_with.map_or(Value::Null, Value::String));
    s.insert(
        "claude_model".into(),
        prefs.claude_model.map_or(Value::Null, Value::String),
    );
    write(data, &s)?;
    Ok(view(&app))
}

/// Settings → Projects → Change…: the host's folder dialog; null when the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn pick_projects_folder(app: AppHandle) -> Result<Option<PrefsView>, Box<Problem>> {
    let dialog = app.dialog().clone();
    let start = projects_folder(&app);
    let picked = tauri::async_runtime::spawn_blocking(move || {
        let mut d = dialog.file().set_title("New projects go in");
        if start.is_dir() {
            d = d.set_directory(&start);
        }
        d.set_can_create_directories(true).blocking_pick_folder()
    })
    .await
    .map_err(|e| Problem::internal(e.to_string()))?;
    let Some(dir) = picked.and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    let data = &app.state::<AppState>().data;
    let mut s = settings_to_change(data)?;
    s.insert("projects_folder".into(), dir.display().to_string().into());
    write(data, &s)?;
    Ok(Some(view(&app)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A settings file Emmet can't read is left as it is, not written over with only what changed.
    #[test]
    fn a_settings_file_that_isnt_json_isnt_written_over() {
        let dir = std::env::temp_dir().join(format!("emmet-prefs-{}", std::process::id()));
        let data = DataDir::at(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(settings_to_change(&data).unwrap().is_empty(), "none yet is empty");
        std::fs::write(data.settings(), "{\"projects_folder\": \"/p\",").unwrap();
        let e = settings_to_change(&data).unwrap_err();
        assert!(e.message.contains("isn't readable"), "{}", e.message);
        std::fs::write(data.settings(), "{\"projects_folder\": \"/p\"}").unwrap();
        assert_eq!(settings_to_change(&data).unwrap()["projects_folder"], "/p");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
