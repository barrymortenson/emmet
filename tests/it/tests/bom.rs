// SPDX-License-Identifier: AGPL-3.0-or-later
//! bom v1 (plan sections 09 and 12) with the parts data pack: Rebrickable ids and names, availability, the user's color
//! naming, and the three exports: CSV with the credit, Rebrickable's parts-list import (no credit line: it would break
//! the import), and a BrickLink wanted list that leaves out and names the lines BrickLink has no id for.

use std::path::Path;

use emmet_it::{emmet, emmet_in, emmet_in_data, project, project_for, redact, shared_data_with_parts, text};
use serde_json::{Value, json};

/// `emmet bom --format <format>` as text, with the pack.
fn export(dir: &Path, format: &str) -> String {
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data_with_parts())
        .args(["bom", "--format", format, "--project"])
        .arg(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn bom_with_the_pack() {
    let dir = project_for("bom-exports", shared_data_with_parts());
    let (v, code) = emmet_in_data(shared_data_with_parts(), &dir, &["bom"]);
    assert_eq!(code, 0, "{v}");
    insta::assert_json_snapshot!("bom-json", redact(v, &[(&dir, "<project>")]));
    insta::assert_snapshot!("bom-csv", export(&dir, "csv"));
    insta::assert_snapshot!("bom-rebrickable", export(&dir, "rebrickable"));
    insta::assert_snapshot!("bom-bricklink-xml", export(&dir, "bricklink-xml"));

    // The left-out lines are envelope warnings too.
    let (v, _) = emmet_in_data(shared_data_with_parts(), &dir, &["bom", "--format", "bricklink-xml"]);
    assert_eq!(v["warnings"][0]["code"], "BRICKLINK_UNMAPPED", "{v}");
    assert!(v["warnings"][0]["message"].as_str().unwrap().contains("3853"), "{v}");
}

/// Color names follow `--color-names`; LDraw's are the default. Unmapped lines stay unknown.
#[test]
fn color_names_follow_the_users_naming() {
    let data = shared_data_with_parts();
    let dir = project_for("bom-exports", data);
    let name = |v: &Value, part: &str| -> String {
        v["data"]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["part"] == part)
            .unwrap()["color_name"]
            .as_str()
            .unwrap()
            .to_string()
    };
    // LDraw 371 is Rebrickable's Warm Tan and BrickLink's Medium Tan; LDraw 368 Neon Yellow is Rebrickable's Vibrant Yellow.
    for (names, tan, yellow) in [
        ("ldraw", "Medium Tan", "Neon Yellow"),
        ("rebrickable", "Warm Tan", "Vibrant Yellow"),
        ("bricklink", "Medium Tan", "Neon Yellow"),
    ] {
        let (v, _) = emmet_in_data(data, &dir, &["bom", "--color-names", names]);
        assert_eq!(v["data"]["color_names"], names, "{v}");
        assert_eq!(
            (name(&v, "3023"), name(&v, "3958")),
            (tan.into(), yellow.into()),
            "{names}"
        );
    }
    // LDraw's own spelling is the default.
    let (v, _) = emmet_in_data(data, &dir, &["bom"]);
    assert_eq!(v["data"]["color_names"], "ldraw");
    let window = v["data"]["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["part"] == "3853")
        .unwrap()
        .clone();
    assert_eq!(
        window["availability"],
        json!({ "status": "unknown", "sets": null, "years": null })
    );
}

/// Without the pack the exports that need its ids say so; the CSV has no credit line.
#[test]
fn exports_without_the_pack() {
    let dir = project("bom-exports");
    let (v, code) = emmet_in(&dir, &["bom", "--format", "rebrickable"]);
    assert_eq!((code, v["error"]["code"].clone()), (4, json!("LIBRARY_MISSING")), "{v}");
    let (v, _) = emmet_in(&dir, &["bom"]);
    assert_eq!(v["data"]["parts_data"], Value::Null);
    let out = emmet()
        .arg("--data-dir")
        .arg(emmet_it::shared_data())
        .args(["bom", "--format", "csv", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    let csv = String::from_utf8(out.stdout).unwrap();
    assert!(csv.starts_with("part,name,color,"), "{csv}");
}
