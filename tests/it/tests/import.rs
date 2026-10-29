// SPDX-License-Identifier: AGPL-3.0-or-later
//! Opening an existing LDraw file (plan section 07's imports, session 4A): `emmet new --import` copies the file into a
//! new project whose model.py brings it in with import_ldraw, so it builds, checks, and has steps like any model, and
//! a file that isn't LDraw is refused.

use emmet_it::{emmet_in, emmet_json, fixture, temp_dir};

#[test]
fn an_imported_ldr_builds_with_its_steps() {
    let dir = temp_dir("import").join("Wagon (imported)");
    let file = fixture("projects/dsl-v1/wagon.ldr");
    let (v, code) = emmet_json(&["new", dir.to_str().unwrap(), "--import", file.to_str().unwrap()]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(
        std::fs::read(dir.join("wagon.ldr")).unwrap(),
        std::fs::read(&file).unwrap()
    );
    let model = std::fs::read_to_string(dir.join("model.py")).unwrap();
    assert!(model.contains("import_ldraw(\"wagon.ldr\")"), "{model}");

    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let placements: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("build/placements.json")).unwrap()).unwrap();
    let parts = placements["parts"].as_array().unwrap();
    let wagon = std::fs::read_to_string(&file).unwrap();
    let lines = wagon.lines().filter(|l| l.trim_start().starts_with("1 ")).count();
    assert_eq!(parts.len(), lines, "every part line of the file is a part");

    let (v, code) = emmet_in(&dir, &["steps"]);
    assert_eq!(code, 0, "{v}");
    let (v, code) = emmet_in(&dir, &["params"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(
        v["data"]["params"].as_array().unwrap().len(),
        0,
        "a fixed model has no knobs yet"
    );
}

#[test]
fn a_file_that_isnt_ldraw_is_refused() {
    let dir = temp_dir("import-bad").join("notes");
    let file = dir.parent().unwrap().join("notes.txt");
    std::fs::write(&file, "not a model").unwrap();
    let (v, code) = emmet_json(&["new", dir.to_str().unwrap(), "--import", file.to_str().unwrap()]);
    assert_ne!(code, 0);
    assert_eq!(v["error"]["message"], "notes.txt isn't an LDraw model file.");
    assert!(!dir.join("model.py").exists(), "nothing is made");
}
