// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet check` (plan section 11) and `emmet bom` against the project fixtures: each broken-* project gets exactly its
//! findings, the cottage gets none at any slider setting, the model's allow and ground entries count, and
//! build/validation.json is reproducible. disconnected_submodel and unknown_part, which today's DSL can't produce, have
//! their goldens in validator_corpus.rs.

use std::path::Path;

use emmet_it::{
    emmet, emmet_in, emmet_in_data, envelope, project, project_for, redact, shared_data, shared_data_with_parts,
    temp_dir,
};
use serde_json::{Value, json};

fn check(dir: &Path, args: &[&str]) -> (Value, i32) {
    let mut all = vec!["check"];
    all.extend_from_slice(args);
    emmet_in(dir, &all)
}

/// The envelope with the project's path and the build's inputs hash (it changes with every Emmet version) masked.
fn snapshot(dir: &Path, v: Value) -> Value {
    let mut v = redact(v, &[(dir, "<project>")]);
    if let Some(h) = v.pointer_mut("/data/inputs_hash").filter(|h| h.is_string()) {
        *h = json!("[inputs_hash]");
    }
    v
}

#[test]
fn each_broken_project_gets_exactly_its_findings() {
    for name in [
        "broken-collision",
        "broken-floating",
        "broken-step-order",
        "broken-step-size",
        "broken-weak-connection",
        "broken-off-grid",
        "broken-obsolete-part",
        "broken-alias-part",
    ] {
        let dir = project(name);
        let (v, code) = check(&dir, &[]);
        assert_eq!(code, 0, "findings aren't errors without --strict: {v}");
        insta::assert_json_snapshot!(name, snapshot(&dir, v));
        let written: Value =
            serde_json::from_slice(&std::fs::read(dir.join("build/validation.json")).unwrap()).unwrap();
        let (again, _) = check(&dir, &[]);
        assert_eq!(
            written, again["data"],
            "{name}: validation.json is the result, and the same twice"
        );
    }
}

#[test]
fn strict_exits_1_on_errors_only() {
    let (v, code) = check(&project("broken-collision"), &["--strict"]);
    assert_eq!((code, v["ok"].clone()), (1, json!(true)), "{v}");
    let (v, code) = check(&project("broken-obsolete-part"), &["--strict"]);
    assert_eq!(code, 0, "warnings don't fail --strict: {v}");
    let (v, code) = check(&project("broken-collision"), &["--rule", "floating", "--strict"]);
    assert_eq!(code, 0, "--rule checks only that rule: {v}");
    let (v, code) = check(&project("broken-collision"), &["--rule", "wobbly"]);
    assert_eq!((code, v["error"]["code"].clone()), (3, json!("PARAM_INVALID")), "{v}");
}

#[test]
fn check_runs_a_stale_model_first() {
    let dir = project("broken-floating");
    assert!(!dir.join("build").exists());
    let (v, _) = check(&dir, &[]);
    assert_eq!(v["data"]["counts"], json!({ "floating": 1 }), "{v}");
    // Lower the floating brick onto the first: the next check sees the edit without a run in between.
    let model = dir.join("model.py");
    let text = std::fs::read_to_string(&model).unwrap();
    std::fs::write(
        &model,
        text.replace("at=(0, 0, 5)", "at=(0, 0, 4)")
            .replace("at=(0, 0, 8)", "at=(0, 0, 7)"),
    )
    .unwrap();
    let (v, _) = check(&dir, &[]);
    assert_eq!(v["data"]["findings"], json!([]), "{v}");
    assert_eq!(v["data"]["clean"], true);
}

#[test]
fn an_allowed_finding_moves_to_allowed() {
    let dir = project("broken-floating");
    let toml = dir.join("emmet.toml");
    let mut text = std::fs::read_to_string(&toml).unwrap();
    text.push_str(
        "\n[[check.allow]]\nrule = \"floating\"\nparts = [\"tower/2\", \"tower/cap\"]\nreason = \"held by hand\"\n",
    );
    std::fs::write(&toml, text).unwrap();
    let (v, _) = check(&dir, &[]);
    assert_eq!(v["data"]["findings"], json!([]), "{v}");
    assert_eq!(v["data"]["allowed"][0]["reason"], "held by hand");
    assert_eq!(v["data"]["clean"], true);
}

/// `m.allow` and `m.ground` (DSL v1, 3B) reach the validator through build/placements.json: an allowed finding moves to
/// `allowed`, and a part named as ground stands.
#[test]
fn the_models_allow_and_ground_count() {
    let dir = project("broken-floating");
    let (v, _) = check(&dir, &[]);
    assert_eq!(v["data"]["counts"], json!({ "floating": 1 }), "{v}");
    // What 3B's DSL writes for m.allow("floating", ["tower/2", "tower/cap"], "…").
    let path = dir.join("build/placements.json");
    let mut doc: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    doc["allow"] =
        json!([{ "rule": "floating", "parts": ["tower/2", "tower/cap"], "reason": "a sign held up by hand" }]);
    std::fs::write(&path, serde_json::to_string(&doc).unwrap()).unwrap();
    let (v, _) = check(&dir, &[]);
    assert_eq!(v["data"]["findings"], json!([]), "{v}");
    assert_eq!(v["data"]["allowed"][0]["reason"], "a sign held up by hand");
    // And m.ground("tower/2"): the group stands.
    doc.as_object_mut().unwrap().remove("allow");
    doc["ground"] = json!(["tower/2"]);
    std::fs::write(&path, serde_json::to_string(&doc).unwrap()).unwrap();
    let (v, _) = check(&dir, &[]);
    assert_eq!(v["data"]["findings"], json!([]), "{v}");

    // The group moved off to the side, a plate off the table: with no baseplate and no ground parts it stands on the
    // table beside the tower. A ground id that names no part doesn't change that; a real one does.
    doc.as_object_mut().unwrap().remove("ground");
    for row in doc["parts"].as_array_mut().unwrap() {
        if ["tower/2", "tower/cap"].contains(&row["id"].as_str().unwrap()) {
            let m = row["matrix"].as_array_mut().unwrap();
            m[12] = json!(m[12].as_f64().unwrap() + 200.0);
            m[13] = json!(m[13].as_f64().unwrap() + 32.0);
        }
    }
    for (ground, floating) in [
        (json!([]), false),
        (json!(["no-such-part"]), false),
        (json!(["base"]), true),
    ] {
        doc["ground"] = ground.clone();
        std::fs::write(&path, serde_json::to_string(&doc).unwrap()).unwrap();
        let (v, _) = check(&dir, &[]);
        let counts = if floating { json!({ "floating": 1 }) } else { json!({}) };
        assert_eq!(v["data"]["counts"], counts, "ground {ground}: {v}");
    }
}

/// emmet.toml's `[build] age` sets step_size's target: broken-step-size's 10-part step is within 18+'s 12 and twice
/// 4+'s 4.
#[test]
fn the_age_setting_sets_the_step_target() {
    let dir = project("broken-step-size");
    let toml = dir.join("emmet.toml");
    let base = std::fs::read_to_string(&toml).unwrap();
    for (age, message) in [
        ("18+", None),
        ("4+", Some("Step 2 adds 10 parts: the build's target is 4 a step.")),
    ] {
        std::fs::write(&toml, format!("{base}\n[build]\nage = \"{age}\"\n")).unwrap();
        let (v, _) = check(&dir, &[]);
        let messages: Vec<&str> = v["data"]["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["message"].as_str().unwrap())
            .collect();
        assert_eq!(messages, message.into_iter().collect::<Vec<_>>(), "age {age}: {v}");
    }
}

/// The rung 1 reference: the cottage template validates clean at its defaults and at the corners of its sliders, with
/// no allow entries (plan section 21, 1D's What good looks like).
#[test]
fn the_cottage_validates_clean_at_every_corner() {
    let dir = temp_dir("check-cottage").join("cottage");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .arg("new")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        !std::fs::read_to_string(dir.join("emmet.toml"))
            .unwrap()
            .contains("[check")
    );
    // Each corner is checked from the defaults with --params, so no setting carries over to the next one.
    let corners: &[&[&str]] = &[
        &[],
        &["width=14", "depth=10"],
        &["width=20", "depth=12"],
        &["width=14", "depth=12", "chimney=false"],
        &["width=20", "depth=10", "season=winter"],
        // The roof lifted off, heavily weathered (old stones never over or beside each other or an opening).
        &["roof=false", "weathering=30", "seed=7"],
        &["season=winter", "weathering=15", "seed=3"],
        // Plaster and roof in other colors, which every part they color must have been made in.
        &["wall_color=Bright Light Orange", "roof_color=Dark Bluish Gray"],
        // The house alone on the table.
        &["baseplate=false"],
        &["width=18", "depth=12", "roof=false", "chimney=false", "season=winter"],
        // Each roof style and wall finish, the wild garden, and the lights on.
        &["roof_style=slate"],
        &["roof_style=shingles"],
        &["roof_style=thatch", "season=winter"],
        // A span that leaves 2 studs over finishes the thatch ridge with curved-top bricks.
        &["roof_style=thatch", "width=14", "chimney=false"],
        &["wall_finish=brick"],
        &["wall_finish=timber", "weathering=30", "seed=5"],
        &["wall_finish=siding", "roof=false"],
        &["garden=wild", "lights=true"],
        // Everything at once, at the widest: the most parts it builds (1,129).
        &[
            "width=20",
            "depth=12",
            "roof_style=thatch",
            "wall_finish=timber",
            "garden=wild",
            "lights=true",
            "season=winter",
            "weathering=30",
        ],
    ];
    for set in corners {
        let mut args = vec!["--strict"];
        if !set.is_empty() {
            args.push("--params");
            args.extend_from_slice(set);
        }
        let (v, code) = check(&dir, &args);
        assert_eq!(code, 0, "{set:?}: {v}");
        assert_eq!(v["data"]["findings"], json!([]), "{set:?}: {v}");
    }
}

/// `--params` (MCP validate's `params`) checks other slider values in one call and leaves params.json as it was: the
/// first live eval's agent wrote params.json from the shell to do this, and left it changed.
#[test]
fn check_with_params_leaves_params_json() {
    let dir = temp_dir("check-params").join("cottage");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .arg("new")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let before = std::fs::read_to_string(dir.join("params.json")).unwrap();
    let (v, code) = check(&dir, &["--strict", "--params", "width=20", "depth=12", "chimney=false"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], json!([]));
    let built: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("build/placements.json")).unwrap()).unwrap();
    assert!(
        built["parts"].as_array().unwrap().len() > 903,
        "the build is the 20 x 12 one, bigger than the defaults' 903 parts"
    );
    assert_eq!(std::fs::read_to_string(dir.join("params.json")).unwrap(), before);
}

/// `emmet run && emmet check` on a clean build/ reproduces validation.json byte for byte (plan section 07).
#[test]
fn validation_json_is_reproducible() {
    let dir = project("broken-collision");
    check(&dir, &[]);
    let first = std::fs::read(dir.join("build/validation.json")).unwrap();
    std::fs::remove_dir_all(dir.join("build")).unwrap();
    emmet_in(&dir, &["run"]);
    check(&dir, &[]);
    assert_eq!(first, std::fs::read(dir.join("build/validation.json")).unwrap());
}

#[test]
fn bom_lists_part_color_quantity() {
    let dir = project("broken-floating");
    let (v, code) = emmet_in(&dir, &["bom"]);
    assert_eq!(code, 0, "{v}");
    insta::assert_json_snapshot!(snapshot(&dir, v));
    assert!(dir.join("build/bom.json").is_file());
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["bom", "--format", "csv", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let csv = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        csv.lines().take(2).collect::<Vec<_>>(),
        [
            "part,name,color,color_name,qty,rebrickable_id,rebrickable_name,availability,sets,years",
            "3001,Brick 2 x 4,4,Red,2,,,unknown,,"
        ],
        "no pack: no credit line, and availability unknown"
    );
    let (v, _) = envelope(
        &emmet()
            .arg("--json")
            .arg("--data-dir")
            .arg(shared_data())
            .args(["bom", "--project"])
            .arg(temp_dir("bom-none"))
            .output()
            .unwrap(),
    );
    assert_eq!(v["error"]["code"], "PROJECT_NOT_FOUND");
}

/// part_unavailable and color_unavailable read the parts data pack: with it, broken-color-unavailable's two Violet
/// bricks are one finding naming both, with the closest colors that brick was made in; without it, availability is
/// unknown and there's no finding (unknown is never unavailable).
#[test]
fn broken_color_unavailable_needs_the_pack() {
    let data = shared_data_with_parts();
    let dir = project_for("broken-color-unavailable", data);
    let (v, code) = emmet_in_data(data, &dir, &["check"]);
    assert_eq!(code, 0, "{v}");
    insta::assert_json_snapshot!("broken-color-unavailable", snapshot(&dir, v));

    let dir = project("broken-color-unavailable");
    let (v, _) = check(&dir, &[]);
    assert_eq!(v["data"]["findings"], json!([]), "{v}");
}
