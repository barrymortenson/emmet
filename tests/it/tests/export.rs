// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet export` (plan section 09): the flat .ldr places every part where placements.json does and keeps the main
//! model's steps; the .mpd and the LPub3D .mpd hold the same parts with no `0 !EMMET` lines, and LPub3D gets each
//! run of a sub-assembly's copies as one callout.

use std::path::Path;

use emmet_core::ldraw;
use emmet_it::{emmet_in, project};
use serde_json::Value;

/// (part, color, matrix rounded to 1e-3) for each part, sorted.
fn parts(rows: impl Iterator<Item = (String, u32, [f64; 16])>) -> Vec<(String, u32, Vec<i64>)> {
    let mut out: Vec<_> = rows
        .map(|(p, c, m)| (p, c, m.iter().map(|v| (v * 1000.0).round() as i64).collect()))
        .collect();
    out.sort();
    out
}

fn built(dir: &Path) -> Vec<(String, u32, Vec<i64>)> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("build/placements.json")).unwrap()).unwrap();
    parts(v["parts"].as_array().unwrap().iter().map(|p| {
        let m: Vec<f64> = p["matrix"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        (
            p["part"].as_str().unwrap().to_string(),
            p["color"].as_u64().unwrap() as u32,
            m.try_into().unwrap(),
        )
    }))
}

fn exported(dir: &Path, format: &str) -> String {
    let (v, code) = emmet_in(dir, &["export", "--format", format]);
    assert_eq!(code, 0, "{v}");
    std::fs::read_to_string(v["data"]["path"].as_str().unwrap()).unwrap()
}

fn read_back(text: &str) -> Vec<(String, u32, Vec<i64>)> {
    let doc = ldraw::parse(text).unwrap();
    parts(
        ldraw::placements::placements(&doc)
            .into_iter()
            .map(|p| (p.part, p.color, p.matrix)),
    )
}

#[test]
fn every_format_places_the_parts_the_build_does() {
    // dsl-v1 has a mirror, add_raw, a technique, and an imported submodel; submodel-x4 one submodel four times.
    for name in ["dsl-v1", "submodel-x4"] {
        let dir = project(name);
        let (v, code) = emmet_in(&dir, &["run"]);
        assert_eq!(code, 0, "{v}");
        let steps = v["data"]["steps"].as_u64().unwrap() as usize;
        let want = built(&dir);
        for format in ["ldr", "mpd", "lpub3d"] {
            let text = exported(&dir, format);
            assert!(!text.contains("!EMMET"), "{name} {format} keeps Emmet's own lines");
            assert_eq!(read_back(&text), want, "{name} {format} places different parts");
            if format == "ldr" {
                assert_eq!(
                    text.lines().filter(|l| *l == "0 STEP").count(),
                    steps,
                    "{name}: one 0 STEP a step"
                );
                assert!(!text.contains("0 FILE"), "{name}: the flat file has no submodels");
            }
        }
    }
}

#[test]
fn lpub3d_puts_a_sub_assemblys_copies_in_one_callout() {
    let dir = project("submodel-x4");
    let text = exported(&dir, "lpub3d");
    let main = text.split("0 NOFILE").next().unwrap();
    let lines: Vec<&str> = main.lines().collect();
    let begin = lines
        .iter()
        .position(|l| *l == "0 !LPUB CALLOUT BEGIN")
        .expect("a callout");
    let end = lines.iter().position(|l| *l == "0 !LPUB CALLOUT END").expect("its end");
    assert_eq!(main.matches("!LPUB CALLOUT BEGIN").count(), 1);
    let inside: Vec<&&str> = lines[begin + 1..end].iter().collect();
    assert_eq!(inside.len(), 4);
    assert!(
        inside
            .iter()
            .all(|l| l.starts_with("1 16 ") && l.ends_with(" tower.ldr")),
        "{inside:?}"
    );
}
