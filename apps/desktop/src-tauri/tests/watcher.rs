// SPDX-License-Identifier: AGPL-3.0-or-later
//! The project watcher (plan section 07): sources only, debounced 150 ms, ignoring build/ and editor temp files.

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use emmet_desktop::watcher::{Watcher, is_source};

fn changes(rx: &mpsc::Receiver<()>, wait: Duration) -> usize {
    let mut n = 0;
    while rx.recv_timeout(wait).is_ok() {
        n += 1;
    }
    n
}

#[test]
fn only_sources_trigger_a_run_and_a_burst_triggers_one() {
    let dir = emmet_core::fsx::canonical(&emmet_it::temp_dir("watch")).unwrap();
    std::fs::create_dir_all(dir.join("models")).unwrap();
    std::fs::create_dir_all(dir.join("build")).unwrap();
    std::fs::write(dir.join("model.py"), "# v0\n").unwrap();
    let (tx, rx) = mpsc::channel();
    let _w = Watcher::start(&dir, move || {
        let _ = tx.send(());
    })
    .unwrap();
    // FSEvents can report the setup writes late; let them pass.
    changes(&rx, Duration::from_millis(600));

    for name in [
        "build/placements.json",
        "build/renders/iso.png",
        ".model.py.swp",
        "model.py~",
        "4913",
        "notes.txt",
        ".emmet/generation",
    ] {
        let p = dir.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "x").unwrap();
    }
    assert_eq!(
        changes(&rx, Duration::from_millis(600)),
        0,
        "a non-source change triggered a run"
    );

    for i in 0..5 {
        std::fs::write(dir.join("model.py"), format!("# v{i}\n")).unwrap();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        changes(&rx, Duration::from_millis(800)),
        1,
        "a burst of saves should run once"
    );

    std::fs::write(dir.join("models/walls.py"), "# walls\n").unwrap();
    assert_eq!(changes(&rx, Duration::from_millis(800)), 1, "models/*.py is a source");
    std::fs::write(dir.join("params.json"), "{}\n").unwrap();
    assert_eq!(changes(&rx, Duration::from_millis(800)), 1, "params.json is a source");
}

#[test]
fn source_rules() {
    let root = Path::new("/p");
    for yes in [
        "model.py",
        "params.json",
        "emmet.toml",
        "models/walls.py",
        "models/a/b.py",
    ] {
        assert!(is_source(root, &root.join(yes)), "{yes}");
    }
    for no in [
        "build/model.mpd",
        "models/__pycache__/x.py",
        "models/.x.py.swp",
        "models/x.py~",
        "CARD.md",
        "other.py",
        ".emmet/trust",
        "#model.py#",
        "/elsewhere/model.py",
    ] {
        assert!(!is_source(root, &root.join(no)), "{no}");
    }
}
