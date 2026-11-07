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
