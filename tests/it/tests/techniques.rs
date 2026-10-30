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
    a_frame_tent,
    angled_hull_panel,
    antenna_mast,
    armchair_and_sofa,
    arrow_slit,
    ball_joint_arm,
    balustrade_parapet,
    bamboo_grove,
    banded_chimney,
    bar_counter,
    bar_spire,
    bar_truss_brace,
    bartizan,
    baseplate_road,
    battlements,
    bay_window,
    beam_spine_floor,
    bench,
    bermed_wall_reveal,
    big_round_eye,
    blade_sign,
    block_beast,
    block_column,
    block_fence_post,
    block_tree,
    block_water_and_falls,
    blossom_tree,
    bolted_pillar,
    bookcase,
    bookshelf_block,
    bracket_side_panel,
    brick_built_arch,
    brick_campfire,
    brick_enemy,
    brick_letters,
    bush,
    buttress,
    cab_over_sleeper,
    cab_seat,
    cactus_column,
    cannon_port,
    canopy,
    canopy_hump,
    car_wheels,
    cargo_pallet,
    cave_mouth,
    chamfered_door_portal,
    checker_light_wall,
    chimney_stack,
    chunky_kart,
    clamshell_pod,
    clapboard_siding,
    cliff_wall,
    clipped_hedge,
    cockpit_canopy,
    color_block_facade,
    console_shell,
    contour_slab,
    contour_terraces,
    control_console,
    controller_slab,
    corbelled_balcony,
    countershade_belly,
    crank_winch,
    crop_field,
    cross_display_stand,
    cross_gable,
    crows_nest,
    crt_cabinet,
    cube_head_character,
    curb_rumble_strip,
    curtain_wall_glazing,
    curved_booth,
    curved_dormer,
    curved_garden_path,
    curved_hood,
    curved_slope_skin,
    curved_wall_end,
    curved_wedge_nose,
    cypress,
    dashboard_and_wheel,
    davit_lifeboat,
    dentil_band,
    dirt_path,
    display_case_counter,
    dock_pier,
    dome_from_quarters,
    dome_window_hood,
    door_frame,
    door_hood,
    doorstep_and_mat,
    double_arch_colonnade,
    downpipe,
    drawbridge,
    dresser_shelf,
    eaves_fascia,
    engine_bay_greebles,
    engine_nozzle,
    exhaust_stack,
    exposed_stud_stonework,
    eyebrow_arch_hood,
    feather_shingles,
    fence,
    fender_crown,
    figure_bed,
    fin_merlon_crenels,
    flagstone_paving,
    flared_exhaust,
    flat_jumper_roof,
    flip_bridge,
    float_on_clear,
    floor_band_bricks,
    floor_lamp_and_candle,
    floor_rug,
    floppy_ear,
    flower_bed,
    footbridge,
    fountain_court,
    four_wide_car,
    front_bumper,
    front_grille,
    front_hinged_hood,
    gable_roof,
    game_pipe,
    garden_boulder,
    garden_gate,
    gear_pair,
    glass_box_cab,
    glow_trim_edge,
    gnarled_tree,
    grass_block,
    grass_edge,
    greeble_strip,
    half_roof_hinge,
    half_timber_band,
    hang_on_side_panel,
    hangar_chevron_floor,
    hanging_planter,
    hatch_on_hinge,
    headlights,
    hearth_fireplace,
    hillside,
    hinge_roof,
    hinged_limb,
    hinged_tailgate,
    hip_roof,
    hover_stand,
    interior_arch_doorway,
    inverted_arch_wheel_well,
    irregular_stone_wall,
    item_block,
    item_launcher,
    jettied_floor,
    jumper_offset,
    kitchen_counter,
    lamp_post,
    landing_leg,
    landing_zone,
    large_display_wheel,
    lattice_shopfront,
    lattice_skirt_foundation,
    lattice_tower,
    lava_inset,
    lawn_with_plant_sockets,
    layered_conifer,
    layered_plate_cap,
    lean_to_awning,
    lift_arm_linkage,
    lift_off_floor,
    lift_off_roof_seat,
    lift_off_window_cab,
    liftarm_ladder_frame,
    light_bar,
    lily_and_reeds,
    lit_window_panel,
    loft_half_floor,
    machicolation,
    mansard_module,
    masonry_wall,
    mast_and_yard,
    micro_boat,
    micro_house_on_wall,
    micro_tower,
    micro_traffic,
    micro_tree_cluster,
    micropolis_block,
    mini_dormer,
    module_link_block,
    mossy_stone_block,
    mowed_pitch,
    muzzle_and_nose,
    oars,
    offroad_fender_flare,
    organic_ground_outline,
    overlap_tile_roof,
    paired_colonnade,
    palm_tree,
    panel_garage_door,
    panel_pilaster,
    pine_tree,
    pipe_column_cluster,
    pixel_mosaic_panel,
    plank_and_stone_wall,
    plank_deck,
    plank_door,
    plank_floor,
    plaster_with_exposed_stones,
    plate_shingle_roof,
    platform_edge,
    pleated_seat,
    polka_dot_cube,
    pond,
    porch_posts,
    portcullis,
    portholes,
    posable_neck_ball,
    potted_plant,
    pyramid_pedestal,
    pyramid_spire_tower,
    quarter_tile_headlights,
    quoin_corner,
    racing_scale_base,
    radar_dish,
    raised_module_base,
    raked_seating_panel,
    rear_spoiler,
    recessed_glow_panel,
    reed_stalks,
    removable_soft_top,
    ribbed_hip_roof,
    ribbed_side_panel,
    ridge_cap,
    ring_light_grille,
    river,
    rock_facet,
    rock_mosaic_panel,
    rock_outcrop,
    rock_spire,
    rolled_sill,
    roof_cresting,
    roof_dormer,
    roof_eaves_overhang_frame,
    roof_hatch,
    rotating_platform,
    rough_stone_ledge,
    round_bastion,
    round_corner_bay,
    round_deck_dome,
    round_funnel,
    round_tail_light_cluster,
    round_tower,
    round_window,
    round_window_frame,
    rounded_car_end,
    rounded_column,
    rudder_hinge,
    rusticated_stone_pilaster,
    scalloped_awning,
    ship_hull,
    shipping_container,
    shop_window,
    shuttered_window,
    side_stud_eyes,
    side_stud_lights,
    side_vent_grille,
    sidewalk,
    sideways_chimney_column,
    signboard_beam,
    six_wide_truck_cab,
    skyline_depth_rows,
    slope_hood_small_window,
    smooth_display_base,
    snot_core,
    snot_panel,
    snot_side_skirt,
    snow_cap,
    snow_drift_edge,
    solar_panel,
    speckled_block_wall,
    sphere_from_caps,
    spiral_dish_stack,
    spliced_chassis,
    stairs,
    start_gantry,
    statuette_group,
    steering_rack,
    stepped_block_terrain,
    stepped_plank_roof,
    stepped_tower_setbacks,
    stepping_stones,
    stick_torch,
    stone_lantern,
    street_cart,
    string_course,
    studs_up_snow_roof,
    sunken_pond_bank,
    surf_line,
    table_and_chairs,
    tail_lights,
    tapered_deck,
    tapered_hood_wedges,
    tapered_tower_cap,
    technic_brace,
    technic_brick_ring_chassis,
    technic_pin_hinge,
    technic_spine,
    teeth_row,
    thick_wall,
    tile_brick_texture,
    tile_stripe_shingles,
    tiled_floor,
    tiled_pond_water,
    tiled_roof_panel,
    tilting_hood,
    timber_frame_panels,
    torch_sconce,
    tower_crown,
    tower_setback_plinth,
    tower_side_fins,
    track_curve,
    track_straight_run,
    track_switch,
    trailer_hitch,
    train_carriage,
    tree,
    truck_spine_chassis,
    turned_post_porch,
    turret,
    twin_arched_window,
    two_story_color_split,
    two_tone_masonry,
    upright_piano,
    vent_fins,
    vintage_chassis,
    vintage_radiator,
    wainscot_split_wall,
    wall_banner,
    wall_lamp_bracket,
    wall_picture_and_clock,
    wall_shelf,
    wall_top_cornice,
    wall_top_tie,
    wall_vent,
    water_tile_field,
    waterfall,
    wedge_wing,
    white_ashlar_wall,
    window_band,
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
