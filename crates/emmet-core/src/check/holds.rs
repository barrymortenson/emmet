// SPDX-License-Identifier: AGPL-3.0-or-later
//! What holds a part without a connector pair (plan section 11): a frame its filler, a tyre its wheel, flexible
//! parts and stickers what they touch, a figure where it touches; and the fallback for a part with no female
//! connector at all.

use std::collections::{HashMap, HashSet};

use glam::{DMat4, DVec3};

use super::boxes::Obb;
use super::{Placed, SEAT, graph};
use crate::index::Part;
use crate::ldraw::geometry::Aabb;

/// The connector data's fallback: a part with no female connector at all (one the LDCad data doesn't cover, whose
/// underside isn't drawn with tube primitives) takes any stud that enters its underside, at least 4 LDU inside its
/// sides (validate v0's rule).
pub(super) fn underside_links(
    placed: &[Placed],
    world: &[Option<Obb>],
    conns: &[graph::Placed],
) -> Vec<graph::Connection> {
    use super::connectors::{Gender, Kind};
    const CELL: f64 = 40.0;
    let cell = |v: f64| (v / CELL).floor() as i64;
    let open: Vec<bool> = placed
        .iter()
        .map(|q| {
            q.info
                .as_ref()
                .is_some_and(|i| i.body.is_some() && !i.connectors.iter().any(|c| c.gender == Gender::Female))
        })
        .collect();
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, w) in world.iter().enumerate() {
        let (true, Some(w)) = (open[i], w) else { continue };
        let b = w.aabb();
        for x in cell(b.min.x)..=cell(b.max.x) {
            for z in cell(b.min.z)..=cell(b.max.z) {
                grid.entry((x, z)).or_default().push(i);
            }
        }
    }
    let mut out = Vec::new();
    if grid.is_empty() {
        return out;
    }
    let inverse: HashMap<usize, DMat4> = grid.values().flatten().map(|&j| (j, placed[j].m.inverse())).collect();
    for w in conns {
        let c = &w.c;
        if c.kind != Kind::Cyl || c.gender != Gender::Male || (c.secs[0].radius - 6.0).abs() > 0.6 || c.len() > 6.0 {
            continue;
        }
        let probe = c.pos + c.axis * (c.len() - 2.0).max(0.0);
        for &j in grid.get(&(cell(probe.x), cell(probe.z))).into_iter().flatten() {
            if j == w.part {
                continue;
            }
            let body = placed[j].info.as_ref().and_then(|i| i.body.as_ref());
            if receives(body, &inverse[&j], c.pos, c.axis, c.len()) {
                out.push(graph::Connection {
                    a: w.part,
                    b: j,
                    kind: Kind::Cyl,
                    at: c.pos + c.axis * c.len() / 2.0,
                    axis: c.axis,
                    radius: 6.0,
                    len: c.len(),
                    toward_a: Some(-c.axis),
                    toward_b: Some(c.axis),
                });
            }
        }
    }
    out
}

/// Whether part `j` (its body box, and its inverse placement) takes a stud whose base is at `base` pointing along
/// `dir` and reaching `len`, in world space: the stud enters it through its underside, inside its sides.
fn receives(body: Option<&Aabb>, inv: &DMat4, base: DVec3, dir: DVec3, len: f64) -> bool {
    const ENTER: f64 = 2.0;
    const INSET: f64 = 4.0;
    let Some(body) = body else { return false };
    let local = inv.transform_point3(base);
    let local_dir = inv.transform_vector3(dir).normalize_or_zero();
    // Into the underside: the stud points up the part's own frame (−Y, so y shrinks along it), its base is at or
    // below the part's bottom, and its tip reaches into the part.
    let bottom = body.max.y;
    if local_dir.y > -0.9 || local.y < bottom - SEAT || local.y - len > bottom - ENTER {
        return false;
    }
    local.x >= body.min.x + INSET
        && local.x <= body.max.x - INSET
        && local.z >= body.min.z + INSET
        && local.z <= body.max.z - INSET
}

/// Window and door frames, and what fills them (glass, panes, shutters, doors): a filler inside its frame isn't a
/// collision.
fn frame(p: &Part) -> bool {
    matches!(p.category.as_str(), "Window" | "Door") && !filler(p) || p.name.contains("Frame")
}

fn filler(p: &Part) -> bool {
    p.category == "Glass"
        || ["Pane", "Shutter", "Glass"].iter().any(|w| p.name.contains(w))
        || p.category == "Door" && !p.name.contains("Frame")
}

fn inside(inner: &Aabb, outer: &Aabb, slack: f64) -> bool {
    (inner.min - outer.min).min_element() >= -slack && (outer.max - inner.max).min_element() >= -slack
}

/// True when one of placements `a` and `b` is a frame and the other fills it (glass in a window, a door in its frame):
/// they don't collide, and the frame holds the filler, which has no studs to hold it.
fn fills(pa: &Part, pb: &Part, wa: &Aabb, wb: &Aabb) -> bool {
    pa.id != pb.id && (frame(pa) && filler(pb) && inside(wb, wa, 2.0) || frame(pb) && filler(pa) && inside(wa, wb, 2.0))
}

fn tyre(p: &Part) -> bool {
    p.category == "Tyre" || p.name.trim_start_matches('~').starts_with("Tyre")
}

/// Parts that follow a path instead of snapping: hose and cable segments, chain links, strings.
pub(crate) fn flexible(p: &Part) -> bool {
    let n = p.name.trim_start_matches('~');
    matches!(p.category.as_str(), "Hose" | "String")
        || [
            "Hose Flexible",
            "Chain Link",
            "Chain Tread",
            "Minifig Chain",
            "Cable",
            "String",
        ]
        .iter()
        .any(|w| n.contains(w))
}

/// A minifigure's or figure's parts (and what they hold: headwear, accessories).
pub(crate) fn figure(p: &Part) -> bool {
    // By name too: an obsolete figure part's category is "Obsolete" (`~Minifig Hips (Obsolete)`).
    let name = p.name.trim_start_matches(['~', '_']);
    ["Minifig", "Figure"]
        .iter()
        .any(|w| p.category.starts_with(w) || name.starts_with(w))
}

pub(crate) fn sticker(p: &Part) -> bool {
    p.category.starts_with("Sticker")
}

/// One part holding another without a connector pair: a frame and its filler, a tyre and the wheel inside it, a
/// flexible part and what it touches along its path, a sticker and the surface it's on, and a figure and what it
/// touches (a figure is placed, not built: models pose figures by hand a millimeter off their joints, and seat or
/// stand them a millimeter off their studs).
fn holds(pa: &Part, pb: &Part, wa: &Aabb, wb: &Aabb) -> bool {
    let center = |b: &Aabb| (b.min + b.max) / 2.0;
    let contains = |outer: &Aabb, p: DVec3| {
        p.cmpge(outer.min - DVec3::splat(2.0)).all() && p.cmple(outer.max + DVec3::splat(2.0)).all()
    };
    // A wheel: a rim, or a part with wheels on it (122c01, Plate 2 x 2 with Red Wheels), whose box holds the tyre's
    // middle.
    let wheel = |p: &Part| !tyre(p) && (p.category == "Wheel" || p.name.contains("Wheel"));
    fills(pa, pb, wa, wb)
        || tyre(pa) && wheel(pb) && contains(wb, center(wa))
        || tyre(pb) && wheel(pa) && contains(wa, center(wb))
        || (flexible(pa) || flexible(pb) || sticker(pa) || sticker(pb) || figure(pa) || figure(pb))
            && touch(wa, wb, 1.0)
}

pub(super) fn touch(a: &Aabb, b: &Aabb, slack: f64) -> bool {
    (a.min - b.max).max_element() <= slack && (b.min - a.max).max_element() <= slack
}

/// Every pair of placements one holds the other (see [`holds`]), found by a sweep of their world boxes along x.
pub(super) fn held_pairs(placed: &[Placed], world: &[Option<Obb>]) -> HashSet<(usize, usize)> {
    let boxes: Vec<Option<Aabb>> = world.iter().map(|o| o.map(|o| o.aabb())).collect();
    let mut order: Vec<usize> = (0..placed.len())
        .filter(|&i| boxes[i].is_some() && placed[i].info.is_some())
        .collect();
    order.sort_by(|&a, &b| boxes[a].unwrap().min.x.total_cmp(&boxes[b].unwrap().min.x));
    let mut out = HashSet::new();
    for (k, &i) in order.iter().enumerate() {
        let bi = boxes[i].unwrap();
        for &j in &order[k + 1..] {
            let bj = boxes[j].unwrap();
            if bj.min.x > bi.max.x + 2.0 {
                break;
            }
            let (pi, pj) = (
                &placed[i].info.as_ref().unwrap().kind,
                &placed[j].info.as_ref().unwrap().kind,
            );
            if touch(&bi, &bj, 2.0) && holds(pi, pj, &bi, &bj) {
                out.insert((i.min(j), i.max(j)));
            }
        }
    }
    out
}
