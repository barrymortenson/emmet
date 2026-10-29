// SPDX-License-Identifier: AGPL-3.0-or-later
//! Saved versions (plan section 07, docs/spec/project-format.md): a copy of the sources (model.py, models/,
//! params.json, emmet.toml, CARD.md), plus that build's model.mpd and placements.json when the build is current and a
//! thumbnail when the app has one, in `history/<n>-<slug>/` with a version.json. A restore first saves the current
//! state as a version, then writes the sources back atomically, so it never loses anything. "Open as a new project"
//! copies a version out to a new folder.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::data::DataDir;
use crate::envelope::{Error, ErrorCode, Result};
use crate::fsx::write_atomic;
use crate::project::{Project, Template};

/// Emmet keeps this many of its own versions; the user's named ones stay until the user deletes them.
pub const KEEP_AUTO: usize = 20;

/// The sources a version holds besides models/: the files at the project's top level.
const TOP: [&str; 4] = ["model.py", "params.json", "emmet.toml", "CARD.md"];

/// `history/<n>-<slug>/version.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Version {
    /// From 1, in the order they were saved.
    pub n: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// When it was saved, in seconds since 1970 (UTC).
    pub time: u64,
    /// Saved by Emmet (before an agent's turn, or before a restore) rather than by the user.
    pub auto: bool,
    /// The build's part count, when the build was current.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parts: Option<u32>,
    /// manifest.json's inputs hash of that build, when it was current.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs_hash: Option<String>,
    /// Its folder in history/.
    pub dir: String,
    /// Whether it has a thumbnail.png and a placements.json to show.
    #[serde(default)]
    pub thumbnail: bool,
    #[serde(default)]
    pub placements: bool,
}

/// What a restore did: the version it saved of the state before, and the version it put back.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Restored {
    pub saved: Version,
    pub restored: Version,
    /// The sources it wrote and the ones it removed (models/ files the version doesn't have), relative to the project.
    pub written: Vec<String>,
    pub removed: Vec<String>,
}

fn history(project: &Project) -> Result<PathBuf> {
    project.own_dir("history")
}

/// A version's folder name: its number, then its name in lowercase words joined by dashes ("3-roof-and-chimney").
pub fn slug(n: u32, name: &str) -> String {
    let mut s = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            s.push(c);
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
        if s.len() >= 40 {
            break;
        }
    }
    let s = s.trim_end_matches('-');
    if s.is_empty() {
        format!("{n}")
    } else {
        format!("{n}-{s}")
    }
}

/// Every version, newest first. A folder without a readable version.json isn't one.
pub fn list(project: &Project) -> Result<Vec<Version>> {
    let dir = project.root.join("history");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut out: Vec<Version> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| {
            let text = std::fs::read_to_string(e.path().join("version.json")).ok()?;
            let v: Version = serde_json::from_str(&text).ok()?;
            // A version.json copied in from elsewhere names its own folder.
            (v.dir == e.file_name().to_string_lossy()).then_some(v)
        })
        .collect();
    out.sort_by_key(|v| std::cmp::Reverse(v.n));
    Ok(out)
}

/// Version `n`, or a plain error naming the ones there are.
pub fn get(project: &Project, n: u32) -> Result<Version> {
    let all = list(project)?;
    all.iter().find(|v| v.n == n).cloned().ok_or_else(|| {
        let have = all.iter().map(|v| v.n.to_string()).collect::<Vec<_>>();
        Error::new(
            ErrorCode::IoError,
            format!("This project has no version {n}."),
            if have.is_empty() {
                "It has no saved versions yet: emmet version save <name> saves one.".to_string()
            } else {
                format!("Its versions are {}.", have.join(", "))
            },
        )
    })
}

/// The folder of version `n`.
pub fn folder(project: &Project, n: u32) -> Result<PathBuf> {
    Ok(project.root.join("history").join(get(project, n)?.dir))
}

/// The project's sources, relative to its root: the top-level files that exist, then models/**/*.py.
fn sources(project: &Project) -> Vec<String> {
    let mut out: Vec<String> = TOP
        .iter()
        .filter(|f| project.root.join(f).is_file())
        .map(|f| f.to_string())
        .collect();
    for p in project.sources() {
        let rel = project.rel(&p);
        if !out.contains(&rel) {
            out.push(rel);
        }
    }
    out
}

/// The sources a version folder holds, relative to it (the same layout as the project).
fn version_sources(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = TOP
        .iter()
        .filter(|f| dir.join(f).is_file())
        .map(|f| f.to_string())
        .collect();
    let mut stack = vec![dir.join("models")];
    let mut models = Vec::new();
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let Ok(kind) = e.file_type() else { continue };
            if kind.is_dir() {
                stack.push(e.path());
            } else if kind.is_file() && e.path().extension().is_some_and(|x| x == "py") {
                models.push(crate::fsx::display_rel(dir, &e.path()));
            }
        }
    }
    models.sort();
    out.extend(models);
    out
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn copy(from: &Path, to: &Path) -> Result<()> {
    let bytes = std::fs::read(from).map_err(|e| Error::io(from, e))?;
    write_atomic(to, &bytes)
}

/// The build's model.mpd, placements.json, and part count, when build/ matches the sources as they are now.
fn current_build(data: &DataDir, project: &Project) -> Option<(String, u32)> {
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(project.root.join("build/manifest.json")).ok()?).ok()?;
    let library = crate::library::installed(data)?.release;
    let parts_data = crate::parts_data::version(data).ok()?;
    let now = project
        .inputs_hash(&Default::default(), &library, parts_data.as_deref())
        .ok()?;
    let hash = manifest["inputs_hash"].as_str()?;
    let fresh = hash == now
        && project.root.join("build/model.mpd").is_file()
        && project.root.join("build/placements.json").is_file();
    fresh.then(|| (hash.to_string(), manifest["parts"].as_u64().unwrap_or(0) as u32))
}

/// Saves the project's sources as a new version. `thumbnail` is a PNG of the model (the app renders one). Emmet's own
/// versions (`auto`) beyond the last [`KEEP_AUTO`] go.
pub fn save(
    data: &DataDir,
    project: &Project,
    name: &str,
    note: Option<&str>,
    auto: bool,
    thumbnail: Option<&[u8]>,
) -> Result<Version> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::new(
            ErrorCode::IoError,
            "A version needs a name.",
            "Give it a few words, like \"Steeper roof\".",
        ));
    }
    let history = history(project)?;
    let n = list(project)?.first().map_or(0, |v| v.n) + 1;
    let dir_name = slug(n, name);
    let dir = history.join(&dir_name);
    if dir.exists() {
        return Err(Error::new(
            ErrorCode::IoError,
            format!("history/{dir_name} already exists."),
            "Move it out of history/ and save again.",
        ));
    }
    for rel in sources(project) {
        copy(&project.root.join(&rel), &dir.join(&rel))?;
    }
    let built = current_build(data, project);
    if built.is_some() {
        copy(&project.root.join("build/model.mpd"), &dir.join("model.mpd"))?;
        copy(
            &project.root.join("build/placements.json"),
            &dir.join("placements.json"),
        )?;
    }
    if let Some(png) = thumbnail {
        write_atomic(&dir.join("thumbnail.png"), png)?;
    }
    let version = Version {
        n,
        name: name.to_string(),
        note: note.map(str::trim).filter(|s| !s.is_empty()).map(String::from),
        time: now(),
        auto,
        parts: built.as_ref().map(|b| b.1),
        inputs_hash: built.map(|b| b.0),
        dir: dir_name,
        thumbnail: thumbnail.is_some(),
        placements: dir.join("placements.json").is_file(),
    };
    let mut text = serde_json::to_string_pretty(&version).expect("json");
    text.push('\n');
    write_atomic(&dir.join("version.json"), text.as_bytes())?;
    if auto {
        prune(project)?;
    }
    Ok(version)
}

/// Removes Emmet's own versions past the newest [`KEEP_AUTO`].
fn prune(project: &Project) -> Result<()> {
    let history = project.root.join("history");
    for v in list(project)?.into_iter().filter(|v| v.auto).skip(KEEP_AUTO) {
        let dir = history.join(&v.dir);
        std::fs::remove_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
    }
    Ok(())
}

/// Puts version `n`'s sources back: first saves the current state as Emmet's own version, then writes each of the
/// version's sources atomically and removes the models/ files and CARD.md it doesn't have.
pub fn restore(data: &DataDir, project: &Project, n: u32) -> Result<Restored> {
    let version = get(project, n)?;
    let dir = project.root.join("history").join(&version.dir);
    let theirs = version_sources(&dir);
    if !theirs.iter().any(|s| s == "model.py") {
        return Err(Error::new(
            ErrorCode::IoError,
            format!("Version {n} has no model.py, so Emmet won't restore it."),
            "Its folder in history/ is incomplete; pick another version.",
        ));
    }
    let saved = save(
        data,
        project,
        &format!("Before restoring version {n}"),
        None,
        true,
        None,
    )?;
    let mine = sources(project);
    let mut written = Vec::new();
    for rel in &theirs {
        let to = project.root.join(rel);
        let bytes = std::fs::read(dir.join(rel)).map_err(|e| Error::io(&dir.join(rel), e))?;
        if std::fs::read(&to).ok().as_deref() != Some(bytes.as_slice()) {
            write_atomic(&to, &bytes)?;
            written.push(rel.clone());
        }
    }
    let mut removed = Vec::new();
    for rel in mine.iter().filter(|m| !theirs.contains(m)) {
        // emmet.toml and params.json stay: a project needs them, and a version always has them.
        if rel == "emmet.toml" || rel == "params.json" {
            continue;
        }
        let path = project.root.join(rel);
        std::fs::remove_file(&path).map_err(|e| Error::io(&path, e))?;
        removed.push(rel.clone());
    }
    Ok(Restored {
        saved,
        restored: version,
        written,
        removed,
    })
}

/// Copies version `n`'s sources into `dest` (missing or empty) as a new project, named after the folder and trusted
/// on this machine.
pub fn open_as_new(data: &DataDir, project: &Project, n: u32, dest: &Path) -> Result<Project> {
    let version = get(project, n)?;
    let dir = project.root.join("history").join(&version.dir);
    // A fresh project first (the folder checks, the machine hash, the trust marker), then the version's sources over it.
    let new = Project::create(data, dest, Template::Empty)?;
    let fresh_config = std::fs::read_to_string(new.root.join("emmet.toml")).map_err(|e| Error::io(&new.root, e))?;
    for rel in version_sources(&dir) {
        if rel == "emmet.toml" {
            continue;
        }
        copy(&dir.join(&rel), &new.root.join(&rel))?;
    }
    // The version's settings and allowed findings, under the new project's own name and machine.
    let mut config: crate::project::Config =
        toml::from_str(&fresh_config).map_err(|e| Error::internal(e.to_string()))?;
    let old = std::fs::read_to_string(dir.join("emmet.toml"))
        .ok()
        .and_then(|t| toml::from_str::<crate::project::Config>(&t).ok());
    if let Some(old) = old {
        config.agent = old.agent;
        config.run = old.run;
        config.check = old.check;
        config.build = old.build;
    }
    let new = Project { root: new.root, config };
    new.write_config()?;
    Project::find(Some(&new.root))
}
