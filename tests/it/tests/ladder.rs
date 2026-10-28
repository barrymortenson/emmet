// SPDX-License-Identifier: AGPL-3.0-or-later
//! The ladder's hand-written reference models (plan section 02): data/ladder/rung2_reference.py runs, validates with
//! no errors at its defaults and at each param's other values, and has at least 6 params that each change the model.

use emmet_it::{emmet, emmet_in, envelope, repo, shared_data, temp_dir};
use serde_json::{Value, json};

#[test]
fn rung2_reference() {
    let source = std::fs::read_to_string(repo().join("data/ladder/rung2_reference.py")).unwrap();
    assert!(
        source.lines().count() < 200,
        "the reference should read like a person's model, under 200 lines"
    );

    let dir = temp_dir("rung2").join("station");
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
    std::fs::write(dir.join("model.py"), &source).unwrap();

    let errors = |v: &Value| {
        v["data"]["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["severity"] == "error")
            .count()
    };
    let (v, code) = emmet_in(&dir, &["check"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], json!([]), "{v:#}");
    let defaults = std::fs::read_to_string(dir.join("build/model.mpd")).unwrap();

    let (v, code) = emmet_in(&dir, &["params"]);
    assert_eq!(code, 0, "{v}");
    let params = v["data"]["params"].as_array().unwrap().clone();
    assert!(params.len() >= 6, "{} params", params.len());

    // Each param at another value: the model still validates with no errors, and it's a different model.
    for p in &params {
        let name = p["name"].as_str().unwrap();
        let others: Vec<String> = match p["type"].as_str().unwrap() {
            "int" => vec![p["min"].to_string(), p["max"].to_string()],
            "bool" => vec![(!p["default"].as_bool().unwrap()).to_string()],
            "choice" => p["choices"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| **c != p["default"])
                .map(|c| c.as_str().unwrap().to_string())
                .collect(),
            "color" => vec!["White".into()],
            other => panic!("{name} is a {other} param; add it here"),
        };
        let default = p["default"].to_string();
        for value in others.into_iter().filter(|o| *o != default) {
            let set = format!("{name}={value}");
            let (v, code) = emmet_in(&dir, &["check", "--params", &set]);
            assert_eq!(code, 0, "{set}: {v}");
            assert_eq!(errors(&v), 0, "{set}: {v:#}");
            let mpd = std::fs::read_to_string(dir.join("build/model.mpd")).unwrap();
            assert_ne!(mpd, defaults, "{set} doesn't change the model");
        }
    }
}
