// SPDX-License-Identifier: AGPL-3.0-or-later
//! Parts search on ldraw-mini (plan section 12): the golden queries, the 20 ms budget, and `emmet part` / `emmet
//! colors` through the CLI.

use std::time::Instant;

use emmet_core::data::DataDir;
use emmet_core::index::{Index, dims};
use emmet_it::{emmet, envelope, shared_library};

fn golden() -> Vec<(String, String)> {
    let text = std::fs::read_to_string(emmet_it::repo().join("tests/it/search-golden.toml")).unwrap();
    let doc: toml::Table = text.parse().unwrap();
    doc["queries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| {
            (
                q["q"].as_str().unwrap().to_string(),
                q["want"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn golden_queries() {
    let index = Index::open(&DataDir::at(shared_library())).unwrap();
    let queries = golden();
    assert!(queries.len() >= 30, "the golden set needs 30+ queries");
    let mut failures = Vec::new();
    for (q, want) in &queries {
        let hits = index.search(q, None, 3).unwrap();
        if !hits.iter().any(|h| &h.id == want) {
            let got: Vec<String> = hits.iter().map(|h| format!("{} ({})", h.id, h.name)).collect();
            failures.push(format!("'{q}': want {want}, got {got:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} queries miss:\n{}",
        failures.len(),
        queries.len(),
        failures.join("\n")
    );
}

#[test]
fn search_is_fast() {
    let index = Index::open(&DataDir::at(shared_library())).unwrap();
    let queries = golden();
    for (q, _) in &queries {
        index.search(q, None, 10).unwrap(); // warm the statement cache
    }
    let started = Instant::now();
    let rounds = 5;
    for _ in 0..rounds {
        for (q, _) in &queries {
            index.search(q, None, 10).unwrap();
        }
    }
    let mean_ms = started.elapsed().as_secs_f64() * 1000.0 / (rounds * queries.len()) as f64;
    println!("search: {mean_ms:.3} ms a query over {} queries", queries.len());
    assert!(mean_ms <= 20.0, "queries average {mean_ms:.2} ms");
}

#[test]
fn dimensions_parse() {
    let d = |q: &str| dims::parse(q).dims;
    assert_eq!(d("brick 2x4"), vec![2.0, 4.0]);
    assert_eq!(d("1 x 2 x 3 brick"), vec![1.0, 2.0, 3.0]);
    assert_eq!(d("2×2 round"), vec![2.0, 2.0]);
    assert_eq!(d("slope 1 x 1 x 2/3"), vec![1.0, 1.0, 2.0 / 3.0]);
    assert_eq!(d("window 1 x 2 x 2.667"), vec![1.0, 2.0, 2.667]);
    assert_eq!(d("2 by 4"), vec![2.0, 4.0]);
    assert_eq!(dims::parse("box 4").dims, Vec::<f64>::new(), "'box' isn't a dimension");
    assert_eq!(dims::parse("slope 45 2x2").rest, "slope 45");
}

#[test]
fn part_and_colors_through_the_cli() {
    let run = |args: &[&str]| {
        let out = emmet()
            .arg("--json")
            .arg("--data-dir")
            .arg(shared_library())
            .args(args)
            .output()
            .unwrap();
        envelope(&out)
    };
    let (v, code) = run(&["part", "3001.DAT"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["id"], "3001");
    assert_eq!(v["data"]["size"], serde_json::json!({ "x": 2.0, "y": 4.0, "z": 3.0 }));
    assert_eq!(
        v["data"]["body_box"],
        serde_json::json!([-40.0, 0.0, -20.0, 40.0, 24.0, 20.0])
    );

    let (v, _) = run(&["part", "3023"]);
    assert_eq!(v["data"]["flags"], serde_json::json!(["alias"]));
    assert_eq!(v["data"]["alias_of"], "3023b");

    let (v, code) = run(&["colors"]);
    assert_eq!(code, 0);
    let colors = v["data"]["colors"].as_array().unwrap();
    let lbg = colors.iter().find(|c| c["code"] == 71).unwrap();
    assert_eq!(lbg["name"], "Light Bluish Gray", "US spelling");
    assert!(
        lbg["aliases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a == "Light_Bluish_Grey")
            && lbg["aliases"].as_array().unwrap().iter().any(|a| a == "lbg")
    );
    assert_eq!(
        colors.iter().find(|c| c["code"] == 47).unwrap()["finish"],
        "transparent"
    );

    let (v, code) = run(&["search", "brick", "2x4", "--limit", "1"]);
    assert_eq!(code, 0);
    assert_eq!(v["data"]["parts"][0]["id"], "3001");
    assert_eq!(v["data"]["parts"].as_array().unwrap().len(), 1);
}

/// Corner shapes sink unless the query asks for a corner, and rise when it does (3D: "4x4 round" found three corner-round
/// parts before the round plate).
#[test]
fn corner_shapes_only_when_asked() {
    let index = Index::open(&DataDir::at(shared_library())).unwrap();
    let names = |q: &str| -> Vec<String> { index.search(q, None, 3).unwrap().into_iter().map(|h| h.name).collect() };
    let plain = names("tile 2x2 round");
    assert!(!plain[0].contains("Corner"), "tile 2x2 round: {plain:?}");
    let asked = names("tile 4x4 corner round");
    assert!(asked[0].contains("Corner"), "tile 4x4 corner round: {asked:?}");
}
