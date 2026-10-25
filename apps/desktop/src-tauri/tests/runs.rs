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
    match &o.placements {
        Some(Ok(p)) => p["parts"].as_array().map_or(0, Vec::len),
        _ => 0,
    }
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
    let asked = Instant::now();
    q.request(&dir);
    let first = finished(&rx);
    assert!(first.canceled, "the slow run should be canceled: {:?}", first.envelope);
    // Closing stdin stopped emmet (and its Python) at once; the queue's 2 s kill is only a backstop.
    assert!(
        asked.elapsed() < Duration::from_millis(1500),
        "the cancel took {:?}: emmet didn't stop when its stdin closed",
        asked.elapsed()
    );
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
fn a_result_older_than_the_scene_on_screen_is_dropped_but_still_ends_the_run() {
    let dir = grid(r#"{"count": 5}"#);
    let (q, rx) = queue();
    q.request(&dir);
    let newer = finished(&rx);
    let state = state();
    let shown = finish(&state, &newer);
    let generation = shown.scene.expect("the first scene is shown").generation;
    let older = Outcome {
        generation: Some(generation - 1),
        ..newer.clone()
    };
    // The window still hears the run ended, so it never stays on "rebuilding".
    let stale = finish(&state, &older);
    assert!(
        stale.stale && stale.scene.is_none() && stale.error.is_none(),
        "{stale:?}"
    );
}

#[test]
fn a_result_the_app_cant_read_is_an_error_not_silence() {
    let dir = grid(r#"{"count": 5}"#);
    let (q, rx) = queue();
    q.request(&dir);
    let ok = finished(&rx);
    let state = state();
    let unreadable = Outcome {
        placements: Some(Err("build/placements.json doesn't parse: EOF".into())),
        ..ok.clone()
    };
    let f = finish(&state, &unreadable);
    assert!(
        f.error.as_ref().is_some_and(|e| e.message.contains("doesn't parse")),
        "{f:?}"
    );
    let wrong_shape = Outcome {
        placements: Some(Ok(serde_json::json!({ "parts": "nope" }))),
        ..ok
    };
    let f = finish(&state, &wrong_shape);
    assert!(f.error.is_some() && !f.stale, "{f:?}");
}

/// A sidecar that crashes without an envelope: what it printed and how it exited reach the error panel.
#[test]
fn a_crash_reports_what_emmet_printed() {
    let dir = emmet_it::temp_dir("crash");
    let fake = if cfg!(windows) {
        let p = dir.join("emmet.cmd");
        std::fs::write(
            &p,
            "@echo off\r\necho thread 'main' panicked at boom 1>&2\r\nexit /b 3\r\n",
        )
        .unwrap();
        p
    } else {
        let p = dir.join("emmet");
        std::fs::write(&p, "#!/bin/sh\necho \"thread 'main' panicked at boom\" >&2\nexit 3\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        p
    };
    let (tx, rx) = mpsc::channel();
    let q = RunQueue::start(fake, emmet_it::shared_data().to_path_buf(), move |ev| {
        let _ = tx.send(ev);
    });
    q.request(&dir);
    let o = finished(&rx);
    assert!(o.envelope.is_none());
    assert_eq!(o.exit, "exit code 3");
    assert!(o.stderr.contains("panicked at boom"), "{:?}", o.stderr);
    let e = finish(&state(), &o).error.expect("an error");
    assert!(
        e.message.contains("exit code 3") && e.message.contains("panicked at boom"),
        "{}",
        e.message
    );
}

fn state() -> AppState {
    AppState::new(
        emmet_core::data::DataDir::at(emmet_it::shared_data()),
        emmet_it::emmet_bin(),
        None,
    )
}
