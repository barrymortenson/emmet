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
    angled_display_stand,
    angled_hull_panel,
    antenna_mast,
    architecture_base,
    armchair_and_sofa,
    arrow_slit,
    axle_round_stack,
    ball_joint_arm,
    balustrade_parapet,
    bamboo_grove,
    banded_chimney,
    banded_round_turret,
    bar_counter,
    bar_spire,
    bar_truss_brace,
    bare_stud_greeble,
    bartizan,
    baseplate_road,
    battlements,
    bay_window,
    beam_spine_floor,
    belfry_louvers,
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
    flying_buttress_arch,
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
    lancet_window,
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
    pinnacle_stack,
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
    pointed_buttress_strip,
    polka_dot_cube,
    pond,
    porch_posts,
    portcullis,
    porthole_band,
    portholes,
    posable_neck_ball,
    potted_plant,
    pyramid_pedestal,
    pyramid_spire_tower,
    quarter_tile_headlights,
    quoin_corner,
    racing_scale_base,
    radar_dish,
    radial_spoke_saucer,
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
    rose_window_ring,
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
    rounded_pier_nose,
    rudder_hinge,
    rusticated_stone_pilaster,
    saucer_petal_panels,
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
    slender_column_capital,
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
    stave_cylinder,
    steep_nave_roof,
    steering_rack,
    stepped_block_terrain,
    stepped_curve_hull,
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
    triangle_tile_floor,
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

/// pixel_mosaic_panel holds together at odd sizes too: its two backing layers' joints once met along the last column
/// of a 5-wide picture, which floated (3D's review). Its example is 10 x 8, so these sizes are checked here.
#[test]
fn pixel_mosaic_panel_holds_at_odd_sizes() {
    let dir = temp_dir("technique-mosaic-odd").join("p");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let (v, code) = emmet_in(&dir, &["trust"]);
    assert_eq!(code, 0, "{v}");
    for (w, h) in [(5, 3), (7, 5), (3, 7), (9, 4)] {
        std::fs::write(
            dir.join("model.py"),
            format!(
                "from emmet_dsl import Model, model\n\n\n@model\ndef mosaic():\n    \"\"\"A checkered picture.\"\"\"\n    \
                 m = Model()\n    with m.step(\"Base\"):\n        m.add(\"3811\", \"Green\", at=(0, 0, 0))\n    \
                 rows = [\"\".join(\"RB\"[(r + c) % 2] for c in range({w})) for r in range({h})]\n    \
                 m.use(\"pixel_mosaic_panel\", x=4, y=4, z=0.5, rows=rows, palette={{\"R\": \"Red\", \"B\": \"Blue\"}})\n    \
                 return m\n"
            ),
        )
        .unwrap();
        let (v, code) = emmet_in(&dir, &["check", "--strict"]);
        assert_eq!(code, 0, "{w} x {h}: {v}");
        assert_eq!(v["data"]["findings"], serde_json::json!([]), "{w} x {h}: {v:#}");
    }
}

/// Stands a technique's example on its lowest level, as a baseplate would. Loose on the table, a group of parts that
/// floats beside something passes as a separate toy put there, which hid floating parts in headlights,
/// bracket_side_panel, and grass_edge until 3D. Only plain parts are grounded: grounding a submodel copy would ground
/// every part inside it, so an example whose lowest parts are all in submodels stays loose. (Model._ids is the DSL's
/// own record of what was placed.)
const GROUND: &str = "low = min(p.box[0][2] for p in m._ids.values())\n    \
                      m.ground(*[i for i, p in m._ids.items() if p.sub is None and p.box[0][2] <= low + 0.01])\n";

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

/// A model of `body` (the lines of a @model function after `m = Model()`) on a baseplate, checked: every finding.
fn findings_of(name: &str, body: &str) -> serde_json::Value {
    let (v, code) = check_of(name, body);
    assert_eq!(code, 0, "{v}");
    v["data"]["findings"].clone()
}

/// [`findings_of`]'s check, as its envelope and exit code.
fn check_of(name: &str, body: &str) -> (serde_json::Value, i32) {
    let dir = temp_dir(&format!("beside-door-{name}")).join("p");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let (v, code) = emmet_in(&dir, &["trust"]);
    assert_eq!(code, 0, "{v}");
    // Each line indented once into the function, keeping its own indentation (a `with` block's body).
    let body: String = body.lines().map(|l| format!("    {l}\n")).collect();
    std::fs::write(
        dir.join("model.py"),
        format!(
            "from emmet_dsl import Model, model\n\n\n@model\ndef beside_a_door():\n    \"\"\"{name} beside a door.\"\"\"\n    \
             m = Model()\n    with m.step(\"Base\"):\n        m.add(\"3811\", \"Green\", at=(0, 0, 0))\n{body}    \
             return m\n"
        ),
    )
    .unwrap();
    emmet_in(&dir, &["check"])
}

/// The techniques a figure walks under or past, each at its defaults against a door_frame's wall (3E, Done when 6):
/// no findings, figure_clearance included, so a canopy or a porch roof clears the door and no post stands in its way.
/// The door is cells 3 to 7 of a wall along y = 8, from z = 0.5 (its top at 18.5).
const DOOR: &str = "wall = m.use(\"door_frame\", x=0, y=8, z=0.5, length=10, rows=ROWS, offset=3, facing=\"front\")";

fn beside_a_door(name: &str, rows: u32, body: &str) {
    let v = findings_of(name, &format!("{}\n{body}", DOOR.replace("ROWS", &rows.to_string())));
    assert_eq!(v, serde_json::json!([]), "{name} beside a door: {v:#}");
}

#[test]
fn canopy_clears_a_door_beside_it() {
    beside_a_door("canopy", 7, "m.use(\"canopy\", x=0, y=2, z=0.5, length=10, depth=6)");
}

#[test]
fn porch_posts_clear_a_door_behind_them() {
    beside_a_door(
        "porch_posts",
        7,
        "m.use(\"porch_posts\", x=1, y=5, z=0.5, length=8, gap=(2, 4))",
    );
}

#[test]
fn turned_post_porch_clears_a_door_behind_it() {
    beside_a_door(
        "turned_post_porch",
        7,
        "m.use(\"turned_post_porch\", x=1, y=5, z=0.5, length=8, depth=3, gap=(2, 4))",
    );
}

#[test]
fn lean_to_awning_clears_a_door_under_it() {
    beside_a_door(
        "lean_to_awning",
        8,
        "m.use(\"lean_to_awning\", x=0, y=8, z=wall, length=10, ground=0.5, clears=wall.door_top)",
    );
}

#[test]
fn wing_plate_porch_roof_clears_a_door_under_it() {
    beside_a_door(
        "wing_plate_porch_roof",
        6,
        "m.use(\"wing_plate_porch_roof\", x=0, y=8, z=wall.door_top, ground=0.5, corbel=False)",
    );
}

#[test]
fn door_hood_clears_its_door() {
    beside_a_door("door_hood", 6, "m.use(\"door_hood\", x=3, y=8, z=wall.door_top)");
}

/// docs/techniques/audit.md has one row for every technique, and only for techniques (3E, Done when 5), each with a
/// verdict: fine, fixed, or limited.
#[test]
fn the_audit_has_a_row_for_every_technique() {
    let audit = std::fs::read_to_string(repo().join("docs/techniques/audit.md")).unwrap();
    let mut rows: Vec<(String, String)> = audit
        .lines()
        .filter(|l| l.starts_with("| `"))
        .map(|l| {
            let cells: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
            (cells[0].trim_matches('`').to_string(), cells[5].to_string())
        })
        .collect();
    rows.sort();
    let names: Vec<&str> = rows.iter().map(|(n, _)| n.as_str()).collect();
    let mut want: Vec<&str> = TECHNIQUES.iter().map(|(n, _)| *n).collect();
    want.sort();
    assert_eq!(
        names, want,
        "docs/techniques/audit.md needs exactly one row per technique"
    );
    for (name, verdict) in &rows {
        assert!(
            ["fine", "fixed", "limited"].iter().any(|v| verdict.starts_with(v)),
            "{name}'s verdict is {verdict:?}: fine, fixed, or limited"
        );
    }
}

#[test]
fn double_arch_colonnade_clears_a_door_behind_it() {
    // 3 studs out from the wall, the door's way out ends where the colonnade's plinth begins; a bay is in line with
    // the door, its columns either side.
    beside_a_door(
        "double_arch_colonnade",
        7,
        "m.use(\"double_arch_colonnade\", x=2, y=4, z=0.5)",
    );
}

#[test]
fn scalloped_awning_clears_a_door_under_it() {
    beside_a_door(
        "scalloped_awning",
        6,
        "m.use(\"scalloped_awning\", x=3, y=8, z=wall.door_top)",
    );
}

#[test]
fn corbelled_balcony_clears_a_door_under_it() {
    // The corbels hang 3 bricks under the slab: on a wall 3 bricks taller than the door, they end at its top.
    beside_a_door("corbelled_balcony", 9, "m.use(\"corbelled_balcony\", x=3, y=8, z=wall)");
}

#[test]
fn cross_gable_clears_a_door_under_it() {
    // The sign hangs 4 plates under the roof: on a wall 2 bricks taller than the door, it clears the door's top.
    beside_a_door(
        "cross_gable",
        8,
        "m.use(\"cross_gable\", x=0, y=8, z=wall, length=10, depth=6, at=3)",
    );
}

/// The seats (3E, building rule 3), each at its defaults on a baseplate: no findings, figure_clearance included; and a
/// 1 x 1 brick at `at`, where a seated figure's legs or shoulder go, is one figure_clearance finding that names the
/// seat.
fn seat_keeps_clear(name: &str, seat: &str, at: (u32, u32), why: &str) {
    assert_eq!(findings_of(name, seat), serde_json::json!([]), "{name} at its defaults");
    let brick = format!("m.add(\"3005\", \"White\", at=({}, {}, 0.5), id=\"post\")", at.0, at.1);
    let v = findings_of(name, &format!("{seat}\n{brick}"));
    let hits: Vec<_> = v
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["rule"] == "figure_clearance")
        .collect();
    assert_eq!(hits.len(), 1, "{name} with a brick in the way: {v:#}");
    assert!(hits[0]["message"].as_str().unwrap().contains(why), "{v:#}");
}

#[test]
fn an_armchair_keeps_clear_the_room_in_front_of_it() {
    // The armchair is x 2 to 6 and y 4 to 6, its seat x 3 to 5 facing -y: the brick stands where the legs go.
    seat_keeps_clear(
        "armchair_and_sofa",
        "m.use(\"armchair_and_sofa\", x=2, y=4, z=0.5)",
        (3, 3),
        "a figure sitting on the armchair",
    );
}

#[test]
fn a_chair_keeps_clear_a_stud_beside_it() {
    // The round set is x 2 to 6 and y 2 to 6; its first chair is at (2, 3) facing +x: the brick stands at its end.
    seat_keeps_clear(
        "table_and_chairs",
        "m.use(\"table_and_chairs\", x=2, y=2, z=0.5)",
        (2, 2),
        "a figure sitting on the chair",
    );
}

#[test]
fn bermed_wall_reveal_clears_a_door_in_its_wall() {
    // In the middle of the baseplate: the hill's cheeks reach 12 studs out from each end of the wall.
    let v = findings_of(
        "bermed_wall_reveal",
        "m.use(\"door_frame\", x=12, y=8, z=0.5, length=8, rows=6, offset=2)\n\
         m.use(\"bermed_wall_reveal\", x=12, y=8, z=0.5, length=8, wall=False)",
    );
    assert_eq!(v, serde_json::json!([]), "bermed_wall_reveal around a door: {v:#}");
}

/// A technique that makes a way through (3E, rule 2), at its defaults: alone, no findings; with a crate (a 1 x 1 brick)
/// at `crate_at` in its way through, one figure_clearance finding.
fn keeps_clear(name: &str, body: &str, crate_at: &str) {
    let v = findings_of(name, body);
    assert_eq!(v, serde_json::json!([]), "{name} alone: {v:#}");
    let v = findings_of(
        name,
        &format!("{body}\nwith m.step(\"Crate\"): m.add(\"3005\", \"Red\", at=({crate_at}))"),
    );
    let rules: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["rule"].as_str().unwrap())
        .collect();
    assert_eq!(
        rules,
        ["figure_clearance"],
        "{name} with a crate in its way through: {v:#}"
    );
}

#[test]
fn chamfered_door_portal_keeps_its_way_through_clear() {
    keeps_clear(
        "chamfered_door_portal",
        "m.use(\"chamfered_door_portal\", x=4, y=8, z=0.5)",
        "6, 6, 0.5",
    );
}

#[test]
fn plank_door_keeps_its_way_through_clear() {
    // The door stands 0.8 stud proud of the wall: the space starts in front of it.
    keeps_clear("plank_door", "m.use(\"plank_door\", x=4, y=8, z=0.5)", "5, 5, 0.5");
}

#[test]
fn interior_arch_doorway_keeps_its_way_through_clear() {
    keeps_clear(
        "interior_arch_doorway",
        "m.use(\"interior_arch_doorway\", x=2, y=8, z=0.5)",
        "5, 10, 0.5",
    );
}

#[test]
fn garden_gate_keeps_its_way_through_clear() {
    keeps_clear("garden_gate", "m.use(\"garden_gate\", x=2, y=8, z=0.5)", "7, 6, 0.5");
    // Open, the gate stands in the front space's last stud, which the space leaves out.
    let v = findings_of("garden_gate", "m.use(\"garden_gate\", x=2, y=8, z=0.5, open=True)");
    assert_eq!(v, serde_json::json!([]), "garden_gate open: {v:#}");
}

#[test]
fn panel_garage_door_keeps_its_way_through_clear() {
    // Behind it only: a car parks in front.
    keeps_clear(
        "panel_garage_door",
        "m.use(\"panel_garage_door\", x=4, y=8, z=0.5)",
        "6, 10, 0.5",
    );
}

#[test]
fn brick_built_arch_keeps_its_way_through_clear() {
    keeps_clear(
        "brick_built_arch",
        "m.use(\"brick_built_arch\", x=0, y=8, z=0.5, length=10)",
        "4, 6, 0.5",
    );
}

#[test]
fn portcullis_keeps_its_way_through_clear() {
    keeps_clear("portcullis", "m.use(\"portcullis\", x=0, y=8, z=0.5)", "4, 12, 0.5");
}

#[test]
fn cave_mouth_keeps_its_way_in_clear() {
    keeps_clear("cave_mouth", "m.use(\"cave_mouth\", x=0, y=8, z=0.5)", "3, 6, 0.5");
}

/// The figure_clearance findings' (rule, parts) in order, leaving the other rules out.
fn clearance_parts(v: &serde_json::Value) -> Vec<Vec<String>> {
    v.as_array()
        .unwrap()
        .iter()
        .filter(|f| f["rule"] == "figure_clearance")
        .map(|f| serde_json::from_value(f["parts"].clone()).unwrap())
        .collect()
}

/// A space declared inside m.repeat or m.mirror goes with each copy, as its parts do (the review of PR #49: every copy's
/// space stayed on the first, so a crate in the second door's way went unseen). The second door is 12 studs on, the
/// mirrored one across x = 12.
#[test]
fn a_space_goes_with_each_repeated_or_mirrored_copy() {
    let door = "m.use(\"door_frame\", x=0, y=8, z=0.5, length=10, rows=7, offset=3, title=f\"Shop {i + 1}\")";
    let v = findings_of(
        "repeat",
        &format!(
            "m.repeat(lambda i: {door}, 2, dx=12)\nwith m.step(\"Crate\"):\n    m.add(\"3005\", \"Brown\", at=(16, 6, 0.5), id=\"crate\")"
        ),
    );
    assert_eq!(clearance_parts(&v), [["crate"]], "{v:#}");
    assert!(v[0]["message"].as_str().unwrap().contains("Shop 2"), "{v:#}");
    let v = findings_of(
        "mirror",
        &format!(
            "with m.mirror(\"x\", about=12):\n    {}\nwith m.step(\"Crate\"):\n    m.add(\"3005\", \"Brown\", at=(18, 6, 0.5), id=\"crate\")",
            door.replace("{i + 1}", "1")
        ),
    );
    assert_eq!(clearance_parts(&v), [["crate"]], "{v:#}");
}

/// A door's space has a box each side of its wall, and the parts in each get their own finding and their own move (the
/// review: one finding measured a crate behind the door against the box in front of it).
#[test]
fn each_side_of_a_door_gets_its_own_finding() {
    let v = findings_of(
        "both-sides",
        &format!(
            "{}\nwith m.step(\"Crates\"):\n    m.add(\"3005\", \"Brown\", at=(4, 6, 0.5), id=\"crate/front\")\n    \
             m.add(\"3005\", \"Brown\", at=(5, 10, 0.5), id=\"crate/back\")",
            DOOR.replace("ROWS", "7")
        ),
    );
    assert_eq!(clearance_parts(&v), [["crate/front"], ["crate/back"]], "{v:#}");
}

/// What can stand in a figure's way: a door frame does (the review: the whole Door category was skipped, frames with
/// the leaves), and a figure, an open door leaf, or a figure on a bench doesn't.
#[test]
fn a_frame_blocks_a_doorway_and_a_figure_or_a_leaf_doesnt() {
    let door = DOOR.replace("ROWS", "7");
    let v = findings_of(
        "frame-across",
        &format!(
            "{door}\nwith m.step(\"Frame\"):\n    m.add(\"60596\", \"White\", at=(3, 6, 0.5), rot=3, id=\"frame\")"
        ),
    );
    assert_eq!(clearance_parts(&v), [["frame"]], "{v:#}");
    let v = findings_of(
        "figure-and-leaf",
        &format!(
            "{door}\nwith m.step(\"Figure and leaf\"):\n    m.add(\"3817b\", \"Blue\", at=(4, 6, 0.5), id=\"legs\")\n    \
             m.add(\"60623\", \"Red\", at=(3, 5, 0.5), rot=3, id=\"leaf\")\nwith m.step(\"Bench\"):\n    \
             m.use(\"bench\", at=(14, 2, 0.5))\nwith m.step(\"Sitter\"):\n    m.add(\"973\", \"Red\", at=(15, 2.5, 5.5), id=\"torso\")"
        ),
    );
    assert_eq!(clearance_parts(&v), Vec::<Vec<String>>::new(), "{v:#}");
}

/// figure_clearance leads the warnings, after the errors (D-53), whatever its source line; m.allow moves it to allowed.
#[test]
fn figure_clearance_leads_the_warnings_and_can_be_allowed() {
    let door = DOOR.replace("ROWS", "7");
    let body = format!(
        "with m.step(\"Nudged\"):\n    m.add(\"3005\", \"Red\", at=(20.2, 2, 0.5), id=\"nudged\")\n{door}\n\
         with m.step(\"Crate\"):\n    m.add(\"3005\", \"Brown\", at=(4, 6, 0.5), id=\"crate\")\n\
         with m.step(\"Loose\"):\n    m.add(\"3005\", \"Red\", at=(25, 25, 10), id=\"loose\")"
    );
    let v = findings_of("order", &body);
    let rules: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["rule"].as_str().unwrap())
        .collect();
    // The errors (the loose brick, and the nudged one, which floats too), then figure_clearance, then off_grid, though
    // the nudged brick's line comes first.
    let at = |r: &str| {
        rules
            .iter()
            .position(|x| *x == r)
            .unwrap_or_else(|| panic!("no {r}: {v:#}"))
    };
    assert!(
        rules[..at("figure_clearance")].iter().all(|r| *r == "floating") && at("figure_clearance") < at("off_grid"),
        "{rules:?}"
    );
    let (v, code) = check_of(
        "allowed",
        &format!("{body}\nm.allow(\"figure_clearance\", \"crate\", \"a display crate; no figure goes in\")"),
    );
    assert_eq!(code, 0, "{v}");
    assert_eq!(
        clearance_parts(&v["data"]["findings"]),
        Vec::<Vec<String>>::new(),
        "{v:#}"
    );
    assert_eq!(v["data"]["allowed"][0]["rule"], "figure_clearance", "{v:#}");
}

/// m.keep_clear refuses a space no part could stand in: no box, or a flat one (the review: they were taken and never
/// fired).
#[test]
fn keep_clear_refuses_a_space_that_could_never_fire() {
    for (box_, says) in [("[]", "needs a box"), ("((0, 0, 1), (4, 0, 10))", "is flat")] {
        let (v, code) = check_of("bad-space", &format!("m.keep_clear({box_}, \"a path\")"));
        assert_ne!(code, 0, "{box_}: {v}");
        assert_eq!(v["error"]["code"], "MODEL_EXEC_FAILED", "{box_}: {v}");
        assert!(v["error"]["message"].as_str().unwrap().contains(says), "{box_}: {v}");
        assert_eq!(v["error"]["details"]["line"], 10, "{box_}: {v}");
    }
}
