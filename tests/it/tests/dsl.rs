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

/// A fresh project whose model.py is `source`, with `files` beside it, trusted.
fn model_project(name: &str, source: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let dir = temp_dir(name).join(name);
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    let (v, code) = envelope(&out);
    assert_eq!(code, 0, "{v}");
    std::fs::write(dir.join("model.py"), source).unwrap();
    for (rel, text) in files {
        std::fs::write(dir.join(rel), text).unwrap();
    }
    dir
}

/// The DSL x of each part's footprint, by id: a 1 x 1 part's origin is half a stud in from its corner.
fn xs(dir: &Path) -> std::collections::BTreeMap<String, f64> {
    placements(dir)
        .iter()
        .map(|p| {
            let z = p["matrix"][14].as_f64().unwrap();
            (p["id"].as_str().unwrap().to_string(), -z / 20.0 - 0.5)
        })
        .collect()
}

#[test]
fn a_submodel_named_like_the_model_gets_its_own_file() {
    // It used to share the main model's file name, so the .mpd referenced itself and export never finished.
    let dir = model_project(
        "submodel-name",
        "from emmet_dsl import Model, model\n\n\ndef unit(m):\n    m.add(\"3005\", \"Red\", at=(0, 0, 0))\n\n\n\
         @model\ndef tower():\n    \"\"\"Two copies of a submodel called tower.\"\"\"\n    m = Model()\n    \
         t = m.submodel(\"tower\", unit)\n    m.place(t, at=(0, 0, 0))\n    m.place(t, at=(2, 0, 0))\n    return m\n",
        &[],
    );
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let mpd = read(&dir, "build/model.mpd");
    let files: Vec<&str> = mpd.lines().filter(|l| l.starts_with("0 FILE ")).collect();
    assert_eq!(files, ["0 FILE tower.ldr", "0 FILE tower-2.ldr"]);
    let (v, code) = emmet_in(&dir, &["export", "--format", "ldr"]);
    assert_eq!((code, v["data"]["parts"].as_u64()), (0, Some(2)), "{v}");
}

#[test]
fn a_mirror_inside_a_repeat_mirrors_each_copy_across_its_own_plane() {
    // The plane used to stay where the first copy put it.
    let dir = model_project(
        "mirror-repeat",
        "from emmet_dsl import Model, model\n\n\n@model\ndef bays():\n    \"\"\"Three bays, each mirrored.\"\"\"\n    \
         m = Model()\n\n    def bay(i):\n        with m.mirror(\"x\", about=3):\n            \
         m.add(\"3005\", \"Red\", at=(0, 0, 0), id=f\"post/{i}\")\n\n    m.repeat(bay, 3, dx=10)\n    return m\n",
        &[],
    );
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let got = xs(&dir);
    for i in 0..3 {
        let x = 10.0 * i as f64;
        assert_eq!(got[&format!("post/{i}")], x);
        // Mirrored across x = 3 within its own bay: the 1-wide post at 0 lands at 5. The mirrored id is a sibling
        // (post/0-mirror-x), so a copy's prefix never takes it in.
        assert_eq!(got[&format!("post/{i}-mirror-x")], x + 5.0);
    }
}

#[test]
fn import_ldraw_reads_direct_colors_and_refuses_a_broken_line() {
    let model = "from emmet_dsl import Model, import_ldraw, model\n\n\n@model\ndef cart():\n    \"\"\"An import.\"\"\"\n    \
                 m = Model()\n    m.place(import_ldraw(\"wagon.mpd\"), at=(0, 0, 0))\n    return m\n";
    // A blank line before the first 0 FILE, and a part in a direct color, which LDraw writes in hex.
    let wagon = "\n0 FILE wagon.ldr\n1 0x2FF0000 0 -24 0 1 0 0 0 1 0 0 0 1 3001.dat\n0 NOFILE\n";
    let dir = model_project("import-colors", model, &[("wagon.mpd", wagon)]);
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(placements(&dir)[0]["color"], 0x2FF_0000);

    // A byte order mark before the first line (Windows editors save one) changes nothing, even when that line is a
    // part: it used to read as "\u{feff}1", not a part line, and the part was quietly left out.
    let bom = "\u{feff}1 0x2FF0000 0 -24 0 1 0 0 0 1 0 0 0 1 3001.dat\n";
    std::fs::write(dir.join("wagon.mpd"), bom).unwrap();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(placements(&dir).len(), 1, "{v}");
    assert_eq!(placements(&dir)[0]["color"], 0x2FF_0000);

    // A part line LDraw can't read is an error at its line, never a part quietly left out.
    std::fs::write(dir.join("wagon.mpd"), wagon.replace(" 3001.dat", "")).unwrap();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "MODEL_EXEC_FAILED");
    assert_eq!(
        (
            v["error"]["details"]["file"].as_str(),
            v["error"]["details"]["line"].as_u64()
        ),
        (Some("wagon.mpd"), Some(3))
    );
}

#[test]
fn a_part_helper_refuses_a_size_that_isnt_whole() {
    // Brick(2.5, 4) used to become Brick(2, 4) without a word.
    let dir = model_project(
        "helper-size",
        "from emmet_dsl import Brick, Model, model\n\n\n@model\ndef half():\n    \"\"\"A half-stud brick.\"\"\"\n    \
         m = Model()\n    m.add(Brick(5 / 2, 4), \"Red\", at=(0, 0, 0))\n    return m\n",
        &[],
    );
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(
        (v["error"]["code"].as_str(), v["error"]["details"]["line"].as_u64()),
        (Some("MODEL_EXEC_FAILED"), Some(8))
    );
}

#[test]
fn a_submodel_counts_toward_the_part_limit_once() {
    // A copy's parts used to count when it was built and again when it was placed: 10,100 parts read as 20,200.
    let dir = model_project(
        "submodel-count",
        "from emmet_dsl import Model, model\n\n\ndef field(m):\n    m.grid(\"3024\", \"Green\", at=(0, 0, 0), nx=100, ny=101)\n\n\n\
         @model\ndef big():\n    \"\"\"One big submodel, placed once.\"\"\"\n    m = Model()\n    \
         m.place(m.submodel(\"field\", field), at=(0, 0, 0))\n    return m\n",
        &[],
    );
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!((code, v["data"]["parts"].as_u64()), (0, Some(10_100)), "{v}");
}

#[test]
fn loose_parts_get_steps_a_layer_at_a_time() {
    // Three bricks added first but on top, then ten on the bottom, none in a step: the bottom layer goes first, split at
    // the step target of 8, and the leftovers share a step with the layer above.
    let dir = model_project(
        "layers",
        "from emmet_dsl import Model, model\n\n\n@model\ndef pile():\n    \"\"\"No steps of its own.\"\"\"\n    \
         m = Model()\n    for i in range(3):\n        m.add(\"3001\", \"Red\", at=(4 * i, 0, 3), id=f\"top/{i}\")\n    \
         for i in range(10):\n        m.add(\"3005\", \"Blue\", at=(i, 0, 0), id=f\"low/{i}\")\n    return m\n",
        &[],
    );
    let (v, code) = emmet_in(&dir, &["steps"]);
    assert_eq!(code, 0, "{v}");
    let steps: Vec<Vec<&str>> = v["data"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            s["part_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i.as_str().unwrap())
                .collect()
        })
        .collect();
    assert_eq!(steps.len(), 2, "{steps:?}");
    assert_eq!(steps[0], (0..8).map(|i| format!("low/{i}")).collect::<Vec<_>>());
    // Within a step, parts keep the order they were added in.
    assert_eq!(steps[1], ["top/0", "top/1", "top/2", "low/8", "low/9"]);
}

#[test]
fn part_helpers_pick_the_plain_current_part() {
    use emmet_core::{data::DataDir, index::Index};
    let index = Index::ensure(&DataDir::at(emmet_it::shared_library())).unwrap();
    let id = |kind: &str, dims: &[u32]| index.resolve_helper(kind, dims).unwrap().id;
    assert_eq!(id("brick", &[2, 4, 1]).as_deref(), Some("3001"));
    // Not "Brick 1 x 2 x 5" (2454b) for a Brick 1 x 2, and the 2-tall one when asked.
    assert_eq!(id("brick", &[1, 2, 1]).as_deref(), Some("3004"));
    assert_eq!(id("brick", &[1, 2, 2]).as_deref(), Some("3245c"));
    // 3040 moved to 3040b; the helper never picks the moved one.
    assert_eq!(id("slope", &[2, 1, 45]).as_deref(), Some("3040b"));
    // A tile "with Groove" counts as the plain tile.
    assert_eq!(id("tile", &[1, 2, 1]).as_deref(), Some("3069b"));
    // A size the library lacks: none, with the nearest sizes there are.
    let missing = index.resolve_helper("brick", &[3, 5, 1]).unwrap();
    assert_eq!(missing.id, None);
    assert!(
        !missing.near.is_empty() && missing.near[0].starts_with("Brick "),
        "{:?}",
        missing.near
    );
}
