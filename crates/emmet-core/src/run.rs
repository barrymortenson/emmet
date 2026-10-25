// SPDX-License-Identifier: AGPL-3.0-or-later
//! Running a model (plan sections 07, 08, 15): `python -I -m emmet_dsl.run` in the project folder with a scrubbed
//! environment, a timeout, and capped output. emmet writes the part geometry and the color table the DSL reads; the
//! DSL writes model.mpd, placements.json, and manifest.json.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
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

#[derive(Debug, Deserialize)]
struct Need {
    part: String,
    file: String,
    line: u32,
}

/// Runs the project's model with params.json plus `overrides`, writing build/.
pub fn run_model(
    data: &DataDir,
    project: &Project,
    overrides: &BTreeMap<String, Value>,
) -> (Result<RunSummary>, RunMeta) {
    let mut meta = RunMeta::default();
    let result = run_inner(data, project, overrides, &mut meta);
    (result, meta)
}

fn run_inner(
    data: &DataDir,
    project: &Project,
    overrides: &BTreeMap<String, Value>,
    meta: &mut RunMeta,
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
    let mut geometry = read_geometry(&geometry_path);
    refresh_geometry(&index, &mut geometry)?;
    let manifest = json!({
        "inputs_hash": hash,
        "emmet": crate::version(),
        "library": library.release,
        "parts_data": null,
        "generation": generation,
    });
    for _pass in 0..3 {
        write_geometry(&geometry_path, &geometry)?;
        let req = Request {
            mode: "run",
            params: &params,
            params_source: source,
            manifest: manifest.clone(),
        };
        let reply = call(&python, project, &req)?;
        if reply.ok {
            let data = reply.data.unwrap_or_default();
            let used: Vec<String> = serde_json::from_value(data["used"].clone()).unwrap_or_default();
            if used.len() != geometry.len() || used.iter().any(|u| !geometry.contains_key(u)) {
                geometry.retain(|k, _| used.contains(k));
                write_geometry(&geometry_path, &geometry)?;
            }
            let parts = data["parts"].as_u64().unwrap_or(0) as usize;
            if parts > MAX_PARTS {
                return Err(too_large(None, None));
            }
            return Ok(RunSummary {
                parts,
                steps: data["steps"].as_u64().unwrap_or(0) as usize,
                submodels: data["submodels"].as_u64().unwrap_or(0) as usize,
                bbox: serde_json::from_value(data["bbox"].clone()).unwrap_or(None),
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
        for need in &reply.need {
            match index.part(&need.part)? {
                Some(part) => {
                    let body = part.body_box.unwrap_or([0.0; 6]);
                    geometry.insert(part.id.clone(), json!({ "name": part.name, "body": body }));
                }
                None => {
                    let mut e = crate::ops::unknown_part(&index, &need.part)?;
                    let d = e.details.get_or_insert_with(Details::default);
                    d.file = Some(need.file.clone());
                    d.line = Some(need.line);
                    e.hint = format!("{} Fix {} line {} and run again.", e.hint, need.file, need.line);
                    return Err(with_excerpt(project, e));
                }
            }
        }
    }
    Err(Error::internal(
        "The model still asked for new parts after three passes.",
    ))
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
    };
    let reply = call(&python, project, &req)?;
    match (reply.ok, reply.data, reply.error) {
        (true, Some(data), _) => Ok(data),
        (_, _, Some(e)) => Err(with_excerpt(project, e)),
        _ => Err(Error::internal(
            "emmet_dsl replied to a schema request without a schema.",
        )),
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

fn read_geometry(path: &Path) -> BTreeMap<String, Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| serde_json::from_value(v["parts"].clone()).ok())
        .unwrap_or_default()
}

/// Rereads every listed part from the index, dropping ids it no longer has, so a stale file can't feed old boxes.
fn refresh_geometry(index: &Index, geometry: &mut BTreeMap<String, Value>) -> Result<()> {
    let ids: Vec<String> = geometry.keys().cloned().collect();
    geometry.clear();
    for id in ids {
        if let Some(part) = index.part(&id)? {
            geometry.insert(
                part.id.clone(),
                json!({ "name": part.name, "body": part.body_box.unwrap_or([0.0; 6]) }),
            );
        }
    }
    Ok(())
}

fn write_geometry(path: &Path, geometry: &BTreeMap<String, Value>) -> Result<()> {
    let text = serde_json::to_string_pretty(&json!({ "parts": geometry })).expect("json") + "\n";
    if std::fs::read_to_string(path).ok().as_deref() != Some(text.as_str()) {
        write_atomic(path, text.as_bytes())?;
    }
    Ok(())
}

fn too_large(file: Option<String>, line: Option<u32>) -> Error {
    Error::new(
        ErrorCode::ModelTooLarge,
        format!("The model places more than {MAX_PARTS} parts."),
        "Reduce repeats, or split the model into submodels.",
    )
    .with_details(Details {
        file,
        line,
        ..Default::default()
    })
}

/// Adds the source line to an error that names a file and line in the project.
fn with_excerpt(project: &Project, mut e: Error) -> Error {
    if let Some(d) = e.details.as_mut()
        && let (Some(file), Some(line), None) = (&d.file, d.line, &d.excerpt)
    {
        let path = project.root.join(file);
        if path.starts_with(&project.root) {
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
fn call(python: &Path, project: &Project, req: &Request) -> Result<Reply> {
    let timeout = project.config.run.timeout.max(0.1);
    let request = json!({
        "mode": req.mode,
        "entry": "model.py",
        "params": req.params,
        "params_source": req.params_source,
        "geometry": "build/part-geometry.json",
        "colors": "build/colors.json",
        "out": "build",
        "timeout": timeout,
        "manifest": req.manifest,
    });
    let mut cmd = Command::new(python);
    cmd.args(["-I", "-B", "-X", "utf8", "-m", "emmet_dsl.run"])
        .current_dir(&project.root)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Windows needs SYSTEMROOT to start Python at all; nothing else is passed through, API keys included.
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
    let mut stdin = child.stdin.take().expect("piped stdin");
    stdin
        .write_all(request.to_string().as_bytes())
        .map_err(|e| Error::internal(format!("writing to Python: {e}")))?;
    drop(stdin);
    let stdout = read_capped(child.stdout.take().expect("piped stdout"), 1 << 20);
    let stderr = read_capped(child.stderr.take().expect("piped stderr"), MAX_LOG);
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
        if started.elapsed() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let out = stdout.join().unwrap_or_default();
    let (log, truncated) = stderr.join().unwrap_or_default();
    let mut log_text = log;
    if truncated {
        log_text.extend_from_slice(b"\n[run.log stopped at 10 MB]\n");
    }
    write_atomic(&project.root.join("build/run.log"), &log_text)?;
    let log_str = String::from_utf8_lossy(&log_text);
    if status.is_none() || log_str.contains("Timeout (") && out.0.is_empty() {
        let (file, line) = timeout_frame(project, &log_str);
        let e = Error::new(
            ErrorCode::ModelTimeout,
            format!("The model ran for more than {} s.", fmt_secs(timeout)),
            "Look for a loop that doesn't end, or reduce repeats; split a big model into submodels. The project's \
             limit is [run] timeout in emmet.toml.",
        )
        .with_details(Details {
            file,
            line,
            ..Default::default()
        });
        return Err(with_excerpt(project, e));
    }
    let text = String::from_utf8_lossy(&out.0);
    let last = text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("");
    serde_json::from_str(last).map_err(|_| {
        Error::internal(format!(
            "emmet_dsl didn't reply (exit {}). The end of build/run.log: {}",
            status.and_then(|s| s.code()).map_or("killed".into(), |c| c.to_string()),
            log_str
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" / ")
        ))
    })
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

/// Reads a pipe on its own thread, keeping the first `cap` bytes and draining the rest.
fn read_capped(mut r: impl Read + Send + 'static, cap: usize) -> std::thread::JoinHandle<(Vec<u8>, bool)> {
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut truncated = false;
        let mut buf = [0u8; 1 << 16];
        while let Ok(n) = r.read(&mut buf) {
            if n == 0 {
                break;
            }
            let room = cap.saturating_sub(kept.len());
            kept.extend_from_slice(&buf[..n.min(room)]);
            truncated |= n > room;
        }
        (kept, truncated)
    })
}
