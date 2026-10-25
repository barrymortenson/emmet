// SPDX-License-Identifier: AGPL-3.0-or-later
//! The run queue against the real emmet sidecar and Python (plan sections 04 and 07): the latest request wins, a run
//! it replaces is canceled, requests that pile up collapse, a run killed from outside ends with no envelope, and the
//! app drops a result older than the scene it shows (the generation check).

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use emmet_desktop::app::{AppState, finish};
use emmet_desktop::runs::{Outcome, RunEvent, RunQueue};

fn queue() -> (RunQueue, mpsc::Receiver<RunEvent>) {
    let (tx, rx) = mpsc::channel();
    let q = RunQueue::start(
        emmet_it::emmet_bin(),
        emmet_it::shared_data().to_path_buf(),
        move |ev| {
            let _ = tx.send(ev);
        },
    );
    (q, rx)
}

fn grid(params: &str) -> PathBuf {
    let dir = emmet_it::project("scene-grid");
    std::fs::write(dir.join("params.json"), params).unwrap();
    dir
}

fn finished(rx: &mpsc::Receiver<RunEvent>) -> Outcome {
    loop {
        match rx.recv_timeout(Duration::from_secs(60)).expect("a run event") {
            RunEvent::Finished(o) => return o,
            RunEvent::Started { .. } => {}
        }
    }
}

fn count(o: &Outcome) -> usize {
    o.placements
        .as_ref()
        .and_then(|p| p["parts"].as_array())
        .map_or(0, Vec::len)
}

#[test]
fn a_run_returns_the_envelope_and_the_placements() {
    let (q, rx) = queue();
    let dir = grid(r#"{"count": 30}"#);
    q.request(&dir);
    let o = finished(&rx);
    assert!(o.ok(), "{:?}", o.envelope);
    assert!(!o.canceled);
    assert_eq!(count(&o), 30);
    assert!(o.generation.is_some());
}

#[test]
fn a_newer_request_cancels_the_run_in_progress() {
    let (q, rx) = queue();
    let dir = grid(r#"{"count": 30, "delay": 20}"#);
    let started = Instant::now();
    q.request(&dir);
    // Wait until the slow run has started, then ask again with the delay gone.
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(30)).unwrap(),
        RunEvent::Started { .. }
    ));
    std::thread::sleep(Duration::from_millis(300));
    std::fs::write(dir.join("params.json"), r#"{"count": 40}"#).unwrap();
    q.request(&dir);
    let first = finished(&rx);
    assert!(first.canceled, "the slow run should be canceled: {:?}", first.envelope);
    let second = finished(&rx);
    assert!(second.ok(), "{:?}", second.envelope);
    assert_eq!(count(&second), 40);
    assert!(
        started.elapsed() < Duration::from_secs(15),
        "the canceled run held up the queue"
    );
}

#[test]
fn requests_that_pile_up_collapse_into_one_run() {
    let (q, rx) = queue();
    let dir = grid(r#"{"count": 20, "delay": 1}"#);
    q.request(&dir);
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(30)).unwrap(),
        RunEvent::Started { .. }
    ));
    for _ in 0..5 {
        q.request(&dir);
    }
    let mut runs = vec![finished(&rx)];
    while let Ok(ev) = rx.recv_timeout(Duration::from_secs(10)) {
        if let RunEvent::Finished(o) = ev {
            runs.push(o);
        }
    }
    // The first run is canceled by the pile, which runs once.
    assert_eq!(
        runs.len(),
        2,
        "{:?}",
        runs.iter().map(|o| o.canceled).collect::<Vec<_>>()
    );
    assert!(runs[0].canceled && runs[1].ok());
}

#[test]
fn a_run_killed_from_outside_ends_with_no_envelope() {
    let (q, rx) = queue();
    let dir = grid(r#"{"count": 10, "delay": 20}"#);
    q.request(&dir);
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(30)).unwrap(),
        RunEvent::Started { .. }
    ));
    std::thread::sleep(Duration::from_millis(500));
    kill_emmet_for(&dir);
    let o = finished(&rx);
    assert!(!o.canceled);
    assert!(o.envelope.is_none() && !o.ok());
}

/// Kills the emmet process running `dir`, as a user might from a task manager.
fn kill_emmet_for(dir: &Path) {
    let dir = dir.display().to_string();
    let status = if cfg!(windows) {
        let filter = format!(
            "$_.Name -eq 'emmet.exe' -and $_.CommandLine -like '*{}*'",
            dir.replace('\'', "''")
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command"])
            .arg(format!("Get-CimInstance Win32_Process | Where-Object {{ {filter} }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force }}"))
            .status()
    } else {
        std::process::Command::new("pkill")
            .args(["-KILL", "-f", &format!("emmet .*{dir}")])
            .status()
    };
    assert!(
        status.is_ok_and(|s| s.success()),
        "couldn't kill the emmet run for {dir}"
    );
}

#[test]
fn a_result_older_than_the_scene_on_screen_is_dropped() {
    let dir = grid(r#"{"count": 5}"#);
    let (q, rx) = queue();
    q.request(&dir);
    let newer = finished(&rx);
    let state = AppState::new(
        emmet_core::data::DataDir::at(emmet_it::shared_data()),
        emmet_it::emmet_bin(),
        None,
    );
    let shown = finish(&state, &newer).expect("the first scene is shown");
    let generation = shown.scene.unwrap().generation;
    let older = Outcome {
        generation: Some(generation - 1),
        ..newer.clone()
    };
    assert!(
        finish(&state, &older).is_none(),
        "an older generation replaced the scene"
    );
    let failed = Outcome {
        envelope: None,
        placements: None,
        ..newer
    };
    let f = finish(&state, &failed).expect("a failure is reported");
    assert!(f.scene.is_none() && f.error.is_some());
}
