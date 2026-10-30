// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Python runtime models run in (plan sections 04, 06, 15; D-05): uv (the bundled sidecar, or `uv` on PATH in
//! dev) creates a Python 3.13 venv in app data, and emmet installs the `emmet_dsl` wheel it builds from the sources
//! embedded in this binary, with no network. A stamp file holds the sources' hash, so a newer emmet reinstalls.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::DataDir;
use crate::envelope::{Error, ErrorCode, Result};
use crate::fsx::{self, write_atomic};

pub const PYTHON_VERSION: &str = "3.13";

macro_rules! dsl_files {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("../../../python/emmet_dsl/src/emmet_dsl/", $name)))),*]
    };
}

/// The `emmet_dsl` package, file by file. `tests/it/tests/python.rs` checks this list against the folder.
pub const DSL_FILES: &[(&str, &str)] = dsl_files![
    "__init__.py",
    "api.py",
    "core.py",
    "emit.py",
    "errors.py",
    "frame.py",
    "ldraw.py",
    "params.py",
    "parts.py",
    "rewrite.py",
    "run.py"
];

macro_rules! technique_files {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("../../../data/techniques/", $name, ".py")))),*]
    };
}

/// The technique library (data/techniques/, plan section 08), installed as `emmet_dsl.techniques`, so `m.use(name)`
/// imports it. `tests/it/tests/techniques.rs` checks this list against the folder.
pub const TECHNIQUES: &[(&str, &str)] = technique_files![
    "angled_hull_panel",
    "antenna_mast",
    "arrow_slit",
    "ball_joint_arm",
    "balustrade_parapet",
    "bartizan",
    "baseplate_road",
    "battlements",
    "bench",
    "bermed_wall_reveal",
    "bracket_side_panel",
    "brick_built_arch",
    "bush",
    "buttress",
    "cab_seat",
    "cannon_port",
    "canopy",
    "cargo_pallet",
    "cave_mouth",
    "chimney_stack",
    "clapboard_siding",
    "cliff_wall",
    "cockpit_canopy",
    "color_block_facade",
    "control_console",
    "corbelled_balcony",
    "crank_winch",
    "crows_nest",
    "curved_hood",
    "curved_wall_end",
    "dentil_band",
    "dirt_path",
    "dock_pier",
    "door_frame",
    "door_hood",
    "downpipe",
    "drawbridge",
    "engine_nozzle",
    "exhaust_stack",
    "exposed_stud_stonework",
    "fence",
    "flagstone_paving",
    "flower_bed",
    "footbridge",
    "front_bumper",
    "front_grille",
    "gable_roof",
    "gear_pair",
    "grass_edge",
    "greeble_strip",
    "half_timber_band",
    "hatch_on_hinge",
    "headlights",
    "hillside",
    "hinge_roof",
    "hinged_tailgate",
    "hip_roof",
    "irregular_stone_wall",
    "jettied_floor",
    "jumper_offset",
    "lamp_post",
    "landing_leg",
    "landing_zone",
    "lattice_skirt_foundation",
    "lift_arm_linkage",
    "lift_off_floor",
    "lift_off_roof_seat",
    "light_bar",
    "machicolation",
    "masonry_wall",
    "mast_and_yard",
    "oars",
    "panel_pilaster",
    "pine_tree",
    "plaster_with_exposed_stones",
    "platform_edge",
    "pond",
    "porch_posts",
    "portcullis",
    "portholes",
    "quoin_corner",
    "radar_dish",
    "rear_spoiler",
    "ridge_cap",
    "river",
    "rock_facet",
    "rock_outcrop",
    "rock_spire",
    "roof_dormer",
    "rotating_platform",
    "rough_stone_ledge",
    "round_tower",
    "rudder_hinge",
    "rusticated_stone_pilaster",
    "ship_hull",
    "shop_window",
    "shuttered_window",
    "sidewalk",
    "snot_panel",
    "snow_cap",
    "solar_panel",
    "spliced_chassis",
    "stairs",
    "string_course",
    "surf_line",
    "tail_lights",
    "technic_brace",
    "technic_pin_hinge",
    "technic_spine",
    "thick_wall",
    "tiled_floor",
    "tilting_hood",
    "timber_frame_panels",
    "torch_sconce",
    "track_curve",
    "track_straight_run",
    "track_switch",
    "trailer_hitch",
    "train_carriage",
    "tree",
    "turret",
    "two_story_color_split",
    "two_tone_masonry",
    "wainscot_split_wall",
    "wall_banner",
    "wall_top_cornice",
    "wall_top_tie",
    "wall_vent",
    "waterfall",
    "wedge_wing",
    "white_ashlar_wall",
    "window_frame",
    "window_hood",
    "window_sill",
    "windscreen",
    "wooden_hoarding",
    "woodpile",
    "worm_drive",
];

/// `__version__` from the embedded `__init__.py`.
pub fn dsl_version() -> &'static str {
    let init = DSL_FILES[0].1;
    init.lines()
        .find_map(|l| l.strip_prefix("__version__ = \"").and_then(|r| r.strip_suffix('"')))
        .expect("emmet_dsl/__init__.py sets __version__")
}

/// Every file the wheel installs under `emmet_dsl/`: the package, then `techniques/`.
fn package_files() -> Vec<(String, &'static str)> {
    let mut files: Vec<(String, &'static str)> = DSL_FILES.iter().map(|(n, t)| (n.to_string(), *t)).collect();
    files.push(("techniques/__init__.py".into(), TECHNIQUES_INIT));
    files.extend(TECHNIQUES.iter().map(|(n, t)| (format!("techniques/{n}.py"), *t)));
    files
}

const TECHNIQUES_INIT: &str = "# SPDX-License-Identifier: Apache-2.0\n\"\"\"Emmet's technique library (data/techniques/ in the Emmet repo); m.use(name) applies one.\"\"\"\n";

fn dsl_hash() -> String {
    let mut all = Vec::new();
    for (name, text) in package_files() {
        all.extend_from_slice(name.as_bytes());
        all.push(0);
        all.extend_from_slice(text.as_bytes());
        all.push(0);
    }
    fsx::sha256_hex(&all)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Status {
    /// The uv emmet would use, or None.
    pub uv: Option<String>,
    /// The venv's Python, when it exists.
    pub python: Option<String>,
    /// The installed emmet_dsl version, when it matches this emmet.
    pub emmet_dsl: Option<String>,
    pub ready: bool,
}

fn venv(data: &DataDir) -> PathBuf {
    data.python().join("venv")
}

/// The venv's interpreter.
pub fn interpreter(data: &DataDir) -> PathBuf {
    if cfg!(windows) {
        venv(data).join("Scripts/python.exe")
    } else {
        venv(data).join("bin/python3")
    }
}

fn stamp(data: &DataDir) -> PathBuf {
    venv(data).join("emmet-dsl.stamp")
}

/// The uv sidecar next to this executable, else `uv` on PATH.
pub fn find_uv() -> Option<PathBuf> {
    let name = format!("uv{}", std::env::consts::EXE_SUFFIX);
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    {
        let sidecar = dir.join(&name);
        if sidecar.is_file() {
            return Some(sidecar);
        }
    }
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(&name))
        .find(|p| p.is_file())
}

pub fn status(data: &DataDir) -> Status {
    let python = interpreter(data);
    let installed = std::fs::read_to_string(stamp(data))
        .ok()
        .filter(|s| s.trim() == dsl_hash());
    let python = python.is_file().then(|| python.display().to_string());
    let ready = python.is_some() && installed.is_some();
    Status {
        uv: find_uv().map(|p| p.display().to_string()),
        python,
        emmet_dsl: installed.map(|_| dsl_version().to_string()),
        ready,
    }
}

fn missing(message: impl Into<String>) -> Error {
    Error::new(ErrorCode::PythonMissing, message, "Run emmet doctor --fix.")
}

/// The interpreter, bootstrapping Python and emmet_dsl first if they're missing or stale.
pub fn ensure(data: &DataDir) -> Result<PathBuf> {
    if status(data).ready {
        return Ok(interpreter(data));
    }
    let _lock = Lock::take(&data.python().join("bootstrap.lock"))?;
    if status(data).ready {
        return Ok(interpreter(data)); // another emmet finished it while we waited
    }
    let uv = find_uv().ok_or_else(|| {
        Error::new(
            ErrorCode::PythonMissing,
            "Python isn't set up for models yet, and uv (which installs it) isn't next to emmet or on PATH.",
            "Install uv (https://docs.astral.sh/uv/), then run emmet doctor --fix.",
        )
    })?;
    let uv_failed = |message: String| {
        Error::new(
            ErrorCode::PythonMissing,
            message,
            "Run emmet doctor --fix. If it fails the same way, check the network (uv downloads Python once).",
        )
    };
    if !interpreter(data).is_file() {
        let venv_cmd = || {
            uv_command(&uv, data)
                .args(["venv", "--quiet", "--python", PYTHON_VERSION])
                .arg(venv(data))
                .output()
                .map_err(|e| missing(format!("Can't run uv at {}: {e}", uv.display())))
        };
        let mut out = venv_cmd()?;
        if !out.status.success() {
            // A half-installed runtime (an interrupted download, files restored from a backup) can trip uv. emmet
            // owns these folders, so clear them and start once more from nothing.
            for dir in ["venv", "installs"] {
                let path = data.python().join(dir);
                let _ = std::fs::remove_dir_all(&path).or_else(|_| std::fs::remove_file(&path));
            }
            out = venv_cmd()?;
        }
        if !out.status.success() {
            let why = tail(&out.stderr);
            return Err(uv_failed(format!(
                "uv couldn't create the Python {PYTHON_VERSION} environment: {why}"
            )));
        }
    }
    let wheel = data
        .python()
        .join(format!("emmet_dsl-{}-py3-none-any.whl", dsl_version()));
    write_atomic(&wheel, &build_wheel())?;
    let out = uv_command(&uv, data)
        .args([
            "pip",
            "install",
            "--quiet",
            "--offline",
            "--no-index",
            "--no-deps",
            "--reinstall",
            "--compile-bytecode",
        ])
        .arg("--python")
        .arg(interpreter(data))
        .arg(&wheel)
        .output()
        .map_err(|e| missing(format!("Can't run uv at {}: {e}", uv.display())))?;
    if !out.status.success() {
        return Err(uv_failed(format!(
            "uv couldn't install emmet_dsl: {}",
            tail(&out.stderr)
        )));
    }
    write_atomic(&stamp(data), dsl_hash().as_bytes())?;
    Ok(interpreter(data))
}

fn tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(3)..].join(" / ")
}

/// uv with its Pythons and cache in app data, no user config, and only the environment it needs (no API keys).
fn uv_command(uv: &Path, data: &DataDir) -> Command {
    let mut cmd = Command::new(uv);
    cmd.env_clear();
    for key in [
        "PATH",
        "HOME",
        "USERPROFILE",
        "SYSTEMROOT",
        "WINDIR",
        "TEMP",
        "TMP",
        "TMPDIR",
        "HTTPS_PROXY",
        "HTTP_PROXY",
        "NO_PROXY",
        "https_proxy",
        "http_proxy",
        "no_proxy",
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
        "LOCALAPPDATA",
        "APPDATA",
        "UV_PYTHON_INSTALL_MIRROR",
    ] {
        if let Some(v) = std::env::var_os(key) {
            cmd.env(key, v);
        }
    }
    cmd.env("UV_PYTHON_INSTALL_DIR", data.python().join("installs"))
        .env("UV_CACHE_DIR", data.python().join("uv-cache"))
        .env("UV_NO_CONFIG", "1")
        .env("UV_PYTHON_PREFERENCE", "only-managed")
        .stdin(Stdio::null());
    cmd
}

/// A wheel (a zip with a dist-info folder) holding the embedded emmet_dsl sources.
fn build_wheel() -> Vec<u8> {
    let version = dsl_version();
    let dist = format!("emmet_dsl-{version}.dist-info");
    let mut files: Vec<(String, Vec<u8>)> = package_files()
        .into_iter()
        .map(|(n, t)| (format!("emmet_dsl/{n}"), t.as_bytes().to_vec()))
        .collect();
    files.push((
        format!("{dist}/METADATA"),
        format!("Metadata-Version: 2.1\nName: emmet-dsl\nVersion: {version}\n").into(),
    ));
    files.push((
        format!("{dist}/WHEEL"),
        b"Wheel-Version: 1.0\nGenerator: emmet\nRoot-Is-Purelib: true\nTag: py3-none-any\n".to_vec(),
    ));
    let mut record = String::new();
    for (name, bytes) in &files {
        let digest = sha2::Digest::finalize(<sha2::Sha256 as sha2::Digest>::new_with_prefix(bytes));
        record.push_str(&format!(
            "{name},sha256={},{}\n",
            crate::base64::encode_url(&digest),
            bytes.len()
        ));
    }
    record.push_str(&format!("{dist}/RECORD,,\n"));
    files.push((format!("{dist}/RECORD"), record.into_bytes()));

    let mut buf = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut buf);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    for (name, bytes) in &files {
        zip.start_file(name.as_str(), opts).expect("wheel entry");
        zip.write_all(bytes).expect("wheel write");
    }
    zip.finish().expect("wheel finish");
    buf.into_inner()
}

/// A lock file held while bootstrapping, so two emmets don't build the venv at once. A lock older than 10 minutes is
/// a crashed bootstrap's and is taken over.
struct Lock(PathBuf);

impl Lock {
    fn take(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
        }
        let started = Instant::now();
        loop {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(_) => return Ok(Self(path.to_path_buf())),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = std::fs::metadata(path)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .is_some_and(|age| age > Duration::from_secs(600));
                    if stale {
                        let _ = std::fs::remove_file(path);
                    } else if started.elapsed() > Duration::from_secs(600) {
                        return Err(missing("Another emmet has been setting up Python for 10 minutes."));
                    } else {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
                Err(e) => return Err(Error::io(path, e)),
            }
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
