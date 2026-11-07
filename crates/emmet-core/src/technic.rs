// SPDX-License-Identifier: AGPL-3.0-or-later
//! Technic connection points (plan section 08, session 4G): a part's pin holes, axle holes, pins, axles, and axle
//! pins, from its connectors (the LDCad data, the primitive table, and the overrides, `check::connectors`). Each has
//! its middle, its axis, and its length, and a name, `1` to `n`, in the order of the middles in the DSL frame at
//! `rot=0` (x, then y, then z), so the agent can say "hole 3" and the DSL can seat a pin in it.
//!
//! A hole drawn in pieces along one line (an axle hole as two halves, a hub as several rings) is one hole. Studs,
//! anti-studs, bar holes, and connectors of a named group (a crane arm's slide) aren't Technic points.

use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::check::connectors::{Connector, Gender, Kind, Shape};

const STUD: f64 = 20.0;
const PLATE: f64 = 8.0;
/// A Technic pin's or axle's radius (4.8 mm), give or take a pin's ridge.
const RADIUS: f64 = 6.0;
const GRIP: f64 = 0.6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PointKind {
    PinHole,
    AxleHole,
    Pin,
    Axle,
    /// One end a pin, the other an axle.
    PinAxle,
}

impl PointKind {
    pub fn is_hole(self) -> bool {
        matches!(self, PointKind::PinHole | PointKind::AxleHole)
    }

    pub fn words(self) -> &'static str {
        match self {
            PointKind::PinHole => "pin hole",
            PointKind::AxleHole => "axle hole",
            PointKind::Pin => "pin",
            PointKind::Axle => "axle",
            PointKind::PinAxle => "axle pin",
        }
    }
}

/// One point in the part's LDraw frame (LDU).
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub name: String,
    pub kind: PointKind,
    /// The middle of the hole, or of the pin or axle.
    pub mid: DVec3,
    /// A unit direction along it. A pin's or axle's runs from its start (an axle pin's pin end) to its end.
    pub axis: DVec3,
    pub len: f64,
    /// A pin or axle whose start is its part's body (an axle stub on a hub, a pin with a stop bush): its far end goes
    /// into a hole.
    pub closed: bool,
}

/// A point as `part_info` and the DSL give it: in the DSL frame (x and y in studs, z in plates), from the corner of the
/// part's footprint when it's placed at (0, 0, 0) with rot=0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Hole {
    /// "1" to "n".
    pub name: String,
    pub kind: PointKind,
    /// The middle of the hole, pin, or axle: (x, y, z) in studs, studs, plates.
    pub at: [f64; 3],
    /// A unit direction along it, in the DSL's axes (x, y, z).
    pub axis: [f64; 3],
    /// How long it is, in studs: a beam's hole is 1, a thin liftarm's 0.5, a pin 2.
    pub length: f64,
}

/// What kind of Technic point a connector is, if any.
fn kind(c: &Connector) -> Option<PointKind> {
    if c.kind != Kind::Cyl || c.group.is_some() {
        return None;
    }
    let near = |r: f64| (r - RADIUS).abs() <= GRIP;
    let axle = c.secs.iter().any(|s| s.shape == Shape::Axle && near(s.radius));
    let round = c.secs.iter().any(|s| s.shape == Shape::Round && near(s.radius));
    let rim = c
        .secs
        .iter()
        .any(|s| s.shape == Shape::Round && (s.radius - 8.0).abs() <= GRIP);
    match c.gender {
        Gender::Female if axle => Some(PointKind::AxleHole),
        // A pin hole runs through (an anti-stud is closed at the part's top) or has a pin hole's rim, and is deeper
        // than an anti-stud.
        Gender::Female if round && (c.open == (true, true) || rim) && c.len() >= 8.0 - 1e-6 => Some(PointKind::PinHole),
        Gender::Male if axle && round => Some(PointKind::PinAxle),
        Gender::Male if axle => Some(PointKind::Axle),
        // Longer than a stud.
        Gender::Male if round && c.len() >= 12.0 - 1e-6 => Some(PointKind::Pin),
        _ => None,
    }
}

/// A segment along a line: start, unit axis, length.
#[derive(Debug, Clone, Copy)]
struct Seg {
    kind: PointKind,
    pos: DVec3,
    axis: DVec3,
    len: f64,
    closed: bool,
}

/// The part's Technic points in its LDraw frame, named in DSL order at rot=0.
pub fn points(connectors: &[Connector]) -> Vec<Point> {
    let mut segs: Vec<Seg> = connectors
        .iter()
        .filter_map(|c| {
            let kind = kind(c)?;
            Some(Seg {
                kind,
                pos: c.pos,
                axis: c.axis,
                len: c.len(),
                closed: !kind.is_hole() && !c.open.0 && c.open.1,
            })
        })
        .collect();
    // Holes drawn in pieces along one line merge; a pin or axle is one connector already.
    let mut merged = true;
    while merged {
        merged = false;
        'outer: for i in 0..segs.len() {
            for j in i + 1..segs.len() {
                if let Some(s) = join(&segs[i], &segs[j]) {
                    segs[i] = s;
                    segs.remove(j);
                    merged = true;
                    break 'outer;
                }
            }
        }
    }
    let all = segs.clone();
    segs.retain(|b| !all.iter().any(|a| inside(a, b)));
    let mut out: Vec<Point> = segs
        .into_iter()
        .map(|s| Point {
            name: String::new(),
            kind: s.kind,
            mid: s.pos + s.axis * s.len / 2.0,
            axis: s.axis,
            len: s.len,
            closed: s.closed,
        })
        .collect();
    let key = |p: &Point| {
        let d = dsl(p.mid);
        let r = |v: f64| (v * 100.0).round() as i64;
        (r(d.x), r(d.y), r(d.z), p.kind)
    };
    out.sort_by_key(key);
    out.dedup_by(|a, b| key(a) == key(b));
    for (i, p) in out.iter_mut().enumerate() {
        p.name = (i + 1).to_string();
    }
    out
}

/// Where `b` lies along `a`'s line, as a range of `a`'s parameter (0 at `a`'s start), when the two are on one line.
fn span(a: &Seg, b: &Seg) -> Option<(f64, f64)> {
    if a.axis.dot(b.axis).abs() < 0.999 {
        return None;
    }
    let d = b.pos - a.pos;
    let t = d.dot(a.axis);
    if (d - a.axis * t).length() > 0.5 {
        return None;
    }
    let end = t + a.axis.dot(b.axis).signum() * b.len;
    Some((t.min(end), t.max(end)))
}

/// Two holes of the same kind on one line that touch or overlap, as one; None otherwise.
fn join(a: &Seg, b: &Seg) -> Option<Seg> {
    if a.kind != b.kind || !a.kind.is_hole() {
        return None;
    }
    let (b0, b1) = span(a, b)?;
    // Touching, give or take a hair.
    if b0 > a.len + 0.5 || b1 < -0.5 {
        return None;
    }
    let (lo, hi) = (b0.min(0.0), b1.max(a.len));
    Some(Seg {
        kind: a.kind,
        pos: a.pos + a.axis * lo,
        axis: a.axis,
        len: hi - lo,
        closed: false,
    })
}

/// Whether `b` is a pin or axle inside the longer pin or axle `a` (a pin's half, drawn as its own primitive).
fn inside(a: &Seg, b: &Seg) -> bool {
    !a.kind.is_hole()
        && !b.kind.is_hole()
        && a.len > b.len + 1e-6
        && span(a, b).is_some_and(|(b0, b1)| b0 >= -0.5 && b1 <= a.len + 0.5)
}

/// An LDraw point in the DSL frame (x = −Z/20, y = X/20, z = −Y/8).
fn dsl(v: DVec3) -> DVec3 {
    DVec3::new(-v.z / STUD, v.x / STUD, -v.y / PLATE)
}

/// An LDraw direction in the DSL's axes, as a unit vector in space (not scaled by stud and plate).
fn dsl_dir(v: DVec3) -> DVec3 {
    DVec3::new(-v.z, v.x, -v.y)
}

fn round(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0 + 0.0
}

/// The points as `part_info` gives them: from the footprint corner of `body` (the part's LDraw body box) at rot=0.
pub fn holes(points: &[Point], body: [f64; 6]) -> Vec<Hole> {
    // The footprint corner: the DSL minimum of the body box.
    let corner = DVec3::new(-body[5] / STUD, body[0] / STUD, -body[4] / PLATE);
    points
        .iter()
        .map(|p| {
            let at = dsl(p.mid) - corner;
            let axis = dsl_dir(p.axis);
            Hole {
                name: p.name.clone(),
                kind: p.kind,
                at: [round(at.x), round(at.y), round(at.z)],
                axis: [round(axis.x), round(axis.y), round(axis.z)],
                length: round(p.len / STUD),
            }
        })
        .collect()
}

/// A direction in words: "x" for a DSL axis, else the vector.
pub fn axis_words(a: [f64; 3]) -> String {
    for (i, name) in ["x", "y", "z"].iter().enumerate() {
        if (a[i].abs() - 1.0).abs() < 1e-3 {
            return (*name).to_string();
        }
    }
    let n = |v: f64| crate::ldraw::num(v);
    format!("({}, {}, {})", n(a[0]), n(a[1]), n(a[2]))
}

/// The points in words, for `part_info`'s text and `emmet part`: "Holes from its footprint corner at rot=0 (x and y
/// in studs, z in plates): 1 pin hole at (0.5, 0.5, 1.25) along z, 1 stud long; …". Empty for none.
pub fn holes_text(holes: &[Hole]) -> String {
    if holes.is_empty() {
        return String::new();
    }
    let n = |v: f64| crate::ldraw::num(v);
    let list: Vec<String> = holes
        .iter()
        .map(|h| {
            format!(
                "{} {} at ({}, {}, {}) along {}, {} long",
                h.name,
                h.kind.words(),
                n(h.at[0]),
                n(h.at[1]),
                n(h.at[2]),
                axis_words(h.axis),
                if h.length == 1.0 {
                    "1 stud".to_string()
                } else {
                    format!("{} studs", n(h.length))
                }
            )
        })
        .collect();
    format!(
        "Holes from its footprint corner at rot=0 (x and y in studs, z in plates): {}",
        list.join("; ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::connectors::parse_one;

    fn at(line: &str, t: DVec3) -> Connector {
        parse_one(line).transformed_world(glam::DMat3::IDENTITY, t).unwrap()
    }

    #[test]
    fn holes_drawn_in_pieces_and_a_pin_s_halves_are_one_and_anti_studs_are_none() {
        // An axle connector's two blind halves (59443) and a brick's anti-stud, a stud, a bar hole.
        let cs = vec![
            at(
                "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=A 6 20] [pos=0 0 -20] [ori=1 0 0 0 0 1 0 -1 0]",
                DVec3::ZERO,
            ),
            at(
                "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=A 6 20] [pos=0 0 20] [ori=1 0 0 0 0 -1 0 1 0]",
                DVec3::ZERO,
            ),
            at(
                "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 20] [pos=0 24 0]",
                DVec3::ZERO,
            ),
            at("!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 6 4]", DVec3::ZERO),
            at(
                "!LDCAD SNAP_CYL [gender=F] [caps=none] [secs=R 4 40] [center=true]",
                DVec3::ZERO,
            ),
            // A pin (3673) and the two halves its file draws as primitives (confric): one pin.
            at(
                "!LDCAD SNAP_CYL [gender=M] [caps=none] [secs=L_ 6.25 2 R 6 16 R 8 4 R 6 16 _L 6.25 2] [center=true] \
                 [ori=0 -1 0 1 0 0 0 0 1] [pos=0 -40 0]",
                DVec3::ZERO,
            ),
            at(
                "!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 8 2 R 6 16 _L 6.25 2] [ori=0 -1 0 1 0 0 0 0 1] [pos=0 -40 0]",
                DVec3::ZERO,
            ),
        ];
        let p = points(&cs);
        assert_eq!(p.len(), 2, "{p:?}");
        let hole = p.iter().find(|p| p.kind == PointKind::AxleHole).unwrap();
        assert_eq!(hole.len, 40.0);
        assert!(hole.mid.length() < 1e-9);
        let pin = p.iter().find(|p| p.kind == PointKind::Pin).unwrap();
        assert_eq!(pin.len, 40.0);
    }

    #[test]
    fn a_beam_s_holes_are_named_along_it() {
        // Three pin holes 20 apart along LDraw Z (DSL −x): named from the lowest DSL x, which is the largest Z.
        let hole = "!LDCAD SNAP_CYL [gender=F] [caps=none] [secs=R 8 2 R 6 16 R 8 2] [center=true] [slide=true]";
        let cs: Vec<Connector> = [-20.0, 0.0, 20.0]
            .iter()
            .map(|z| at(hole, DVec3::new(0.0, 0.0, *z)))
            .collect();
        let p = points(&cs);
        assert_eq!(p.iter().map(|p| p.mid.z).collect::<Vec<_>>(), vec![20.0, 0.0, -20.0]);
        assert_eq!(p[2].name, "3");
        let h = holes(&p, [-10.0, -10.0, -30.0, 10.0, 10.0, 30.0]);
        assert_eq!(h[0].at, [0.5, 0.5, 1.25]);
        assert_eq!(h[2].at, [2.5, 0.5, 1.25]);
        assert_eq!(axis_words(h[0].axis), "z");
        assert_eq!(h[0].length, 1.0);
    }
}
