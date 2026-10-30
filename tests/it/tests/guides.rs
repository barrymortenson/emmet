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
/// is set; a scale or style in the user's own words (a scale's name given as the style, say) gets a note and the list;
/// no settings get the list and what to do.
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
    // A blank scale is no scale; a style that names a scale isn't a style, so it doesn't bring a second scale's guide.
    let (names, v) = with("[build]\nscale = \"  \"\nstyle = \"minifig\"\n");
    assert_eq!(names, "figure-scale");
    assert!(v["data"]["note"].as_str().unwrap().contains("\"minifig\""), "{v:#}");
    let (names, v) = with("[build]\nscale = \"Dollhouse\"\nstyle = \"Blocky\"\n");
    assert_eq!(names, "blocky");
    assert!(
        v["data"]["note"].as_str().unwrap().contains("scale \"Dollhouse\""),
        "{v:#}"
    );
    // A blank name is no name.
    let (v, code) = emmet_in(&dir, &["guide", " "]);
    assert_eq!(code, 0, "{v:#}");
    assert_eq!(v["data"]["guides"][0]["name"], "blocky", "{v:#}");
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

/// A project whose emmet.toml can't be read is an error, never a quiet "no scale or style is set": the agent reads the
/// guide before its first draft and would build at the wrong scale.
#[test]
fn a_project_that_cant_be_read_is_an_error() {
    let dir = temp_dir("guide-broken").join("p");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let toml = std::fs::read_to_string(dir.join("emmet.toml")).unwrap();
    std::fs::write(dir.join("emmet.toml"), format!("{toml}\n[build]\nscale = [oops\n")).unwrap();
    let (v, code) = emmet_in(&dir, &["guide"]);
    assert_ne!(code, 0, "{v:#}");
    assert_eq!(v["ok"], false, "{v:#}");
    // A folder named with --project that isn't a project is an error too.
    let (v, code) = emmet_in(&temp_dir("guide-nowhere"), &["guide"]);
    assert_ne!(code, 0, "{v:#}");
    assert_eq!(v["error"]["code"], "PROJECT_NOT_FOUND", "{v:#}");
    // A name still answers, whatever the project.
    let (v, code) = emmet_in(&dir, &["guide", "micro"]);
    assert_eq!(code, 0, "{v:#}");
}

/// Each guide file is a `# Title` line, an `Also:` line, then its text, and no name answers to two guides: `find`
/// takes the first match, so a clash would send one guide's readers to the other without a word.
#[test]
fn every_guide_file_has_a_title_and_names_no_other_guide_answers_to() {
    let key = |s: &str| -> String {
        s.chars()
            .filter(char::is_ascii_alphanumeric)
            .map(|c| c.to_ascii_lowercase())
            .collect()
    };
    let dir = repo().join("crates/emmet-cli/doctrine/guides");
    let mut owner: std::collections::BTreeMap<String, String> = Default::default();
    let mut files = 0;
    for e in std::fs::read_dir(&dir).unwrap() {
        let path = e.unwrap().path();
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&path).unwrap();
        let mut lines = text.lines();
        assert!(
            lines.next().unwrap_or("").starts_with("# "),
            "{stem}.md: line 1 isn't '# Title'"
        );
        let also = lines
            .next()
            .unwrap_or("")
            .strip_prefix("Also: ")
            .unwrap_or_else(|| panic!("{stem}.md: line 2 isn't 'Also: ...'"));
        assert!(
            !lines.any(|l| l.starts_with("Also: ")),
            "{stem}.md: a second 'Also:' line"
        );
        for name in std::iter::once(stem.as_str()).chain(also.split(',').map(str::trim)) {
            if let Some(other) = owner.insert(key(name), stem.clone()) {
                assert_eq!(other, stem, "'{name}' answers to both {other} and {stem}");
            }
        }
        files += 1;
    }
    assert_eq!(files, 19);
}
