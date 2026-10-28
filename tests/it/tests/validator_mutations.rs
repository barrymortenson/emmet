// SPDX-License-Identifier: AGPL-3.0-or-later
//! The validator's recall (plan section 11): faults injected by the seeded generator in `emmet_it::corpus` into the
//! committed OMR fixtures and the rung 1 reference (the cottage), each with the rule that must catch it. Gates: at
//! least 95 % caught overall and 90 % for each error rule. The nightly job runs the same generator over the nightly
//! corpus (`validator_corpus.rs`).

use emmet_core::ldraw::placements::Placement;
use emmet_it::corpus;
use emmet_it::{emmet, fixture, shared_data, shared_library, temp_dir};

/// The cottage template's placements at its defaults.
fn cottage() -> Vec<Placement> {
    let dir = temp_dir("mutations-cottage").join("cottage");
    let run = |args: &[&str]| {
        let out = emmet()
            .arg("--data-dir")
            .arg(shared_data())
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run(&["new", dir.to_str().unwrap()]);
    run(&["run", "--project", dir.to_str().unwrap()]);
    let doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("build/placements.json")).unwrap()).unwrap();
    serde_json::from_value(doc["parts"].clone()).unwrap()
}

#[test]
fn injected_faults_are_caught() {
    let mut models: Vec<(String, Vec<Placement>)> = corpus::models_in(&fixture("omr"))
        .iter()
        .map(|f| {
            (
                f.file_name().unwrap().to_string_lossy().into_owned(),
                corpus::placements_of(f),
            )
        })
        .collect();
    models.push(("cottage".into(), cottage()));
    let recall = corpus::recall(shared_library(), &models, 4);
    println!("{}", recall.table());
    recall.gate();
}
