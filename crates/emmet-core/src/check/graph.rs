// SPDX-License-Identifier: AGPL-3.0-or-later
//! Connector matching and the connection graph (plan section 11): every placed part's connectors in the world, the
//! pairs that mate, and the parts they join.
//!
//! Two connectors mate when their kinds and genders fit and they sit on one line: their axes parallel (within about
//! 8°) and no more than 0.5 LDU apart across it (1 LDU for a round joint: a pin, axle, bar, clip, finger, or plug).
//! Along it, a cylinder must reach into its mate: a stud all the way into an anti-stud (its length, give or take
//! 0.5 LDU), a round joint at least half way, and a bar or pin anywhere along a hole or clip that slides. Section radii must fit (no male wider than the hole it's in) and at least one must grip (the same radius).

use std::collections::{HashMap, HashSet};

use glam::{DMat3, DMat4, DVec3};

use super::connectors::{Connector, Gender, Kind, Shape};

/// How far apart a stud and its anti-stud's axes may be, and how far a cylinder may fall short of its seat.
pub const TOLERANCE: f64 = 0.5;
/// How far apart a round joint's axes may be (a pin, axle, bar, clip, hinge finger, or plug): models place them by
/// hand and they have play, so official files are often 1 LDU off (Barry, 2026-10-28).
pub const JOINT: f64 = 1.0;
/// How parallel two mating axes must be (cos 8°).
const PARALLEL: f64 = 0.99;
/// How close a male section's radius must be to its hole's to grip, and how much wider it may be (a pin's ridge).
const GRIP: f64 = 0.6;
const CELL: f64 = 10.0;
/// A bar's radius (3.2 mm).
const BAR: f64 = 4.0;

/// A connector of a placed part, in the world.
#[derive(Debug, Clone)]
pub struct Placed {
    /// The placement it belongs to.
    pub part: usize,
    pub c: Connector,
}

/// Two placements joined by a pair of connectors.
#[derive(Debug, Clone)]
pub struct Connection {
    pub a: usize,
    pub b: usize,
    pub kind: Kind,
    /// Where the pair seats: the middle of their overlap.
    pub at: DVec3,
    /// The pair's axis, and the radius and length of the seat, for the collision rule's exemption.
    pub axis: DVec3,
    pub radius: f64,
    pub len: f64,
    /// From the seat toward part `a`'s side, and toward `b`'s, when the shapes' closed ends say (a stud's base is in
    /// its part, an anti-stud's far end in its): the way each part was put on.
    pub toward_a: Option<DVec3>,
    pub toward_b: Option<DVec3>,
}

/// Every placement's connectors in the world. `parts[i]` is placement i's matrix and connectors (empty for an
/// unknown part).
pub fn world(parts: &[(DMat4, std::rc::Rc<Vec<Connector>>)]) -> Vec<Placed> {
    let mut out = Vec::new();
    for (i, (m, cs)) in parts.iter().enumerate() {
        let r = DMat3::from_mat4(*m);
        let t = m.w_axis.truncate();
        for c in cs.iter() {
            if let Some(w) = c.transformed_world(r, t) {
                out.push(Placed { part: i, c: w });
            }
        }
    }
    out
}

fn cell(p: DVec3) -> (i64, i64, i64) {
    (
        (p.x / CELL).floor() as i64,
        (p.y / CELL).floor() as i64,
        (p.z / CELL).floor() as i64,
    )
}

fn end(c: &Connector) -> DVec3 {
    c.pos + c.axis * c.len()
}

/// Every mating pair of connectors between different placements.
pub fn connections(conns: &[Placed]) -> Vec<Connection> {
    // The receiving side (females, clips, every finger and generic shape) goes in a grid by the cells its segment
    // passes through, widened by the tolerance; the other side looks up the cells its own points fall in.
    let receiving = |c: &Connector| c.gender == Gender::Female || matches!(c.kind, Kind::Finger | Kind::Gen);
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    for (i, p) in conns.iter().enumerate() {
        if !receiving(&p.c) {
            continue;
        }
        let (a, b) = (p.c.pos, end(&p.c));
        let pad = DVec3::splat(JOINT + 1.0);
        let (lo, hi) = (cell(a.min(b) - pad), cell(a.max(b) + pad));
        for x in lo.0..=hi.0 {
            for y in lo.1..=hi.1 {
                for z in lo.2..=hi.2 {
                    grid.entry((x, y, z)).or_default().push(i);
                }
            }
        }
    }
    let mut out = Vec::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    for (i, p) in conns.iter().enumerate() {
        // Males look for what takes them; fingers and generic shapes look for each other (each pair once).
        let looks = p.c.gender == Gender::Male && p.c.kind == Kind::Cyl || matches!(p.c.kind, Kind::Finger | Kind::Gen);
        if !looks {
            continue;
        }
        let len = p.c.len();
        let steps = (len / (CELL / 2.0)).ceil().max(1.0) as usize;
        let mut cells = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let k = cell(p.c.pos + p.c.axis * (len * s as f64 / steps as f64));
            if !cells.contains(&k) {
                cells.push(k);
            }
        }
        for k in cells {
            for &j in grid.get(&k).into_iter().flatten() {
                let q = &conns[j];
                if q.part == p.part || j == i {
                    continue;
                }
                let key = if p.c.kind == Kind::Cyl {
                    (i, j)
                } else {
                    (i.min(j), i.max(j))
                };
                if !seen.insert(key) {
                    continue;
                }
                if let Some(c) = mate(&p.c, &q.c) {
                    out.push(Connection {
                        a: p.part,
                        b: q.part,
                        ..c
                    });
                }
            }
        }
    }
    out
}

/// For debugging: connectors of other placements within `reach` LDU of `c`'s line that could be its mate (opposite
/// gender, or both fingers or generic shapes), with whether they mate.
#[doc(hidden)]
pub fn near(conns: &[Placed], i: usize, reach: f64) -> Vec<(usize, &'static str, f64)> {
    let p = &conns[i];
    let mut out = Vec::new();
    for (j, q) in conns.iter().enumerate() {
        if q.part == p.part {
            continue;
        }
        let pair = p.c.kind == q.c.kind && matches!(p.c.kind, Kind::Finger | Kind::Gen) || p.c.gender != q.c.gender;
        let d =
            q.c.pos
                .distance(p.c.pos)
                .min(q.c.pos.distance(end(&p.c)))
                .min(end(&q.c).distance(p.c.pos));
        if !pair || d > reach + p.c.len() + q.c.len() {
            continue;
        }
        let (m, f) = if p.c.gender == Gender::Male && p.c.kind == Kind::Cyl {
            (&p.c, &q.c)
        } else {
            (&q.c, &p.c)
        };
        let off = (m.pos - f.pos) - f.axis * (m.pos - f.pos).dot(f.axis);
        if off.length() <= reach {
            let why = match (try_mate(m, f), try_mate(f, m)) {
                (Ok(_), _) | (_, Ok(_)) => "mates",
                (Err(w), _) => w,
            };
            out.push((j, why, off.length()));
        }
    }
    out
}

/// Whether two connectors mate, and where. `a` is the male cylinder, or either finger or generic shape.
fn mate(a: &Connector, b: &Connector) -> Option<Connection> {
    try_mate(a, b).ok()
}

/// [`mate`], or why not.
fn try_mate(a: &Connector, b: &Connector) -> Result<Connection, &'static str> {
    let fail = |why| Err(why);
    if a.group != b.group {
        return fail("group");
    }
    let dot = a.axis.dot(b.axis);
    if dot.abs() < PARALLEL && !(a.free || b.free) {
        return fail("not parallel");
    }
    match (a.kind, b.kind) {
        (Kind::Gen, Kind::Gen) => {
            if a.gender == b.gender {
                return fail("same gender");
            }
            if a.pos.distance(b.pos) > JOINT {
                return fail("apart");
            }
            let (r, l) = (a.secs[0].radius.max(b.secs[0].radius), a.len().max(b.len()));
            Ok(seat(a.pos, a.axis, r, l, Kind::Gen))
        }
        (Kind::Finger, Kind::Finger) => {
            // Fingers of one named group are made for each other (a window and its glass: LDCad gives them radii 4
            // and 2); without a group, their radii must match.
            let (ra, rb) = (a.secs[0].radius, b.secs[0].radius);
            if a.group.is_none() && (ra - rb).abs() > GRIP {
                return fail("radius");
            }
            // A hinge: the two sets of fingers on one line, together on it. LDCad's finger runs start at a point and
            // go one way, which a part turned over reverses (3937 and 3938 in 889-1 start at one point and run
            // apart), so starts within a run's length count as together.
            let d = a.pos - b.pos;
            let t = d.dot(b.axis);
            if (d - b.axis * t).length() > JOINT + 1e-3 {
                return fail("off the line");
            }
            if t.abs() > a.len().max(b.len()) {
                return fail("no overlap");
            }
            Ok(seat(
                (a.pos + b.pos) / 2.0,
                b.axis,
                ra,
                a.len().max(b.len()),
                Kind::Finger,
            ))
        }
        (Kind::Cyl, Kind::Clip) => {
            let (lo, hi) = along(a, b, dot).ok_or("off the line")?;
            let r = b.secs[0].radius;
            let grip = sections_over(a, b, dot)
                .filter(|(sa, _, l)| (sa.radius - r).abs() <= GRIP && *l > TOLERANCE)
                .count();
            if grip == 0 {
                return fail("radius");
            }
            if hi - lo <= TOLERANCE {
                return fail("no overlap");
            }
            Ok(seat(b.pos + b.axis * (lo + hi) / 2.0, b.axis, r, hi - lo, Kind::Clip))
        }
        (Kind::Cyl, Kind::Cyl) => {
            if a.gender != Gender::Male || b.gender != Gender::Female {
                return fail("genders");
            }
            // A stud enters an anti-stud from its open side: a stud (closed at its base) into an anti-stud (closed at its
            // far end) runs the same way. Short centered pins (a minifig's shoulder, a wheel's) seat either way round.
            if stud(a) && !a.open.0 && !b.open.1 && dot < 0.0 {
                return fail("closed ends face each other");
            }
            let (lo, hi) = along(a, b, dot).ok_or("off the line")?;
            let overlap = hi - lo;
            let shorter = a.len().min(b.len());
            if overlap <= TOLERANCE {
                return fail("no overlap");
            }
            // A stud goes all the way in; a pin, axle, or bar holds once half in (files put wheels on their pins a
            // little short).
            let need = if stud(a) { shorter - TOLERANCE } else { shorter / 2.0 };
            if !(a.slide || b.slide) && overlap < need {
                return fail("not all the way in");
            }
            let mut grip = 0.0f64;
            for (sa, sb, l) in sections_over(a, b, dot) {
                if l <= TOLERANCE || sa.shape == Shape::Flex || sb.shape == Shape::Flex {
                    continue;
                }
                // An axle hole takes an axle, and a bar (3.2 mm, a snug fit in its cross), but no round pin; a round or
                // square hole takes a pin, an axle, or a stud.
                let bar = sa.shape == Shape::Round && sa.radius <= BAR + GRIP;
                if sb.shape == Shape::Axle && sa.shape != Shape::Axle && !bar {
                    return fail("round in an axle hole");
                }
                if sa.radius > sb.radius + GRIP {
                    return fail("too wide");
                }
                if (sa.radius - sb.radius).abs() <= GRIP || sb.shape == Shape::Axle && bar {
                    grip += l;
                }
            }
            if grip <= TOLERANCE {
                return fail("no grip");
            }
            let r = a.secs.iter().map(|s| s.radius).fold(0.0, f64::max);
            Ok(Connection {
                toward_a: (!a.open.0 && a.open.1).then_some(-a.axis),
                toward_b: (b.open.0 && !b.open.1).then_some(b.axis),
                ..seat(b.pos + b.axis * (lo + hi) / 2.0, b.axis, r, overlap, Kind::Cyl)
            })
        }
        _ => fail("kinds"),
    }
}

fn seat(at: DVec3, axis: DVec3, radius: f64, len: f64, kind: Kind) -> Connection {
    Connection {
        a: 0,
        b: 0,
        kind,
        at,
        axis,
        radius,
        len,
        toward_a: None,
        toward_b: None,
    }
}

/// A stud: one round section a stud's width, no longer than a stud is tall.
fn stud(c: &Connector) -> bool {
    c.kind == Kind::Cyl && c.secs.len() == 1 && (c.secs[0].radius - 6.0).abs() <= GRIP && c.len() <= 6.0
}

/// How far apart two connectors' axes may be: [`TOLERANCE`] for a stud, [`JOINT`] for any round joint.
fn reach(a: &Connector, b: &Connector) -> f64 {
    if stud(a) || stud(b) { TOLERANCE } else { JOINT }
}

/// Where `a` lies along `b`'s axis, as the overlap of the two segments in `b`'s parameter (0 at `b`'s start), when
/// `a`'s axis is within [`reach`] of `b`'s line.
fn along(a: &Connector, b: &Connector, dot: f64) -> Option<(f64, f64)> {
    let d = a.pos - b.pos;
    let t = d.dot(b.axis);
    if (d - b.axis * t).length() > reach(a, b) + 1e-3 {
        return None;
    }
    let (a0, a1) = {
        let e = t + dot.signum() * a.len();
        (t.min(e), t.max(e))
    };
    let (lo, hi) = (a0.max(0.0), a1.min(b.len()));
    (hi > lo).then_some((lo, hi))
}

/// Each pair of `a`'s and `b`'s sections that overlap along `b`'s axis, with the length they share.
fn sections_over<'c>(
    a: &'c Connector,
    b: &'c Connector,
    dot: f64,
) -> impl Iterator<Item = (&'c super::connectors::Section, &'c super::connectors::Section, f64)> {
    let t = (a.pos - b.pos).dot(b.axis);
    let sign = dot.signum();
    let mut sa_ranges = Vec::new();
    let mut s = 0.0;
    for sec in a.secs.iter() {
        let (u, v) = (t + sign * s, t + sign * (s + sec.len));
        sa_ranges.push((sec, u.min(v), u.max(v)));
        s += sec.len;
    }
    let mut sb_ranges = Vec::new();
    let mut s = 0.0;
    for sec in b.secs.iter() {
        sb_ranges.push((sec, s, s + sec.len));
        s += sec.len;
    }
    sa_ranges.into_iter().flat_map(move |(sa, a0, a1)| {
        sb_ranges.clone().into_iter().filter_map(move |(sb, b0, b1)| {
            let l = a1.min(b1) - a0.max(b0);
            (l > 0.0).then_some((sa, sb, l))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::connectors::parse_one;

    fn at(line: &str, t: DVec3) -> Connector {
        parse_one(line).transformed_world(DMat3::IDENTITY, t).unwrap()
    }

    const STUD: &str = "!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 6 4]";
    const ANTI: &str = "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=S 6 4] [pos=0 8 0]";

    #[test]
    fn a_stud_mates_with_the_anti_stud_on_top_of_it_only() {
        let stud = at(STUD, DVec3::ZERO);
        // A plate on top: its underside (8 below its origin) at the stud's base.
        assert!(mate(&stud, &at(ANTI, DVec3::new(0.0, -8.0, 0.0))).is_some());
        assert!(
            mate(&stud, &at(ANTI, DVec3::new(0.4, -8.0, 0.0))).is_some(),
            "within the tolerance"
        );
        assert!(
            mate(&stud, &at(ANTI, DVec3::new(1.0, -8.0, 0.0))).is_none(),
            "a millimeter off"
        );
        // A plate a plate too high doesn't reach the stud; one 2 LDU too high only half takes it.
        assert!(mate(&stud, &at(ANTI, DVec3::new(0.0, -16.0, 0.0))).is_none());
        assert!(mate(&stud, &at(ANTI, DVec3::new(0.0, -10.0, 0.0))).is_none());
        // A brick's deep anti-stud takes the stud too.
        let deep = at(
            "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 20] [pos=0 24 0]",
            DVec3::new(0.0, -24.0, 0.0),
        );
        assert!(mate(&stud, &deep).is_some());
    }

    #[test]
    fn a_bar_mates_with_a_clip_anywhere_along_it_but_a_stud_doesnt() {
        let bar = at(
            "!LDCAD SNAP_CYL [gender=M] [caps=none] [secs=R 4 60] [center=true] [slide=true] [pos=0 30 0]",
            DVec3::ZERO,
        );
        let clip = |y: f64| {
            at(
                "!LDCAD SNAP_CLP [radius=4] [length=8] [center=true]",
                DVec3::new(0.0, y, 0.0),
            )
        };
        assert!(mate(&bar, &clip(10.0)).is_some());
        assert!(mate(&bar, &clip(50.0)).is_some());
        assert!(mate(&bar, &clip(80.0)).is_none(), "off the bar's end");
        assert!(
            mate(&at(STUD, DVec3::new(0.0, 4.0, 0.0)), &clip(0.0)).is_none(),
            "a stud is too wide for a clip"
        );
    }

    #[test]
    fn a_pin_slides_in_a_hole_and_an_axle_hole_refuses_a_round_pin() {
        let hole = "!LDCAD SNAP_CYL [gender=F] [caps=none] [secs=R 8 2 R 6 16 R 8 2] [center=true] [slide=true]";
        let pin = at(
            "!LDCAD SNAP_CYL [gender=M] [caps=none] [secs=L_ 6.25 2 R 6 16 R 8 2 R 6 4] [center=true]",
            DVec3::ZERO,
        );
        // The pin's 16 LDU shaft in the hole's, its collar in the hole's wide mouth.
        assert!(mate(&pin, &at(hole, DVec3::new(0.0, 2.0, 0.0))).is_some());
        // Pushed in so far that its collar is in the narrow part: too wide.
        assert!(mate(&pin, &at(hole, DVec3::new(0.0, -6.0, 0.0))).is_none());
        let axle_hole = at(
            "!LDCAD SNAP_CYL [gender=F] [caps=none] [secs=A 6 20] [center=true]",
            DVec3::ZERO,
        );
        assert!(mate(&pin, &axle_hole).is_none());
        // A bar is a snug fit in an axle hole (a lightsaber blade in a bush, in 10124-1).
        let bar = at(
            "!LDCAD SNAP_CYL [gender=M] [caps=none] [secs=R 4 40] [center=true]",
            DVec3::ZERO,
        );
        assert!(mate(&bar, &axle_hole).is_some());
        // A wheel's pin half in its hole holds; a stud must go all the way.
        let wheel_pin = at("!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 4 12]", DVec3::ZERO);
        let wheel = |y: f64| {
            at(
                "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 4 12]",
                DVec3::new(0.0, y, 0.0),
            )
        };
        assert!(mate(&wheel_pin, &wheel(-5.0)).is_some());
        assert!(mate(&wheel_pin, &wheel(-7.0)).is_none());
    }

    #[test]
    fn hinge_fingers_turned_over_still_mesh() {
        // 3937 and 3938 (889-1): runs that start at one point and go opposite ways; a run a whole hinge away doesn't.
        let base = at(
            "!LDCAD SNAP_FGR [group=hgBrC] [genderOfs=F] [seq=18 4 18] [radius=4] [ori=0 1 0 -1 0 0 0 0 1]",
            DVec3::ZERO,
        );
        let top = at(
            "!LDCAD SNAP_FGR [group=hgBrC] [genderOfs=M] [seq=18 4 18] [radius=4] [ori=0 -1 0 1 0 0 0 0 1]",
            DVec3::ZERO,
        );
        assert!(mate(&top, &base).is_some());
        let far = at(
            "!LDCAD SNAP_FGR [group=hgBrC] [genderOfs=M] [seq=18 4 18] [radius=4] [ori=0 -1 0 1 0 0 0 0 1]",
            DVec3::new(100.0, 0.0, 0.0),
        );
        assert!(mate(&far, &base).is_none());
    }

    #[test]
    fn fingers_of_one_group_mesh_whatever_their_radii() {
        // 4033, a train window, and 3855b, its glass (2150-1): group win-1-4-3, radii 4 and 2.
        let window = at(
            "!LDCAD SNAP_FGR [group=win-1-4-3] [genderOfs=F] [seq=72] [radius=4]",
            DVec3::ZERO,
        );
        let glass = at(
            "!LDCAD SNAP_FGR [group=win-1-4-3] [genderOfs=M] [seq=72] [radius=2]",
            DVec3::ZERO,
        );
        assert!(mate(&glass, &window).is_some());
        let plain = at("!LDCAD SNAP_FGR [genderOfs=M] [seq=72] [radius=2]", DVec3::ZERO);
        let other = at("!LDCAD SNAP_FGR [genderOfs=F] [seq=72] [radius=4]", DVec3::ZERO);
        assert!(mate(&plain, &other).is_none());
    }

    #[test]
    fn a_round_joint_takes_1_ldu_of_hand_placement() {
        // A wheel 1 LDU off its holder's pin (6375-1) still seats; 2 LDU off doesn't.
        let hole = "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 4 10 R 5 2] [pos=0 0 8] [ori=1 0 0 0 0 -1 0 1 0]";
        let pin = at(
            "!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 4 11 R 4.25 1] [ori=1 0 0 0 0 -1 0 1 0]",
            DVec3::new(0.0, 0.0, 8.0),
        );
        assert!(mate(&pin, &at(hole, DVec3::new(1.0, 0.0, 0.0))).is_some());
        assert!(mate(&pin, &at(hole, DVec3::new(2.0, 0.0, 0.0))).is_none());
    }

    #[test]
    fn a_minifig_arm_seats_either_way_round_but_a_stud_only_one_way() {
        // A centered shoulder pin and the torso's hole, the pin's axis reversed (an arm posed the other way, 2150-1).
        let pin = at(
            "!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 5 2] [center=true] [ori=1 0 0 0 -1 0 0 0 -1]",
            DVec3::ZERO,
        );
        let hole = at(
            "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 5 2 R 6 5] [pos=0 1 0]",
            DVec3::ZERO,
        );
        assert!(mate(&pin, &hole).is_some());
        // A stud pointing down at an anti-stud that opens upward doesn't enter it.
        let down = at(
            "!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 6 4] [ori=1 0 0 0 -1 0 0 0 -1]",
            DVec3::new(0.0, -4.0, 0.0),
        );
        assert!(mate(&down, &at(ANTI, DVec3::new(0.0, -8.0, 0.0))).is_none());
    }
}
