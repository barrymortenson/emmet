// SPDX-License-Identifier: AGPL-3.0-or-later
//! Path cases that differ on Windows (plan section 12): spaces and non-ASCII in the project path, a relative
//! --project, finding the project from a subfolder, part ids written with a different case or a .dat suffix, a path
//! over Windows' old 260-character limit, and a project named in another case (5B).
//! The LDraw reference cases (backslashes, mixed case) are in ldraw.rs. Both run on macOS and Windows.

use emmet_it::{emmet, envelope, shared_data, temp_dir};

/// `emmet --json --data-dir <shared> <args…>`.
fn json(args: &[&std::ffi::OsStr]) -> (serde_json::Value, i32) {
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(args)
        .output()
        .unwrap();
    envelope(&out)
}

#[test]
fn a_project_path_with_spaces_and_accents() {
    let parent = temp_dir("paths");
    let dir = parent.join("Mes maquettes é/My Cottage");
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .arg("new")
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(envelope(&out).1, 0, "{}", emmet_it::text(&out));

    // A relative --project, from the parent folder.
    let out = emmet()
        .current_dir(&parent)
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(["run", "--project", "Mes maquettes é/My Cottage"])
        .output()
        .unwrap();
    let (v, code) = envelope(&out);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["parts"], 905);
    assert!(v["meta"]["project"].as_str().unwrap().ends_with("My Cottage"), "{v}");

    // No --project at all, from a folder inside the project: emmet looks upward for emmet.toml.
    std::fs::create_dir_all(dir.join("models/roofs")).unwrap();
    let out = emmet()
        .current_dir(dir.join("models/roofs"))
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .arg("run")
        .output()
        .unwrap();
    assert_eq!(envelope(&out).1, 0, "{}", emmet_it::text(&out));
}

#[test]
fn part_ids_and_errors_use_forward_slashes_and_lowercase() {
    let dir = temp_dir("ids").join("ids");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let model = "# SPDX-License-Identifier: AGPL-3.0-or-later\nfrom emmet_dsl import Model, model\n\n\n@model\ndef ids():\n    \"\"\"Part ids in other spellings.\"\"\"\n    m = Model()\n    m.add(\"3001.DAT\", \"red\", at=(0, 0, 0))\n    m.add(\" 3062B \", \"LIGHT_BLUISH_GREY\", at=(4, 0, 0))\n    return m\n";
    std::fs::write(dir.join("model.py"), model).unwrap();
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(["run", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    let (v, code) = envelope(&out);
    assert_eq!(code, 0, "{v}");
    let mpd = std::fs::read_to_string(dir.join("build/model.mpd")).unwrap();
    assert!(mpd.contains(" 3001.dat\n") && mpd.contains(" 3062b.dat\n"), "{mpd}");
    assert!(mpd.contains("\n1 71 "), "LIGHT_BLUISH_GREY is code 71: {mpd}");
    assert!(!mpd.contains('\r'), "LF line ends on every OS");
}

/// Windows held paths to 260 characters for decades, and many programs still do; a project deep in a synced or shared
/// folder passes that. The project, its build, and Python's working folder all sit past the limit here.
#[test]
fn a_project_under_a_path_over_260_characters() {
    let mut dir = temp_dir("long");
    while dir.as_os_str().len() < 300 {
        dir.push("a folder with a long name");
    }
    let dir = dir.join("Cottage");
    let (v, code) = json(&["new".as_ref(), dir.as_os_str()]);
    assert_eq!(code, 0, "{v}");
    let (v, code) = json(&["run".as_ref(), "--project".as_ref(), dir.as_os_str()]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["parts"], 905);
    assert!(dir.join("build/model.mpd").is_file());
    let (v, code) = json(&["check".as_ref(), "--project".as_ref(), dir.as_os_str()]);
    assert_eq!(code, 0, "{v}");
}

/// Windows and macOS open a folder whatever case its name is typed in. A project trusted under one spelling is the
/// same project under another: it stays trusted and builds.
#[test]
fn a_project_named_in_another_case_is_the_same_project() {
    let dir = temp_dir("case").join("Cottage");
    let (v, code) = json(&["new".as_ref(), dir.as_os_str()]);
    assert_eq!(code, 0, "{v}");
    let shouted = std::path::PathBuf::from(dir.to_string_lossy().to_uppercase());
    if !shouted.join("emmet.toml").exists() {
        return; // a disk that tells cases apart: these are two folders
    }
    let (v, code) = json(&["run".as_ref(), "--project".as_ref(), shouted.as_os_str()]);
    assert_eq!(code, 0, "trusted, and built: {v}");
    assert_eq!(v["data"]["parts"], 905);
}

/// OneDrive, a virus scanner, or the search indexer opens a file for a moment just after it changes, and Windows
/// refuses to replace a file another program holds. Emmet's writes wait for it instead of failing the build.
#[cfg(windows)]
#[test]
fn a_write_waits_for_a_program_holding_the_file() {
    use std::os::windows::fs::OpenOptionsExt;
    let path = temp_dir("held").join("params.json");
    std::fs::write(&path, "old").unwrap();
    // Share mode 0: no one else may read, write, or replace it while this handle is open.
    let holder = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        drop(holder);
    });
    emmet_core::fsx::write_atomic(&path, b"new").expect("the write waits for the holder");
    release.join().unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
}
