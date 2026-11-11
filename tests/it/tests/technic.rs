// SPDX-License-Identifier: AGPL-3.0-or-later
//! Technic by holes (session 4G, plan sections 08 and 11): part_info's holes, pins, and axles against the parts' own
//! LDraw geometry; a beam laid on a 5-12-13 triangle and pinned at both ends; a pin that misses its hole; and a wheel
//! on an axle through its hub.

use emmet_core::data::DataDir;
use emmet_core::ldraw::geometry::{LibFiles, Measurer};
use emmet_core::ldraw::{Line, normalize_ref};
use emmet_it::{emmet, emmet_in, emmet_json, project, shared_data, shared_library, temp_dir, text};
use glam::{DMat3, DVec3};
use serde_json::Value;

/// A primitive that draws a Technic hole, pin, or axle, in the part's frame: its name, where it is, and its own y axis
/// (the way it runs).
struct Prim {
    name: String,
    pos: DVec3,
    axis: DVec3,
}

/// Every hole, pin, and axle primitive a part's file draws, through its subparts: read from the LDraw files alone,
/// not from the connector data part_info uses.
fn prims(m: &mut Measurer, reference: &str, mat: DMat3, at: DVec3, depth: usize, out: &mut Vec<Prim>) {
    let Some(lines) = m.lines(reference) else { return };
    for line in lines.iter() {
        let Line::Sub { pos, matrix, file, .. } = line else {
            continue;
        };
        let (mat2, at2) = (mat * *matrix, mat * *pos + at);
        let name = normalize_ref(file);
        let base = name.rsplit('/').next().unwrap_or(&name).to_string();
        if family(&base).is_some() && !name.starts_with("s/") {
            out.push(Prim {
                name: base,
                pos: at2,
                axis: mat2.y_axis.normalize_or_zero(),
            });
        } else if depth < 12 {
            prims(m, &name, mat2, at2, depth + 1, out);
        }
    }
}

/// Which kind of point a primitive draws: a pin hole, an axle hole, a pin, or an axle (the half-hole edges, npeghol,
/// are part of a hole drawn with others).
fn family(base: &str) -> Option<&'static str> {
    let starts = |p: &[&str]| p.iter().any(|p| base.starts_with(p));
    if starts(&["peghol", "beamhol", "connhol"]) {
        Some("pin_hole")
    } else if starts(&["axlehol", "axl2hol", "axl3hol", "axl4hol", "axlehole"]) {
        Some("axle_hole")
    } else if starts(&["connect", "confric"]) {
        Some("pin")
    } else if starts(&["axleend", "axle.dat"]) {
        Some("axle")
    } else {
        None
    }
}

/// What a point's kind may be drawn with: an axle's body is drawn with axle-hole primitives (axlehol8), an axle pin's
/// with both.
fn drawn_with(kind: &str, fam: &str) -> bool {
    match kind {
        "axle" => fam == "axle" || fam == "axle_hole",
        "pin_axle" => fam != "pin_hole",
        k => k == fam,
    }
}

/// 20 Technic parts, every kind: straight beams, thin and bent liftarms, bricks with holes, pins, axles, an axle pin,
/// a cross block, and a brake disc.
const TWENTY: [&str; 20] = [
    "32523", "32524", "40490", "41239", "32278", "32063", "6632", "60483", "32140", "32526", "3700", "3894", "3703",
    "6541", "2780", "6558", "3749", "4519", "32184", "65416",
];

#[test]
fn part_info_s_holes_match_the_parts_ldraw_geometry() {
    let lib = LibFiles::probe(&DataDir::at(shared_library()).ldraw());
    let mut m = Measurer::new(&lib);
    for id in TWENTY {
        let (v, code) = emmet_json(&["part", id]);
        assert_eq!(code, 0, "{v}");
        let d = &v["data"];
        let holes = d["holes"].as_array().unwrap();
        assert!(!holes.is_empty(), "{id} has no holes");
        let b: Vec<f64> = d["body_box"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        let mut drawn = Vec::new();
        prims(
            &mut m,
            &format!("{id}.dat"),
            DMat3::IDENTITY,
            DVec3::ZERO,
            0,
            &mut drawn,
        );
        // Each point: its middle in the part's LDraw frame (from the footprint corner at rot=0) and its axis.
        let corner = DVec3::new(-b[5] / 20.0, b[0] / 20.0, -b[4] / 8.0);
        let mut lines: Vec<(&str, DVec3, DVec3, f64)> = Vec::new();
        for h in holes {
            let f = |k: &str, i: usize| h[k][i].as_f64().unwrap();
            let dsl = DVec3::new(f("at", 0), f("at", 1), f("at", 2)) + corner;
            let mid = DVec3::new(20.0 * dsl.y, -8.0 * dsl.z, -20.0 * dsl.x);
            let axis = DVec3::new(f("axis", 1), -f("axis", 2), -f("axis", 0));
            let half = h["length"].as_f64().unwrap() * 20.0 / 2.0;
            let kind = h["kind"].as_str().unwrap();
            // Drawn there: a primitive of its kind on its line, within its length, running its way.
            let backed = drawn.iter().any(|p| {
                let d = p.pos - mid;
                let t = d.dot(axis);
                drawn_with(kind, family(&p.name).unwrap())
                    && p.axis.dot(axis).abs() > 0.99
                    && (d - axis * t).length() <= 0.5
                    && t.abs() <= half + 0.5
            });
            assert!(
                backed,
                "{id}'s point {} ({kind}) at {mid} along {axis} isn't drawn there",
                h["name"]
            );
            lines.push((kind, mid, axis, half));
        }
        // And none missing: every primitive is on one of the points.
        for p in &drawn {
            let on = lines.iter().any(|(kind, mid, axis, half)| {
                let d = p.pos - *mid;
                let t = d.dot(*axis);
                drawn_with(kind, family(&p.name).unwrap()) && (d - *axis * t).length() <= 0.5 && t.abs() <= half + 0.5
            });
            assert!(on, "{id} draws a {} at {} that part_info doesn't give", p.name, p.pos);
        }
    }
}

/// Every Technic beam, liftarm, brick with holes, pin, and axle in ldraw-mini has holes, pins, or axles in part_info.
#[test]
fn every_technic_beam_brick_pin_and_axle_has_points() {
    let data = DataDir::at(shared_library());
    let parts = data.ldraw().join("parts");
    let mut checked = 0;
    for e in std::fs::read_dir(&parts).unwrap() {
        let path = e.unwrap().path();
        let Some(id) = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".dat")
            .map(String::from)
        else {
            continue;
        };
        let first = std::fs::read_to_string(&path).unwrap();
        let name = first
            .lines()
            .next()
            .unwrap_or("")
            .trim_start_matches("0 ")
            .trim()
            .to_string();
        let technic = [
            "Technic Beam",
            "Technic Liftarm",
            "Technic Brick",
            "Technic Pin",
            "Technic Axle",
        ]
        .iter()
        .any(|p| name.starts_with(p));
        if !technic {
            continue;
        }
        let info = emmet_core::ops::part_info(&data, &id).unwrap();
        assert!(!info.holes.is_empty(), "{id} ({name}) has no holes in part_info");
        checked += 1;
    }
    assert!(checked >= 40, "only {checked} Technic parts in ldraw-mini");
}

/// The technic-brace golden project (`fault` as its docstring says), checked, with its connections.
fn brace(params: &[&str]) -> (Value, i32) {
    checked(&project("technic-brace"), params)
}

/// A project of `body` (the lines of the @model function after `m = Model()`), checked.
fn check_of(name: &str, body: &str, params: &[&str]) -> (Value, i32) {
    let dir = temp_dir(&format!("technic-{name}")).join("p");
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
    let body: String = body.lines().map(|l| format!("    {l}\n")).collect();
    std::fs::write(
        dir.join("model.py"),
        format!(
            "from emmet_dsl import Model, model\n\n\n@model\ndef build():\n    \"\"\"{name}.\"\"\"\n    \
             m = Model()\n{body}    return m\n"
        ),
    )
    .unwrap();
    checked(&dir, params)
}

/// `emmet check` in a project after setting `params`, with the connections (`links`) when it ran.
fn checked(dir: &std::path::Path, params: &[&str]) -> (Value, i32) {
    if !params.is_empty() {
        let mut args = vec!["params"];
        for p in params {
            args.extend(["--set", p]);
        }
        let (v, code) = emmet_in(dir, &args);
        assert_eq!(code, 0, "{v}");
    }
    let (v, code) = emmet_in(dir, &["check"]);
    if code == 0 {
        // The connections, for the tests that need them.
        let lib = LibFiles::probe(&DataDir::at(shared_data()).ldraw());
        let index = emmet_core::index::Index::ensure(&DataDir::at(shared_data())).unwrap();
        let project = emmet_core::project::Project::find(Some(dir)).unwrap();
        let (placements, _) = emmet_core::ops::built_placements(&project).unwrap();
        let links = emmet_core::check::links(&index, &lib, &placements).unwrap();
        let mut v = v;
        v["links"] = serde_json::json!(links);
        return (v, code);
    }
    (v, code)
}

#[test]
fn a_beam_on_a_5_12_13_triangle_seats_a_pin_at_each_end() {
    let (v, code) = brace(&[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
    // Each pin holds both beams it passes through.
    let links: Vec<(String, String)> = v["links"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| (l[0].as_str().unwrap().to_string(), l[1].as_str().unwrap().to_string()))
        .collect();
    for (pin, beams) in [("pin/low", ["base", "brace"]), ("pin/high", ["brace", "top"])] {
        for beam in beams {
            assert!(
                links
                    .iter()
                    .any(|(a, b)| a == pin && b == beam || a == beam && b == pin),
                "{pin} doesn't hold {beam}: {links:?}"
            );
        }
    }
}

#[test]
fn a_pin_out_of_its_hole_is_a_finding_that_names_the_hole() {
    // Half a stud to one side: it runs into the brace's body, and nothing holds it or the top beam.
    let (v, _) = brace(&["fault=1"]);
    let findings = v["data"]["findings"].as_array().unwrap();
    let rules: Vec<&str> = findings.iter().map(|f| f["rule"].as_str().unwrap()).collect();
    assert!(rules.contains(&"collision") && rules.contains(&"floating"), "{v:#}");
    for f in findings
        .iter()
        .filter(|f| f["parts"].as_array().unwrap().iter().any(|p| p == "pin/high"))
    {
        if f["rule"] == "collision" || f["rule"] == "floating" {
            let hint = f["hint"].as_str().unwrap();
            assert!(hint.contains("0.5 studs to one side of brace's hole 14"), "{hint}");
            assert!(hint.contains("m.insert"), "{hint}");
        }
    }
    // In the top beam but not the brace: the top beam and its pin hang on nothing.
    let (v, _) = brace(&["fault=2"]);
    let findings = v["data"]["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{v:#}");
    assert_eq!(findings[0]["rule"], "floating");
    assert_eq!(findings[0]["parts"], serde_json::json!(["top", "pin/high"]));
    let hint = findings[0]["hint"].as_str().unwrap();
    assert!(hint.starts_with("pin/high only touches brace's hole 14."), "{hint}");
}

/// A motorcycle wheel between two fork legs, on an axle through their end holes and its hub: no collision where the
/// axle runs through the hub's bore, and the wheel isn't "held by 1 stud" (the e-bike retry allowed both, 4D). The fork
/// is held up (`m.ground`) with the wheel off the table.
#[test]
fn a_wheel_on_an_axle_through_its_hub_checks_clean() {
    // Held up off the table, so the wheel isn't the model's lowest part (which stands on the table).
    let body = r#"with m.step("Table"):
    m.add("3703", "Black", at=(10, 0, 0), id="table")
with m.step("Fork"):
    left = m.beam("32524", "Black", at=(0, 0.5, 40), run=0, rise=6, id="leg/left")
    right = m.beam("32524", "Black", at=(0, 4.5, 40), run=0, rise=6, id="leg/right")
with m.step("Wheel"):
    axle = m.insert("44294", "Light Bluish Gray", left.hole(1), out="+y", out_by=4, id="axle")
    hub = left.hole(1)
    m.add("88517c01", "Black", at=(hub.at[0], 2.5, hub.at[2]), rot=1, hole=3, id="wheel")
m.ground("leg/left", "leg/right")"#;
    let (v, code) = check_of("wheel", body, &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
    let links = v["links"].as_array().unwrap();
    for part in ["leg/left", "leg/right", "wheel"] {
        assert!(
            links
                .iter()
                .any(|l| l[0] == "axle" && l[1] == part || l[1] == "axle" && l[0] == part),
            "the axle doesn't hold {part}: {links:?}"
        );
    }
}

/// A part slid onto an axle after earlier parts are on it either side is out of order: it can't go on (Barry's fifth
/// e-bike session, 2026-11-10, its brake discs and sprocket). The wheel between the legs, put on after the axle runs
/// through both, is a step_order warning naming the axle and the legs; on with the axle, it's clean; and a bush on
/// the axle's free end afterward is fine, since that end is open.
#[test]
fn a_part_slid_onto_an_axle_after_both_its_ends_are_shut_is_out_of_order() {
    let fork = r#"with m.step("Table"):
    m.add("3703", "Black", at=(10, 0, 0), id="table")
with m.step("Fork"):
    left = m.beam("32524", "Black", at=(0, 0.5, 40), run=0, rise=6, id="leg/left")
    right = m.beam("32524", "Black", at=(0, 4.5, 40), run=0, rise=6, id="leg/right")
hub = left.hole(1)
"#;
    let axle = r#"axle = m.insert("44294", "Light Bluish Gray", left.hole(1), out="+y", out_by=4, id="axle")
"#;
    let wheel = r#"m.add("88517c01", "Black", at=(hub.at[0], 2.5, hub.at[2]), rot=1, hole=3, id="wheel")
"#;
    let bush = r#"with m.step("Bush"):
    m.add("3713", "Black", at=(hub.at[0], axle.max[1] - 0.5, hub.at[2]), rot=1, hole="1", id="bush")
m.ground("leg/left", "leg/right")"#;
    let step = |name: &str, lines: &str| {
        format!(
            "with m.step(\"{name}\"):\n{}",
            lines.lines().map(|l| format!("    {l}\n")).collect::<String>()
        )
    };
    // The axle through both legs first, the wheel after: shut in.
    let late = format!("{fork}{}{}{bush}", step("Axle", axle), step("Wheel", wheel));
    let (v, code) = check_of("shut-in", &late, &[]);
    assert_eq!(code, 0, "{v}");
    let findings = v["data"]["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{v:#}");
    let f = &findings[0];
    assert_eq!(
        (f["rule"].as_str(), f["severity"].as_str()),
        (Some("step_order"), Some("warning")),
        "{f:#}"
    );
    assert_eq!(f["parts"], serde_json::json!(["wheel"]), "{f:#}");
    let message = f["message"].as_str().unwrap();
    for word in ["axle", "leg/left", "leg/right", "can't slide on"] {
        assert!(message.contains(word), "{message}");
    }
    assert!(f["hint"].as_str().unwrap().contains("step 3 or before"), "{f:#}");
    // The wheel on with its axle: clean, the bush on the open end included.
    let together = format!("{fork}{}{bush}", step("Wheel", &format!("{axle}{wheel}")));
    let (v, code) = check_of("on-with-it", &together, &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
}

/// The DSL says when a triangle doesn't land on holes, and which ones do; and a round pin won't go in an axle hole.
#[test]
fn beams_and_pins_that_cant_fit_say_why() {
    let (v, code) = check_of(
        "no-triangle",
        "m.beam(\"32278\", \"Black\", at=(0, 0.5, 1.25), run=12, rise=6)",
        &[],
    );
    assert_ne!(code, 0);
    let e = &v["error"];
    assert_eq!(e["code"], "MODEL_EXEC_FAILED");
    assert!(e["message"].as_str().unwrap().contains("reach 13.416 studs"), "{e}");
    assert!(
        e["hint"]
            .as_str()
            .unwrap()
            .contains("run 5, rise 12 (or 12, 5) for holes 13 apart"),
        "{e}"
    );
    let (v, code) = check_of(
        "pin-in-axle-hole",
        "arm = m.add(\"6632\", \"Black\", at=(0, 0, 0))\nm.insert(\"2780\", \"Black\", arm.hole(1))",
        &[],
    );
    assert_ne!(code, 0);
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("doesn't fit hole 1 of"),
        "{}",
        v["error"]
    );
}

/// A whole display e-bike from the bike techniques at their defaults, which fit a fat-tire e-bike's photo.
const BIKE: &str = r#"frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
swingarm = m.use("swingarm_shock", frame=frame)
fork = m.use("suspension_fork", frame=frame, swingarm=swingarm, fender=True)
m.use("chain_drive", frame=frame, swingarm=swingarm)
m.use("kickstand", swingarm=swingarm)
m.use("handlebars", fork=fork)
seat = m.use("bike_seat_rack", frame=frame)
m.use("battery_case", frame=frame, seat=seat)
m.use("front_fender", fork=fork)"#;

/// The connections of a checked model, as (a, b, kind).
fn links_of(v: &Value) -> Vec<(String, String, String)> {
    v["links"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            let s = |i: usize| l[i].as_str().unwrap().to_string();
            (s(0), s(1), s(2))
        })
        .collect()
}

/// The links between a group of parts and the rest, as (the group's part, the other part, kind).
fn crossing(links: &[(String, String, String)], inside: impl Fn(&str) -> bool) -> Vec<(String, String, String)> {
    links
        .iter()
        .filter(|(a, b, _)| inside(a) != inside(b))
        .map(|(a, b, k)| {
            if inside(a) {
                (a.clone(), b.clone(), k.clone())
            } else {
                (b.clone(), a.clone(), k.clone())
            }
        })
        .collect()
}

/// The display bike's fork steers (Barry, on 4G's review page: the bars turn on a triple tree and a pivot): the fork,
/// its front wheel, brakes, fender, and bars join the frame at one joint only, the steerer turning in the head bearing.
#[test]
fn the_fork_turns_on_its_steerer_alone() {
    let body = r#"frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
swingarm = m.use("swingarm_shock", frame=frame)
fork = m.use("suspension_fork", frame=frame, swingarm=swingarm, fender=True)
m.use("front_fender", fork=fork)
m.use("handlebars", fork=fork)"#;
    let (v, code) = check_of("steering", body, &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
    let front = |id: &str| {
        ["fork", "brakes", "front-fender", "handlebars"]
            .iter()
            .any(|p| id.starts_with(p))
    };
    let across = crossing(&links_of(&v), front);
    assert!(
        !across.is_empty()
            && across.iter().all(|(fork, frame, kind)| {
                fork.starts_with("fork-steerer") && frame.starts_with("frame-head") && kind == "cyl"
            }),
        "the fork joins the frame elsewhere too: {across:?}"
    );
}

/// Barry's second e-bike hand session (2026-11-09): "the fork design is not stable", the wheel hung on the shocks
/// alone. Now the front wheel, its axle, and the sliders that carry it join the rest of the bike only by the sliders
/// on the fork tubes, which they slide along, and by the front axle through the shocks' eyes: the shocks are the
/// springs, and the tubes guide the wheel.
#[test]
fn the_front_wheel_slides_on_the_fork_against_its_springs() {
    let body = r#"frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
swingarm = m.use("swingarm_shock", frame=frame)
m.use("suspension_fork", frame=frame, swingarm=swingarm)"#;
    let (v, code) = check_of("fork-springs", body, &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
    // The sliders are the step's first part on each side, then the fixed block, the shock, and its pin.
    let sliders = ["fork-sliders-and-shocks/1", "fork-sliders-and-shocks/5"];
    let shocks = ["fork-sliders-and-shocks/3", "fork-sliders-and-shocks/7"];
    let tubes = ["fork-fork-tubes/1", "fork-fork-tubes/2"];
    let wheel = |id: &str| id.starts_with("fork-front-wheel") || sliders.contains(&id);
    let across = crossing(&links_of(&v), wheel);
    for (w, other, kind) in &across {
        let slides = sliders.contains(&w.as_str()) && tubes.contains(&other.as_str()) && kind == "cyl";
        // The front wheel step: the wheel, its two discs, then the axle (in last, through them all), and its bushes.
        let springs = w == "fork-front-wheel/4" && shocks.contains(&other.as_str());
        assert!(
            slides || springs,
            "the front wheel hangs on {other} through {w} ({kind}): {across:?}"
        );
    }
    for (slider, tube) in sliders.iter().zip(tubes) {
        assert!(
            across.iter().any(|(w, o, _)| w == slider && o == tube),
            "{slider} isn't on {tube}: {across:?}"
        );
    }
    for shock in shocks {
        assert!(
            across.iter().any(|(_, o, _)| o == shock),
            "the front axle isn't through {shock}: {across:?}"
        );
    }
}

/// Barry's second e-bike hand session: the rear suspension was "designed to be non-functioning", its shock mounts
/// swinging free on one pin. Now the swingarm joins the frame only at its pivot axle and through the one shock, whose
/// top is on an axle fixed in the frame's down tubes: the wheel can only rise by squeezing the shock.
#[test]
fn the_rear_wheel_rises_only_by_squeezing_the_shock() {
    let body = r#"frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
m.use("swingarm_shock", frame=frame)"#;
    let (v, code) = check_of("rear-shock", body, &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
    // The frame's cross axles: the pivot, then the crank (the head's goes in last, with the head). The shock step: the
    // axle through the arms, its two bushes, the shock, the half bushes on the frame's mount, and the mount axle,
    // pushed in last through the down tubes, the half bushes, and the shock's eye: it's the frame's side of the joint.
    let (pivot, mount, shock) = ("frame-cross-axles/1", "swingarm-shock/7", "swingarm-shock/4");
    let swing = |id: &str| {
        id.starts_with("swingarm")
            && ![
                "swingarm-shock/4",
                "swingarm-shock/5",
                "swingarm-shock/6",
                "swingarm-shock/7",
            ]
            .contains(&id)
    };
    let links = links_of(&v);
    let across = crossing(&links, swing);
    assert!(
        across.iter().all(|(_, other, _)| other == pivot || other == shock),
        "the swingarm joins the frame elsewhere: {across:?}"
    );
    assert!(
        across.iter().any(|(_, o, k)| o == pivot && k == "cyl"),
        "it isn't on the pivot axle: {across:?}"
    );
    assert!(
        across.iter().any(|(w, o, _)| w == "swingarm-shock/1" && o == shock),
        "{across:?}"
    );
    assert!(
        links
            .iter()
            .any(|(a, b, _)| (a == shock && b == mount) || (a == mount && b == shock)),
        "the shock's top isn't on the frame's mount axle: {links:?}"
    );
}

/// Barry (2026-11-09): "the kickstand should not be coming off the center needs to be of the rear left swing arm". Its
/// pin goes through the left arm, and its foot is on the table with the wheels.
#[test]
fn the_kickstand_is_on_the_left_swingarm_arm() {
    let body = r#"frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
swingarm = m.use("swingarm_shock", frame=frame)
m.use("suspension_fork", frame=frame, swingarm=swingarm)
leg = m.use("kickstand", swingarm=swingarm)
table = swingarm["rear_axle"][2] - 2.5 * swingarm["radius"]
foot = min(h.at[2] for h in leg.holes) - 1.25  # the lowest hole's round end, half a stud below it
assert abs(foot - table) < 0.05, (foot, table)
assert leg.min[1] > swingarm["rear_axle"][1] + 4, leg.min"#;
    let (v, code) = check_of("kickstand", body, &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
    let links = links_of(&v);
    let pin = "kickstand/2";
    for part in ["kickstand/1", "swingarm-arms/2"] {
        assert!(
            links
                .iter()
                .any(|(a, b, _)| (a == pin && b == part) || (a == part && b == pin)),
            "the kickstand's pin doesn't hold {part}: {links:?}"
        );
    }
}

/// Barry's second e-bike hand session: "the axels needs to have something to retain them". On the whole bike, every
/// axle that runs through a round hole has a part gripping it (an axle hole: a bush, a crank arm, a cross block, a
/// gear) on each side of one, so it can't slide out either way; an axle in no round hole is held by what grips it.
#[test]
fn every_axle_on_the_bike_is_held() {
    let check = r#"
import math
from emmet_dsl.core import Handle
AXLES = {"4519", "3705", "32073", "3706", "44294", "3707", "3737", "3708", "50451"}
placed = [p for p in m._ids.values() if p.sub is None]
holes = {p.id: Handle(p).holes for p in placed}
loose = []
for p in placed:
    if p.part not in AXLES:
        continue
    (ax,) = [h for h in holes[p.id] if h.kind == "axle"]
    a, d = (ax.at[0], ax.at[1], ax.at[2] / 2.5), ax.axis
    on = []
    for q in placed:
        if q is p:
            continue
        for h in holes[q.id]:
            if h.kind not in ("pin_hole", "axle_hole") or abs(sum(h.axis[i] * d[i] for i in range(3))) < 0.99:
                continue
            w = (h.at[0] - a[0], h.at[1] - a[1], h.at[2] / 2.5 - a[2])
            t = sum(w[i] * d[i] for i in range(3))
            if abs(t) < ax.length / 2 and math.dist(w, tuple(t * d[i] for i in range(3))) < 0.05:
                on.append((t, h.kind))
    grips = [t for t, k in on if k == "axle_hole"]
    rounds = [t for t, k in on if k == "pin_hole"]
    if rounds:
        held = any(g < r for g in grips for r in rounds) and any(g > r for g in grips for r in rounds)
    else:
        held = bool(grips)
    if not held:
        loose.append(f"{p.id} ({p.part})")
assert not loose, f"axles that can slide out: {loose}"
"#;
    let (v, code) = check_of("held", &format!("{BIKE}\n{check}"), &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
}

/// The display-bike techniques take a photo's proportions and build a whole e-bike that checks clean: on its wheels
/// and its kickstand with no base, the wheels about two diameters apart and level, a seat, a rack, a battery, and a
/// front fender, from the techniques alone 200 parts or more (the agent's detail comes on top).
#[test]
fn a_bike_sized_to_its_photo_checks_clean() {
    let body = format!(
        "{BIKE}\nrear, front = swingarm[\"rear_axle\"], fork[\"front_axle\"]\nassert 21 <= front[0] - rear[0] <= 25, \
         (rear, front)\nassert abs(front[2] - rear[2]) < 0.01, (rear, front)\nassert len(m._ids) >= 200, len(m._ids)"
    );
    let (v, code) = check_of("e-bike", &body, &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]), "{v:#}");
}
