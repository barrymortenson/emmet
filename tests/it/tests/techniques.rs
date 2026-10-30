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
    armchair_and_sofa,
    arrow_slit,
    ball_joint_arm,
    balustrade_parapet,
    banded_chimney,
    bar_counter,
    bartizan,
    baseplate_road,
    battlements,
    bay_window,
    bench,
    bermed_wall_reveal,
    blade_sign,
    bookcase,
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
    color_block_facade,
    control_console,
    corbelled_balcony,
    crank_winch,
    cross_display_stand,
    cross_gable,
    crows_nest,
    curved_booth,
    curved_dormer,
    curved_hood,
    curved_wall_end,
    dentil_band,
    dirt_path,
    display_case_counter,
    dock_pier,
    dome_window_hood,
    door_frame,
    door_hood,
    doorstep_and_mat,
    downpipe,
    drawbridge,
    dresser_shelf,
    eaves_fascia,
    engine_nozzle,
    exhaust_stack,
    exposed_stud_stonework,
    eyebrow_arch_hood,
    fence,
    figure_bed,
    flagstone_paving,
    floor_lamp_and_candle,
    floor_rug,
    flower_bed,
    footbridge,
    front_bumper,
    front_grille,
    gable_roof,
    gear_pair,
    grass_edge,
    greeble_strip,
    half_roof_hinge,
    half_timber_band,
    hanging_planter,
    hatch_on_hinge,
    headlights,
    hearth_fireplace,
    hillside,
    hinge_roof,
    hinged_tailgate,
    hip_roof,
    interior_arch_doorway,
    irregular_stone_wall,
    jettied_floor,
    jumper_offset,
    kitchen_counter,
    lamp_post,
    landing_leg,
    landing_zone,
    lattice_shopfront,
    lattice_skirt_foundation,
    lean_to_awning,
    lift_arm_linkage,
    lift_off_floor,
    lift_off_roof_seat,
    light_bar,
    loft_half_floor,
    machicolation,
    mansard_module,
    masonry_wall,
    mast_and_yard,
    mini_dormer,
    oars,
    overlap_tile_roof,
    panel_garage_door,
    panel_pilaster,
    pine_tree,
    plank_door,
    plank_floor,
    plaster_with_exposed_stones,
    plate_shingle_roof,
    platform_edge,
    pond,
    porch_posts,
    portcullis,
    portholes,
    pyramid_spire_tower,
    quoin_corner,
    radar_dish,
    rear_spoiler,
    ribbed_hip_roof,
    ridge_cap,
    river,
    rock_facet,
    rock_outcrop,
    rock_spire,
    roof_cresting,
    roof_dormer,
    roof_eaves_overhang_frame,
    roof_hatch,
    rotating_platform,
    rough_stone_ledge,
    round_corner_bay,
    round_tower,
    round_window,
    round_window_frame,
    rudder_hinge,
    rusticated_stone_pilaster,
    scalloped_awning,
    ship_hull,
    shop_window,
    shuttered_window,
    sidewalk,
    sideways_chimney_column,
    signboard_beam,
    slope_hood_small_window,
    snot_panel,
    snow_cap,
    solar_panel,
    spliced_chassis,
    stairs,
    string_course,
    studs_up_snow_roof,
    surf_line,
    table_and_chairs,
    tail_lights,
    tapered_tower_cap,
    technic_brace,
    technic_pin_hinge,
    technic_spine,
    thick_wall,
    tile_stripe_shingles,
    tiled_floor,
    tiled_roof_panel,
    tilting_hood,
    timber_frame_panels,
    torch_sconce,
    track_curve,
    track_straight_run,
    track_switch,
    trailer_hitch,
    train_carriage,
    tree,
    turned_post_porch,
    turret,
    twin_arched_window,
    two_story_color_split,
    two_tone_masonry,
    upright_piano,
    wainscot_split_wall,
    wall_banner,
    wall_lamp_bracket,
    wall_picture_and_clock,
    wall_shelf,
    wall_top_cornice,
    wall_top_tie,
    wall_vent,
    waterfall,
    wedge_wing,
    white_ashlar_wall,
    window_box,
    window_frame,
    window_hood,
    window_sill,
    windscreen,
    wing_plate_porch_roof,
    wood_stove,
    wooden_hoarding,
    woodpile,
    worm_drive,
);

/// Every technique is named, as a whole word (so pine_tree doesn't count for tree), by when to reach for it: in the
/// doctrine's (`emmet rules`) "Build with the library" section, or in a build guide (`emmet guide`, session 3D), so
/// agents use the library without being told to (session 3C's What good looks like). The doctrine keeps the common
/// ones; the guides name the rest by scale, style, and subject, so the doctrine every session reads stays short.
#[test]
fn the_doctrine_or_a_guide_names_every_technique() {
    let out = emmet().arg("rules").output().unwrap();
    let doctrine = String::from_utf8_lossy(&out.stdout);
    let start = doctrine
        .find("## Build with the library")
        .expect("the doctrine has a library section");
    let section = &doctrine[start + 3..];
    let mut text = section[..section.find("\n## ").unwrap_or(section.len())].to_string();
    let (list, code) = emmet_json(&["guide"]);
    assert_eq!(code, 0, "{list}");
    for g in list["data"]["list"].as_array().unwrap() {
        let (v, code) = emmet_json(&["guide", g["name"].as_str().unwrap()]);
        assert_eq!(code, 0, "{v}");
        text.push_str(v["data"]["guides"][0]["text"].as_str().unwrap());
    }
    let words: std::collections::HashSet<&str> =
        text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).collect();
    let missing: Vec<&str> = TECHNIQUES
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| !words.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "neither the library section of crates/emmet-cli/doctrine/EMMET.md nor a guide in \
         crates/emmet-cli/doctrine/guides/ names {missing:?}"
    );
}

/// Stands a technique's example on its lowest level, as a baseplate would. Loose on the table, a group of parts that
/// floats beside something passes as a separate toy put there, which hid floating lenses in headlights and a floating
/// panel in bracket_side_panel until 3D. (Model._ids is the DSL's own record of what was placed.)
const GROUND: &str = "low = min(p.box[0][2] for p in m._ids.values())\n    \
                      m.ground(*[i for i, p in m._ids.items() if p.box[0][2] <= low + 0.01])\n";

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
             {GROUND}    return m\n"
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
