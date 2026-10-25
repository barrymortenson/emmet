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
    /// build/placements.json after a successful run.
    pub placements: Option<Value>,
    pub generation: Option<u64>,
    pub ms: u64,
}

impl Outcome {
    pub fn ok(&self) -> bool {
        self.envelope.as_ref().is_some_and(|e| e["ok"] == true) && self.placements.is_some()
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

/// Runs emmet once. Returns the outcome and, if a request came in meanwhile, that request (the run was canceled).
fn run_once(emmet: &Path, data: &Path, project: &Path, rx: &mpsc::Receiver<PathBuf>) -> (Outcome, Option<PathBuf>) {
    let started = Instant::now();
    let mut outcome = Outcome {
        project: project.to_path_buf(),
        canceled: false,
        envelope: None,
        placements: None,
        generation: None,
        ms: 0,
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
        .stderr(Stdio::null());
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
    let mut newer = None;
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(p) => {
                let mut latest = p;
                while let Ok(p) = rx.try_recv() {
                    latest = p;
                }
                newer = Some(latest);
                cancel(&mut child);
                break;
            }
            Err(RecvTimeoutError::Timeout) => {}
            // The app is closing: stop the run with it.
            Err(RecvTimeoutError::Disconnected) => {
                cancel(&mut child);
                break;
            }
        }
    }
    let text = reader.join().unwrap_or_default();
    outcome.ms = started.elapsed().as_millis() as u64;
    if newer.is_some() {
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
            outcome.placements = std::fs::read_to_string(project.join("build/placements.json"))
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok());
        }
    }
    (outcome, None)
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
