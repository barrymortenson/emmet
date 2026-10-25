// SPDX-License-Identifier: AGPL-3.0-or-later
//! The emmet_dsl sources emmet embeds and installs (crates/emmet-core/src/python.rs) are the package's files.

use emmet_core::python::{DSL_FILES, dsl_version};

#[test]
fn every_emmet_dsl_file_is_embedded() {
    let dir = emmet_it::repo().join("python/emmet_dsl/src/emmet_dsl");
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".py"))
        .collect();
    on_disk.sort();
    let mut embedded: Vec<String> = DSL_FILES.iter().map(|(n, _)| n.to_string()).collect();
    embedded.sort();
    assert_eq!(embedded, on_disk, "add new emmet_dsl files to DSL_FILES");
    for (name, text) in DSL_FILES {
        let disk = std::fs::read_to_string(dir.join(name)).unwrap();
        assert_eq!(text.replace("\r\n", "\n"), disk.replace("\r\n", "\n"), "{name}");
    }
    assert_eq!(dsl_version(), env!("CARGO_PKG_VERSION"));
}
