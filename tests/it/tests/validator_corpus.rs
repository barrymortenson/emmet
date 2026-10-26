// SPDX-License-Identifier: AGPL-3.0-or-later
//! The validator on official models (plan section 11's precision): the committed OMR fixtures, which are buildable by
//! definition, have no findings; an .mpd with a part the library doesn't have gets `unknown_part`. 3A adds the
//! mutation generator (recall) and the nightly corpus.

use std::path::{Path, PathBuf};

use emmet_core::check::{Rule, Validation};
use emmet_core::data::DataDir;
use emmet_core::ldraw;
use emmet_it::{fixture, shared_library};

fn validate(data: &Path, mpd: &Path) -> Validation {
    let bytes = std::fs::read(mpd).unwrap_or_else(|e| panic!("{}: {e}", mpd.display()));
    let doc = ldraw::parse(&String::from_utf8_lossy(&bytes)).unwrap_or_else(|e| panic!("{}: {e}", mpd.display()));
    let placements = ldraw::placements::placements(&doc);
    assert!(!placements.is_empty(), "{} places no parts", mpd.display());
    emmet_core::ops::validate_placements(&DataDir::at(data), &placements, &Rule::ALL, &[], None)
        .unwrap_or_else(|e| panic!("{}: {e}", mpd.display()))
}

fn mpds(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "mpd" || x == "ldr"))
        .collect();
    out.sort();
    out
}

fn report(v: &Validation) -> String {
    v.findings
        .iter()
        .map(|f| format!("  {} {} {:?}: {}", f.id, f.rule.name(), f.parts, f.message))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_omr_fixtures_have_no_findings() {
    let files = mpds(&fixture("omr"));
    assert_eq!(
        files.len(),
        5,
        "the five smallest stud-built OMR models (3A brings the corpus to 10)"
    );
    let mut bad = Vec::new();
    for f in &files {
        let v = validate(shared_library(), f);
        if !v.findings.is_empty() {
            bad.push(format!("{}:\n{}", f.file_name().unwrap().to_string_lossy(), report(&v)));
        }
    }
    assert!(
        bad.is_empty(),
        "official models must validate clean:\n{}",
        bad.join("\n")
    );
}

/// The fixtures are the OMR's files unmodified, and ATTRIBUTION.md credits each one (ADR 0003).
#[test]
fn the_omr_fixtures_are_unmodified_and_credited() {
    let attribution = std::fs::read_to_string(fixture("omr/ATTRIBUTION.md")).unwrap();
    for f in mpds(&fixture("omr")) {
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
    let v = validate(shared_library(), &fixture("validator/unknown-part.mpd"));
    insta::assert_json_snapshot!(v);
}

/// Not run by default: validates every .mpd in `EMMET_CORPUS_DIR` against the library in `EMMET_CORPUS_DATA` (a
/// data dir with the full library) and prints the findings. For choosing fixtures and checking a change.
#[test]
#[ignore]
fn corpus_dir() {
    let dir = PathBuf::from(std::env::var("EMMET_CORPUS_DIR").expect("EMMET_CORPUS_DIR"));
    let data = PathBuf::from(std::env::var("EMMET_CORPUS_DATA").expect("EMMET_CORPUS_DATA"));
    for f in mpds(&dir) {
        let started = std::time::Instant::now();
        let v = validate(&data, &f);
        println!(
            "{} — {} findings in {} ms\n{}",
            f.file_name().unwrap().to_string_lossy(),
            v.findings.len(),
            started.elapsed().as_millis(),
            report(&v)
        );
    }
}
