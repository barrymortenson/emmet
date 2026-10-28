// SPDX-License-Identifier: AGPL-3.0-or-later
//! The DSL v1 (plan section 08) through the real emmet: every call `emmet api` documents is used by a golden project
//! that validates clean, a submodel placed four times is one file with four references (and the BOM, steps, and ids
//! follow), and `emmet params --write-default` changes one literal in model.py.

use std::path::Path;

use emmet_it::{emmet, emmet_in, emmet_json, envelope, project, repo, shared_data, temp_dir};
use serde_json::Value;

/// The model files the goldens are: the fixture projects, the templates, and the ladder's references.
fn golden_sources() -> String {
    let mut all = String::new();
    let projects = repo().join("tests/fixtures/projects");
    for entry in std::fs::read_dir(&projects).unwrap() {
        let model = entry.unwrap().path().join("model.py");
        if let Ok(text) = std::fs::read_to_string(&model) {
            all.push_str(&text);
        }
    }
    for rel in [
        "crates/emmet-core/templates/cottage/model.py",
        "crates/emmet-core/templates/empty/model.py",
        "data/ladder/rung2_reference.py",
    ] {
        all.push_str(&std::fs::read_to_string(repo().join(rel)).unwrap());
    }
    all
}

#[test]
fn every_api_call_is_documented_and_used() {
    let (v, code) = emmet_json(&["api"]);
    assert_eq!(code, 0, "{v}");
    let api = v["data"].as_str().unwrap();
    // "### Model.wall(…)" is a call on a Model; "## import_ldraw(…)" a function. Classes and properties aren't calls.
    let mut calls: Vec<String> = Vec::new();
    for line in api.lines() {
        if let Some(rest) = line.strip_prefix("### Model.") {
            calls.push(format!("m.{}(", rest.split('(').next().unwrap()));
        } else if let Some(rest) = line
            .strip_prefix("## ")
            .filter(|r| !r.starts_with("class ") && r.contains('('))
        {
            let name = rest.split('(').next().unwrap();
            calls.push(if name == "model" {
                "@model".into()
            } else {
                format!("{name}(")
            });
        }
    }
    // Plan section 08's calls, each documented.
    for call in [
        "@model",
        "m.add(",
        "m.step(",
        "m.wall(",
        "m.row(",
        "m.grid(",
        "m.stack(",
        "m.fill(",
        "m.repeat(",
        "m.mirror(",
        "m.submodel(",
        "m.place(",
        "m.use(",
        "m.allow(",
        "import_ldraw(",
        "m.add_raw(",
        "Brick(",
        "Plate(",
        "Tile(",
        "Slope(",
    ] {
        assert!(calls.iter().any(|c| c == call), "{call} isn't in emmet api");
    }
    let goldens = golden_sources();
    let unused: Vec<&String> = calls.iter().filter(|c| !goldens.contains(c.as_str())).collect();
    assert!(unused.is_empty(), "no golden project uses {unused:?}");
    assert!(goldens.contains("Rot("), "no golden project uses Rot");

    // The goldens made for this run and validate with no errors (the cottage and the rung 2 reference have their own
    // tests in check.rs and ladder.rs).
    for name in ["dsl-v1", "submodel-x4"] {
        let dir = project(name);
        let (v, code) = emmet_in(&dir, &["check"]);
        assert_eq!(code, 0, "{v}");
        assert_eq!(v["data"]["findings"], serde_json::json!([]), "{name}: {v:#}");
    }
}

fn read(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap()
}

fn placements(dir: &Path) -> Vec<Value> {
    let v: Value = serde_json::from_str(&read(dir, "build/placements.json")).unwrap();
    v["parts"].as_array().unwrap().clone()
}

#[test]
fn a_submodel_placed_four_times() {
    let dir = project("submodel-x4");
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["submodels"], 1);

    // The .mpd: the main file and one submodel file, which the main file references four times.
    let mpd = read(&dir, "build/model.mpd");
    let files: Vec<&str> = mpd.lines().filter(|l| l.starts_with("0 FILE ")).collect();
    assert_eq!(files, ["0 FILE towers.ldr", "0 FILE tower.ldr"]);
    let references = mpd
        .lines()
        .filter(|l| l.starts_with("1 ") && l.ends_with(" tower.ldr"))
        .count();
    assert_eq!(references, 4);

    // The BOM counts every copy: 4 bases, 12 column bricks, 4 caps, and the plate.
    let (v, code) = emmet_in(&dir, &["bom"]);
    assert_eq!(code, 0, "{v}");
    let qty = |part: &str| -> u64 {
        v["data"]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|l| l["part"] == part)
            .map(|l| l["qty"].as_u64().unwrap())
            .sum()
    };
    assert_eq!((qty("3003"), qty("3004"), qty("3022"), qty("3958")), (4, 12, 4, 1));
    assert_eq!(v["data"]["totals"]["pieces"], 21);

    // The steps build the tower once, in its own steps, and place it × 4 in one step.
    let (v, code) = emmet_in(&dir, &["steps"]);
    assert_eq!(code, 0, "{v}");
    let sub = &v["data"]["submodels"][0];
    assert_eq!(
        (sub["name"].as_str(), sub["count"].as_u64(), sub["parts"].as_u64()),
        (Some("Tower"), Some(4), Some(5))
    );
    assert_eq!(sub["steps"].as_array().unwrap().len(), 3);
    let towers = &v["data"]["steps"][1];
    assert_eq!(towers["title"], "Towers");
    assert_eq!(towers["submodels"][0]["count"], 4);
    assert_eq!(
        towers["submodels"][0]["ids"],
        serde_json::json!(["tower/1", "tower/2", "tower/3", "tower/4"])
    );

    // Every id is unique, each copy's parts carry its id, and they say which copy and step placed them.
    let parts = placements(&dir);
    let mut ids: Vec<&str> = parts.iter().map(|p| p["id"].as_str().unwrap()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), total, "duplicate ids");
    let cap = parts.iter().find(|p| p["id"] == "tower/3/cap").expect("tower/3/cap");
    assert_eq!(cap["submodel"], serde_json::json!(["tower.ldr"]));
    assert_eq!(cap["copies"], serde_json::json!([{ "id": "tower/3", "step": 2 }]));

    // Editing the submodel changes all four copies, and the .mpd still holds it once.
    let model = read(&dir, "model.py").replace(r#""3022", "Dark Red""#, r#""3022", "Blue""#);
    std::fs::write(dir.join("model.py"), model).unwrap();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let caps: Vec<u64> = placements(&dir)
        .iter()
        .filter(|p| p["part"] == "3022")
        .map(|p| p["color"].as_u64().unwrap())
        .collect();
    assert_eq!(caps, [1, 1, 1, 1]);
    assert_eq!(read(&dir, "build/model.mpd").matches("0 FILE tower.ldr").count(), 1);
}

#[test]
fn write_default_rewrites_one_literal() {
    let dir = temp_dir("write-default").join("demo");
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .arg("new")
        .arg(&dir)
        .output()
        .unwrap();
    let (v, code) = envelope(&out);
    assert_eq!(code, 0, "{v}");
    let before = read(&dir, "model.py");
    let (v, code) = emmet_in(&dir, &["params", "--set", "width=12", "roof_color=Dark Green"]);
    assert_eq!(code, 0, "{v}");

    let (v, code) = emmet_in(&dir, &["params", "--write-default", "width"]);
    assert_eq!(code, 0, "{v}");
    let width = v["data"]["params"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "width")
        .unwrap();
    assert_eq!(
        (width["default"].as_u64(), v["data"]["values"]["width"].as_u64()),
        (Some(12), Some(12))
    );
    let (v, code) = emmet_in(&dir, &["params", "--write-default", "roof_color"]);
    assert_eq!(code, 0, "{v}");

    // Only the two literals changed; params.json no longer overrides them.
    let after = read(&dir, "model.py");
    let expected = before
        .replacen("Range(6, 16)] = 10,", "Range(6, 16)] = 12,", 1)
        .replacen(
            r#"roof_color: Color = "Dark Red","#,
            r#"roof_color: Color = "Dark Green","#,
            1,
        );
    assert_ne!(expected, before, "the template's signature changed; update this test");
    assert_eq!(after, expected);
    assert_eq!(read(&dir, "params.json").trim(), "{}");

    // A default that isn't a plain value is refused at its line.
    let model = after.replacen("Range(6, 16)] = 12,", "Range(6, 16)] = 6 + 6,", 1);
    std::fs::write(dir.join("model.py"), &model).unwrap();
    emmet_in(&dir, &["params", "--set", "width=8"]);
    let (v, code) = emmet_in(&dir, &["params", "--write-default", "width"]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "PARAM_INVALID");
    assert_eq!(read(&dir, "model.py"), model);
}
