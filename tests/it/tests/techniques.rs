// SPDX-License-Identifier: AGPL-3.0-or-later
//! The technique library (plan section 08, data/techniques/): each technique's example builds on ldraw-mini and
//! validates with no findings, it has a render, and `emmet technique` gives its doc, an example that uses it through
//! `m.use`, its parts, and its provenance note.

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
    angled_hull_panel,
    antenna_mast,
    arrow_slit,
    ball_joint_arm,
    bartizan,
    baseplate_road,
    battlements,
    bench,
    bracket_side_panel,
    brick_built_arch,
    bush,
    buttress,
    cab_seat,
    cannon_port,
    canopy,
    cargo_pallet,
    cave_mouth,
    chimney_stack,
    clapboard_siding,
    cliff_wall,
    cockpit_canopy,
    control_console,
    crank_winch,
    crows_nest,
    curved_hood,
    dirt_path,
    dock_pier,
    door_frame,
    door_hood,
    downpipe,
    drawbridge,
    engine_nozzle,
    exhaust_stack,
    fence,
    flagstone_paving,
    flower_bed,
    footbridge,
    front_bumper,
    front_grille,
    gable_roof,
    gear_pair,
    grass_edge,
    greeble_strip,
    half_timber_band,
    hatch_on_hinge,
    headlights,
    hillside,
    hinge_roof,
    hinged_tailgate,
    hip_roof,
    jettied_floor,
    jumper_offset,
    lamp_post,
    landing_leg,
    landing_zone,
    lift_arm_linkage,
    lift_off_floor,
    lift_off_roof_seat,
    light_bar,
    machicolation,
    masonry_wall,
    mast_and_yard,
    oars,
    pine_tree,
    platform_edge,
    pond,
    porch_posts,
    portcullis,
    portholes,
    quoin_corner,
    radar_dish,
    rear_spoiler,
    ridge_cap,
    river,
    rock_facet,
    rock_outcrop,
    rock_spire,
    roof_dormer,
    rotating_platform,
    round_tower,
    rudder_hinge,
    ship_hull,
    shop_window,
    shuttered_window,
    sidewalk,
    snot_panel,
    snow_cap,
    solar_panel,
    spliced_chassis,
    stairs,
    surf_line,
    tail_lights,
    technic_brace,
    technic_pin_hinge,
    technic_spine,
    thick_wall,
    tiled_floor,
    tilting_hood,
    torch_sconce,
    track_curve,
    track_straight_run,
    track_switch,
    trailer_hitch,
    train_carriage,
    tree,
    turret,
    wall_banner,
    wall_top_tie,
    wall_vent,
    waterfall,
    wedge_wing,
    window_frame,
    window_hood,
    window_sill,
    windscreen,
    wooden_hoarding,
    woodpile,
    worm_drive,
);

/// The doctrine (`emmet rules`) names every technique in its "Build with the library" section, as a whole name (so
/// pine_tree doesn't count for tree), by when to reach for it, so agents use the library without being told to
/// (session 3C's What good looks like).
#[test]
fn the_doctrine_names_every_technique() {
    let out = emmet().arg("rules").output().unwrap();
    let doctrine = String::from_utf8_lossy(&out.stdout);
    let start = doctrine
        .find("## Build with the library")
        .expect("the doctrine has a library section");
    let section = &doctrine[start + 3..];
    let section = &section[..section.find("\n## ").unwrap_or(section.len())];
    let words: std::collections::HashSet<&str> = section
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .collect();
    let missing: Vec<&str> = TECHNIQUES
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| !words.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "crates/emmet-cli/doctrine/EMMET.md's library section doesn't name {missing:?}"
    );
}

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
    // Every technique says where its method came from (3C, D-48), and none is derived from a model's file.
    let provenance = t["provenance"].as_str().unwrap();
    assert!(!provenance.is_empty(), "{name} has no PROVENANCE note");
    let lower = provenance.to_lowercase();
    assert!(
        !lower.contains(".mpd") && !lower.contains(".ldr") && !lower.contains("omr_models"),
        "{name}'s provenance names a model file: {provenance}"
    );

    let png = std::fs::read(repo().join(format!("data/techniques/{name}.png")))
        .unwrap_or_else(|_| panic!("{name} has no render; run scripts/render-techniques {name}"));
    check_png(&png, 512);
}
