// SPDX-License-Identifier: AGPL-3.0-or-later
//! The technique library (plan section 08, data/techniques/): each technique's example builds on ldraw-mini and
//! validates with no findings, it has a render, and `emmet technique` gives its doc, an example that uses it through
//! `m.use`, and its parts.

use emmet_core::python::TECHNIQUES;
use emmet_it::{check_png, emmet, emmet_in, emmet_json, repo, shared_data, temp_dir, text};

/// One test per technique, and a test that the list here is every technique there is.
macro_rules! techniques {
    ($($name:ident),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                builds_clean(stringify!($name));
            }
        )*

        #[test]
        fn every_technique_has_a_test() {
            let mut tested = vec![$(stringify!($name)),*];
            tested.sort();
            let mut embedded: Vec<&str> = TECHNIQUES.iter().map(|(n, _)| *n).collect();
            embedded.sort();
            assert_eq!(tested, embedded, "add the new technique to techniques! here");
            let mut on_disk: Vec<String> = std::fs::read_dir(repo().join("data/techniques"))
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .filter_map(|n| n.strip_suffix(".py").map(String::from))
                .collect();
            on_disk.sort();
            assert_eq!(on_disk, embedded, "add the new technique to TECHNIQUES in crates/emmet-core/src/python.rs");
        }
    };
}

techniques!(
    bench,
    canopy,
    gable_roof,
    lamp_post,
    platform_edge,
    track_curve,
    track_straight_run,
    track_switch,
);

fn builds_clean(name: &str) {
    let dir = temp_dir(&format!("technique-{name}")).join(name);
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(
        dir.join("model.py"),
        format!(
            "from emmet_dsl import Model, model\nfrom emmet_dsl.techniques import {name} as technique\n\n\n@model\n\
             def example():\n    \"\"\"The technique on its own.\"\"\"\n    m = Model()\n    technique.example(m)\n    \
             return m\n"
        ),
    )
    .unwrap();
    let (v, code) = emmet_in(&dir, &["trust"]);
    assert_eq!(code, 0, "{v}");
    let (v, code) = emmet_in(&dir, &["check"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(
        v["data"]["findings"],
        serde_json::json!([]),
        "{name} isn't clean: {v:#}"
    );

    let (v, code) = emmet_json(&["technique", name]);
    assert_eq!(code, 0, "{v}");
    let t = &v["data"]["technique"];
    assert!(!t["doc"].as_str().unwrap().is_empty(), "{name} has no doc");
    // Whitespace aside, so a call the formatter wraps still counts.
    let snippet: String = t["snippet"].as_str().unwrap().split_whitespace().collect();
    assert!(
        snippet.contains(&format!("m.use(\"{name}\"")),
        "{name}'s example doesn't use it through m.use: {t:#}"
    );
    assert!(!t["parts"].as_array().unwrap().is_empty(), "{name} lists no PARTS");

    let png = std::fs::read(repo().join(format!("data/techniques/{name}.png")))
        .unwrap_or_else(|_| panic!("{name} has no render; run scripts/render-techniques {name}"));
    check_png(&png, 512);
}
