// SPDX-License-Identifier: AGPL-3.0-or-later
//! The build guides (session 3D, `crates/emmet-cli/doctrine/guides/`): every scale and style the app's Options offer
//! has one, each answers to its other names, `emmet guide` with no name gives the ones for the project's `[build]`,
//! and every technique a guide names exists.

use std::collections::BTreeSet;

use emmet_core::python::TECHNIQUES;
use emmet_it::{emmet, emmet_in, emmet_json, repo, shared_data, temp_dir, text};
use serde_json::Value;

/// The choice ids of one build setting in the Options tab (`apps/desktop/src/Options.tsx`'s BRIEF).
fn options(setting: &str) -> Vec<String> {
    let src = std::fs::read_to_string(repo().join("apps/desktop/src/Options.tsx")).unwrap();
    let start = src
        .find(&format!("  {setting}: {{"))
        .unwrap_or_else(|| panic!("Options.tsx has no {setting}"));
    let block = &src[start..];
    let block = &block[..block.find("\n  },").expect("the setting's block ends")];
    block
        .split("{ id: \"")
        .skip(1)
        .map(|s| s[..s.find('"').unwrap()].to_string())
        .collect()
}

fn guide(name: &str) -> Value {
    let (v, code) = emmet_json(&["guide", name]);
    assert_eq!(code, 0, "no guide for {name:?}: {v:#}");
    v["data"]["guides"][0].clone()
}

#[test]
fn every_scale_and_style_the_options_offer_has_a_guide() {
    for (setting, kind) in [("scale", "scale"), ("style", "style")] {
        let ids = options(setting);
        assert!(ids.len() >= 6, "{setting}: {ids:?}");
        for id in ids {
            let g = guide(&id);
            assert_eq!(g["kind"], kind, "{id}: {g:#}");
            assert!(g["text"].as_str().unwrap().len() > 800, "{id}'s guide is too thin");
        }
    }
}

#[test]
fn a_guide_answers_to_its_other_names() {
    for (asked, name) in [
        ("Figure scale", "figure-scale"),
        ("minifig", "figure-scale"),
        ("Car 1:8", "car-1-8"),
        ("voxel", "blocky"),
        ("sci-fi", "space"),
        ("garden", "landscaping"),
        ("furniture", "interiors"),
        ("trucks", "vehicles"),
    ] {
        assert_eq!(guide(asked)["name"], name, "{asked}");
    }
    let (v, code) = emmet_json(&["guide", "steampunkk"]);
    assert_ne!(code, 0);
    assert_eq!(v["error"]["code"], "PARAM_INVALID");
}

/// With no name, `guide` reads the project's `[build]`: its scale's and style's guides, figure scale's when no scale
/// is set; a style in the user's own words gets a note and the list; no settings get the list and what to do.
#[test]
fn with_no_name_it_gives_the_guides_for_the_build_settings() {
    let dir = temp_dir("guide-build").join("p");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let toml = std::fs::read_to_string(dir.join("emmet.toml")).unwrap();
    let with = |build: &str| {
        std::fs::write(dir.join("emmet.toml"), format!("{toml}\n{build}")).unwrap();
        let (v, code) = emmet_in(&dir, &["guide"]);
        assert_eq!(code, 0, "{v:#}");
        let names: Vec<&str> = v["data"]["guides"]
            .as_array()
            .map(|a| a.iter().map(|g| g["name"].as_str().unwrap()).collect())
            .unwrap_or_default();
        (names.join(" "), v)
    };

    let (names, v) = with("[build]\nscale = \"Micropolis\"\nstyle = \"Town\"\n");
    assert_eq!(names, "micropolis town");
    assert!(
        v["data"].get("list").is_none() && v["data"].get("note").is_none(),
        "{v:#}"
    );

    let (names, v) = with("[build]\nscale = \"Figure scale\"\nstyle = \"Steampunk harbor\"\n");
    assert_eq!(names, "figure-scale");
    assert!(
        v["data"]["note"].as_str().unwrap().contains("Steampunk harbor"),
        "{v:#}"
    );
    assert!(!v["data"]["list"].as_array().unwrap().is_empty());

    // With no scale set, it's figure scale (the doctrine's default), so that guide comes too.
    let (names, _) = with("[build]\nstyle = \"Blocky\"\n");
    assert_eq!(names, "figure-scale blocky");
    let (names, v) = with("");
    assert_eq!(names, "figure-scale");
    assert!(
        v["data"]["note"].as_str().unwrap().contains("No scale or style is set"),
        "{v:#}"
    );
}

/// A technique name in backticks in a guide is a technique the library has, so a guide never sends the agent to one
/// that doesn't exist.
#[test]
fn every_technique_a_guide_names_exists() {
    let (v, code) = emmet_json(&["guide"]);
    assert_eq!(code, 0, "{v:#}");
    let have: BTreeSet<&str> = TECHNIQUES.iter().map(|(n, _)| *n).collect();
    for g in v["data"]["list"].as_array().unwrap() {
        let name = g["name"].as_str().unwrap();
        let text = guide(name)["text"].as_str().unwrap().to_string();
        let missing: Vec<&str> = text
            .split('`')
            .skip(1)
            .step_by(2)
            .filter(|t| {
                !t.is_empty()
                    && t.chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            })
            .filter(|t| t.chars().next().unwrap().is_ascii_lowercase() && !have.contains(t))
            .collect();
        assert!(
            missing.is_empty(),
            "the {name} guide names techniques that don't exist: {missing:?}"
        );
    }
}
