// SPDX-License-Identifier: AGPL-3.0-or-later
//! The project folder (plan section 07, docs/spec/project-format.md): finding it, creating it, workspace trust
//! (D-27), params.json, the inputs hash, and the generation counter.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::data::DataDir;
use crate::envelope::{Details, Error, ErrorCode, Result};
use crate::fsx::{self, write_atomic};

pub const SCHEMA: u32 = 1;
/// The longest `[run] timeout` a project may set, in seconds.
pub const MAX_TIMEOUT: f64 = 3600.0;
const MANIFEST_FILE: &str = "emmet.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub schema: u32,
    pub name: String,
    pub created_by: String,
    pub library: String,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default, skip_serializing_if = "RunConfig::is_default")]
    pub run: RunConfig,
    #[serde(default, skip_serializing_if = "CheckConfig::is_empty")]
    pub check: CheckConfig,
    #[serde(default, skip_serializing_if = "BuildConfig::is_empty")]
    pub build: BuildConfig,
}

/// `[build]`: the build settings (plan section 14's Options tab), which steer the agent; each is Any until set. The
/// app writes them, a change asks the agent to rework the model, and the validator reads the Age.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildConfig {
    /// What it stands on: "None", "16 × 16", "32 × 32", or "48 × 48".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// What the details are sized for: "Micro", "Micropolis", "Midi", "Figure scale", "Large", or "Car 1:8".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
    /// The Age build setting: "4+", "6+", "9+", "13+", or "18+" (plan section 08's step rules).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<String>,
    /// How much the agent adds beyond the shape: "Low", "Medium", or "High".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// How many pieces: "Small" (50–250), "Medium" (250–1,000), "Large" (1,000–2,500), or "Huge" (2,500+).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pieces: Option<String>,
    /// The look: one of the suggestions ("Town", "Medieval", …) or the user's own words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

impl BuildConfig {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The most parts a step should add for the Age setting: 4 for 4+, 12 for 18+, and 8 for 6+ or Any.
    pub fn step_target(&self) -> usize {
        match self.age.as_deref().map(str::trim) {
            Some("4+") => 4,
            Some("18+") => 12,
            _ => 8,
        }
    }
}

/// `[check]`: findings the user has decided to allow (`[[check.allow]] rule = "floating", parts = ["flag/1"],
/// reason = "…"`). The validator lists them under `allowed` instead of as findings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CheckConfig {
    #[serde(default)]
    pub allow: Vec<crate::check::Allow>,
}

impl CheckConfig {
    fn is_empty(&self) -> bool {
        self.allow.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub preferred: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    /// Seconds a model may run before MODEL_TIMEOUT.
    #[serde(default = "default_timeout")]
    pub timeout: f64,
}

impl RunConfig {
    fn is_default(&self) -> bool {
        self.timeout == default_timeout()
    }
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
        }
    }
}

fn default_timeout() -> f64 {
    30.0
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub config: Config,
}

pub enum Template {
    Cottage,
    Empty,
    /// An existing LDraw file (.ldr or .mpd), copied into the project and brought in with import_ldraw (plan
    /// section 07's imports).
    Import(PathBuf),
}

/// An imported project's model.py: the file as fixed geometry, which the agent can make parametric.
fn import_model(file: &str) -> String {
    format!(
        "# SPDX-License-Identifier: Apache-2.0\nfrom emmet_dsl import Model, import_ldraw, model\n\n\n@model\ndef imported():\n    \"\"\"{file}, opened in Emmet: a fixed model with no parameters yet.\"\"\"\n    m = Model()\n    m.place(import_ldraw({}), at=(0, 0, 0))\n    return m\n",
        serde_json::to_string(file).expect("json")
    )
}

const COTTAGE: &str = include_str!("../templates/cottage/model.py");
const EMPTY: &str = include_str!("../templates/empty/model.py");

impl Project {
    /// The project at `dir`, or with no dir, the nearest folder with an emmet.toml from the current directory up.
    pub fn find(dir: Option<&Path>) -> Result<Self> {
        let not_found = |from: &Path| {
            Error::new(
                ErrorCode::ProjectNotFound,
                format!("There's no Emmet project (emmet.toml) at {}.", from.display()),
                "Run emmet new <dir> to make one, or pass --project <dir>.",
            )
            .at_file(from.display().to_string())
        };
        let root = match dir {
            Some(d) => {
                let d = std::path::absolute(d).unwrap_or_else(|_| d.to_path_buf());
                if !d.join(MANIFEST_FILE).is_file() {
                    return Err(not_found(&d));
                }
                d
            }
            None => {
                let cwd = std::env::current_dir().map_err(|e| Error::io(Path::new("."), e))?;
                cwd.ancestors()
                    .find(|a| a.join(MANIFEST_FILE).is_file())
                    .map(Path::to_path_buf)
                    .ok_or_else(|| not_found(&cwd))?
            }
        };
        let root = fsx::canonical(&root).map_err(|e| Error::io(&root, e))?;
        let path = root.join(MANIFEST_FILE);
        let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
        let config: Config = toml::from_str(&text).map_err(|e| {
            let line = e.span().map(|s| line_of(&text, s.start));
            Error::new(
                ErrorCode::IoError,
                format!("emmet.toml doesn't parse: {}", e.message()),
                "Fix emmet.toml (docs/spec/project-format.md describes it).",
            )
            .with_details(Details {
                file: Some(MANIFEST_FILE.into()),
                line,
                ..Default::default()
            })
        })?;
        if config.schema > SCHEMA {
            return Err(Error::new(
                ErrorCode::IoError,
                format!(
                    "This project uses project format {}; this Emmet reads up to {SCHEMA}.",
                    config.schema
                ),
                "Update Emmet to open it.",
            )
            .at_file(MANIFEST_FILE));
        }
        let t = config.run.timeout;
        if !(t.is_finite() && t > 0.0 && t <= MAX_TIMEOUT) {
            let line = text
                .lines()
                .position(|l| l.trim_start().starts_with("timeout"))
                .map(|i| i as u32 + 1);
            return Err(Error::new(
                ErrorCode::IoError,
                format!("[run] timeout = {t} in emmet.toml isn't a number of seconds from 0 to {MAX_TIMEOUT}."),
                "Set it to the seconds a model may run, for example timeout = 30, or remove it for the default.",
            )
            .with_details(Details {
                file: Some(MANIFEST_FILE.into()),
                line,
                ..Default::default()
            }));
        }
        Ok(Self { root, config })
    }

    /// `emmet new`: a new project in `dir` (which must be missing or empty), trusted on this machine.
    pub fn create(data: &DataDir, dir: &Path, template: Template) -> Result<Self> {
        if dir.exists() && std::fs::read_dir(dir).map_err(|e| Error::io(dir, e))?.next().is_some() {
            return Err(Error::new(
                ErrorCode::IoError,
                format!("{} already exists and isn't empty.", dir.display()),
                "Pick a new folder name, or empty that folder first.",
            )
            .at_file(dir.display().to_string()));
        }
        let stem = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "model".into());
        let name = title_case(&stem);
        let library = crate::library::installed(data).map_or(crate::library::RELEASE.to_string(), |i| i.release);
        let machine = machine_hash(data)?;
        let toml = format!(
            "# The Emmet project file (docs/spec/project-format.md).\nschema = {SCHEMA}\nname = {}\ncreated_by = \"{machine}\"\nlibrary = \"{library}\"\n\n[agent]\npreferred = \"claude\"\n",
            toml::Value::String(name.clone())
        );
        let imported;
        let (model, what) = match &template {
            Template::Cottage => (
                COTTAGE,
                "A small cottage on a green 32 x 32 baseplate: stone-based walls with a real door and glazed windows, a\ngable roof, a chimney, a path with flowers, and a room inside (a bed, a table, and a stove).",
            ),
            Template::Empty => (EMPTY, "A green 32 x 32 baseplate, ready for a model."),
            Template::Import(file) => {
                let name = import_name(file)?;
                let bytes = std::fs::read(file).map_err(|e| Error::io(file, e))?;
                write_atomic(&dir.join(&name), &bytes)?;
                imported = import_model(&name);
                (
                    imported.as_str(),
                    "An LDraw model opened in Emmet: fixed geometry, with no parameters yet.",
                )
            }
        };
        let w = |rel: &str, text: &str| write_atomic(&dir.join(rel), text.as_bytes());
        w(MANIFEST_FILE, &toml)?;
        w("model.py", model)?;
        w("params.json", "{}\n")?;
        w("CARD.md", &card(&name, what))?;
        w("AGENTS.md", &agents(&name))?;
        w(".gitignore", "build/\n.emmet/\n")?;
        link_claude_md(dir)?;
        let project = Self::find(Some(dir))?;
        project.trust(data)?;
        Ok(project)
    }

    /// Writes emmet.toml from `config` (the UI's build settings and allowed findings), atomically.
    pub fn write_config(&self) -> Result<()> {
        let body = toml::to_string(&self.config).map_err(|e| Error::internal(format!("emmet.toml: {e}")))?;
        let text = format!("# The Emmet project file (docs/spec/project-format.md).\n{body}");
        write_atomic(&self.root.join(MANIFEST_FILE), text.as_bytes())
    }

    /// The LDraw file an imported project brings in, while model.py is still the one Emmet wrote for it (once the
    /// agent makes it parametric, it's a model like any other).
    pub fn imported_file(&self) -> Option<String> {
        let text = std::fs::read_to_string(self.root.join("model.py")).ok()?;
        let at = text.find("import_ldraw(")? + "import_ldraw(".len();
        let end = text[at..].find(')')?;
        let name: String = serde_json::from_str(&text[at..at + end]).ok()?;
        (text.replace("\r\n", "\n") == import_model(&name)).then_some(name)
    }

    pub fn rel(&self, path: &Path) -> String {
        fsx::display_rel(&self.root, path)
    }

    /// A folder inside the project that emmet writes, refused when it's a symlink or a file (a shared project can't point
    /// emmet's writes elsewhere).
    pub fn own_dir(&self, name: &str) -> Result<PathBuf> {
        let dir = self.root.join(name);
        if let Ok(meta) = std::fs::symlink_metadata(&dir) {
            if meta.file_type().is_symlink() || !meta.is_dir() {
                return Err(Error::new(
                    ErrorCode::IoError,
                    format!("{name}/ in this project is a link or a file, not a folder; Emmet won't write through it."),
                    format!("Delete {name}/ and run again; Emmet recreates it."),
                )
                .at_file(name));
            }
        } else {
            std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
        }
        Ok(dir)
    }

    pub fn build_dir(&self) -> Result<PathBuf> {
        self.own_dir("build")
    }

    /// Trusted when `.emmet/trust` holds `trust_token` (sha256 of this install's secret and the project's canonical
    /// path) and `.emmet/` isn't a link (D-27).
    pub fn is_trusted(&self, data: &DataDir) -> bool {
        let marker = self.root.join(".emmet/trust");
        let linked = std::fs::symlink_metadata(self.root.join(".emmet")).is_ok_and(|m| m.file_type().is_symlink());
        !linked
            && std::fs::read_to_string(marker)
                .ok()
                .zip(trust_token(data, &self.root).ok())
                .is_some_and(|(m, t)| m.trim() == t)
    }

    pub fn trust(&self, data: &DataDir) -> Result<()> {
        let dir = self.own_dir(".emmet")?;
        write_atomic(
            &dir.join("trust"),
            format!("{}\n", trust_token(data, &self.root)?).as_bytes(),
        )
    }

    pub fn require_trusted(&self, data: &DataDir) -> Result<()> {
        if self.is_trusted(data) {
            return Ok(());
        }
        Err(Error::new(
            ErrorCode::ProjectUntrusted,
            format!(
                "{} isn't trusted on this machine (it was made elsewhere, or moved), so its model.py won't run until you \
                 trust it.",
                self.config.name
            ),
            "Read model.py first; it's code. Then trust the project in the app, or run emmet trust.",
        )
        .at_file("model.py"))
    }

    /// The overrides in params.json.
    pub fn params(&self) -> Result<BTreeMap<String, Value>> {
        let path = self.root.join("params.json");
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(e) => return Err(Error::io(&path, e)),
        };
        serde_json::from_str(&text).map_err(|e| {
            Error::new(
                ErrorCode::ParamInvalid,
                format!("params.json isn't a JSON object of parameter values: {e}"),
                "Fix params.json, or delete it to use the model's defaults.",
            )
            .with_details(Details {
                file: Some("params.json".into()),
                line: Some(e.line() as u32),
                excerpt: text.lines().nth(e.line().saturating_sub(1)).map(str::to_string),
                ..Default::default()
            })
        })
    }

    pub fn write_params(&self, values: &BTreeMap<String, Value>) -> Result<()> {
        let mut text = serde_json::to_string_pretty(values).expect("json");
        text.push('\n');
        write_atomic(&self.root.join("params.json"), text.as_bytes())
    }

    /// The model's sources: model.py, models/**/*.py, params.json, and emmet.toml, sorted by path.
    pub fn sources(&self) -> Vec<PathBuf> {
        let mut out: Vec<PathBuf> = ["model.py", "params.json", MANIFEST_FILE]
            .iter()
            .map(|f| self.root.join(f))
            .filter(|p| p.is_file())
            .collect();
        let mut stack = vec![self.root.join("models")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                let p = e.path();
                // file_type doesn't follow links: a link in models/ could point anywhere, or back at itself.
                let Ok(kind) = e.file_type() else { continue };
                if kind.is_dir() && !p.ends_with("__pycache__") {
                    stack.push(p);
                } else if kind.is_file() && p.extension().is_some_and(|x| x == "py") {
                    out.push(p);
                }
            }
        }
        out.sort_by_key(|p| self.rel(p));
        out
    }

    /// sha256 of the sources (line endings normalized), the run's parameter overrides, and the Emmet, library, and
    /// parts data versions (plan section 07).
    pub fn inputs_hash(
        &self,
        overrides: &BTreeMap<String, Value>,
        library: &str,
        parts_data: Option<&str>,
    ) -> Result<String> {
        let mut h = Vec::new();
        let mut line = |k: &str, v: &str| h.extend_from_slice(format!("{k} {v}\n").as_bytes());
        line("emmet", crate::version());
        line("library", library);
        line("parts_data", parts_data.unwrap_or("none"));
        line("params", &serde_json::to_string(overrides).expect("json"));
        for path in self.sources() {
            let bytes = std::fs::read(&path).map_err(|e| Error::io(&path, e))?;
            let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
            h.extend_from_slice(format!("file {} {}\n", self.rel(&path), text.len()).as_bytes());
            h.extend_from_slice(text.as_bytes());
        }
        Ok(fsx::sha256_hex(&h))
    }

    /// Bumps and returns the run counter in `.emmet/generation`.
    pub fn next_generation(&self) -> Result<u64> {
        let dir = self.own_dir(".emmet")?;
        let path = dir.join("generation");
        let n = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0)
            + 1;
        write_atomic(&path, format!("{n}\n").as_bytes())?;
        Ok(n)
    }
}

/// The file name an import keeps: the original's, if it's a plain .ldr or .mpd name.
fn import_name(file: &Path) -> Result<String> {
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = Path::new(&name).extension().map(|e| e.to_string_lossy().to_lowercase());
    if !matches!(ext.as_deref(), Some("ldr" | "mpd")) {
        return Err(Error::new(
            ErrorCode::IoError,
            format!("{name} isn't an LDraw model file."),
            "Open an .ldr or .mpd file, like the ones Studio, LDCad, and LeoCAD save.",
        )
        .at_file(file.display().to_string()));
    }
    let bad = |c: char| c.is_control() || matches!(c, '/' | '\\' | '"' | '\'');
    Ok(if name.chars().any(bad) {
        format!("model.{}", ext.unwrap_or_default())
    } else {
        name
    })
}

/// This install's machine hash for `created_by`: `m:` plus 16 hex digits of sha256 of a random secret kept in app
/// data. It isn't personal data; it tells this machine's projects from others' (D-27). It's shared with the project,
/// so trust never rests on it.
pub fn machine_hash(data: &DataDir) -> Result<String> {
    let id = machine_secret(data)?;
    Ok(format!(
        "m:{}",
        &fsx::sha256_hex(format!("emmet-machine:{id}").as_bytes())[..16]
    ))
}

/// What `.emmet/trust` must hold: sha256 of the machine secret and the project's canonical path. Only this machine
/// can make it, nothing shared with a project (created_by included) reveals it, and a moved project needs trusting
/// again.
fn trust_token(data: &DataDir, root: &Path) -> Result<String> {
    let id = machine_secret(data)?;
    let root = fsx::canonical(root).unwrap_or_else(|_| root.to_path_buf());
    Ok(fsx::sha256_hex(
        format!("emmet-trust:{id}\0{}", root.display()).as_bytes(),
    ))
}

/// The random secret in app data, made on first use. It never leaves the data dir. Two emmets starting at once must
/// agree on it, so a new secret is published with a hard link, which fails if the file already exists; the loser
/// reads the winner's.
fn machine_secret(data: &DataDir) -> Result<String> {
    let path = data.machine_id();
    let read = || {
        std::fs::read_to_string(&path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| s.len() >= 32)
    };
    if let Some(id) = read() {
        return Ok(id);
    }
    let id = fsx::random_hex(4);
    let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
    write_atomic(&tmp, format!("{id}\n").as_bytes())?;
    let linked = std::fs::hard_link(&tmp, &path);
    let _ = std::fs::remove_file(&tmp);
    match linked {
        Ok(()) => Ok(id),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => read().ok_or_else(|| {
            Error::new(
                ErrorCode::IoError,
                format!("{} is damaged: it should hold one long random id.", path.display()),
                "Delete it and try again. Projects trusted with the old id will need trusting again.",
            )
            .at_file(path.display().to_string())
        }),
        Err(e) => Err(Error::io(&path, e)),
    }
}

/// The 1-based line holding byte `offset` of `text`.
fn line_of(text: &str, offset: usize) -> u32 {
    text[..offset.min(text.len())].matches('\n').count() as u32 + 1
}

fn title_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn card(name: &str, what: &str) -> String {
    format!(
        "# {name}\n\n## What it is\n\n{what}\n\n## Design notes\n\n- The baseplate is half a plate thick, so the walls start at z = 0.5.\n\n## Don't\n\n- Don't edit anything in build/; emmet run regenerates it.\n\n<!-- emmet:auto -->\n<!-- /emmet:auto -->\n"
    )
}

/// The pointer every agent gets (plan section 10): the app passes it through Claude Code's `--append-system-prompt`
/// and Codex's `developerInstructions`, and a new project's AGENTS.md (and CLAUDE.md) starts with it, so a terminal
/// agent behaves the same way. The doctrine itself stays in the binary and is read through the `rules` tool.
pub const AGENT_POINTER: &str = "You are operating Emmet, which builds brick models from real LDraw parts. Before \
anything else, call the emmet `rules` tool (in a terminal without Emmet's MCP server: `emmet rules`) and follow it.";

fn agents(name: &str) -> String {
    format!(
        "# {name}: an Emmet project\n\n{AGENT_POINTER}\n\nThis folder is a brick model. The model is the Python in model.py, written with emmet_dsl, and build/ is\ngenerated from it: never edit build/.\n\n- Edit model.py (and models/*.py), then run `emmet run --json` and fix what it reports. Errors name the file\n  and line, with a hint.\n- Find parts with `emmet search <words>` (for example `emmet search brick 2x4`) and check one with\n  `emmet part <id>`. List colors with `emmet colors`, and parameters with `emmet params`.\n- Units: x runs right and y runs back, in studs; z runs up, in plates (3 plates make a brick). `at=` is the\n  lowest corner of the part's body, studs left out, after `rot` quarter turns.\n- CARD.md says what this model is, its design notes, and what not to do. Keep it current.\n- No file I/O outside this folder and no network in model code.\n"
    )
}

/// CLAUDE.md points at AGENTS.md: a symlink, or a copy where symlinks need extra rights (Windows).
fn link_claude_md(dir: &Path) -> Result<()> {
    let target = dir.join("CLAUDE.md");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("AGENTS.md", &target).map_err(|e| Error::io(&target, e))
    }
    #[cfg(not(unix))]
    {
        std::fs::copy(dir.join("AGENTS.md"), &target)
            .map(|_| ())
            .map_err(|e| Error::io(&target, e))
    }
}
