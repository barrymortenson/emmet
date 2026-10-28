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

/// Each connector override (data/connections/*.toml) has a test model that validates with no errors, in which every
/// placement of the overridden part connects to another part (plan section 11).
#[test]
fn each_override_holds_in_its_test_model() {
    let data = emmet_core::data::DataDir::at(shared_library());
    let index = emmet_core::index::Index::ensure(&data).unwrap();
    let lib = emmet_core::ldraw::geometry::LibFiles::probe(&data.ldraw());
    let overrides = emmet_core::check::connectors::overrides();
    assert!(!overrides.is_empty());
    for o in overrides.values() {
        let path = fixture(&format!("validator/connections/{}", o.test));
        let placements = placements_of(&path);
        let v = validate(shared_library(), &placements);
        assert_eq!(v.errors(), 0, "{}: {}\n{}", o.part, o.test, report(&v));
        let links = emmet_core::check::links(&index, &lib, &placements).unwrap();
        // The part itself, or (for a subpart) every part that includes it.
        let users: Vec<&String> = placements
            .iter()
            .filter(|p| p.part == o.part || o.part.starts_with("s/"))
            .map(|p| &p.id)
            .collect();
        assert!(!users.is_empty(), "{}: {} doesn't use it", o.part, o.test);
        for id in users {
            assert!(
                links.iter().any(|(a, b, _)| a == id || b == id),
                "{}: {id} in {} connects to nothing",
                o.part,
                o.test
            );
        }
    }
}

/// Track joins exactly (plan section 11's What good looks like: rung 2's loop depends on it): in a layout built from
/// the end sleepers' frames, each joint is two connections (a knob in a socket, a socket on a knob), plastic to
/// plastic and plastic to the 9V switch; a piece pulled a stud away joins nothing, and a curve turned back into the
/// switch collides with it.
#[test]
fn track_joins_exactly() {
    let data = emmet_core::data::DataDir::at(shared_library());
    let index = emmet_core::index::Index::ensure(&data).unwrap();
    let lib = emmet_core::ldraw::geometry::LibFiles::probe(&data.ldraw());
    let placements = placements_of(&fixture("validator/connections/track.ldr"));
    let joints = |p: &[emmet_core::ldraw::placements::Placement]| {
        let mut out: Vec<(String, String)> = emmet_core::check::links(&index, &lib, p)
            .unwrap()
            .into_iter()
            .filter(|(_, _, kind)| kind == "gen")
            .map(|(a, b, _)| (a, b))
            .collect();
        out.sort();
        out
    };
    let pair = |a: &str, b: &str| (a.to_string(), b.to_string());
    let all = vec![
        pair("part/1", "part/2"),
        pair("part/1", "part/2"),
        pair("part/2", "part/3"),
        pair("part/2", "part/3"),
        pair("part/3", "part/4"),
        pair("part/3", "part/4"),
        pair("part/4", "part/5"),
        pair("part/4", "part/5"),
        pair("part/4", "part/6"),
        pair("part/4", "part/6"),
    ];
    let mut got = joints(&placements);
    got.iter_mut().for_each(|(a, b)| {
        if a > b {
            std::mem::swap(a, b)
        }
    });
    got.sort();
    assert_eq!(got, all);
    assert_eq!(validate(shared_library(), &placements).errors(), 0);
    // The straight on the switch's exit, pulled a stud away along itself.
    let mut apart = placements.clone();
    let m = &mut apart[4].matrix;
    let (dx, dz) = (m[0] * 20.0, m[2] * 20.0);
    m[12] += dx;
    m[14] += dz;
    assert!(!joints(&apart).iter().any(|(a, b)| a == "part/5" || b == "part/5"));
    // The branch's curve turned the other way round runs back into the switch.
    let mut back = placements.clone();
    let turned = glam::DMat4::from_cols_array(&back[5].matrix) * glam::DMat4::from_rotation_y(std::f64::consts::PI);
    back[5].matrix = turned.to_cols_array();
    let v = validate(shared_library(), &back);
    assert!(
        v.findings
            .iter()
            .any(|f| f.rule == emmet_core::check::Rule::Collision || f.rule == emmet_core::check::Rule::Floating),
        "{}",
        report(&v)
    );
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

/// The goldens for the rules today's DSL can't produce (it has no submodels yet, and raises on an unknown part): .mpd
/// files with exactly their findings.
#[test]
fn a_part_the_library_lacks_is_unknown() {
    let v = validate(shared_library(), &placements_of(&fixture("validator/unknown-part.mpd")));
    insta::assert_json_snapshot!(v);
}

#[test]
fn a_submodel_placed_off_its_model_is_disconnected() {
    let v = validate(
        shared_library(),
        &placements_of(&fixture("validator/disconnected-submodel.mpd")),
    );
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

/// Not run by default: builds the collision proxy of every part in `EMMET_CORPUS_DATA`'s library and prints the
/// time and the box counts (for sizing the index).
#[test]
#[ignore]
fn proxy_stats() {
    let data = PathBuf::from(std::env::var("EMMET_CORPUS_DATA").expect("EMMET_CORPUS_DATA"));
    let lib = emmet_core::ldraw::geometry::LibFiles::scan(&data.join("ldraw")).unwrap();
    let parts: Vec<String> = lib.parts().iter().map(|(n, _)| n.to_string()).collect();
    let started = std::time::Instant::now();
    let mut m = emmet_core::ldraw::geometry::Measurer::new(&lib);
    let mut counts = Vec::new();
    for p in &parts {
        let t = std::time::Instant::now();
        let b = emmet_core::check::proxy::build(&mut m, p);
        counts.push((b.len(), t.elapsed().as_millis(), p.clone()));
    }
    counts.sort();
    let total: usize = counts.iter().map(|c| c.0).sum();
    println!("{} parts in {:?}, {total} boxes", parts.len(), started.elapsed());
    for c in counts.iter().rev().take(10) {
        println!("  {:?}", c);
    }
    let mut slow = counts.clone();
    slow.sort_by_key(|c| c.1);
    for c in slow.iter().rev().take(10) {
        println!("  slow {:?}", c);
    }
    for id in [
        "3001", "3003", "3020", "3024", "3062b", "3039", "3811", "53401", "4070", "60592",
    ] {
        if let Some(c) = counts.iter().find(|c| c.2 == format!("{id}.dat")) {
            println!("  {id}: {} boxes", c.0);
        }
    }
}

/// Not run by default: where a stud fits under each part in `EMMET_PROXY_PARTS` (comma-separated ids): stud-grid
/// positions where a stud's volume (6 LDU around its axis, 4 LDU up from the underside) is empty in the part's
/// proxy and the part is solid right above it. For measuring anti-studs the LDCad data lacks (track).
#[test]
#[ignore]
fn anti_studs() {
    let data = emmet_core::data::DataDir::at(shared_library());
    let lib = emmet_core::ldraw::geometry::LibFiles::probe(&data.ldraw());
    for id in std::env::var("EMMET_PROXY_PARTS")
        .expect("EMMET_PROXY_PARTS")
        .split(',')
    {
        let mut m = emmet_core::ldraw::geometry::Measurer::new(&lib);
        let boxes = emmet_core::check::proxy::build(&mut m, &format!("{id}.dat"));
        let bottom = boxes.iter().map(|b| b[4]).max().unwrap_or(0) as f64;
        let (x0, x1) = (
            boxes.iter().map(|b| b[0]).min().unwrap() as f64,
            boxes.iter().map(|b| b[3]).max().unwrap() as f64,
        );
        let (z0, z1) = (
            boxes.iter().map(|b| b[2]).min().unwrap() as f64,
            boxes.iter().map(|b| b[5]).max().unwrap() as f64,
        );
        let hits = |lo: [f64; 3], hi: [f64; 3]| {
            boxes
                .iter()
                .any(|b| (0..3).all(|k| (b[k] as f64) < hi[k] && lo[k] < (b[k + 3] as f64)))
        };
        let mut found = Vec::new();
        let mut x = (x0 / 10.0).floor() * 10.0;
        while x <= x1 {
            let mut z = (z0 / 10.0).floor() * 10.0;
            while z <= z1 {
                let stud_free = !hits([x - 4.0, bottom - 4.0, z - 4.0], [x + 4.0, bottom, z + 4.0]);
                let roof = hits([x - 1.0, bottom - 8.0, z - 1.0], [x + 1.0, bottom - 4.0, z + 1.0]);
                let gripped = hits([x - 8.0, bottom - 4.0, z - 8.0], [x + 8.0, bottom, z + 8.0]);
                if stud_free && roof && gripped {
                    found.push(format!("({x}, {z})"));
                }
                z += 10.0;
            }
            x += 10.0;
        }
        println!(
            "{id}: bottom y={bottom}, {} positions: {}",
            found.len(),
            found.join(" ")
        );
    }
}
