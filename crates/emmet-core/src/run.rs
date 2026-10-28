// SPDX-License-Identifier: AGPL-3.0-or-later
//! Running a model (plan sections 07, 08, 15): `python -I -B -X utf8 -m emmet_dsl.run` in the project folder with a scrubbed
//! environment, a timeout, and capped output. emmet writes the part geometry and the color table the DSL reads; the
//! DSL writes model.mpd, placements.json, and manifest.json.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::data::DataDir;
use crate::envelope::{Details, Error, ErrorCode, Result};
use crate::fsx::write_atomic;
use crate::index::Index;
use crate::project::Project;

/// Caps from plan section 15.
pub const MAX_PARTS: usize = 20_000;
const MAX_LOG: usize = 10 << 20;
/// Passes over a model that keeps asking for new parts. Each pass reports every part it lacks, so a model needs two
/// unless the parts it picks depend on other parts' sizes.
const MAX_PASSES: usize = 16;
/// How long to wait, after Python exits, for the rest of its output. A process the model started and left running
/// can hold the pipes open; emmet doesn't wait for it.
const PIPE_GRACE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Bbox {
    /// [x, y, z] in studs, studs, plates.
    pub min: [f64; 3],
    pub max: [f64; 3],
}

/// What `run_model` returns (plan section 09).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunSummary {
    pub parts: usize,
    pub steps: usize,
    pub submodels: usize,
    /// The box the model covers, or None for an empty model.
    pub bbox: Option<Bbox>,
    pub ms: u64,
    pub generation: u64,
}

/// What a run knows about itself, for the envelope's meta.
#[derive(Debug, Clone, Default)]
pub struct RunMeta {
    pub inputs_hash: Option<String>,
    pub generation: Option<u64>,
}

/// One request to the DSL.
pub(crate) struct Request<'a> {
    pub mode: &'a str,
    pub params: &'a BTreeMap<String, Value>,
    pub params_source: &'a str,
    pub manifest: Value,
    /// More keys for the mode (write_default's param, value, and kind).
    pub extra: Value,
}

/// The DSL's reply line.
#[derive(Debug, Deserialize)]
struct Reply {
    ok: bool,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    error: Option<Error>,
    #[serde(default)]
    need: Vec<Need>,
}

/// The data of a successful run reply.
#[derive(Debug, Deserialize)]
struct RunData {
    parts: usize,
    steps: usize,
    submodels: usize,
    bbox: Option<Bbox>,
    used: Vec<String>,
}

/// A part (or a part helper, `Brick(2, 4)`) a pass lacked.
#[derive(Debug, Deserialize)]
struct Need {
    #[serde(default)]
    part: Option<String>,
    /// The helper's key ("brick 2x4"), with its kind and dims.
    #[serde(default)]
    helper: Option<String>,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    dims: Vec<u32>,
    file: String,
    line: u32,
}

/// Runs the project's model with params.json plus `overrides`, writing build/.
pub fn run_model(
    data: &DataDir,
    project: &Project,
    overrides: &BTreeMap<String, Value>,
) -> (Result<RunSummary>, RunMeta) {
    run_model_until(data, project, overrides, &AtomicBool::new(false))
}

/// [`run_model`], stopped as soon as `cancel` is set: Python is killed and the run fails with "canceled". The app's
/// run queue cancels a run it no longer needs this way (through `emmet run --cancel-on-eof`).
pub fn run_model_until(
    data: &DataDir,
    project: &Project,
    overrides: &BTreeMap<String, Value>,
    cancel: &AtomicBool,
) -> (Result<RunSummary>, RunMeta) {
    let mut meta = RunMeta::default();
    let result = run_inner(data, project, overrides, &mut meta, cancel);
    (result, meta)
}

fn run_inner(
    data: &DataDir,
    project: &Project,
    overrides: &BTreeMap<String, Value>,
    meta: &mut RunMeta,
    cancel: &AtomicBool,
) -> Result<RunSummary> {
    let started = Instant::now();
    project.require_trusted(data)?;
    let library = crate::library::require(data)?;
    let index = Index::ensure(data)?;
    let python = crate::python::ensure(data)?;
    let mut params = project.params()?;
    let source = if overrides.is_empty() {
        "params.json"
    } else {
        "params.json and --params"
    };
    params.extend(overrides.clone());
    let hash = project.inputs_hash(overrides, &library.release)?;
    meta.inputs_hash = Some(hash.clone());
    let generation = project.next_generation()?;
    meta.generation = Some(generation);
    let build = project.build_dir()?;
    write_colors(&index, &build)?;

    let geometry_path = build.join("part-geometry.json");
    let (mut geometry, mut helpers) = read_geometry(&geometry_path);
    refresh_geometry(&index, &mut geometry, &mut helpers)?;
    let manifest = json!({
        "inputs_hash": hash,
        "emmet": crate::version(),
        "library": library.release,
        "parts_data": null,
        "generation": generation,
    });
    // Each pass that meets parts it has no geometry for reports them all; a pass can only add parts, so this ends
    // once every part the model uses is known. The cap is a backstop against a reply that never settles.
    for _pass in 0..MAX_PASSES {
        write_geometry(&geometry_path, &geometry, &helpers)?;
        let req = Request {
            mode: "run",
            params: &params,
            params_source: source,
            manifest: manifest.clone(),
            extra: Value::Null,
        };
        let reply = call(&python, project, &req, cancel)?;
        if reply.ok {
            let data: RunData = serde_json::from_value(reply.data.unwrap_or_default())
                .map_err(|e| Error::internal(format!("emmet_dsl's run reply doesn't match what emmet expects: {e}")))?;
            if data.used.len() != geometry.len() || data.used.iter().any(|u| !geometry.contains_key(u)) {
                geometry.retain(|k, _| data.used.contains(k));
                write_geometry(&geometry_path, &geometry, &helpers)?;
            }
            if data.parts > MAX_PARTS {
                return Err(too_large(None, None));
            }
            return Ok(RunSummary {
                parts: data.parts,
                steps: data.steps,
                submodels: data.submodels,
                bbox: data.bbox,
                ms: started.elapsed().as_millis() as u64,
                generation,
            });
        }
        if let Some(e) = reply.error {
            return Err(with_excerpt(project, e));
        }
        if reply.need.is_empty() {
            return Err(Error::internal(
                "emmet_dsl replied without a result, an error, or missing parts.",
            ));
        }
        let before = geometry.len() + helpers.len();
        for need in &reply.need {
            let at = |mut e: Error| {
                let d = e.details.get_or_insert_with(Default::default);
                d.file = Some(need.file.clone());
                d.line = Some(need.line);
                e.hint = format!("{} Fix {} line {} and run again.", e.hint, need.file, need.line);
                with_excerpt(project, e)
            };
            let id = match (&need.part, &need.helper) {
                (Some(id), _) => id.clone(),
                // A helper the library lacks goes back as null with the sizes it has; the DSL says so (or, for a
                // run of pieces, does without that size).
                (None, Some(key)) => {
                    let found = index.resolve_helper(&need.kind, &need.dims)?;
                    helpers.insert(key.clone(), json!({ "id": found.id, "near": found.near }));
                    match found.id {
                        Some(id) => id,
                        None => continue,
                    }
                }
                (None, None) => return Err(Error::internal("emmet_dsl asked for nothing in a need.")),
            };
            if geometry.contains_key(&id) {
                continue;
            }
            match index.part(&id)? {
                Some(part) => match part.body_box {
                    Some(body) => {
                        geometry.insert(part.id.clone(), json!({ "name": part.name, "body": body }));
                    }
                    // Without a body box the part can't be placed by its footprint; placing it at a zero box would
                    // put it in the wrong spot without a word.
                    None => {
                        return Err(at(Error::new(
                            ErrorCode::UnknownPart,
                            format!(
                                "Part '{}' ({}) has no geometry in the LDraw library to place it by.",
                                part.id, part.name
                            ),
                            "Use another part (emmet search finds one), or check the library with emmet library verify.",
                        )));
                    }
                },
                None => return Err(at(crate::ops::unknown_part(&index, &id)?)),
            }
        }
        if geometry.len() + helpers.len() == before {
            return Err(Error::internal(
                "emmet_dsl asked again for parts emmet had already sent.",
            ));
        }
    }
    Err(Error::internal(format!(
        "The model still asked for new parts after {MAX_PASSES} passes."
    )))
}

/// The parameter schema and the current values (`mode: schema`), for `emmet params`.
pub fn schema(data: &DataDir, project: &Project, overrides: &BTreeMap<String, Value>) -> Result<Value> {
    project.require_trusted(data)?;
    crate::library::require(data)?;
    let index = Index::ensure(data)?;
    let python = crate::python::ensure(data)?;
    let build = project.build_dir()?;
    write_colors(&index, &build)?;
    let mut params = project.params()?;
    let source = if overrides.is_empty() {
        "params.json"
    } else {
        "the new values"
    };
    params.extend(overrides.clone());
    let req = Request {
        mode: "schema",
        params: &params,
        params_source: source,
        manifest: Value::Null,
        extra: Value::Null,
    };
    let reply = call(&python, project, &req, &AtomicBool::new(false))?;
    match (reply.ok, reply.data, reply.error) {
        (true, Some(data), _) => Ok(data),
        (_, _, Some(e)) => Err(with_excerpt(project, e)),
        _ => Err(Error::internal(
            "emmet_dsl replied to a schema request without a schema.",
        )),
    }
}

/// Rewrites one parameter's default in model.py to `value` (`mode: write_default`), without running the model.
pub fn write_default(data: &DataDir, project: &Project, param: &str, value: &Value, kind: &str) -> Result<()> {
    project.require_trusted(data)?;
    let python = crate::python::ensure(data)?;
    let req = Request {
        mode: "write_default",
        params: &BTreeMap::new(),
        params_source: "",
        manifest: Value::Null,
        extra: json!({ "param": param, "value": value, "kind": kind }),
    };
    let reply = call(&python, project, &req, &AtomicBool::new(false))?;
    match (reply.ok, reply.error) {
        (true, _) => Ok(()),
        (_, Some(e)) => Err(with_excerpt(project, e)),
        _ => Err(Error::internal("emmet_dsl replied to write_default without a result.")),
    }
}

fn write_colors(index: &Index, build: &Path) -> Result<()> {
    let colors = index.colors()?;
    let text = serde_json::to_string_pretty(&json!({ "colors": colors })).expect("json") + "\n";
    let path = build.join("colors.json");
    if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
        write_atomic(&path, text.as_bytes())?;
    }
    Ok(())
}

/// part-geometry.json: the body boxes by part id, and the part helpers resolved so far ("brick 2x4" → 3001).
fn read_geometry(path: &Path) -> (BTreeMap<String, Value>, BTreeMap<String, Value>) {
    let file = std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .unwrap_or_default();
    let get = |key: &str| serde_json::from_value(file[key].clone()).unwrap_or_default();
    (get("parts"), get("helpers"))
}

/// Rereads every listed part from the index, dropping ids it no longer has, so a stale file can't feed old boxes; and
/// resolves the helpers again, since the library may have changed.
fn refresh_geometry(
    index: &Index,
    geometry: &mut BTreeMap<String, Value>,
    helpers: &mut BTreeMap<String, Value>,
) -> Result<()> {
    for (key, value) in helpers.iter_mut() {
        let (kind, dims) = helper_key(key);
        let found = index.resolve_helper(&kind, &dims)?;
        *value = json!({ "id": found.id, "near": found.near });
    }
    let ids: Vec<String> = geometry.keys().cloned().collect();
    geometry.clear();
    for id in ids {
        // A part without a body box drops out, so the next pass asks for it and gets the error.
        if let Some(part) = index.part(&id)?
            && let Some(body) = part.body_box
        {
            geometry.insert(part.id.clone(), json!({ "name": part.name, "body": body }));
        }
    }
    Ok(())
}

/// "brick 1x2x5" → ("brick", [1, 2, 5]); "slope 2x1 45" → ("slope", [2, 1, 45]); "plate 2x4" → ("plate", [2, 4, 1]).
fn helper_key(key: &str) -> (String, Vec<u32>) {
    let mut words = key.split_whitespace();
    let kind = words.next().unwrap_or("").to_string();
    let mut dims: Vec<u32> = words
        .next()
        .unwrap_or("")
        .split('x')
        .filter_map(|n| n.parse().ok())
        .collect();
    dims.extend(words.filter_map(|n| n.parse::<u32>().ok()));
    if dims.len() == 2 {
        dims.push(1);
    }
    (kind, dims)
}

fn write_geometry(path: &Path, geometry: &BTreeMap<String, Value>, helpers: &BTreeMap<String, Value>) -> Result<()> {
    let text = serde_json::to_string_pretty(&json!({ "parts": geometry, "helpers": helpers })).expect("json") + "\n";
    if std::fs::read_to_string(path).ok().as_deref() != Some(text.as_str()) {
        write_atomic(path, text.as_bytes())?;
    }
    Ok(())
}

fn too_large(file: Option<String>, line: Option<u32>) -> Error {
    Error::new(
        ErrorCode::ModelTooLarge,
        format!("The model places more than {MAX_PARTS} parts."),
        "Reduce repeats, or use larger parts.",
    )
    .with_details(Details {
        file,
        line,
        ..Default::default()
    })
}

/// Adds the source line to an error that names a file and line in the project. Only a plain relative path inside
/// the project is read: a model can raise an error naming any file, and `..` or an absolute path would read outside.
fn with_excerpt(project: &Project, mut e: Error) -> Error {
    if let Some(d) = e.details.as_mut()
        && let (Some(file), Some(line), None) = (&d.file, d.line, &d.excerpt)
    {
        let rel = Path::new(file);
        if rel.components().all(|c| matches!(c, std::path::Component::Normal(_))) {
            let path = project.root.join(rel);
            d.excerpt = std::fs::read_to_string(&path).ok().and_then(|t| {
                t.lines()
                    .nth(line.saturating_sub(1) as usize)
                    .map(|l| l.trim_end().to_string())
            });
        }
    }
    e
}

/// Runs the DSL once: request on stdin, reply on stdout, stderr to build/run.log.
fn call(python: &Path, project: &Project, req: &Request, cancel: &AtomicBool) -> Result<Reply> {
    let timeout = project.config.run.timeout.max(0.1);
    let mut request = json!({
        "mode": req.mode,
        "entry": "model.py",
        "params": req.params,
        "params_source": req.params_source,
        "geometry": "build/part-geometry.json",
        "colors": "build/colors.json",
        "out": "build",
        "timeout": timeout,
        "step_target": project.config.build.step_target(),
        "manifest": req.manifest,
    });
    if let (Some(all), Some(more)) = (request.as_object_mut(), req.extra.as_object()) {
        all.extend(more.clone());
    }
    let mut cmd = Command::new(python);
    cmd.args(["-I", "-B", "-X", "utf8", "-m", "emmet_dsl.run"])
        .current_dir(&project.root)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Windows needs SYSTEMROOT and WINDIR to start Python at all; nothing else is passed through, API keys included.
    for key in ["SYSTEMROOT", "WINDIR"] {
        if let Some(v) = std::env::var_os(key) {
            cmd.env(key, v);
        }
    }
    let mut child = cmd.spawn().map_err(|e| {
        Error::new(
            ErrorCode::PythonMissing,
            format!("Can't start {}: {e}", python.display()),
            "Run emmet doctor --fix.",
        )
    })?;
    // A failed write means Python died before reading its request; what it printed says why, so carry on and
    // report that rather than the broken pipe.
    let mut stdin = child.stdin.take().expect("piped stdin");
    let _ = stdin.write_all(request.to_string().as_bytes());
    drop(stdin);
    let stdout = Pipe::read(child.stdout.take().expect("piped stdout"), 1 << 20);
    let stderr = Pipe::read(child.stderr.take().expect("piped stderr"), MAX_LOG);
    let started = Instant::now();
    // faulthandler in the DSL dumps the stack and exits at the timeout; the kill is the backstop.
    let deadline = Duration::from_secs_f64(timeout + 5.0);
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| Error::internal(format!("waiting for Python: {e}")))?
        {
            break Some(status);
        }
        if cancel.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::new(
                ErrorCode::Internal,
                "The run was canceled.",
                "A newer run replaced it; nothing to fix.",
            ));
        }
        if started.elapsed() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let grace = Instant::now() + PIPE_GRACE;
    while !(stdout.done() && stderr.done()) && Instant::now() < grace {
        std::thread::sleep(Duration::from_millis(2));
    }
    let (out, _) = stdout.take();
    let (mut log_text, truncated) = stderr.take();
    if truncated {
        log_text.extend_from_slice(b"\n[run.log stopped at 10 MB]\n");
    }
    write_atomic(&project.root.join("build/run.log"), &log_text)?;
    let log_str = String::from_utf8_lossy(&log_text);
    if status.is_none() || log_str.contains("Timeout (") && out.is_empty() {
        let (file, line) = timeout_frame(project, &log_str);
        let e = Error::new(
            ErrorCode::ModelTimeout,
            format!("The model ran for more than {} s.", fmt_secs(timeout)),
            "Look for a loop that doesn't end, or reduce repeats. The project's limit is [run] timeout in emmet.toml.",
        )
        .with_details(Details {
            file,
            line,
            ..Default::default()
        });
        return Err(with_excerpt(project, e));
    }
    let text = String::from_utf8_lossy(&out);
    let last = text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("");
    serde_json::from_str(last).map_err(|_| {
        // emmet_dsl replies to every exception, so no reply means the process itself died: os._exit, a crash in
        // native code, or the OS killing it.
        let tail: Vec<&str> = log_str.lines().filter(|l| !l.trim().is_empty()).collect();
        let tail = tail[tail.len().saturating_sub(3)..].join(" / ");
        Error::new(
            ErrorCode::ModelExecFailed,
            format!("Python stopped without finishing the model ({}).", how_it_ended(status)),
            format!("build/run.log has everything it printed; it ends: {tail}"),
        )
        .at_file("build/run.log")
    })
}

fn how_it_ended(status: Option<std::process::ExitStatus>) -> String {
    let Some(status) = status else { return "killed".into() };
    #[cfg(unix)]
    if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(&status) {
        return format!("signal {signal}");
    }
    status
        .code()
        .map_or_else(|| "no exit code".into(), |c| format!("exit code {c}"))
}

fn fmt_secs(s: f64) -> String {
    crate::ldraw::num((s * 10.0).round() / 10.0)
}

/// The innermost frame of the project's own code in a faulthandler dump ("most recent call first").
fn timeout_frame(project: &Project, log: &str) -> (Option<String>, Option<u32>) {
    let root = project.root.display().to_string();
    for line in log.lines() {
        let Some(rest) = line.trim().strip_prefix("File \"") else {
            continue;
        };
        let Some((file, tail)) = rest.split_once('"') else {
            continue;
        };
        // `File "/abs/model.py", line 10 in forever`
        let Some(n) = tail
            .trim_start_matches([',', ' '])
            .strip_prefix("line ")
            .and_then(|t| t.split_whitespace().next())
        else {
            continue;
        };
        if let (Some(rel), Ok(n)) = (Path::new(file).strip_prefix(&root).ok(), n.parse()) {
            return (Some(rel.to_string_lossy().replace('\\', "/")), Some(n));
        }
    }
    (None, None)
}

/// One of Python's output pipes, read on its own thread into a shared buffer that keeps the first `cap` bytes and
/// drains the rest, so a chatty model never blocks on a full pipe.
struct Pipe {
    buf: std::sync::Arc<std::sync::Mutex<(Vec<u8>, bool)>>,
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Pipe {
    fn read(mut r: impl Read + Send + 'static, cap: usize) -> Self {
        let buf = std::sync::Arc::new(std::sync::Mutex::new((Vec::new(), false)));
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (b, d) = (buf.clone(), done.clone());
        std::thread::spawn(move || {
            let mut chunk = [0u8; 1 << 16];
            loop {
                match r.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut g = b.lock().expect("pipe buffer");
                        let room = cap.saturating_sub(g.0.len());
                        g.0.extend_from_slice(&chunk[..n.min(room)]);
                        g.1 |= n > room;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(_) => break,
                }
            }
            d.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        Self { buf, done }
    }

    fn done(&self) -> bool {
        self.done.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// What was read so far, and whether the cap cut it.
    fn take(&self) -> (Vec<u8>, bool) {
        let mut g = self.buf.lock().expect("pipe buffer");
        (std::mem::take(&mut g.0), g.1)
    }
}
