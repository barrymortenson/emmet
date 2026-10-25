// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet run --cancel-on-eof` (the app's run queue cancels a run this way): closing stdin stops the run and its
//! Python at once, with the "canceled" envelope, rather than waiting out the model.

use std::io::Read;
use std::process::Stdio;
use std::time::{Duration, Instant};

#[test]
fn closing_stdin_cancels_the_run() {
    let dir = emmet_it::project("scene-grid");
    std::fs::write(dir.join("params.json"), r#"{"count": 10, "delay": 20}"#).unwrap();
    let mut child = emmet_it::emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(emmet_it::shared_data())
        .arg("--project")
        .arg(&dir)
        .args(["run", "--cancel-on-eof"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    // Python is asleep in the model by now.
    std::thread::sleep(Duration::from_millis(1500));
    let closed = Instant::now();
    drop(child.stdin.take());
    let status = child.wait().unwrap();
    assert!(
        closed.elapsed() < Duration::from_secs(2),
        "the run took {:?} to stop",
        closed.elapsed()
    );
    let mut out = String::new();
    child.stdout.take().unwrap().read_to_string(&mut out).unwrap();
    let env: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(!status.success());
    assert_eq!(env["error"]["message"], "The run was canceled.");
}
