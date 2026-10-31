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

/// A project fixture (or the rung 2 reference, or the cottage template) built, with the spaces it keeps clear.
fn built(name: &str) -> (Vec<Placement>, Vec<emmet_core::check::KeepClear>) {
    let dir = temp_dir(&format!("mutations-{name}")).join(name);
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
    match name {
        "cottage" => run(&["new", dir.to_str().unwrap()]),
        "rung2_reference" => {
            run(&["new", "--template", "empty", dir.to_str().unwrap()]);
            std::fs::copy(
                emmet_it::repo().join("data/ladder/rung2_reference.py"),
                dir.join("model.py"),
            )
            .unwrap();
        }
        _ => {
            emmet_it::copy_tree(&fixture(&format!("projects/{name}")), &dir);
            run(&["trust", "--project", dir.to_str().unwrap()]);
        }
    }
    run(&["run", "--project", dir.to_str().unwrap()]);
    let doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("build/placements.json")).unwrap()).unwrap();
    (
        serde_json::from_value(doc["parts"].clone()).unwrap(),
        serde_json::from_value(doc["keep_clear"].clone()).unwrap_or_default(),
    )
}

/// figure_clearance's recall (3E, Done when 3): a bench moved against a post, what stands over a door's approach lowered
/// a brick, and a wall across a doorway, in the models that keep spaces clear; each model is clean before. At least
/// 90 % caught, on at least 3 faults of each kind.
#[test]
fn figure_faults_are_caught() {
    let data = shared_data();
    let index = emmet_core::index::Index::ensure(&emmet_core::data::DataDir::at(data)).unwrap();
    let mut made: std::collections::BTreeMap<corpus::FigureFault, (usize, usize)> = Default::default();
    let mut missed = Vec::new();
    for name in ["figure-clearance-clean", "rung2_reference", "cottage"] {
        let (placements, keep) = built(name);
        let v = corpus::validate_keeping(data, &placements, &keep);
        assert!(
            v.findings
                .iter()
                .all(|f| f.rule != emmet_core::check::Rule::FigureClearance),
            "{name} isn't clean before the faults:\n{}",
            corpus::report(&v)
        );
        for m in corpus::figure_mutants(&index, &placements, &keep) {
            let v = corpus::validate_keeping(data, &m.placements, &m.keep);
            let e = made.entry(m.fault).or_default();
            e.1 += 1;
            if m.caught(&v) {
                e.0 += 1;
            } else {
                missed.push(format!("{:?} in {name}: {}\n{}", m.fault, m.what, corpus::report(&v)));
            }
        }
    }
    let (caught, total) = made.values().fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    println!("| Fault | Caught |\n|---|---|");
    for (f, (c, t)) in &made {
        println!("| {f:?} | {c} / {t} |");
    }
    println!("| **figure_clearance** | {caught} / {total} |");
    for f in [
        corpus::FigureFault::BenchToPost,
        corpus::FigureFault::LowerCanopy,
        corpus::FigureFault::WallAcross,
    ] {
        assert!(
            made.get(&f).is_some_and(|e| e.1 >= 3),
            "{f:?} has fewer than 3 faults: {made:?}"
        );
    }
    assert!(
        caught * 100 >= total * 90,
        "figure_clearance recall under 90 %:\n{}",
        missed.join("\n")
    );
}
