// SPDX-License-Identifier: AGPL-3.0-or-later
//! The run queue (plan sections 04 and 07): one `emmet run` at a time, through the emmet sidecar. The latest request
//! wins: a request that arrives while a run is going cancels it (by closing its stdin, which `--cancel-on-eof` turns
//! into killing its Python) and starts over, and requests that pile up collapse into one.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use serde_json::Value;

/// How long a canceled run gets to stop before it's killed.
const CANCEL_GRACE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub enum RunEvent {
    Started { project: PathBuf },
    Finished(Outcome),
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub project: PathBuf,
    /// A newer request replaced this run; its result means nothing.
    pub canceled: bool,
    /// emmet's JSON envelope, or None when emmet printed none (it crashed or was killed).
    pub envelope: Option<Value>,
    /// build/placements.json after a successful run, or why it couldn't be read.
    pub placements: Option<Result<Value, String>>,
    pub generation: Option<u64>,
    pub ms: u64,
    /// How emmet exited ("exit code 2", "killed") and the end of what it printed to stderr, for a run that printed no
    /// envelope.
    pub exit: String,
    pub stderr: String,
}

impl Outcome {
    pub fn ok(&self) -> bool {
        self.envelope.as_ref().is_some_and(|e| e["ok"] == true) && matches!(self.placements, Some(Ok(_)))
    }
}

pub struct RunQueue {
    tx: mpsc::Sender<PathBuf>,
}

impl RunQueue {
    /// `emmet` is the sidecar binary; every run gets `--data-dir data`.
    pub fn start(emmet: PathBuf, data: PathBuf, on_event: impl Fn(RunEvent) + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel::<PathBuf>();
        std::thread::spawn(move || {
            let mut next = None;
            loop {
                let mut project = match next.take() {
                    Some(p) => p,
                    None => match rx.recv() {
                        Ok(p) => p,
                        Err(_) => return,
                    },
                };
                while let Ok(p) = rx.try_recv() {
                    project = p;
                }
                on_event(RunEvent::Started {
                    project: project.clone(),
                });
                let (outcome, newer) = run_once(&emmet, &data, &project, &rx);
                next = newer;
                on_event(RunEvent::Finished(outcome));
            }
        });
        Self { tx }
    }

    /// Asks for a run of `project`. It starts at once, canceling a run in progress.
    pub fn request(&self, project: &Path) {
        let _ = self.tx.send(project.to_path_buf());
    }
}

/// Runs emmet once. Returns the outcome and the latest request that came in meanwhile, if one did (the run was
/// canceled for it, unless it was still setting up Python).
fn run_once(emmet: &Path, data: &Path, project: &Path, rx: &mpsc::Receiver<PathBuf>) -> (Outcome, Option<PathBuf>) {
    let started = Instant::now();
    let mut outcome = Outcome {
        project: project.to_path_buf(),
        canceled: false,
        envelope: None,
        placements: None,
        generation: None,
        ms: 0,
        exit: String::new(),
        stderr: String::new(),
    };
    let mut cmd = Command::new(emmet);
    cmd.arg("--json")
        .arg("--data-dir")
        .arg(data)
        .arg("--project")
        .arg(project)
        .args(["run", "--cancel-on-eof"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    no_window(&mut cmd);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            outcome.envelope = Some(serde_json::json!({
                "ok": false,
                "error": { "code": "INTERNAL", "message": format!("Can't start {}: {e}", emmet.display()),
                           "hint": "Reinstall Emmet; the emmet sidecar is missing." }
            }));
            return (outcome, None);
        }
    };
    let mut stdout = child.stdout.take().expect("piped stdout");
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let mut stderr = child.stderr.take().expect("piped stderr");
    let errors = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let mut newer = None;
    let mut status = None;
    let mut canceled = false;
    loop {
        if let Ok(Some(s)) = child.try_wait() {
            status = Some(s);
            break;
        }
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(p) => {
                let mut latest = p;
                while let Ok(p) = rx.try_recv() {
                    latest = p;
                }
                newer = Some(latest);
                // A run that's still setting up Python finishes first: killing emmet then would leave its bootstrap
                // lock behind, and the next run would wait on it for ten minutes.
                if emmet_core::python::status(&emmet_core::data::DataDir::at(data)).ready {
                    cancel(&mut child);
                    canceled = true;
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            // The app is closing: stop the run with it.
            Err(RecvTimeoutError::Disconnected) => {
                cancel(&mut child);
                canceled = true;
                break;
            }
        }
    }
    let text = reader.join().unwrap_or_default();
    let err = errors.join().unwrap_or_default();
    // The last few lines are what explain a crash; a run's stderr is otherwise empty (the model's goes to run.log).
    let tail: Vec<&str> = err.lines().filter(|l| !l.trim().is_empty()).collect();
    outcome.stderr = tail[tail.len().saturating_sub(5)..].join("\n");
    outcome.exit = match status.and_then(|s| s.code()) {
        Some(code) => format!("exit code {code}"),
        None => "killed".into(),
    };
    outcome.ms = started.elapsed().as_millis() as u64;
    if canceled {
        outcome.canceled = true;
        return (outcome, newer);
    }
    outcome.envelope = text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .and_then(|l| serde_json::from_str(l).ok());
    if let Some(env) = &outcome.envelope {
        outcome.generation = env["meta"]["generation"].as_u64();
        if env["ok"] == true {
            let path = project.join("build").join("placements.json");
            outcome.placements = Some(
                std::fs::read_to_string(&path)
                    .map_err(|e| format!("can't read {}: {e}", path.display()))
                    .and_then(|t| {
                        serde_json::from_str(&t).map_err(|e| format!("{} doesn't parse: {e}", path.display()))
                    }),
            );
        }
    }
    (outcome, newer)
}

/// Closes stdin so emmet stops its Python and exits, then kills it if it hasn't within the grace period.
fn cancel(child: &mut Child) {
    drop(child.stdin.take());
    let until = Instant::now() + CANCEL_GRACE;
    while Instant::now() < until {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// The sidecar is a console program: on Windows, keep it from opening a console window over the app.
pub fn no_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}
