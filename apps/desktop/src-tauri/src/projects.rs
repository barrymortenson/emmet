// SPDX-License-Identifier: AGPL-3.0-or-later
//! The project commands of the frozen design (session 4A): New without a dialog into the projects folder, Open… (one
//! file picker: a project's emmet.toml, or an LDraw file to import, docs/decisions.md 2026-10-28), Trust this
//! project, the build settings, allowed findings, a knob's Keep and Reset, versions, and exports. The window names no
//! path: the host picks files in its own dialogs, makes folders in the projects folder, and reveals the last export
//! itself.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use emmet_core::project::{BuildConfig, Project, Template};
use emmet_core::versions;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::app::{AppState, ParamsView, Problem, ProjectView, Scene};

/// The build settings (plan section 14), each null until set ("Any").
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct BuildSettings {
    pub base: Option<String>,
    pub scale: Option<String>,
    pub age: Option<String>,
    pub detail: Option<String>,
    pub pieces: Option<String>,
    pub style: Option<String>,
}

impl From<&BuildConfig> for BuildSettings {
    fn from(b: &BuildConfig) -> Self {
        Self {
            base: b.base.clone(),
            scale: b.scale.clone(),
            age: b.age.clone(),
            detail: b.detail.clone(),
            pieces: b.pieces.clone(),
            style: b.style.clone(),
        }
    }
}

/// Each setting's choices, as the frozen design names them (docs/design/shared/data.js). Style also takes the user's
/// own words.
pub const BASES: [&str; 4] = ["None", "16 × 16", "32 × 32", "48 × 48"];
pub const SCALES: [&str; 6] = ["Micro", "Micropolis", "Midi", "Figure scale", "Large", "Car 1:8"];
pub const AGES: [&str; 5] = ["4+", "6+", "9+", "13+", "18+"];
pub const DETAILS: [&str; 3] = ["Low", "Medium", "High"];
pub const PIECES: [&str; 4] = ["Small", "Medium", "Large", "Huge"];
/// The longest style the user may type.
const STYLE_MAX: usize = 60;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum NewKind {
    /// The Cottage example.
    Cottage,
    /// An empty baseplate, for a new model from a prompt.
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VersionView {
    pub n: u32,
    pub name: String,
    /// Seconds since 1970, UTC.
    pub time: u64,
    pub auto: bool,
    pub parts: Option<u32>,
    /// A data: URL of its thumbnail, when it has one.
    pub thumbnail: Option<String>,
    /// Whether it can be opened to compare (it has the build's placements).
    pub viewable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    Csv,
    Rebrickable,
    BricklinkXml,
    Mpd,
    Lpub3d,
}

/// What an export wrote: the file's name and where, in words ("Downloads").
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Exported {
    pub file: String,
    pub folder: String,
}

/// The last export's path, for Show in folder.
static LAST_EXPORT: Mutex<Option<PathBuf>> = Mutex::new(None);

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, Box<Problem>> + Send + 'static,
) -> Result<T, Box<Problem>> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| Problem::internal(e.to_string()))?
}

fn bad(message: impl Into<String>, hint: impl Into<String>) -> Box<Problem> {
    Box::new(Problem {
        code: "PARAM_INVALID".into(),
        hint: hint.into(),
        ..*Problem::internal(message)
    })
}

/// A name as one plain file or folder name: no separators, no characters Windows refuses, no control characters, and
/// no leading dots (so never "." or ".."). A project's name comes from its emmet.toml, which a shared project writes.
fn file_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_control() || "/\\:*?\"<>|".contains(c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let words = clean.split_whitespace().collect::<Vec<_>>().join(" ");
    let clean = words.trim_start_matches(['.', ' ']);
    if clean.is_empty() {
        "Model".into()
    } else {
        clean.to_string()
    }
}

/// A folder in `parent` named `base`, or "`base` 2", "`base` 3", … when that's taken.
fn unique(parent: &Path, base: &str) -> PathBuf {
    let clean = file_name(base);
    (1..)
        .map(|n| parent.join(if n == 1 { clean.clone() } else { format!("{clean} {n}") }))
        .find(|p| !p.exists())
        .expect("a free name")
}

/// Makes a project in the projects folder named `name` (or the next free "`name` 2"), with `template`, named as its
/// folder is, and opens it.
async fn create(app: AppHandle, name: &str, template: Template) -> Result<ProjectView, Box<Problem>> {
    let folder = crate::prefs::projects_folder(&app);
    let dir = unique(&folder, name);
    let data = app.state::<AppState>().data.clone();
    let made = dir.clone();
    blocking(move || {
        std::fs::create_dir_all(made.parent().unwrap_or(&made))
            .map_err(|e| emmet_core::envelope::Error::io(&made, e))?;
        let mut project = Project::create(&data, &made, template)?;
        // The folder's own name, in sentence case as the user reads it ("New model 2"), not emmet new's title case.
        project.config.name = made
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        project.write_config()?;
        Ok(())
    })
    .await?;
    crate::app::open(&app, &dir)
}

/// New (⌘N): the Cottage example or an empty baseplate, in the projects folder, with no dialog (Barry's note 7).
#[tauri::command]
#[specta::specta]
pub async fn new_project(app: AppHandle, kind: NewKind) -> Result<ProjectView, Box<Problem>> {
    match kind {
        NewKind::Cottage => create(app, "Cottage", Template::Cottage).await,
        NewKind::Empty => create(app, "New model", Template::Empty).await,
    }
}

/// Open… (⌘O): one file picker (docs/decisions.md, 2026-10-28). A project's emmet.toml opens its folder; an .ldr or
/// .mpd becomes an imported project in the projects folder. Null when the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn open_dialog(app: AppHandle) -> Result<Option<ProjectView>, Box<Problem>> {
    let dialog = app.dialog().clone();
    let picked = blocking(move || {
        Ok(dialog
            .file()
            .set_title("Open a project or an LDraw file")
            .add_filter("Emmet project or LDraw model", &["toml", "ldr", "mpd"])
            .blocking_pick_file())
    })
    .await?;
    let Some(file) = picked.and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if name == "emmet.toml" {
        let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
        return crate::app::open(&app, &dir).map(Some);
    }
    if name.ends_with(".ldr") || name.ends_with(".mpd") {
        let stem = file
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        return create(app, &format!("{stem} (imported)"), Template::Import(file))
            .await
            .map(Some);
    }
    Err(bad(
        format!("Emmet can't open {name}."),
        "Pick a project's emmet.toml, or an LDraw .ldr or .mpd file.",
    ))
}

/// Trust this project (D-27): writes the trust marker for the open project, opens it again (no longer read-only),
/// and builds it.
#[tauri::command]
#[specta::specta]
pub async fn trust_project(app: AppHandle) -> Result<ProjectView, Box<Problem>> {
    let state = app.state::<AppState>();
    let (data, project) = (state.data.clone(), state.project()?);
    let root = project.root.clone();
    blocking(move || Ok(emmet_core::ops::trust(&data, &project)?)).await?;
    crate::app::open(&app, &root)
}

/// The open project, refused when it isn't trusted: the panel's changes all lead to a run of its model.py.
fn trusted(app: &AppHandle) -> Result<(emmet_core::data::DataDir, Project), Box<Problem>> {
    let state = app.state::<AppState>();
    let project = state.project()?;
    project.require_trusted(&state.data)?;
    Ok((state.data.clone(), project))
}

/// Saves the build settings in emmet.toml's [build]. The agent reworks the model when the user presses Rebuild (the
/// window sends it the settings), so nothing runs here but the watcher's rebuild of the changed emmet.toml.
#[tauri::command]
#[specta::specta]
pub fn set_build_settings(app: AppHandle, settings: BuildSettings) -> Result<BuildSettings, Box<Problem>> {
    let check = |v: &Option<String>, what: &str, choices: &[&str]| match v.as_deref() {
        Some(x) if !choices.contains(&x) => Err(bad(
            format!("There's no {what} '{x}'."),
            format!("Pick one of {}.", choices.join(", ")),
        )),
        _ => Ok(()),
    };
    check(&settings.base, "base", &BASES)?;
    check(&settings.scale, "scale", &SCALES)?;
    check(&settings.age, "age", &AGES)?;
    check(&settings.detail, "detail", &DETAILS)?;
    check(&settings.pieces, "pieces", &PIECES)?;
    let style = settings.style.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if style.is_some_and(|s| s.chars().count() > STYLE_MAX || s.chars().any(char::is_control)) {
        return Err(bad(
            "That style is too long.",
            format!("Say it in {STYLE_MAX} characters or fewer."),
        ));
    }
    let (_, mut project) = trusted(&app)?;
    project.config.build = BuildConfig {
        base: settings.base,
        scale: settings.scale,
        age: settings.age,
        detail: settings.detail,
        pieces: settings.pieces,
        style: style.map(String::from),
    };
    project.write_config()?;
    Ok(BuildSettings::from(&project.config.build))
}

/// It's fine: allows a finding of the build on screen (its rule and parts) in emmet.toml's [[check.allow]].
#[tauri::command]
#[specta::specta]
pub fn allow_finding(app: AppHandle, rule: String, parts: Vec<String>) -> Result<(), Box<Problem>> {
    let (_, mut project) = trusted(&app)?;
    let allow = &mut project.config.check.allow;
    if !allow.iter().any(|a| a.rule == rule && a.parts == parts) {
        allow.push(emmet_core::check::Allow {
            rule,
            parts,
            reason: "Allowed in Emmet's Checks".into(),
        });
    }
    Ok(project.write_config()?)
}

/// Check it again: takes an allowed finding (its rule and parts) out of emmet.toml's [[check.allow]].
#[tauri::command]
#[specta::specta]
pub fn unallow_finding(app: AppHandle, rule: String, parts: Vec<String>) -> Result<(), Box<Problem>> {
    let (_, mut project) = trusted(&app)?;
    let want: BTreeSet<&String> = parts.iter().collect();
    project
        .config
        .check
        .allow
        .retain(|a| !(a.rule == rule && a.parts.iter().collect::<BTreeSet<_>>() == want));
    Ok(project.write_config()?)
}

/// Keep: writes each knob's current value into model.py as its default (`emmet params --write-default`), so the
/// change is the model's own.
#[tauri::command]
#[specta::specta]
pub async fn keep_params(app: AppHandle, names: Vec<String>) -> Result<ParamsView, Box<Problem>> {
    let (data, project) = trusted(&app)?;
    let handle = app.clone();
    let params = blocking(move || {
        let state = handle.state::<AppState>();
        let _one = state.lock_params();
        let mut last = None;
        for name in &names {
            last = Some(emmet_core::ops::write_default(&data, &project, name)?);
        }
        Ok(match last {
            Some(p) => p,
            None => emmet_core::ops::get_params(&data, &project)?,
        })
    })
    .await?;
    crate::app::params_view(params)
}

/// Reset: drops the knobs' drafts from params.json, so the model's defaults apply again.
#[tauri::command]
#[specta::specta]
pub async fn reset_params(app: AppHandle, names: Vec<String>) -> Result<ParamsView, Box<Problem>> {
    let (data, project) = trusted(&app)?;
    let handle = app.clone();
    let params = blocking(move || {
        let state = handle.state::<AppState>();
        let _one = state.lock_params();
        let mut values = project.params()?;
        for name in &names {
            values.remove(name);
        }
        project.write_params(&values)?;
        Ok(emmet_core::ops::get_params(&data, &project)?)
    })
    .await?;
    crate::app::params_view(params)
}

fn version_view(project: &Project, v: &versions::Version) -> VersionView {
    let dir = project.root.join("history").join(&v.dir);
    let thumbnail = v
        .thumbnail
        .then(|| std::fs::read(dir.join("thumbnail.png")).ok())
        .flatten()
        .map(|png| format!("data:image/png;base64,{}", emmet_core::base64::encode(&png)));
    VersionView {
        n: v.n,
        name: v.name.clone(),
        time: v.time,
        auto: v.auto,
        parts: v.parts,
        thumbnail,
        viewable: v.placements,
    }
}

/// Save version (⌘S): the sources as a version named `name`, with the window's render of the model (a base64 PNG).
#[tauri::command]
#[specta::specta]
pub async fn save_version(
    app: AppHandle,
    name: String,
    thumbnail: Option<String>,
) -> Result<VersionView, Box<Problem>> {
    let state = app.state::<AppState>();
    let (data, project) = (state.data.clone(), state.project()?);
    blocking(move || {
        let png = thumbnail.as_deref().and_then(emmet_core::base64::decode);
        // A PNG, and a small one: the window renders the thumbnail at 128 px.
        let png = png.filter(|p| p.starts_with(b"\x89PNG") && p.len() < 2_000_000);
        let v = versions::save(&data, &project, &name, None, false, png.as_deref())?;
        Ok(version_view(&project, &v))
    })
    .await
}

/// The versions list, newest first.
#[tauri::command]
#[specta::specta]
pub async fn list_versions(app: AppHandle) -> Result<Vec<VersionView>, Box<Problem>> {
    let project = app.state::<AppState>().project()?;
    blocking(move || {
        Ok(versions::list(&project)?
            .iter()
            .map(|v| version_view(&project, v))
            .collect())
    })
    .await
}

/// A version opened read-only to compare: its build's placements as a scene. Nothing runs.
#[tauri::command]
#[specta::specta]
pub async fn version_scene(app: AppHandle, n: u32) -> Result<Scene, Box<Problem>> {
    let project = app.state::<AppState>().project()?;
    blocking(move || {
        let dir = versions::folder(&project, n)?;
        crate::app::scene_at(&dir.join("placements.json")).ok_or_else(|| {
            bad(
                format!("Version {n} has no build to show."),
                "Restore it to build it again.",
            )
        })
    })
    .await
}

/// Restore: saves the current state as a version first, then puts version `n`'s sources back. The watcher builds it.
#[tauri::command]
#[specta::specta]
pub async fn restore_version(app: AppHandle, n: u32) -> Result<VersionView, Box<Problem>> {
    let state = app.state::<AppState>();
    let (data, project) = (state.data.clone(), state.project()?);
    blocking(move || {
        let r = versions::restore(&data, &project, n)?;
        Ok(version_view(&project, &r.saved))
    })
    .await
}

/// Open as new project: copies version `n` into the projects folder as "<name> version <n>" and opens it.
#[tauri::command]
#[specta::specta]
pub async fn open_version_as_new(app: AppHandle, n: u32) -> Result<ProjectView, Box<Problem>> {
    let state = app.state::<AppState>();
    let (data, project) = (state.data.clone(), state.project()?);
    let dir = unique(
        &crate::prefs::projects_folder(&app),
        &format!("{} version {n}", project.config.name),
    );
    let dest = dir.clone();
    blocking(move || {
        std::fs::create_dir_all(dest.parent().unwrap_or(&dest))
            .map_err(|e| emmet_core::envelope::Error::io(&dest, e))?;
        let mut new = versions::open_as_new(&data, &project, n, &dest)?;
        new.config.name = dest
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        new.write_config()?;
        Ok(())
    })
    .await?;
    crate::app::open(&app, &dir)
}

/// Export (⌘E): writes the parts list or the model into Downloads, named for the project ("Cottage parts.csv").
#[tauri::command]
#[specta::specta]
pub async fn export(app: AppHandle, format: ExportFormat) -> Result<Exported, Box<Problem>> {
    let state = app.state::<AppState>();
    let (data, project) = (state.data.clone(), state.project()?);
    let downloads = app
        .path()
        .download_dir()
        .map_err(|e| Problem::internal(e.to_string()))?;
    let folder = crate::prefs::in_words(&app, &downloads);
    let name = file_name(&project.config.name);
    let file = match format {
        ExportFormat::Csv => format!("{name} parts.csv"),
        ExportFormat::Rebrickable => format!("{name} Rebrickable.csv"),
        ExportFormat::BricklinkXml => format!("{name} BrickLink.xml"),
        ExportFormat::Mpd => format!("{name}.mpd"),
        ExportFormat::Lpub3d => format!("{name} LPub3D.mpd"),
    };
    let path = unique_file(&downloads, &file);
    if path.parent() != Some(downloads.as_path()) {
        return Err(Problem::internal(format!(
            "Emmet won't write {} outside Downloads.",
            path.display()
        )));
    }
    let out = path.clone();
    blocking(move || {
        let names = emmet_core::bom::ColorNames::preferred(&data);
        let text = match format {
            ExportFormat::Csv => Some(emmet_core::bom::csv(&emmet_core::ops::bom(&data, &project, names)?)),
            ExportFormat::Rebrickable => {
                Some(emmet_core::bom::rebrickable_csv(&emmet_core::ops::bom(&data, &project, names)?)?.0)
            }
            ExportFormat::BricklinkXml => {
                Some(emmet_core::bom::bricklink_xml(&emmet_core::ops::bom(&data, &project, names)?)?.0)
            }
            ExportFormat::Mpd | ExportFormat::Lpub3d => None,
        };
        match text {
            Some(t) => emmet_core::fsx::write_atomic(&out, t.as_bytes())?,
            None => {
                let f = if matches!(format, ExportFormat::Mpd) {
                    emmet_core::export::Format::Mpd
                } else {
                    emmet_core::export::Format::Lpub3d
                };
                emmet_core::export::export(&data, &project, f, Some(&out))?;
            }
        }
        Ok(())
    })
    .await?;
    *LAST_EXPORT.lock().expect("export") = Some(path.clone());
    Ok(Exported {
        file: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        folder,
    })
}

/// `file` in `dir`, or "name 2.ext", "name 3.ext", … when that's taken.
fn unique_file(dir: &Path, file: &str) -> PathBuf {
    let p = Path::new(file);
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = p
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (1..)
        .map(|n| {
            dir.join(if n == 1 {
                file.to_string()
            } else {
                format!("{stem} {n}{ext}")
            })
        })
        .find(|p| !p.exists())
        .expect("a free name")
}

/// Show in folder: the last export, selected in Finder, or its folder in File Explorer.
#[tauri::command]
#[specta::specta]
pub fn reveal_export() -> Result<(), Box<Problem>> {
    let Some(path) = LAST_EXPORT.lock().expect("export").clone() else {
        return Err(Problem::internal("Nothing has been exported yet."));
    };
    // File Explorer gets the folder, not /select, whose comma-separated argument a file name could bend.
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("explorer");
        c.arg(path.parent().unwrap_or(&path));
        c
    } else {
        let mut c = std::process::Command::new("open");
        c.arg("-R").arg(&path);
        c
    };
    cmd.spawn()
        .map_err(|e| Problem::internal(format!("Couldn't show the folder: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shared project names itself in emmet.toml, and the host makes files and folders from that name.
    #[test]
    fn a_name_becomes_one_plain_file_name() {
        for (name, want) in [
            ("Cottage", "Cottage"),
            ("../../Library/x", "Library x"),
            ("..", "Model"),
            (".hidden", "hidden"),
            ("a\\b:c*d?\"e<f>g|h", "a b c d e f g h"),
            ("tab\there\nand", "tab here and"),
            ("   ", "Model"),
        ] {
            let got = file_name(name);
            assert_eq!(got, want, "{name:?}");
            assert_eq!(
                Path::new(&got).components().count(),
                1,
                "{name:?} → {got:?} is one component"
            );
        }
    }
}
