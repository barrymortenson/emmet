// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet new` → `emmet run` on the cottage template: the goldens, the reproducibility check (plan section 07),
//! and the warm-run budget.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use emmet_core::ldraw;
use emmet_it::{emmet, emmet_in, envelope, shared_data, temp_dir, text};
use serde_json::Value;

/// `emmet new <tmp>/demo` with the shared data dir.
fn new_demo(name: &str) -> std::path::PathBuf {
    let dir = temp_dir(name).join("demo");
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
    dir
}

/// Every file in build/ but run.log, with manifest.json's run block left out: what a rebuild must reproduce.
fn build_snapshot(project: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(project.join("build")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name == "run.log" {
            continue;
        }
        let mut bytes = std::fs::read(&path).unwrap();
        if name == "manifest.json" {
            let mut v: Value = serde_json::from_slice(&bytes).unwrap();
            v.as_object_mut().unwrap().remove("run");
            bytes = serde_json::to_vec(&v).unwrap();
        }
        out.insert(name, bytes);
    }
    out
}

#[test]
fn new_then_run_matches_the_golden() {
    let demo = new_demo("golden");
    for f in [
        "emmet.toml",
        "model.py",
        "params.json",
        "CARD.md",
        "AGENTS.md",
        "CLAUDE.md",
        ".gitignore",
        ".emmet/trust",
    ] {
        assert!(demo.join(f).is_file(), "emmet new wrote {f}");
    }
    assert_eq!(
        std::fs::read_to_string(demo.join("CLAUDE.md")).unwrap(),
        std::fs::read_to_string(demo.join("AGENTS.md")).unwrap()
    );

    let (v, code) = emmet_in(&demo, &["run"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["parts"], 165);
    assert_eq!(v["data"]["steps"], 39);
    assert_eq!(
        v["data"]["bbox"],
        serde_json::json!({ "min": [0.0, 0.0, 0.0], "max": [32.0, 32.0, 36.5] })
    );
    assert_eq!(v["meta"]["generation"], 1);
    assert_eq!(v["meta"]["inputs_hash"].as_str().unwrap().len(), 64);

    let mpd = std::fs::read_to_string(demo.join("build/model.mpd")).unwrap();
    insta::assert_snapshot!("cottage_mpd", mpd);
    let placements: Value =
        serde_json::from_slice(&std::fs::read(demo.join("build/placements.json")).unwrap()).unwrap();
    insta::assert_json_snapshot!("cottage_placements", placements);

    // placements.json says what the .mpd says, read back by the Rust LDraw layer.
    let ours = ldraw::placements::placements(&ldraw::parse(&mpd).unwrap());
    let theirs = placements["parts"].as_array().unwrap();
    assert_eq!(ours.len(), theirs.len());
    for (a, b) in ours.iter().zip(theirs) {
        assert_eq!(a.id, b["id"].as_str().unwrap());
        assert_eq!(a.part, b["part"].as_str().unwrap());
        assert_eq!(u64::from(a.color), b["color"].as_u64().unwrap());
        assert_eq!(u64::from(a.step), b["step"].as_u64().unwrap());
        assert_eq!(a.src, b["src"].as_str().unwrap());
        let m: Vec<f64> = b["matrix"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        assert_eq!(a.matrix.to_vec(), m, "{}", a.id);
    }
    let manifest: Value = serde_json::from_slice(&std::fs::read(demo.join("build/manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["inputs_hash"], v["meta"]["inputs_hash"]);
    assert_eq!(manifest["library"], "ldraw-mini");
    assert_eq!(manifest["run"]["generation"], 1);
}

#[test]
fn clean_rebuild_is_byte_identical() {
    let demo = new_demo("rebuild");
    let (v, code) = emmet_in(&demo, &["run"]);
    assert_eq!(code, 0, "{v}");
    let first = build_snapshot(&demo);
    assert!(
        first.contains_key("model.mpd") && first.contains_key("placements.json") && first.contains_key("manifest.json")
    );

    // A warm rerun changes nothing but the run block.
    let (v2, _) = emmet_in(&demo, &["run"]);
    assert_eq!(build_snapshot(&demo), first);
    assert_eq!(v2["meta"]["generation"], 2);
    assert_eq!(v2["meta"]["inputs_hash"], v["meta"]["inputs_hash"]);

    // So does a clean one, from no build/ at all (two passes: the first finds which parts the model uses).
    std::fs::remove_dir_all(demo.join("build")).unwrap();
    let (v3, code) = emmet_in(&demo, &["run"]);
    assert_eq!(code, 0, "{v3}");
    let again = build_snapshot(&demo);
    for (name, bytes) in &first {
        assert!(
            again.get(name) == Some(bytes),
            "build/{name} differs after a clean rebuild"
        );
    }
    assert_eq!(again.len(), first.len());
}

#[test]
fn warm_run_is_fast() {
    let demo = new_demo("warm");
    assert_eq!(emmet_in(&demo, &["run"]).1, 0);
    let mut best = f64::MAX;
    for _ in 0..5 {
        let started = Instant::now();
        let out = emmet()
            .arg("--data-dir")
            .arg(shared_data())
            .arg("run")
            .arg("--project")
            .arg(&demo)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        best = best.min(started.elapsed().as_secs_f64() * 1000.0);
    }
    println!("warm emmet run on the cottage: {best:.0} ms (best of 5)");
    // What good looks like: 400 ms or less.
    assert!(best <= 400.0, "warm run took {best:.0} ms");
}

#[test]
fn params_get_set_and_run_overrides() {
    let demo = new_demo("params");
    let (v, code) = emmet_in(&demo, &["params"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["model"], "cottage");
    let names: Vec<&str> = v["data"]["params"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "width",
            "depth",
            "wall_rows",
            "wall_color",
            "roof_color",
            "chimney",
            "roof",
            "weathering",
            "seed",
            "baseplate"
        ]
    );
    let width = &v["data"]["params"][0];
    assert_eq!(
        (width["type"].as_str(), width["min"].as_f64(), width["max"].as_f64()),
        (Some("int"), Some(6.0), Some(16.0))
    );
    assert_eq!(width["help"], "how many studs wide the house is");
    assert_eq!(v["data"]["params"][1]["step"], 2.0);
    assert_eq!(v["data"]["values"]["wall_color"], "Tan");

    let (v, code) = emmet_in(&demo, &["params", "--set", "width=12", "chimney=false"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["changed"], serde_json::json!(["chimney", "width"]));
    let file: Value = serde_json::from_str(&std::fs::read_to_string(demo.join("params.json")).unwrap()).unwrap();
    assert_eq!(file, serde_json::json!({ "chimney": false, "width": 12 }));

    // Setting a value back to its default drops it from params.json.
    emmet_in(&demo, &["params", "--set", "width=10"]);
    let file: Value = serde_json::from_str(&std::fs::read_to_string(demo.join("params.json")).unwrap()).unwrap();
    assert_eq!(file, serde_json::json!({ "chimney": false }));

    // --params applies to one run and doesn't touch params.json; the inputs hash changes with it.
    let (a, _) = emmet_in(&demo, &["run"]);
    let (b, code) = emmet_in(&demo, &["run", "--params", "width=14", "wall_color=\"Dark Tan\""]);
    assert_eq!(code, 0, "{b}");
    assert_ne!(a["meta"]["inputs_hash"], b["meta"]["inputs_hash"]);
    assert!(b["data"]["parts"].as_u64() > a["data"]["parts"].as_u64());
    let file: Value = serde_json::from_str(&std::fs::read_to_string(demo.join("params.json")).unwrap()).unwrap();
    assert_eq!(file, serde_json::json!({ "chimney": false }));

    // An odd depth isn't on the slider's step.
    let (v, code) = emmet_in(&demo, &["params", "--set", "depth=7"]);
    assert_eq!(code, 3, "{v}");
    assert_eq!(v["error"]["code"], "PARAM_INVALID");
}

/// The character knobs: weathering changes the bricks, the same seed gives the same build byte for byte (build/ is
/// reproducible), another seed another arrangement, and roof=false takes the roof off.
#[test]
fn the_knobs_change_the_build_and_a_seed_reproduces_it() {
    let demo = new_demo("knobs");
    let placements = |params: &[&str]| {
        let mut args = vec!["run", "--params"];
        args.extend_from_slice(params);
        let (v, code) = emmet_in(&demo, &args);
        assert_eq!(code, 0, "{params:?}: {v}");
        (
            std::fs::read(demo.join("build/placements.json")).unwrap(),
            v["data"]["parts"].as_u64().unwrap(),
        )
    };
    let (plain, parts) = placements(&["weathering=0"]);
    let (old, _) = placements(&["weathering=30", "seed=7"]);
    assert_ne!(old, plain, "weathering changes no brick");
    assert_eq!(
        placements(&["weathering=30", "seed=7"]).0,
        old,
        "the same seed builds differently"
    );
    assert_ne!(
        placements(&["weathering=30", "seed=8"]).0,
        old,
        "another seed builds the same"
    );
    // The chimney is shorter without a roof too, so the count alone doesn't show the roof came off.
    let (bytes, roofless) = placements(&["roof=false"]);
    assert!(roofless < parts, "roof=false keeps {roofless} of {parts} parts");
    let built: Value = serde_json::from_slice(&bytes).unwrap();
    let roof: Vec<&str> = built["parts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["id"].as_str().filter(|id| id.starts_with("roof-")))
        .collect();
    assert!(roof.is_empty(), "roof=false keeps the roof: {roof:?}");
}

#[test]
fn every_parameter_combination_at_the_edges_builds() {
    let demo = new_demo("edges");
    for (w, d, rows) in [(6, 4, 6), (16, 12, 9), (7, 10, 7), (11, 6, 8)] {
        for chimney in ["true", "false"] {
            let args = [
                "run",
                "--params",
                &format!("width={w}"),
                &format!("depth={d}"),
                &format!("wall_rows={rows}"),
            ];
            let mut args = args.to_vec();
            let c = format!("chimney={chimney}");
            args.push(&c);
            let (v, code) = emmet_in(&demo, &args);
            assert_eq!(code, 0, "{w}x{d}x{rows} chimney={chimney}: {v}");
        }
    }
}

#[test]
fn the_dsl_spec_example_runs() {
    // docs/spec/dsl.md opens with a model; it has to work as written.
    let spec = std::fs::read_to_string(emmet_it::repo().join("docs/spec/dsl.md")).unwrap();
    let example = spec
        .split("```python\n")
        .nth(1)
        .and_then(|s| s.split("```").next())
        .expect("a python block");
    let dir = temp_dir("dsl-example").join("wall");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    std::fs::write(dir.join("model.py"), example).unwrap();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    // The baseplate, two courses of wall (one brick, then two staggered), and four lamps of four parts each.
    assert_eq!(
        (v["data"]["parts"].as_u64(), v["data"]["submodels"].as_u64()),
        (Some(20), Some(1))
    );
    let (v, code) = emmet_in(&dir, &["check"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
    let (v, code) = emmet_in(&dir, &["run", "--params", "length=12"]);
    assert_eq!(
        (code, v["error"]["code"].as_str()),
        (3, Some("PARAM_INVALID")),
        "12 isn't on the steps of 8: {v}"
    );
}
