// SPDX-License-Identifier: AGPL-3.0-or-later
//! The app's state, its typed commands and events (tauri-specta), and the glue between the project, the watcher, the
//! run queue, and the bus. The webview names no filesystem path except `open_recent`'s, which must be on the list.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock, mpsc};
use std::time::Duration;

use emmet_core::data::DataDir;
use emmet_core::envelope::Error;
use emmet_core::project::Project;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::recent::{self, Recent};
use crate::runs::{Outcome, RunEvent, RunQueue};
use crate::watcher::Watcher;

/// How long the app waits for its own window to render for the bus.
const BUS_RENDER_TIMEOUT: Duration = Duration::from_secs(25);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ProjectView {
    pub root: String,
    pub name: String,
    pub trusted: bool,
    /// Whether the watcher is running; when it isn't, edits don't rebuild on their own (use Rebuild).
    pub watching: bool,
    /// The LDraw file an imported project brings in, while its model.py is still the one Emmet wrote (the banner's
    /// Make it parametric).
    pub imported: Option<String>,
    /// The build settings in emmet.toml's [build].
    pub build: crate::projects::BuildSettings,
}

/// An error for the UI: emmet's error envelope, with the user's file and line when there is one.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Problem {
    pub code: String,
    pub message: String,
    pub hint: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub excerpt: Option<String>,
}

impl From<Error> for Box<Problem> {
    fn from(e: Error) -> Self {
        let v = serde_json::to_value(&e).unwrap_or_default();
        Box::new(Problem::from_envelope(&v))
    }
}

impl Problem {
    pub(crate) fn from_envelope(e: &Value) -> Self {
        let s = |v: &Value| v.as_str().map(String::from);
        let d = &e["details"];
        Self {
            code: s(&e["code"]).unwrap_or_else(|| "INTERNAL".into()),
            message: s(&e["message"]).unwrap_or_default(),
            hint: s(&e["hint"]).unwrap_or_default(),
            file: s(&d["file"]),
            line: d["line"].as_u64().map(|l| l as u32),
            excerpt: s(&d["excerpt"]),
        }
    }

    pub(crate) fn internal(message: impl Into<String>) -> Box<Self> {
        Box::new(Self {
            code: "INTERNAL".into(),
            message: message.into(),
            hint: String::new(),
            file: None,
            line: None,
            excerpt: None,
        })
    }
}

/// One placed part, as build/placements.json lists it.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Placement {
    pub id: String,
    pub part: String,
    pub color: u32,
    pub matrix: Vec<f64>,
    pub step: u32,
    pub submodel: Vec<String>,
    #[serde(default)]
    pub src: Option<String>,
}

/// A built model on screen: what the viewer shows, and the run it came from.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Scene {
    pub generation: u64,
    pub model: String,
    pub placements: Vec<Placement>,
    pub parts: u32,
    pub steps: u32,
    /// How long the run took, or null for a scene read from an earlier build/.
    pub ms: Option<u64>,
    /// The validator's findings for this build (the Checks drawer), or null when they couldn't be made.
    #[serde(default)]
    pub checks: Option<Checks>,
    /// The parts list (the Parts drawer), or null when it couldn't be made.
    #[serde(default)]
    pub bom: Option<Bom>,
    /// The Steps tab's tree, or null when build/ has no steps.json.
    #[serde(default)]
    pub tree: Option<StepsTree>,
}

/// A line of the model's source that placed a finding's parts, with its text.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SourceLine {
    pub file: String,
    pub line: u32,
    /// The line as written, or null when the file can't be read (or isn't inside the project).
    pub text: Option<String>,
}

/// One validator finding (plan section 11), for the Checks drawer.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Finding {
    pub id: String,
    pub rule: String,
    /// error, warning, or info.
    pub severity: String,
    pub parts: Vec<String>,
    pub source: Vec<SourceLine>,
    pub message: String,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Checks {
    pub findings: Vec<Finding>,
    /// Findings the user allowed (It's fine) or the model did (m.allow): Check it again brings one back.
    #[serde(default)]
    pub allowed: Vec<AllowedView>,
    pub errors: u32,
    /// No errors and no warnings.
    pub clean: bool,
    /// Why the validator couldn't run, when it couldn't.
    pub problem: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AllowedView {
    pub rule: String,
    pub parts: Vec<String>,
    pub reason: String,
}

/// The Steps tab's tree (build/steps.json): the sub-assemblies, each with its uses and sub-steps, then the steps.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct StepsTree {
    pub steps: Vec<StepView>,
    pub submodels: Vec<SubmodelView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct StepView {
    pub n: u32,
    pub title: String,
    pub group: String,
    /// The parts this step adds, and the copies of sub-assemblies it puts on.
    pub parts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SubmodelView {
    pub name: String,
    /// Its file in model.mpd, which placements name in their submodel path.
    pub file: String,
    /// The "× 5" on its row.
    pub count: u32,
    /// Parts in one copy.
    pub parts: u32,
    pub steps: Vec<StepView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BomLine {
    pub part: String,
    pub name: String,
    pub color: u32,
    /// In the user's color naming (Settings → Data).
    pub color_name: String,
    pub qty: u32,
    /// available, color_unavailable, part_unavailable, or unknown (never read as unavailable).
    pub availability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Bom {
    pub lines: Vec<BomLine>,
    pub pieces: u32,
    /// Different parts, in any color.
    pub parts: u32,
    /// Whether the parts data pack was there for availability (the Parts list's Rebrickable credit).
    pub parts_data: bool,
    /// Why there's no parts list for a build that ran (a damaged parts data pack, say), or null.
    pub problem: Option<String>,
}

/// Settings → Data: the parts data pack and the BOM's color naming.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DataSettings {
    /// The installed pack's date, or null when there's none.
    pub parts_data: Option<String>,
    /// ldraw, rebrickable, or bricklink.
    pub color_names: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UiState {
    pub project: Option<ProjectView>,
    pub running: bool,
    /// The last good model, which stays on screen while a run is going or after one fails.
    pub scene: Option<Scene>,
    /// Why the last run failed, if it did.
    pub error: Option<Problem>,
    /// Something the user should know that isn't a run's error: a project that didn't open at launch, a bus that
    /// didn't start.
    pub notice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AppInfo {
    pub recent: Vec<Recent>,
    /// The library release, or null when none is installed.
    pub library: Option<String>,
    /// The release this Emmet installs, and whether the installed one is older (the old library's banner).
    pub pinned: String,
    pub library_old: bool,
    /// The parts data pack's date, or null.
    pub parts_data: Option<String>,
    /// Whether Python and emmet_dsl are set up for models (`emmet doctor --fix` does it).
    pub python: bool,
    /// Emmet's version.
    pub version: String,
    /// Milliseconds since the app started, from the first thing run() does (the warm start's timing, plan section 14's
    /// budget).
    pub uptime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ParamSpec {
    pub name: String,
    /// int, float, bool, choice, or color.
    #[serde(rename = "type")]
    pub kind: String,
    pub default: Value,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub choices: Option<Vec<Value>>,
    pub help: String,
    pub line: u32,
    /// One of the model's knobs, which the panel shows (`@model(knobs=...)`).
    #[serde(default)]
    pub knob: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ParamsView {
    pub model: String,
    pub doc: String,
    pub file: String,
    pub params: Vec<ParamSpec>,
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ColorView {
    pub code: u32,
    pub name: String,
    pub rgb: String,
}

/// What render mode renders: the project's build/placements.json, from one view.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RenderJob {
    pub scene: Scene,
    pub view: String,
    pub size: u32,
    pub highlight: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct ProjectOpened {
    pub project: ProjectView,
    pub scene: Option<Scene>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct RunStarted {}

/// A run ended. It carries a new scene, or why it failed (the last good scene stays), or neither with `stale` set:
/// the run built a model older than the one on screen (the generation check), so nothing changes but the status.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct RunFinished {
    pub scene: Option<Scene>,
    pub error: Option<Problem>,
    pub stale: bool,
}

/// The bus asks the window to render: show `scene` if it's newer than what's on screen, render, and answer with
/// `bus_reply(id, …)`.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct BusRender {
    pub id: u64,
    pub view: String,
    pub size: u32,
    pub highlight: Vec<String>,
    pub scene: Option<Scene>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct BusHighlight {
    pub ids: Vec<String>,
}

/// `camera` from an agent: turn the viewer to `view`.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct BusCamera {
    pub view: String,
}

/// The views the viewer has (plan section 13), as the bus and the window name them.
const VIEWS: [&str; 6] = ["iso", "front", "top", "back", "left", "right"];

/// What the user sees in the viewer, as the window last said: the bus's `selection` and `camera` answer from it.
#[derive(Default)]
struct Viewing {
    selection: Vec<String>,
    view: String,
}

struct Open {
    root: PathBuf,
    view: ProjectView,
    _watcher: Option<Watcher>,
}

#[derive(Default)]
struct Inner {
    project: Option<Open>,
    scene: Option<Scene>,
    /// Whether `scene` came from a run or render this session. A scene read from an earlier build/ at open doesn't
    /// hold back the next run: its generation could be from a counter that has since been reset.
    scene_current: bool,
    error: Option<Problem>,
    running: bool,
    notice: Option<String>,
}

impl Inner {
    /// Shows `scene` unless one from a later run is already on screen (the generation check). True if it was taken.
    fn accept(&mut self, scene: Scene) -> bool {
        if self.scene_current && self.scene.as_ref().is_some_and(|s| s.generation > scene.generation) {
            return false;
        }
        self.scene = Some(scene);
        self.scene_current = true;
        self.error = None;
        true
    }
}

struct BusAnswer {
    png: Option<String>,
    camera: Option<Value>,
    error: Option<String>,
}

pub struct AppState {
    pub data: DataDir,
    pub(crate) emmet: PathBuf,
    pub(crate) chat: crate::chat::Chat,
    pub(crate) jobs: crate::setup::Jobs,
    inner: Mutex<Inner>,
    runs: OnceLock<RunQueue>,
    pending: Mutex<HashMap<u64, mpsc::Sender<BusAnswer>>>,
    next: AtomicU64,
    /// One params.json write at a time: each reads the file, checks the values with Python, and writes, so two at
    /// once could lose one's change. Held on a blocking thread, never across an await.
    params: Mutex<()>,
    viewing: Mutex<Viewing>,
    pub render: Option<RenderJob>,
}

impl AppState {
    pub fn new(data: DataDir, emmet: PathBuf, render: Option<RenderJob>) -> Self {
        Self {
            data,
            emmet,
            chat: crate::chat::Chat::default(),
            jobs: crate::setup::Jobs::default(),
            inner: Mutex::new(Inner::default()),
            runs: OnceLock::new(),
            pending: Mutex::new(HashMap::new()),
            next: AtomicU64::new(1),
            params: Mutex::new(()),
            viewing: Mutex::new(Viewing {
                selection: vec![],
                view: "iso".into(),
            }),
            render,
        }
    }

    /// Starts the run queue; its events go to the window.
    pub fn start_runs(&self, app: &AppHandle) {
        let handle = app.clone();
        let queue = RunQueue::start(self.emmet.clone(), self.data.root().to_path_buf(), move |ev| {
            on_run_event(&handle, ev)
        });
        let _ = self.runs.set(queue);
    }

    pub(crate) fn request_run(&self, root: &Path) {
        if let Some(q) = self.runs.get() {
            q.request(root);
        }
    }

    pub(crate) fn open_root(&self) -> Option<PathBuf> {
        self.inner
            .lock()
            .expect("state")
            .project
            .as_ref()
            .map(|o| o.root.clone())
    }

    /// Marks the project at `root` open, as [`open`] does, without a window, a watcher, or a run: for the run tests.
    #[doc(hidden)]
    pub fn open_without_window(&self, root: &Path) {
        self.inner.lock().expect("state").project = Some(Open {
            root: root.to_path_buf(),
            view: ProjectView {
                root: root.display().to_string(),
                name: String::new(),
                trusted: true,
                watching: false,
                imported: None,
                build: crate::projects::BuildSettings::default(),
            },
            _watcher: None,
        });
    }

    /// The open project's view as its files say now, when `project` is the open one: a restore wrote back its
    /// emmet.toml (the build settings) and model.py (whether it's still an import).
    pub fn refresh_view(&self, project: &Project) -> Option<ProjectView> {
        let mut inner = self.inner.lock().expect("state");
        let open = inner.project.as_mut().filter(|o| o.root == project.root)?;
        open.view.imported = project.imported_file();
        open.view.build = crate::projects::BuildSettings::from(&project.config.build);
        Some(open.view.clone())
    }

    /// Records something to tell the user that isn't a run's error (see [`UiState::notice`]).
    pub fn notify(&self, notice: impl Into<String>) {
        self.inner.lock().expect("state").notice = Some(notice.into());
    }

    /// One params.json write at a time (see [`AppState::params`]).
    pub(crate) fn lock_params(&self) -> std::sync::MutexGuard<'_, ()> {
        self.params.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn project(&self) -> Result<Project, Box<Problem>> {
        let root = self
            .open_root()
            .ok_or_else(|| Problem::internal("No project is open."))?;
        Ok(Project::find(Some(&root))?)
    }
}

/// Reads a scene from the project's build/, if it has one.
fn read_scene(root: &Path) -> Option<Scene> {
    let placements: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("build/placements.json")).ok()?).ok()?;
    let manifest: Value = std::fs::read_to_string(root.join("build/manifest.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    scene_from(&placements, &manifest, None)
}

fn scene_from(placements: &Value, manifest: &Value, ms: Option<u64>) -> Option<Scene> {
    let list: Vec<Placement> = serde_json::from_value(placements["parts"].clone()).ok()?;
    Some(Scene {
        generation: manifest["run"]["generation"].as_u64().unwrap_or(0),
        model: placements["model"].as_str().unwrap_or_default().to_string(),
        parts: list.len() as u32,
        steps: manifest["steps"].as_u64().unwrap_or(0) as u32,
        placements: list,
        ms,
        checks: None,
        bom: None,
        tree: None,
    })
}

/// A scene from a placements.json alone: a saved version's build, opened to compare.
pub(crate) fn scene_at(placements: &Path) -> Option<Scene> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(placements).ok()?).ok()?;
    scene_from(&v, &Value::Null, None)
}

/// The Steps tab's tree from build/steps.json.
fn steps_tree(root: &Path) -> Option<StepsTree> {
    let text = std::fs::read_to_string(root.join("build/steps.json")).ok()?;
    let steps: emmet_core::steps::Steps = serde_json::from_str(&text).ok()?;
    let view = |list: &[emmet_core::steps::Step]| {
        list.iter()
            .map(|s| StepView {
                n: s.n,
                title: s.title.clone(),
                group: s.group.clone(),
                parts: (s.part_ids.len() + s.submodels.iter().map(|p| p.count).sum::<usize>()) as u32,
            })
            .collect::<Vec<_>>()
    };
    Some(StepsTree {
        steps: view(&steps.steps),
        submodels: steps
            .submodels
            .iter()
            .map(|m| SubmodelView {
                name: m.name.clone(),
                file: m.file.clone(),
                count: m.count as u32,
                parts: m.parts as u32,
                steps: view(&m.steps),
            })
            .collect(),
    })
}

/// Adds the validator's findings and the parts list to a scene read from the project's build/: the same
/// build/validation.json and build/bom.json `emmet check` and `emmet bom` write, made in-process.
fn annotate(data: &DataDir, root: &Path, scene: &mut Scene) {
    let project = match Project::find(Some(root)) {
        Ok(p) => p,
        Err(e) => {
            scene.checks = Some(Checks {
                findings: vec![],
                allowed: vec![],
                errors: 0,
                clean: false,
                problem: Some(e.message),
            });
            return;
        }
    };
    scene.tree = steps_tree(root);
    scene.checks = Some(match emmet_core::ops::check_build(data, &project, &[]) {
        Ok(v) => Checks {
            allowed: v
                .allowed
                .iter()
                .map(|a| AllowedView {
                    rule: a.rule.clone(),
                    parts: a.parts.clone(),
                    reason: a.reason.clone(),
                })
                .collect(),
            errors: v.errors() as u32,
            clean: v.clean,
            findings: v
                .findings
                .into_iter()
                .map(|f| Finding {
                    id: f.id,
                    rule: f.rule.name().into(),
                    severity: serde_json::to_value(f.severity)
                        .ok()
                        .and_then(|s| s.as_str().map(String::from))
                        .unwrap_or_default(),
                    parts: f.parts,
                    source: f
                        .source
                        .into_iter()
                        .map(|s| SourceLine {
                            text: source_text(&project.root, &s.file, s.line),
                            file: s.file,
                            line: s.line,
                        })
                        .collect(),
                    message: f.message,
                    hint: f.hint,
                })
                .collect(),
            problem: None,
        },
        Err(e) => Checks {
            findings: vec![],
            allowed: vec![],
            errors: 0,
            clean: false,
            problem: Some(format!("{} {}", e.message, e.hint)),
        },
    });
    let names = emmet_core::bom::ColorNames::preferred(data);
    scene.bom = Some(match emmet_core::ops::bom_build(data, &project, names) {
        Ok(b) => Bom {
            pieces: b.totals.pieces as u32,
            parts: b.totals.parts as u32,
            parts_data: b.parts_data.is_some(),
            problem: None,
            lines: b
                .lines
                .into_iter()
                .map(|l| BomLine {
                    availability: serde_json::to_value(l.availability.status)
                        .ok()
                        .and_then(|s| s.as_str().map(String::from))
                        .unwrap_or_default(),
                    part: l.part,
                    name: l.name,
                    color: l.color,
                    color_name: l.color_name,
                    qty: l.qty as u32,
                })
                .collect(),
        },
        Err(e) => Bom {
            lines: vec![],
            pieces: 0,
            parts: 0,
            parts_data: false,
            problem: Some(format!("{} {}", e.message, e.hint)),
        },
    });
}

/// Line `line` of `file`, which must be a plain relative path inside the project: the validator names the model's own
/// files, but a model could make its source lines name anything.
fn source_text(root: &Path, file: &str, line: u32) -> Option<String> {
    let rel = Path::new(file);
    if !rel.components().all(|c| matches!(c, std::path::Component::Normal(_))) {
        return None;
    }
    let path = crate::paths::inside(root, &root.join(rel))?;
    let text = std::fs::read_to_string(path).ok()?;
    text.lines()
        .nth(line.checked_sub(1)? as usize)
        .map(|l| l.trim_end().to_string())
}

fn on_run_event(app: &AppHandle, ev: RunEvent) {
    let state = app.state::<AppState>();
    match ev {
        RunEvent::Started { project } => {
            {
                let mut inner = state.inner.lock().expect("state");
                if inner.project.as_ref().map(|p| p.root.as_path()) != Some(project.as_path()) {
                    return;
                }
                inner.running = true;
            }
            let _ = RunStarted {}.emit(app);
        }
        RunEvent::Finished(o) => {
            if o.canceled {
                return;
            }
            if let Some(finished) = finish(&state, &o) {
                let _ = finished.emit(app);
            }
        }
    }
}

/// Applies a run's outcome to the state and says what the window should show. A run of the open project always ends
/// with one, so the window never stays on "rebuilding"; a run of a project that's no longer open ends with none.
pub fn finish(state: &AppState, o: &Outcome) -> Option<RunFinished> {
    // The scene, checks, and parts list are made before the state is locked: the validator takes a few tens of ms.
    let built = match (&o.envelope, &o.placements) {
        (Some(env), Some(Ok(placements))) if env["ok"] == true => {
            let manifest = std::fs::read_to_string(o.project.join("build").join("manifest.json"))
                .ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                .unwrap_or_default();
            Some(scene_from(placements, &manifest, Some(o.ms)).map(|mut scene| {
                scene.generation = o.generation.unwrap_or(scene.generation);
                annotate(&state.data, &o.project, &mut scene);
                scene
            }))
        }
        _ => None,
    };
    let mut inner = state.inner.lock().expect("state");
    // Checked under the lock that applies the result: another project can open while the scene above is made, and
    // its window would take this one's scene as its own.
    if inner.project.as_ref().map(|p| p.root.as_path()) != Some(o.project.as_path()) {
        return None;
    }
    inner.running = false;
    let failed = |inner: &mut Inner, error: Problem| {
        inner.error = Some(error.clone());
        RunFinished {
            scene: None,
            error: Some(error),
            stale: false,
        }
    };
    let error = match (&o.envelope, &o.placements) {
        (Some(env), Some(Ok(_))) if env["ok"] == true => {
            let Some(Some(scene)) = built else {
                return Some(failed(
                    &mut inner,
                    *Problem::internal("build/placements.json isn't a list of placements this app can read."),
                ));
            };
            if !inner.accept(scene.clone()) {
                return Some(RunFinished {
                    scene: None,
                    error: None,
                    stale: true,
                });
            }
            return Some(RunFinished {
                scene: Some(scene),
                error: None,
                stale: false,
            });
        }
        (Some(env), Some(Err(why))) if env["ok"] == true => {
            *Problem::internal(format!("The run finished, but the app can't use its result: {why}."))
        }
        (Some(env), _) if env["ok"] == true => {
            *Problem::internal("The run finished but wrote no build/placements.json.")
        }
        (Some(env), _) => Problem::from_envelope(&env["error"]),
        (None, _) => {
            let said = if o.stderr.is_empty() {
                String::new()
            } else {
                format!(" It said: {}", o.stderr)
            };
            Problem {
                hint: "Run it again with Rebuild. If it keeps stopping, run emmet run in a terminal to see why.".into(),
                ..*Problem::internal(format!("emmet stopped before it finished the run ({}).{said}", o.exit))
            }
        }
    };
    Some(failed(&mut inner, error))
}

/// Opens the project at `dir`: watches it, shows its last build, and runs it if that's stale or missing.
pub fn open(app: &AppHandle, dir: &Path) -> Result<ProjectView, Box<Problem>> {
    let state = app.state::<AppState>();
    let project = Project::find(Some(dir))?;
    let root = project.root.clone();
    // One chat per project: switching projects ends the old one (its history stays in its .emmet/).
    if state.open_root().as_deref() != Some(root.as_path()) {
        state.chat.close();
    }
    let handle = app.clone();
    let watched = root.clone();
    let watcher = Watcher::start(&root, move || handle.state::<AppState>().request_run(&watched)).ok();
    let view = ProjectView {
        root: root.display().to_string(),
        name: project.config.name.clone(),
        trusted: project.is_trusted(&state.data),
        watching: watcher.is_some(),
        imported: project.imported_file(),
        build: crate::projects::BuildSettings::from(&project.config.build),
    };
    let scene = read_scene(&root).map(|mut s| {
        annotate(&state.data, &root, &mut s);
        s
    });
    let fresh = emmet_core::library::require(&state.data)
        .and_then(|lib| {
            let parts_data = emmet_core::parts_data::version(&state.data)?;
            project.inputs_hash(&BTreeMap::new(), &lib.release, parts_data.as_deref())
        })
        .ok()
        .zip(
            std::fs::read_to_string(root.join("build/manifest.json"))
                .ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok()),
        )
        .is_some_and(|(hash, m)| m["inputs_hash"].as_str() == Some(hash.as_str()))
        && scene.is_some();
    {
        let mut inner = state.inner.lock().expect("state");
        inner.project = Some(Open {
            root: root.clone(),
            view: view.clone(),
            _watcher: watcher,
        });
        inner.scene = scene.clone();
        inner.scene_current = false;
        inner.error = None;
        inner.running = false;
    }
    recent::touch(&state.data, &root, &view.name);
    crate::menu::refresh(app);
    let _ = ProjectOpened {
        project: view.clone(),
        scene,
    }
    .emit(app);
    if !fresh {
        state.request_run(&root);
    }
    Ok(view)
}

/// Answers a bus message for the open project (plan section 09, docs/spec/bus.md).
pub fn on_bus(app: &AppHandle, msg: emmet_bus::Message) -> emmet_bus::Reply {
    use emmet_bus::{Message, Reply};
    let state = app.state::<AppState>();
    let open = state.open_root();
    let is_open = |project: &str| {
        let want = emmet_core::fsx::canonical(Path::new(project)).ok();
        want.is_some() && want == open
    };
    match msg {
        Message::Highlight { project, ids } => {
            if !is_open(&project) {
                return Reply::undelivered("that project isn't open in Emmet");
            }
            let _ = BusHighlight { ids: ids.clone() }.emit(app);
            Reply {
                delivered: true,
                highlighted: ids,
                ..Default::default()
            }
        }
        Message::Camera { project, view } => {
            if !is_open(&project) {
                return Reply::undelivered("that project isn't open in Emmet");
            }
            let mut viewing = state.viewing.lock().expect("viewing");
            if let Some(v) = view.filter(|v| VIEWS.contains(&v.as_str())) {
                let _ = BusCamera { view: v.clone() }.emit(app);
                viewing.view = v;
            }
            Reply {
                delivered: true,
                camera: Some(serde_json::json!({ "view": viewing.view })),
                ..Default::default()
            }
        }
        Message::Selection { project } => {
            if !is_open(&project) {
                return Reply::undelivered("that project isn't open in Emmet");
            }
            let viewing = state.viewing.lock().expect("viewing");
            Reply {
                delivered: true,
                camera: Some(serde_json::json!({ "view": viewing.view })),
                selected: viewing.selection.clone(),
                ..Default::default()
            }
        }
        Message::Render {
            project,
            view,
            size,
            highlight,
        } => {
            if !is_open(&project) {
                return Reply::undelivered("that project isn't open in Emmet");
            }
            let Some(root) = open.clone() else {
                return Reply::undelivered("that project isn't open in Emmet");
            };
            let id = state.next.fetch_add(1, Ordering::SeqCst);
            let (tx, rx) = mpsc::channel();
            state.pending.lock().expect("pending").insert(id, tx);
            // emmet render ran the model if it was out of date, so build/ may be newer than the window: show it
            // first, through the same generation check as a run.
            let scene = read_scene(&root)
                .map(|mut s| {
                    annotate(&state.data, &root, &mut s);
                    s
                })
                .filter(|s| state.inner.lock().expect("state").accept(s.clone()));
            let event = BusRender {
                id,
                view,
                size,
                highlight: highlight.clone(),
                scene,
            };
            if event.emit(app).is_err() {
                state.pending.lock().expect("pending").remove(&id);
                return Reply::undelivered("the window isn't listening");
            }
            let answer = rx.recv_timeout(BUS_RENDER_TIMEOUT);
            state.pending.lock().expect("pending").remove(&id);
            match answer {
                Ok(a) => Reply {
                    delivered: true,
                    png: a.png,
                    camera: a.camera,
                    error: a.error,
                    highlighted: highlight,
                    ..Default::default()
                },
                Err(_) => Reply::undelivered("the window didn't render in time"),
            }
        }
        msg => crate::chat::route(&state.chat.hub, msg),
    }
}

/// The window says what the user has selected and which view they're looking at, as either changes: what an agent's
/// `selection` and `camera` read.
#[tauri::command]
#[specta::specta]
pub fn viewer_state(state: State<'_, AppState>, selection: Vec<String>, view: String) {
    let mut viewing = state.viewing.lock().expect("viewing");
    viewing.selection = selection.into_iter().take(20_000).collect();
    if VIEWS.contains(&view.as_str()) {
        viewing.view = view;
    }
}

#[tauri::command]
#[specta::specta]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    let library = emmet_core::library::require(&state.data).ok().map(|i| i.release);
    AppInfo {
        recent: recent::list(&state.data),
        library_old: library.as_deref().is_some_and(crate::setup::library_old),
        library,
        pinned: crate::setup::library_pin().0,
        parts_data: emmet_core::parts_data::version(&state.data).ok().flatten(),
        python: emmet_core::python::status(&state.data).ready,
        version: env!("CARGO_PKG_VERSION").into(),
        uptime_ms: crate::started().elapsed().as_millis() as u64,
    }
}

#[tauri::command]
#[specta::specta]
pub fn ui_state(state: State<'_, AppState>) -> UiState {
    let inner = state.inner.lock().expect("state");
    UiState {
        project: inner.project.as_ref().map(|o| o.view.clone()),
        running: inner.running,
        scene: inner.scene.clone(),
        error: inner.error.clone(),
        notice: inner.notice.clone(),
    }
}

/// Opens a project from the recents list. Any other path is refused.
#[tauri::command]
#[specta::specta]
pub fn open_recent(app: AppHandle, path: String) -> Result<ProjectView, Box<Problem>> {
    let data = app.state::<AppState>().data.clone();
    let root = recent::find(&data, &path).ok_or_else(|| {
        Box::new(Problem {
            hint: "Open it with Open… instead.".into(),
            ..*Problem::internal(format!("{path} isn't a recent project."))
        })
    })?;
    open(&app, &root)
}

#[tauri::command]
#[specta::specta]
pub async fn get_params(app: AppHandle) -> Result<ParamsView, Box<Problem>> {
    let state = app.state::<AppState>();
    let (data, project) = (state.data.clone(), state.project()?);
    let params = tauri::async_runtime::spawn_blocking(move || emmet_core::ops::get_params(&data, &project))
        .await
        .map_err(|e| Problem::internal(e.to_string()))??;
    params_view(params)
}

/// Checks the values against the model's schema and writes params.json; the watcher then runs the model.
#[tauri::command]
#[specta::specta]
pub async fn set_params(app: AppHandle, values: BTreeMap<String, Value>) -> Result<ParamsView, Box<Problem>> {
    let (data, project) = {
        let state = app.state::<AppState>();
        (state.data.clone(), state.project()?)
    };
    let handle = app.clone();
    let params = tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        let _one = state.params.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        emmet_core::ops::set_params(&data, &project, &values)
    })
    .await
    .map_err(|e| Problem::internal(e.to_string()))??;
    params_view(params)
}

pub(crate) fn params_view(p: emmet_core::ops::Params) -> Result<ParamsView, Box<Problem>> {
    serde_json::to_value(p)
        .and_then(serde_json::from_value)
        .map_err(|e| Problem::internal(format!("the params reply: {e}")))
}

#[tauri::command]
#[specta::specta]
pub async fn list_colors(app: AppHandle) -> Result<Vec<ColorView>, Box<Problem>> {
    let data = app.state::<AppState>().data.clone();
    let list = tauri::async_runtime::spawn_blocking(move || emmet_core::ops::list_colors(&data, None))
        .await
        .map_err(|e| Problem::internal(e.to_string()))??;
    Ok(list
        .colors
        .into_iter()
        .map(|c| ColorView {
            code: c.code,
            name: c.name,
            rgb: c.rgb,
        })
        .collect())
}

/// Settings → Data: the installed parts data pack's date and the BOM's color naming.
#[tauri::command]
#[specta::specta]
pub async fn data_settings(app: AppHandle) -> Result<DataSettings, Box<Problem>> {
    let data = app.state::<AppState>().data.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<DataSettings, Box<Problem>> {
        let pack = emmet_core::parts_data::PartsData::open(&data)?;
        let names = serde_json::to_value(emmet_core::bom::ColorNames::preferred(&data)).unwrap_or_default();
        Ok(DataSettings {
            parts_data: pack.map(|p| p.manifest().date.clone()),
            color_names: names.as_str().unwrap_or("ldraw").to_string(),
        })
    })
    .await
    .map_err(|e| Problem::internal(e.to_string()))?
}

/// Saves the BOM's color naming (ldraw, rebrickable, or bricklink) in the data dir's settings.json, which `emmet bom`
/// and the MCP `bom` tool read too, and rebuilds the open project's parts list.
#[tauri::command]
#[specta::specta]
pub fn set_color_names(state: State<'_, AppState>, names: String) -> Result<(), Box<Problem>> {
    let value: emmet_core::bom::ColorNames = serde_json::from_value(Value::String(names.clone())).map_err(|_| {
        Box::new(Problem {
            code: "PARAM_INVALID".into(),
            message: format!("There's no color naming '{names}'."),
            hint: "Use ldraw, rebrickable, or bricklink.".into(),
            file: None,
            line: None,
            excerpt: None,
        })
    })?;
    let mut settings = crate::prefs::settings_to_change(&state.data)?;
    settings.insert("color_names".into(), serde_json::to_value(value).expect("json"));
    crate::prefs::write(&state.data, &settings)?;
    if let Some(root) = state.open_root() {
        state.request_run(&root);
    }
    Ok(())
}

/// Runs the open project's model again.
#[tauri::command]
#[specta::specta]
pub fn rebuild(state: State<'_, AppState>) -> Result<(), Box<Problem>> {
    let root = state
        .open_root()
        .ok_or_else(|| Problem::internal("No project is open."))?;
    state.request_run(&root);
    Ok(())
}

/// The window's answer to a [`BusRender`].
#[tauri::command]
#[specta::specta]
pub fn bus_reply(
    state: State<'_, AppState>,
    id: u64,
    png: Option<String>,
    camera: Option<Value>,
    error: Option<String>,
) {
    if let Some(tx) = state.pending.lock().expect("pending").remove(&id) {
        let _ = tx.send(BusAnswer { png, camera, error });
    }
}

/// Render mode: what to render.
#[tauri::command]
#[specta::specta]
pub fn render_job(state: State<'_, AppState>) -> Option<RenderJob> {
    state.render.clone()
}

/// Render mode: the PNG (base64) and camera. Writes them where `--out` says and exits.
#[tauri::command]
#[specta::specta]
pub fn render_done(png: String, camera: Value) {
    crate::finish_render(Ok((png, camera)));
}

/// Render mode: why the render failed. Exits with an error.
#[tauri::command]
#[specta::specta]
pub fn render_failed(message: String) {
    crate::finish_render(Err(message));
}

/// The render mode's job from `--render <project>`: the project's last build.
pub fn render_job_for(dir: &Path, view: &str, size: u32, highlight: Vec<String>) -> Result<RenderJob, String> {
    let project = Project::find(Some(dir)).map_err(|e| e.message)?;
    let scene = read_scene(&project.root).ok_or_else(|| {
        format!(
            "{} has no build/placements.json; run emmet run first.",
            project.root.display()
        )
    })?;
    Ok(RenderJob {
        scene,
        view: view.into(),
        size,
        highlight,
    })
}
