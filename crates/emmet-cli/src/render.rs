// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet render` and the MCP `render` tool (plan sections 09 and 13, D-24): a PNG through the viewer's own code. The
//! running app renders it over the bus when it has the project open; when the bus doesn't deliver (no app, another
//! project open, or no answer in time), emmet starts the app's render mode, a hidden window that renders and exits
//! (ADR 0005).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use emmet_bus::Message;
use emmet_core::data::DataDir;
use emmet_core::envelope::{Error, ErrorCode, Result};
use emmet_core::project::Project;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VIEWS: [&str; 6] = ["iso", "front", "back", "left", "right", "top"];
/// How long the open app may take to answer (its own window gives up at 25 s).
const BUS_TIMEOUT: Duration = Duration::from_secs(30);
/// How long render mode may take (a cold WebView2 on CI loads parts slowly; the app gives up at 100 s).
const RENDER_MODE_TIMEOUT: Duration = Duration::from_secs(120);

/// What a render produced; the PNG itself goes to a file (CLI) or into image content (MCP).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Rendered {
    pub view: String,
    pub size: u32,
    /// "app" (the running app, over the bus) or "render-mode" (a hidden window of the app).
    pub via: String,
    /// The render's camera: `{view, position, target, fov}` in the viewer's frame.
    pub camera: Option<serde_json::Value>,
    pub highlighted: Vec<String>,
    /// Where the PNG was written (CLI only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub bytes: usize,
}

pub struct Png {
    pub bytes: Vec<u8>,
    pub info: Rendered,
}

/// Renders the project's current model, running it first if build/ is missing or out of date.
pub fn render(data: &DataDir, project: &Project, view: &str, size: u32, highlight: &[String]) -> Result<Png> {
    if !VIEWS.contains(&view) {
        return Err(Error::new(
            ErrorCode::ParamInvalid,
            format!("There's no view '{view}'."),
            format!("Use one of {}.", VIEWS.join(", ")),
        ));
    }
    let size = size.clamp(16, 4096);
    emmet_core::ops::ensure_built(data, project)?;
    let highlight = &expand_findings(project, highlight)?;
    let root = project.root.display().to_string();
    let reply = emmet_bus::send(
        data,
        Message::Render {
            project: root.clone(),
            view: view.into(),
            size,
            highlight: highlight.to_vec(),
        },
        BUS_TIMEOUT,
    );
    if reply.delivered {
        if let Some(e) = reply.error {
            return Err(unavailable(format!("The Emmet app couldn't render: {e}")));
        }
        let bytes = reply
            .png
            .as_deref()
            .and_then(emmet_core::base64::decode)
            .ok_or_else(|| unavailable("The Emmet app replied without a PNG."))?;
        return Ok(Png {
            info: info(view, size, "app", reply.camera, reply.highlighted, &bytes),
            bytes,
        });
    }
    let why = reply.reason.unwrap_or_default();
    render_mode(data, project, view, size, highlight).map_err(|mut e| {
        // Say why the open app wasn't used, too.
        e.message = format!("{} (The open app wasn't used: {why}.)", e.message);
        e
    })
}

fn info(
    view: &str,
    size: u32,
    via: &str,
    camera: Option<serde_json::Value>,
    highlighted: Vec<String>,
    bytes: &[u8],
) -> Rendered {
    Rendered {
        view: view.into(),
        size,
        via: via.into(),
        camera,
        highlighted,
        path: None,
        bytes: bytes.len(),
    }
}

/// The highlight list with each finding id (`F-0003`) replaced by its parts, from build/validation.json.
pub fn expand_findings(project: &Project, highlight: &[String]) -> Result<Vec<String>> {
    if !highlight.iter().any(|h| is_finding_id(h)) {
        return Ok(highlight.to_vec());
    }
    let path = project.root.join("build/validation.json");
    let validation: Option<emmet_core::check::Validation> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok());
    let mut out = Vec::new();
    for h in highlight {
        if !is_finding_id(h) {
            out.push(h.clone());
            continue;
        }
        let finding = validation.as_ref().and_then(|v| v.finding(h)).ok_or_else(|| {
            Error::new(
                ErrorCode::ParamInvalid,
                format!("There's no finding {h} in build/validation.json."),
                "Run validate (emmet check) first; finding ids come from its latest result.",
            )
        })?;
        out.extend(finding.parts.iter().cloned());
    }
    Ok(out)
}

pub fn is_finding_id(s: &str) -> bool {
    let s = s.trim();
    s.len() > 2 && s[..2].eq_ignore_ascii_case("F-") && s[2..].bytes().all(|b| b.is_ascii_digit())
}

fn unavailable(message: impl Into<String>) -> Error {
    Error::new(
        ErrorCode::RenderUnavailable,
        message,
        "Open Emmet, or check the GPU drivers (Windows).",
    )
}

/// The app binary: `EMMET_APP`, else next to this emmet (the app bundle's sidecar layout, or target/<profile>/ in
/// development). Names match exactly: on a case-insensitive disk `Emmet` would otherwise find this `emmet`.
fn app_binary() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("EMMET_APP") {
        return Some(PathBuf::from(p));
    }
    let me = std::env::current_exe().ok()?;
    let names: Vec<String> = std::fs::read_dir(me.parent()?)
        .ok()?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    ["Emmet", "emmet-desktop"]
        .iter()
        .map(|n| format!("{n}{}", std::env::consts::EXE_SUFFIX))
        .find(|n| names.contains(n))
        .map(|n| me.with_file_name(n))
}

/// `Emmet --render <project> --view … --size … --out …` in a hidden window, then the PNG it wrote.
fn render_mode(data: &DataDir, project: &Project, view: &str, size: u32, highlight: &[String]) -> Result<Png> {
    let app = app_binary()
        .ok_or_else(|| unavailable("The Emmet app isn't installed next to emmet, so render mode can't run."))?;
    // One file per render: an MCP server can render several views at once.
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let out = project
        .build_dir()?
        .join("renders")
        .join(format!(".render-{}-{n}.png", std::process::id()));
    std::fs::create_dir_all(out.parent().expect("renders/")).map_err(|e| Error::io(&out, e))?;
    let _ = std::fs::remove_file(&out);
    let mut cmd = Command::new(&app);
    cmd.arg("--render")
        .arg(&project.root)
        .args(["--view", view, "--size", &size.to_string(), "--out"])
        .arg(&out)
        .arg("--data-dir")
        .arg(data.root())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if !highlight.is_empty() {
        cmd.args(["--highlight", &highlight.join(",")]);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| unavailable(format!("Can't start {}: {e}", app.display())))?;
    let stderr = child.stderr.take().expect("piped stderr");
    let log = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::Read::read_to_string(&mut std::io::BufReader::new(stderr), &mut s);
        s
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) if started.elapsed() > RENDER_MODE_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(unavailable(format!("waiting for the render: {e}"))),
        }
    };
    let log = log.join().unwrap_or_default();
    // The app's own "render failed: …" line, else a panic's, never Rust's "note: run with RUST_BACKTRACE" line.
    let failed = |why: String| {
        let lines: Vec<&str> = log
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("note:"))
            .collect();
        let said = lines
            .iter()
            .rev()
            .find(|l| l.starts_with("render failed:"))
            .or_else(|| lines.iter().rev().find(|l| l.contains("panicked at")))
            .or(lines.last())
            .copied()
            .unwrap_or("");
        unavailable(if said.is_empty() { why } else { format!("{why}: {said}") })
    };
    match status {
        None => {
            return Err(failed(format!(
                "The render took over {} s",
                RENDER_MODE_TIMEOUT.as_secs()
            )));
        }
        Some(s) if !s.success() => return Err(failed("The render failed".into())),
        Some(_) => {}
    }
    let bytes = std::fs::read(&out).map_err(|_| failed("The render wrote no PNG".into()))?;
    let _ = std::fs::remove_file(&out);
    let camera = std::fs::read_to_string(out.with_extension("json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok());
    let _ = std::fs::remove_file(out.with_extension("json"));
    Ok(Png {
        info: info(view, size, "render-mode", camera, highlight.to_vec(), &bytes),
        bytes,
    })
}

/// The default output path: build/renders/<view>.png.
pub fn default_out(project: &Project, view: &str) -> Result<PathBuf> {
    Ok(project.build_dir()?.join("renders").join(format!("{view}.png")))
}

pub fn write_png(path: &Path, png: &Png) -> Result<()> {
    emmet_core::fsx::write_atomic(path, &png.bytes)
}
