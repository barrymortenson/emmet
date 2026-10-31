// SPDX-License-Identifier: AGPL-3.0-or-later
//! Saved versions (plan section 07, session 4A) through the real emmet: save, change every kind of source, restore,
//! and the sources come back byte for byte, with the state before the restore saved first; open as a new project
//! makes a trusted project that builds; Emmet's own versions stop at 20.

use std::collections::BTreeMap;
use std::path::Path;

use emmet_it::{emmet_in, emmet_json, temp_dir};
use serde_json::Value;

/// Every source in the project, by its path inside it: model.py, models/**, params.json, emmet.toml, CARD.md.
fn sources(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for top in ["model.py", "params.json", "emmet.toml", "CARD.md"] {
        if let Ok(b) = std::fs::read(root.join(top)) {
            out.insert(top.to_string(), b);
        }
    }
    let mut stack = vec![root.join("models")];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            if e.file_type().unwrap().is_dir() {
                stack.push(e.path());
            } else if e.path().extension().is_some_and(|x| x == "py") {
                let rel = e
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read(e.path()).unwrap());
            }
        }
    }
    out
}

fn ok(v: &Value, code: i32) -> &Value {
    assert_eq!(code, 0, "{v}");
    &v["data"]
}

fn new_cottage(name: &str) -> std::path::PathBuf {
    let dir = temp_dir(name).join("cottage");
    let (v, code) = emmet_json(&["new", dir.to_str().unwrap()]);
    ok(&v, code);
    dir
}

#[test]
fn save_change_restore_gives_the_same_bytes() {
    let dir = new_cottage("versions-roundtrip");
    // A models/ file and a built model, so the version holds a submodule and the build's model.mpd.
    std::fs::create_dir_all(dir.join("models")).unwrap();
    std::fs::write(dir.join("models/trim.py"), "# trim helpers\r\nWIDTH = 2\n").unwrap();
    let (v, code) = emmet_in(&dir, &["run"]);
    ok(&v, code);
    let before = sources(&dir);

    let (v, code) = emmet_in(
        &dir,
        &["version", "save", "Roof and chimney", "--note", "the first good one"],
    );
    let saved = ok(&v, code).clone();
    assert_eq!(saved["n"], 1);
    assert_eq!(saved["dir"], "1-roof-and-chimney");
    assert_eq!(saved["auto"], false);
    assert_eq!(saved["note"], "the first good one");
    assert!(
        saved["parts"].as_u64().unwrap() > 100,
        "a current build's part count: {saved}"
    );
    let folder = dir.join("history/1-roof-and-chimney");
    assert!(folder.join("model.mpd").is_file() && folder.join("placements.json").is_file());

    // Change every kind of source: edit model.py and a submodule, add one, remove CARD.md, set a param, and a build
    // setting in emmet.toml.
    let model = std::fs::read_to_string(dir.join("model.py")).unwrap();
    std::fs::write(dir.join("model.py"), model.replace("cottage", "cottage, changed")).unwrap();
    std::fs::write(dir.join("models/trim.py"), "WIDTH = 3\n").unwrap();
    std::fs::write(dir.join("models/extra.py"), "X = 1\n").unwrap();
    std::fs::remove_file(dir.join("CARD.md")).unwrap();
    std::fs::write(dir.join("params.json"), "{\n  \"width\": 14\n}\n").unwrap();
    let toml = std::fs::read_to_string(dir.join("emmet.toml")).unwrap();
    std::fs::write(
        dir.join("emmet.toml"),
        format!("{toml}\n[build]\nstyle = \"Steampunk\"\n"),
    )
    .unwrap();
    let changed = sources(&dir);
    assert_ne!(changed, before);

    let (v, code) = emmet_in(&dir, &["version", "restore", "1"]);
    let r = ok(&v, code).clone();
    assert_eq!(
        sources(&dir),
        before,
        "the restore gives back every source byte for byte"
    );
    assert_eq!(r["removed"], serde_json::json!(["models/extra.py"]));

    // The restore saved the changed state first, as Emmet's own version, and it holds exactly what was there.
    assert_eq!(r["saved"]["n"], 2);
    assert_eq!(r["saved"]["auto"], true);
    assert_eq!(r["saved"]["name"], "Before restoring version 1");
    let kept = dir.join("history").join(r["saved"]["dir"].as_str().unwrap());
    assert_eq!(
        sources(&kept),
        changed,
        "nothing is lost: the state before the restore is a version"
    );
    assert!(
        !kept.join("model.mpd").exists(),
        "a stale build isn't kept with the version"
    );

    // Restoring that one brings the changes back.
    let (v, code) = emmet_in(&dir, &["version", "restore", "2"]);
    ok(&v, code);
    assert_eq!(sources(&dir), changed);

    let (v, code) = emmet_in(&dir, &["version", "list"]);
    let list: Vec<(u64, bool)> = ok(&v, code)
        .as_array()
        .unwrap()
        .iter()
        .map(|v| (v["n"].as_u64().unwrap(), v["auto"].as_bool().unwrap()))
        .collect();
    assert_eq!(list, [(3, true), (2, true), (1, false)], "newest first");
}

#[test]
fn open_as_new_project_is_trusted_and_builds() {
    let dir = new_cottage("versions-open");
    let toml = std::fs::read_to_string(dir.join("emmet.toml")).unwrap();
    std::fs::write(dir.join("emmet.toml"), format!("{toml}\n[build]\nage = \"9+\"\n")).unwrap();
    let (v, code) = emmet_in(&dir, &["version", "save", "Nine plus"]);
    ok(&v, code);
    let dest = dir.parent().unwrap().join("Cottage version 1");
    let (v, code) = emmet_in(&dir, &["version", "open", "1", dest.to_str().unwrap()]);
    ok(&v, code);
    assert_eq!(
        std::fs::read(dest.join("model.py")).unwrap(),
        std::fs::read(dir.join("model.py")).unwrap()
    );
    let config = std::fs::read_to_string(dest.join("emmet.toml")).unwrap();
    assert!(
        config.contains("name = \"Cottage Version 1\""),
        "named for its folder: {config}"
    );
    assert!(
        config.contains("age = \"9+\""),
        "the version's build settings come along: {config}"
    );
    assert!(
        !dest.join("history").exists(),
        "the new project starts without the old one's versions"
    );
    // Trusted on this machine, so it runs.
    let (v, code) = emmet_in(&dest, &["run"]);
    ok(&v, code);
}

#[test]
fn emmet_keeps_its_last_twenty_versions_and_every_named_one() {
    let dir = new_cottage("versions-prune");
    let (v, code) = emmet_in(&dir, &["version", "save", "Mine"]);
    ok(&v, code);
    // Each restore of version 1 saves Emmet's own version first.
    for _ in 0..22 {
        let (v, code) = emmet_in(&dir, &["version", "restore", "1"]);
        ok(&v, code);
    }
    let (v, code) = emmet_in(&dir, &["version", "list"]);
    let list = ok(&v, code).as_array().unwrap().clone();
    let auto = list.iter().filter(|v| v["auto"] == true).count();
    assert_eq!(auto, 20);
    assert!(list.iter().any(|v| v["name"] == "Mine"), "the user's version stays");
    assert_eq!(list[0]["n"], 23, "the newest is kept");
    assert_eq!(
        std::fs::read_dir(dir.join("history")).unwrap().count(),
        21,
        "pruned folders are gone from history/"
    );
}

#[test]
fn a_missing_version_or_name_says_so() {
    let dir = new_cottage("versions-errors");
    let (v, code) = emmet_in(&dir, &["version", "restore", "4"]);
    assert_ne!(code, 0);
    assert_eq!(v["error"]["message"], "This project has no version 4.");
    let (v, code) = emmet_in(&dir, &["version", "save", "  "]);
    assert_ne!(code, 0);
    assert_eq!(v["error"]["message"], "A version needs a name.");
    // A version.json that names another folder isn't a version (a copied folder can't stand in for one).
    let fake = dir.join("history/9-elsewhere");
    std::fs::create_dir_all(&fake).unwrap();
    std::fs::write(
        fake.join("version.json"),
        r#"{"n":9,"name":"x","time":0,"auto":false,"dir":"../../outside"}"#,
    )
    .unwrap();
    let (v, code) = emmet_in(&dir, &["version", "list"]);
    assert_eq!(ok(&v, code).as_array().unwrap().len(), 0);
}

#[test]
fn an_untrusted_projects_versions_cant_be_restored_or_opened() {
    let dir = new_cottage("versions-untrusted");
    let (v, code) = emmet_in(&dir, &["version", "save", "Mine"]);
    ok(&v, code);
    // Moved, so no longer trusted: its history/ is as untrusted as its model.py.
    let moved = dir.parent().unwrap().join("moved");
    std::fs::rename(&dir, &moved).unwrap();
    let (v, code) = emmet_in(&moved, &["version", "restore", "1"]);
    assert_ne!(code, 0);
    assert_eq!(v["error"]["code"], "PROJECT_UNTRUSTED");
    let dest = moved.parent().unwrap().join("copy");
    let (v, code) = emmet_in(&moved, &["version", "open", "1", dest.to_str().unwrap()]);
    assert_eq!(v["error"]["code"], "PROJECT_UNTRUSTED", "{v}");
    assert_ne!(code, 0);
    assert!(!dest.exists(), "no trusted copy of untrusted code is made");
}

#[cfg(unix)]
#[test]
fn a_linked_models_folder_or_source_is_refused() {
    let dir = new_cottage("versions-links");
    let outside = dir.parent().unwrap().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.py"), "SECRET = 1\n").unwrap();

    // A save won't read a source through a link.
    std::os::unix::fs::symlink(&outside, dir.join("models")).unwrap();
    let (v, code) = emmet_in(&dir, &["version", "save", "Linked"]);
    assert_ne!(code, 0, "{v}");
    assert!(v["error"]["message"].as_str().unwrap().contains("is a link"), "{v}");
    std::fs::remove_file(dir.join("models")).unwrap();

    // A version with a models/ file, restored into a project whose models/ became a link to an empty folder (so the
    // save before the restore has nothing to refuse): the write is what's refused, and nothing lands outside.
    std::fs::create_dir_all(dir.join("models")).unwrap();
    std::fs::write(dir.join("models/trim.py"), "WIDTH = 2\n").unwrap();
    let (v, code) = emmet_in(&dir, &["version", "save", "With trim"]);
    ok(&v, code);
    let empty = dir.parent().unwrap().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    std::fs::remove_dir_all(dir.join("models")).unwrap();
    std::os::unix::fs::symlink(&empty, dir.join("models")).unwrap();
    let (v, code) = emmet_in(&dir, &["version", "restore", "1"]);
    assert_ne!(code, 0, "{v}");
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with("models/trim.py is a link"),
        "{v}"
    );
    assert_eq!(
        std::fs::read_dir(&empty).unwrap().count(),
        0,
        "the restore wrote nothing outside the project"
    );
    assert_eq!(
        std::fs::read_to_string(outside.join("secret.py")).unwrap(),
        "SECRET = 1\n"
    );

    // A version whose own model.py is a link (a shared project's crafted history/) is refused before anything is
    // saved or written.
    std::fs::remove_file(dir.join("models")).unwrap();
    let version = dir.join("history/1-with-trim/model.py");
    std::fs::remove_file(&version).unwrap();
    std::os::unix::fs::symlink(outside.join("secret.py"), &version).unwrap();
    let folders = std::fs::read_dir(dir.join("history")).unwrap().count();
    let model = std::fs::read(dir.join("model.py")).unwrap();
    let (v, code) = emmet_in(&dir, &["version", "restore", "1"]);
    assert_ne!(code, 0, "{v}");
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with("model.py is a link"),
        "{v}"
    );
    assert_eq!(
        std::fs::read_dir(dir.join("history")).unwrap().count(),
        folders,
        "nothing saved"
    );
    assert_eq!(std::fs::read(dir.join("model.py")).unwrap(), model, "nothing written");
}

#[cfg(unix)]
#[test]
fn a_restore_that_failed_can_be_tried_again() {
    let dir = new_cottage("versions-retry");
    let (v, code) = emmet_in(&dir, &["version", "save", "Mine"]);
    ok(&v, code);
    // The save before the restore stops partway, at a linked models/ file.
    let outside = dir.parent().unwrap().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.py"), "SECRET = 1\n").unwrap();
    std::os::unix::fs::symlink(&outside, dir.join("models")).unwrap();
    let (v, code) = emmet_in(&dir, &["version", "restore", "1"]);
    assert_ne!(code, 0, "{v}");
    // With the link gone, the same restore (and its save of the same name) works.
    std::fs::remove_file(dir.join("models")).unwrap();
    let (v, code) = emmet_in(&dir, &["version", "restore", "1"]);
    ok(&v, code);
    let (v, code) = emmet_in(&dir, &["version", "list"]);
    assert_eq!(
        ok(&v, code).as_array().unwrap().len(),
        2,
        "Mine and one save before the restore"
    );
}

#[test]
fn restoring_the_oldest_of_emmets_own_versions_works() {
    let dir = new_cottage("versions-oldest");
    let (v, code) = emmet_in(&dir, &["version", "save", "Mine"]);
    ok(&v, code);
    // Versions 2 to 21: Emmet's own, as many as it keeps.
    for _ in 0..20 {
        let (v, code) = emmet_in(&dir, &["version", "restore", "1"]);
        ok(&v, code);
    }
    // Version 2 is the oldest kept; the save before restoring it would make it the 21st.
    let (v, code) = emmet_in(&dir, &["version", "restore", "2"]);
    let data = ok(&v, code);
    assert_eq!(data["restored"]["n"], 2, "{v}");
    let (v, code) = emmet_in(&dir, &["version", "list"]);
    let list = ok(&v, code).as_array().unwrap().clone();
    assert_eq!(
        list.iter().filter(|v| v["auto"] == true).count(),
        20,
        "pruned after the restore"
    );
}

#[test]
fn an_imported_projects_version_keeps_its_ldraw_file() {
    let dir = temp_dir("versions-import").join("Wagon");
    let file = emmet_it::fixture("projects/dsl-v1/wagon.ldr");
    let (v, code) = emmet_json(&["new", dir.to_str().unwrap(), "--import", file.to_str().unwrap()]);
    ok(&v, code);
    let (v, code) = emmet_in(&dir, &["version", "save", "As imported"]);
    ok(&v, code);
    assert!(dir.join("history/1-as-imported/wagon.ldr").is_file());
    let dest = dir.parent().unwrap().join("Wagon version 1");
    let (v, code) = emmet_in(&dir, &["version", "open", "1", dest.to_str().unwrap()]);
    ok(&v, code);
    // The copy's model.py brings in wagon.ldr, so the copy has it, and builds.
    assert_eq!(
        std::fs::read(dest.join("wagon.ldr")).unwrap(),
        std::fs::read(&file).unwrap()
    );
    let (v, code) = emmet_in(&dest, &["run"]);
    ok(&v, code);
    // An edited LDraw file comes back on restore.
    std::fs::write(dir.join("wagon.ldr"), "0 edited\n").unwrap();
    let (v, code) = emmet_in(&dir, &["version", "restore", "1"]);
    ok(&v, code);
    assert_eq!(
        std::fs::read(dir.join("wagon.ldr")).unwrap(),
        std::fs::read(&file).unwrap()
    );
}

/// An agent's turn that changed nothing keeps no version (4B): the sources match the turn's version until any of them
/// is edited, added, or removed, and only Emmet's own version can be discarded, never one saved by name.
#[test]
fn a_turn_that_changed_nothing_is_told_apart_and_only_emmets_own_version_goes() {
    use emmet_core::{data::DataDir, project::Project, versions};
    let dir = new_cottage("versions-unchanged");
    let data = DataDir::at(temp_dir("versions-unchanged-data"));
    let project = Project::find(Some(&dir)).unwrap();
    let turn = versions::save(&data, &project, "Before Claude's turn", None, true, None)
        .unwrap()
        .n;
    let same = || versions::unchanged_since(&Project::find(Some(&dir)).unwrap(), turn).unwrap();
    assert!(same());

    let model = std::fs::read(dir.join("model.py")).unwrap();
    std::fs::write(dir.join("model.py"), [&model[..], b"\n# taller\n"].concat()).unwrap();
    assert!(!same(), "an edit");
    std::fs::write(dir.join("model.py"), &model).unwrap();
    assert!(same(), "the edit undone");
    std::fs::create_dir_all(dir.join("models")).unwrap();
    std::fs::write(dir.join("models/trim.py"), "WIDTH = 2\n").unwrap();
    assert!(!same(), "a new models/ file");
    std::fs::remove_file(dir.join("models/trim.py")).unwrap();
    assert!(same());
    let params = std::fs::read(dir.join("params.json")).unwrap();
    std::fs::remove_file(dir.join("params.json")).unwrap();
    assert!(!same(), "a removed source");
    std::fs::write(dir.join("params.json"), &params).unwrap();

    versions::discard(&project, turn).unwrap();
    assert!(versions::get(&project, turn).is_err());
    let named = versions::save(&data, &project, "Mine", None, false, None).unwrap().n;
    assert!(versions::discard(&project, named).is_err());
    assert!(versions::get(&project, named).is_ok());
}
