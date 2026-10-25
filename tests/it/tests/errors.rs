// SPDX-License-Identifier: AGPL-3.0-or-later
//! A golden for every plan section 09 error code 1A can raise: the exact envelope, the user's file and line where
//! there is one, a hint, and the exit code. RENDER_UNAVAILABLE arrives with render (1D); INTERNAL is the catch-all.

use std::path::Path;

use emmet_it::{
    copy_tree, emmet, emmet_in, envelope, fixture, library_zip, project, redact, sha256, shared_data, temp_dir,
};
use serde_json::Value;

/// Checks what every error envelope must have, then snapshots it with paths and timings redacted.
#[track_caller]
fn golden(name: &str, (v, code): (Value, i32), exit: i32, paths: &[(&Path, &str)]) -> Value {
    assert_eq!(v["ok"], false, "{v}");
    assert_eq!(code, exit, "exit code: {v}");
    let e = &v["error"];
    assert!(e["message"].as_str().is_some_and(|m| !m.is_empty()), "a message: {v}");
    assert!(e["hint"].as_str().is_some_and(|h| h.len() > 10), "a hint: {v}");
    let v = redact(v, paths);
    insta::assert_json_snapshot!(name, v);
    v
}

/// Asserts the error points at `file:line` with the line's text.
#[track_caller]
fn at(v: &Value, file: &str, line: u64) {
    let d = &v["error"]["details"];
    assert_eq!(
        (d["file"].as_str(), d["line"].as_u64()),
        (Some(file), Some(line)),
        "{v}"
    );
    assert!(
        d["excerpt"].as_str().is_some_and(|x| !x.trim().is_empty()),
        "an excerpt: {v}"
    );
}

#[test]
fn project_not_found() {
    let dir = temp_dir("no-project");
    let v = golden("project_not_found", emmet_in(&dir, &["run"]), 2, &[(&dir, "[dir]")]);
    assert!(v["error"]["hint"].as_str().unwrap().contains("emmet new"));
}

#[test]
fn project_untrusted() {
    let dir = temp_dir("untrusted").join("frame");
    copy_tree(&fixture("projects/frame"), &dir);
    let v = golden("project_untrusted", emmet_in(&dir, &["run"]), 3, &[(&dir, "[project]")]);
    assert!(v["error"]["hint"].as_str().unwrap().contains("emmet trust"));
    assert!(!dir.join("build/model.mpd").exists(), "nothing ran");
}

#[test]
fn model_exec_failed_syntax_error() {
    let dir = project("syntax-error");
    let model = "# SPDX-License-Identifier: AGPL-3.0-or-later\nfrom emmet_dsl import Model, model\n\n\n@model\ndef broken():\n    \"\"\"A model with a syntax error on line 9.\"\"\"\n    m = Model()\n    m.add(\"3001\", \"Red\", at=(0, 0, 0)\n    return m\n";
    std::fs::write(dir.join("model.py"), model).unwrap();
    let v = golden(
        "model_exec_failed_syntax",
        emmet_in(&dir, &["run"]),
        3,
        &[(&dir, "[project]")],
    );
    assert_eq!(v["error"]["code"], "MODEL_EXEC_FAILED");
    at(&v, "model.py", 9);
}

#[test]
fn model_exec_failed_name_error() {
    let dir = project("name-error");
    let v = golden(
        "model_exec_failed_name",
        emmet_in(&dir, &["run"]),
        3,
        &[(&dir, "[project]")],
    );
    at(&v, "model.py", 12);
    assert!(
        v["error"]["hint"]
            .as_str()
            .unwrap()
            .starts_with("Did you mean 'width'?"),
        "{v}"
    );
}

#[test]
fn model_exec_failed_in_a_submodule() {
    let dir = project("submodule");
    let v = golden(
        "unknown_color_in_submodule",
        emmet_in(&dir, &["run"]),
        3,
        &[(&dir, "[project]")],
    );
    assert_eq!(v["error"]["code"], "UNKNOWN_COLOR");
    // The file is relative to the project with / separators on every OS.
    at(&v, "models/walls.py", 9);
}

#[test]
fn model_timeout() {
    let dir = project("timeout");
    let v = golden("model_timeout", emmet_in(&dir, &["run"]), 3, &[(&dir, "[project]")]);
    assert_eq!(v["error"]["code"], "MODEL_TIMEOUT");
    at(&v, "model.py", 10);
    let log = std::fs::read_to_string(dir.join("build/run.log")).unwrap();
    assert!(log.contains("Timeout"), "run.log keeps the stack dump: {log}");
}

#[test]
fn model_too_large() {
    let dir = project("too-large");
    let v = golden("model_too_large", emmet_in(&dir, &["run"]), 3, &[(&dir, "[project]")]);
    assert_eq!(v["error"]["code"], "MODEL_TOO_LARGE");
    at(&v, "model.py", 11);
}

#[test]
fn unknown_part() {
    let dir = project("unknown-part");
    let v = golden("unknown_part", emmet_in(&dir, &["run"]), 3, &[(&dir, "[project]")]);
    at(&v, "model.py", 10);
    assert_eq!(v["error"]["details"]["suggestions"][0], "3001");
}

#[test]
fn unknown_part_from_part_info() {
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(["part", "3001x"])
        .output()
        .unwrap();
    let v = golden("unknown_part_info", envelope(&out), 3, &[]);
    assert_eq!(v["error"]["code"], "UNKNOWN_PART");
}

#[test]
fn unknown_color() {
    let dir = project("unknown-color");
    let v = golden("unknown_color", emmet_in(&dir, &["run"]), 3, &[(&dir, "[project]")]);
    at(&v, "model.py", 10);
    assert!(
        v["error"]["hint"].as_str().unwrap().contains("'Light Bluish Gray'"),
        "{v}"
    );
}

#[test]
fn param_invalid_out_of_range() {
    let dir = project("bad-param");
    let v = golden(
        "param_invalid_range",
        emmet_in(&dir, &["run"]),
        3,
        &[(&dir, "[project]")],
    );
    assert_eq!(v["error"]["code"], "PARAM_INVALID");
    // The line that declares the parameter, with its range and default in the hint.
    at(&v, "model.py", 9);
    assert!(
        v["error"]["hint"]
            .as_str()
            .unwrap()
            .contains("from 2 to 16; the default is 4"),
        "{v}"
    );
}

#[test]
fn param_invalid_unknown_name_and_bad_json() {
    let dir = project("bad-param");
    std::fs::write(dir.join("params.json"), "{ \"widht\": 6 }\n").unwrap();
    let v = golden(
        "param_invalid_name",
        emmet_in(&dir, &["run"]),
        3,
        &[(&dir, "[project]")],
    );
    assert!(
        v["error"]["hint"].as_str().unwrap().contains("Did you mean 'width'?"),
        "{v}"
    );
    at(&v, "params.json", 1);

    std::fs::write(dir.join("params.json"), "{\n  \"width\": 6,\n}\n").unwrap();
    let v = golden(
        "param_invalid_json",
        emmet_in(&dir, &["run"]),
        3,
        &[(&dir, "[project]")],
    );
    at(&v, "params.json", 3);
}

#[test]
fn library_missing() {
    let data = temp_dir("no-library");
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(&data)
        .args(["search", "brick"])
        .output()
        .unwrap();
    let v = golden("library_missing", envelope(&out), 4, &[(&data, "[data]")]);
    assert!(v["error"]["hint"].as_str().unwrap().contains("emmet library install"));
}

#[test]
fn library_corrupt() {
    let data = temp_dir("corrupt");
    let zip = library_zip(&[]);
    let file = data.join("complete.zip");
    std::fs::write(&file, &zip).unwrap();
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(&data)
        .args([
            "library",
            "install",
            "--release",
            "ldraw-mini",
            "--sha256",
            &sha256(&zip),
            "--zip",
        ])
        .arg(&file)
        .output()
        .unwrap();
    assert!(out.status.success());
    std::fs::write(data.join("ldraw/parts/3001.dat"), "0 changed\n").unwrap();
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(&data)
        .args(["library", "verify"])
        .output()
        .unwrap();
    let v = golden("library_corrupt", envelope(&out), 4, &[(&data, "[data]")]);
    assert_eq!(v["error"]["details"]["file"], "[data]/ldraw/parts/3001.dat");
}

#[test]
fn python_missing() {
    // A data dir with the library but no Python, and no uv to install it with.
    let data = temp_dir("no-python");
    let zip = library_zip(&[]);
    let file = data.join("complete.zip");
    std::fs::write(&file, &zip).unwrap();
    let install = emmet()
        .arg("--data-dir")
        .arg(&data)
        .args([
            "library",
            "install",
            "--release",
            "ldraw-mini",
            "--sha256",
            &sha256(&zip),
            "--zip",
        ])
        .arg(&file)
        .output()
        .unwrap();
    assert!(install.status.success());
    let dir = project("frame");
    let trust = emmet()
        .arg("--data-dir")
        .arg(&data)
        .args(["trust", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(trust.status.success());
    let empty = temp_dir("empty-path");
    let out = emmet()
        .env("PATH", &empty)
        .arg("--json")
        .arg("--data-dir")
        .arg(&data)
        .args(["run", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    let v = golden(
        "python_missing",
        envelope(&out),
        4,
        &[(&data, "[data]"), (&dir, "[project]")],
    );
    assert!(v["error"]["hint"].as_str().unwrap().contains("emmet doctor --fix"));
}

#[test]
fn io_error_build_is_a_link() {
    let dir = project("frame");
    let elsewhere = temp_dir("elsewhere");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&elsewhere, dir.join("build")).unwrap();
    #[cfg(windows)]
    std::fs::write(dir.join("build"), "a file where build/ should be").unwrap();
    let v = golden(
        "io_error_build_link",
        emmet_in(&dir, &["run"]),
        3,
        &[(&dir, "[project]")],
    );
    assert_eq!(v["error"]["code"], "IO_ERROR");
    assert_eq!(v["error"]["details"]["file"], "build");
    assert_eq!(
        std::fs::read_dir(&elsewhere).unwrap().count(),
        0,
        "nothing written through the link"
    );
}

#[test]
fn io_error_new_into_a_full_folder() {
    let dir = temp_dir("full");
    std::fs::write(dir.join("notes.txt"), "mine").unwrap();
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .arg("new")
        .arg(&dir)
        .output()
        .unwrap();
    let v = golden("io_error_new_full", envelope(&out), 3, &[(&dir, "[dir]")]);
    assert_eq!(v["error"]["code"], "IO_ERROR");
}

#[test]
fn a_marker_copied_from_created_by_doesnt_trust() {
    // created_by travels with a project, so a shared project must not be able to trust itself with it.
    let demo = temp_dir("forged").join("demo");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .arg("new")
        .arg(&demo)
        .output()
        .unwrap();
    assert!(out.status.success());
    let toml = std::fs::read_to_string(demo.join("emmet.toml")).unwrap();
    let created_by = toml
        .lines()
        .find_map(|l| l.strip_prefix("created_by = \""))
        .unwrap()
        .trim_end_matches('"');
    let forged = temp_dir("forged-copy").join("frame");
    copy_tree(&fixture("projects/frame"), &forged);
    std::fs::create_dir_all(forged.join(".emmet")).unwrap();
    std::fs::write(forged.join(".emmet/trust"), format!("{created_by}\n")).unwrap();
    let (v, code) = emmet_in(&forged, &["run"]);
    assert_eq!(
        (code, v["error"]["code"].as_str()),
        (3, Some("PROJECT_UNTRUSTED")),
        "{v}"
    );
    // A trusted project's marker doesn't carry over to another folder either.
    std::fs::copy(demo.join(".emmet/trust"), forged.join(".emmet/trust")).unwrap();
    assert_eq!(emmet_in(&forged, &["run"]).0["error"]["code"], "PROJECT_UNTRUSTED");
}

#[test]
fn projects_trusted_at_once_on_a_fresh_install_stay_trusted() {
    // CI caught this: with no machine secret yet, parallel `emmet trust` runs each made one and overwrote the others',
    // so markers written a moment earlier stopped matching.
    let data = temp_dir("fresh-secret");
    let zip = library_zip(&[]);
    let file = data.join("complete.zip");
    std::fs::write(&file, &zip).unwrap();
    let install = emmet()
        .arg("--data-dir")
        .arg(&data)
        .args([
            "library",
            "install",
            "--release",
            "ldraw-mini",
            "--sha256",
            &sha256(&zip),
            "--zip",
        ])
        .arg(&file)
        .output()
        .unwrap();
    assert!(install.status.success());
    let dirs: Vec<_> = (0..8)
        .map(|i| {
            let d = temp_dir(&format!("fresh-{i}")).join("frame");
            copy_tree(&fixture("projects/frame"), &d);
            d
        })
        .collect();
    let children: Vec<_> = dirs
        .iter()
        .map(|d| {
            emmet()
                .arg("--data-dir")
                .arg(&data)
                .args(["trust", "--project"])
                .arg(d)
                .spawn()
                .unwrap()
        })
        .collect();
    for mut c in children {
        assert!(c.wait().unwrap().success());
    }
    let empty = temp_dir("fresh-path");
    for d in &dirs {
        // run checks trust first; with no uv on PATH it then stops at PYTHON_MISSING, without downloading Python.
        let out = emmet()
            .env("PATH", &empty)
            .arg("--json")
            .arg("--data-dir")
            .arg(&data)
            .args(["run", "--project"])
            .arg(d)
            .output();
        let (v, _) = envelope(&out.unwrap());
        assert_eq!(v["error"]["code"], "PYTHON_MISSING", "{}: {v}", d.display());
    }
}
