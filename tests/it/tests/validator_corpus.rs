// SPDX-License-Identifier: AGPL-3.0-or-later
//! The validator on official models (plan section 11's precision): the committed OMR fixtures and the nightly corpus,
//! which are buildable by definition, have no error findings; the rule fixtures in `tests/fixtures/validator/` get
//! exactly their findings. Recall is `validator_mutations.rs`.

use std::path::PathBuf;

use emmet_core::check::Severity;
use emmet_it::corpus::{self, models_in, placements_of, report, validate};
use emmet_it::{fixture, shared_library};

/// Official models are buildable, so no error findings (plan section 11's precision gate). Their warnings and info
/// (moved and obsolete part ids, a big step) are snapshotted, so a new one is a reviewed change.
#[test]
fn the_omr_fixtures_have_no_errors() {
    let files = models_in(&fixture("omr"));
    assert_eq!(files.len(), 10, "the committed OMR fixtures (ATTRIBUTION.md)");
    let mut bad = Vec::new();
    let mut rest = Vec::new();
    for f in &files {
        let name = f.file_name().unwrap().to_string_lossy().into_owned();
        let v = validate(shared_library(), &placements_of(f));
        if v.errors() > 0 {
            bad.push(format!("{name}:\n{}", report(&v)));
        }
        for f in v.findings.iter().filter(|f| f.severity != Severity::Error) {
            rest.push(format!(
                "{name} {} {} {:?}: {}",
                f.id,
                f.rule.name(),
                f.parts,
                f.message
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "official models must have no errors:\n{}",
        bad.join("\n")
    );
    insta::assert_snapshot!(rest.join("\n"));
}

/// The fixtures are the OMR's files unmodified, and ATTRIBUTION.md credits each one (ADR 0003).
#[test]
fn the_omr_fixtures_are_unmodified_and_credited() {
    let attribution = std::fs::read_to_string(fixture("omr/ATTRIBUTION.md")).unwrap();
    for f in models_in(&fixture("omr")) {
        let name = f.file_name().unwrap().to_string_lossy().into_owned();
        let bytes = std::fs::read(&f).unwrap();
        let row = attribution
            .lines()
            .find(|l| l.starts_with(&format!("| `{name}`")))
            .unwrap_or_else(|| panic!("ATTRIBUTION.md has no row for {name}"));
        assert!(
            row.contains(&emmet_it::sha256(&bytes)),
            "{name} isn't the file ATTRIBUTION.md lists"
        );
        let text = String::from_utf8_lossy(&bytes);
        let author = text
            .lines()
            .find_map(|l| l.strip_prefix("0 Author: "))
            .expect("an author line");
        assert!(
            row.contains(author.trim()),
            "{name}: ATTRIBUTION.md doesn't name {author}"
        );
        assert!(text.contains("0 !LICENSE "), "{name} keeps its license line");
    }
}

#[test]
fn a_part_the_library_lacks_is_unknown() {
    let v = validate(shared_library(), &placements_of(&fixture("validator/unknown-part.mpd")));
    insta::assert_json_snapshot!(v);
}

/// The nightly corpus (`tests/fixtures/omr/nightly.json`, fetched by `scripts/omr-corpus` into
/// `EMMET_CORPUS_DIR`), against the full library in the data dir `EMMET_CORPUS_DATA`: no error findings on any model,
/// and the mutation gates over all of them. `.github/workflows/nightly.yml` runs it; by hand:
///
/// ```sh
/// scripts/omr-corpus /tmp/omr && EMMET_CORPUS_DIR=/tmp/omr EMMET_CORPUS_DATA=<data dir> \
///   cargo test -p emmet-it --test validator_corpus -- --ignored corpus_nightly --nocapture
/// ```
#[test]
#[ignore]
fn corpus_nightly() {
    let dir = PathBuf::from(std::env::var("EMMET_CORPUS_DIR").expect("EMMET_CORPUS_DIR"));
    let data = PathBuf::from(std::env::var("EMMET_CORPUS_DATA").expect("EMMET_CORPUS_DATA"));
    let list: serde_json::Value = serde_json::from_slice(&std::fs::read(fixture("omr/nightly.json")).unwrap()).unwrap();
    let mut models = Vec::new();
    let mut bad = Vec::new();
    println!("| Model | Parts | Errors | Warnings | ms |\n|---|---|---|---|---|");
    for m in list["models"].as_array().unwrap() {
        let file = m["file"].as_str().unwrap();
        let bytes = std::fs::read(dir.join(file)).unwrap_or_else(|e| panic!("{file}: {e} (run scripts/omr-corpus)"));
        assert_eq!(
            emmet_it::sha256(&bytes),
            m["sha256"].as_str().unwrap(),
            "{file} isn't the listed file"
        );
        let placements = placements_of(&dir.join(file));
        let started = std::time::Instant::now();
        let v = validate(&data, &placements);
        let ms = started.elapsed().as_millis();
        let warnings = v.findings.iter().filter(|f| f.severity == Severity::Warning).count();
        println!("| {file} | {} | {} | {warnings} | {ms} |", placements.len(), v.errors());
        if v.errors() > 0 {
            bad.push(format!("{file}:\n{}", report(&v)));
        }
        models.push((file.to_string(), placements));
    }
    let recall = corpus::recall(&data, &models, 3);
    println!("\n{}", recall.table());
    assert!(
        bad.is_empty(),
        "official models must have no errors:\n{}",
        bad.join("\n")
    );
    recall.gate();
}

/// Not run by default: validates every model in `EMMET_CORPUS_DIR` against the library in `EMMET_CORPUS_DATA` and
/// prints the findings. For choosing models and checking a change.
#[test]
#[ignore]
fn corpus_dir() {
    let dir = PathBuf::from(std::env::var("EMMET_CORPUS_DIR").expect("EMMET_CORPUS_DIR"));
    let data = PathBuf::from(std::env::var("EMMET_CORPUS_DATA").expect("EMMET_CORPUS_DATA"));
    let index = emmet_core::index::Index::ensure(&emmet_core::data::DataDir::at(&data)).unwrap();
    for f in models_in(&dir) {
        let placements = placements_of(&f);
        let started = std::time::Instant::now();
        let v = validate(&data, &placements);
        println!(
            "{} — {} findings in {} ms",
            f.file_name().unwrap().to_string_lossy(),
            v.findings.len(),
            started.elapsed().as_millis(),
        );
        for x in &v.findings {
            let names: Vec<String> = x
                .parts
                .iter()
                .filter_map(|id| placements.iter().find(|p| &p.id == id))
                .map(|p| {
                    let name = index.part(&p.part).ok().flatten().map(|q| q.name).unwrap_or_default();
                    format!("{} {} [{name}]", p.id, p.part)
                })
                .collect();
            println!("  {} {} {}: {}", x.id, x.rule.name(), names.join(" | "), x.message);
        }
    }
}

/// Not run by default: prints the connectors and connections of the parts in `EMMET_EXPLAIN_PARTS` (comma-separated
/// ids) of the model `EMMET_EXPLAIN` against `EMMET_CORPUS_DATA` (default: ldraw-mini).
#[test]
#[ignore]
fn explain() {
    let file = PathBuf::from(std::env::var("EMMET_EXPLAIN").expect("EMMET_EXPLAIN"));
    let data = std::env::var("EMMET_CORPUS_DATA").map_or_else(|_| shared_library().to_path_buf(), PathBuf::from);
    let ids: Vec<String> = std::env::var("EMMET_EXPLAIN_PARTS")
        .unwrap_or_default()
        .split(',')
        .map(String::from)
        .collect();
    let data = emmet_core::data::DataDir::at(&data);
    let index = emmet_core::index::Index::ensure(&data).unwrap();
    let lib = emmet_core::ldraw::geometry::LibFiles::probe(&data.ldraw());
    println!(
        "{}",
        emmet_core::check::explain(&index, &lib, &placements_of(&file), &ids).unwrap()
    );
}
