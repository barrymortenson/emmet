// SPDX-License-Identifier: AGPL-3.0-or-later
//! The rules on the connection graph (plan section 11): floating, disconnected_submodel, weak_connection, and
//! off_grid.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use glam::{DMat3, DVec3};

use super::connectors::Kind;
use super::{GROUND, Model, ON_THE_TABLE, Options, PLATE, Raw, Rule, SEAT, STUD, UnionFind, dsl_box, graph, holds};
use super::{name, num, plural, round};
use crate::ldraw::geometry::Aabb;
use crate::ldraw::placements::Placement;

/// The copies a placement sits in, from the outermost: their ids, or the submodel names when the writer didn't name
/// copies (each name then counts as one copy).
pub(super) fn copy_path(p: &Placement) -> Vec<String> {
    if p.copies.len() == p.submodel.len() {
        p.copies.iter().map(|c| c.id.clone()).collect()
    } else {
        p.submodel.clone()
    }
}

/// The parts the model stands on: its lowest level, any baseplate, and the model's `m.ground` parts.
pub(super) fn ground(m: &Model, options: &Options) -> Vec<bool> {
    (0..m.placed.len())
        .map(|i| {
            m.kind(i).is_some_and(|k| k.category == "Baseplate")
                || options.ground.contains(&m.placed[i].p.id)
                || m.boxes[i].is_some_and(|b| b.max.y >= m.lowest - GROUND)
        })
        .collect()
}

/// The groups of connected parts that stand on nothing: no part of theirs is ground. Without a baseplate or ground
/// parts, a model can be several things on the table (a car, a figure beside it), each at its own height off it, so a
/// group with nothing of the model under it, within two plates of the lowest point, stands too.
pub(super) fn floating_groups(m: &Model, options: &Options) -> Vec<Vec<usize>> {
    let n = m.placed.len();
    let mut sets = UnionFind::new(n);
    for (a, b) in m.pairs() {
        sets.join(a, b);
    }
    let ground = ground(m, options);
    let mut grounded = vec![false; n];
    for i in (0..n).filter(|&i| ground[i]) {
        let r = sets.root(i);
        grounded[r] = true;
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in (0..n).filter(|&i| m.boxes[i].is_some()) {
        if !grounded[sets.root(i)] {
            groups.entry(sets.root(i)).or_default().push(i);
        }
    }
    let loose = options.ground.is_empty() && !(0..n).any(|i| m.kind(i).is_some_and(|k| k.category == "Baseplate"));
    if loose {
        groups.retain(|_, parts| over_something(m, parts) || bottom(m, parts) < m.lowest - ON_THE_TABLE);
    }
    groups.into_values().collect()
}

fn bottom(m: &Model, parts: &[usize]) -> f64 {
    parts
        .iter()
        .map(|&i| m.boxes[i].unwrap().max.y)
        .fold(f64::NEG_INFINITY, f64::max)
}

/// Some part outside `parts` is under one of them (by footprint, a stud's inset inside its sides).
pub(super) fn over_something(m: &Model, parts: &[usize]) -> bool {
    let members: HashSet<usize> = parts.iter().copied().collect();
    parts.iter().any(|&i| {
        let a = m.boxes[i].unwrap();
        m.boxes.iter().enumerate().any(|(j, b)| {
            !members.contains(&j)
                && b.is_some_and(|b| {
                    b.min.y >= a.max.y - 1.0
                        && a.min.x + 2.0 < b.max.x
                        && b.min.x + 2.0 < a.max.x
                        && a.min.z + 2.0 < b.max.z
                        && b.min.z + 2.0 < a.max.z
                })
        })
    })
}

fn union(m: &Model, parts: &[usize]) -> Aabb {
    let mut all = m.boxes[parts[0]].unwrap();
    for &i in &parts[1..] {
        let b = m.boxes[i].unwrap();
        all = Aabb {
            min: all.min.min(b.min),
            max: all.max.max(b.max),
        };
    }
    all
}

/// The highest part under a box, outside `parts`, by footprint: (its index, the gap in plates from its top to the
/// box's bottom).
fn below(m: &Model, parts: &[usize], all: &Aabb) -> Option<(usize, f64)> {
    let members: BTreeSet<usize> = parts.iter().copied().collect();
    let (i, b) = m
        .boxes
        .iter()
        .enumerate()
        .filter(|(i, b)| !members.contains(i) && b.is_some())
        .map(|(i, b)| (i, b.unwrap()))
        .filter(|(_, b)| {
            b.min.x < all.max.x
                && b.max.x > all.min.x
                && b.min.z < all.max.z
                && b.max.z > all.min.z
                && b.min.y >= all.max.y - SEAT
        })
        .max_by(|a, b| b.1.min.y.total_cmp(&a.1.min.y))?;
    Some((i, round(dsl_box(all).min[2] - dsl_box(&b).max[2])))
}

/// floating findings, leaving out the groups a disconnected_submodel finding already names.
pub(super) fn floating(m: &Model, groups: &[Vec<usize>], skip: &HashSet<usize>) -> Vec<Raw> {
    groups
        .iter()
        .filter(|g| !g.iter().all(|i| skip.contains(i)))
        .map(|parts| {
            let all = union(m, parts);
            let here = dsl_box(&all);
            let z = num(here.min[2]);
            let message = if parts.len() == 1 {
                let q = &m.placed[parts[0]];
                format!(
                    "{} ({}) at z={z} isn't connected to anything that holds it: no stud, clip, pin, or hinge.",
                    q.p.id,
                    name(q)
                )
            } else {
                format!(
                    "{} parts at z={z} aren't connected to the rest of the model: no stud, clip, pin, or hinge holds \
                     them.",
                    parts.len()
                )
            };
            let (they, them, start, rest) = if parts.len() == 1 {
                ("It", "it", "starts", "rests")
            } else {
                ("They", "them", "start", "rest")
            };
            let hint = match below(m, parts, &all) {
                Some((i, gap)) if gap > 0.0 => {
                    let q = &m.placed[i];
                    format!(
                        "{they} {start} {} above {} ({}), which ends at z={}: lower {them} by {}, or put a part under \
                         {them}.",
                        plural(gap, "plate"),
                        q.p.id,
                        name(q),
                        num(here.min[2] - gap),
                        plural(gap, "plate")
                    )
                }
                Some((i, _)) => {
                    let q = &m.placed[i];
                    format!(
                        "{they} {rest} on {} ({}) without a stud between: put {them} on whole-stud positions, or on a \
                         part with studs.",
                        q.p.id,
                        name(q)
                    )
                }
                None => format!(
                    "Nothing is under {them}: give {them} a part to stand on, or move {them} down onto the model."
                ),
            };
            Raw::new(Rule::Floating, parts.clone(), here.min, message, hint)
        })
        .collect()
}

/// disconnected_submodel: a submodel copy none of whose parts touches a part outside it, placed in a model that has
/// parts of its own to hold it (often a placement offset bug). Only the outermost such copy is reported. Returns the
/// findings and the parts they name.
pub(super) fn disconnected_submodels(m: &Model) -> (Vec<Raw>, HashSet<usize>) {
    let paths: Vec<Vec<String>> = m.placed.iter().map(|q| copy_path(q.p)).collect();
    let mut copies: BTreeMap<Vec<String>, Vec<usize>> = BTreeMap::new();
    for (i, path) in paths.iter().enumerate() {
        for d in 1..=path.len() {
            copies.entry(path[..d].to_vec()).or_default().push(i);
        }
    }
    let pairs = m.pairs();
    let mut out = Vec::new();
    let mut named: HashSet<usize> = HashSet::new();
    let mut reported: Vec<Vec<String>> = Vec::new();
    for (key, members) in &copies {
        if reported.iter().any(|r| key.starts_with(r)) || members.len() == m.placed.len() {
            continue;
        }
        let parent = &key[..key.len() - 1];
        if !paths.iter().any(|p| p == parent) {
            continue;
        }
        let inside: HashSet<usize> = members.iter().copied().collect();
        if pairs.iter().any(|&(a, b)| inside.contains(&a) != inside.contains(&b)) {
            continue;
        }
        let parts: Vec<usize> = members.iter().copied().filter(|&i| m.boxes[i].is_some()).collect();
        if parts.is_empty() {
            continue;
        }
        let q = &m.placed[parts[0]];
        let depth = key.len() - 1;
        let sub = q.p.submodel.get(depth).cloned().unwrap_or_default();
        let step =
            q.p.copies
                .get(depth)
                .map(|c| format!(" in step {}", c.step))
                .unwrap_or_default();
        let all = union(m, &parts);
        let message = format!(
            "Submodel '{sub}' (copy {}){step} isn't connected to the model around it: none of its {} parts touches a \
             part outside it.",
            key.last().unwrap(),
            parts.len()
        );
        let hint = match below(m, &parts, &all) {
            Some((i, gap)) if gap > 0.0 => format!(
                "The copy sits {} above {} ({}): move where it's placed down by {}.",
                plural(gap, "plate"),
                m.placed[i].p.id,
                name(&m.placed[i]),
                plural(gap, "plate")
            ),
            Some((i, _)) => format!(
                "The copy rests on {} ({}) without a stud between them: move where it's placed onto whole-stud \
                 positions.",
                m.placed[i].p.id,
                name(&m.placed[i])
            ),
            None => "Nothing is under the copy: move where it's placed onto the model.".into(),
        };
        named.extend(parts.iter().copied());
        reported.push(key.clone());
        out.push(Raw::new(
            Rule::DisconnectedSubmodel,
            parts,
            dsl_box(&all).min,
            message,
            hint,
        ));
    }
    (out, named)
}

/// How far a part may reach past the studs that hold it before weak_connection warns (plan section 11).
const REACH: f64 = 4.0 * STUD;

/// weak_connection: a part held by one or two studs (and nothing else) that reaches more than 4 studs past them, or a
/// span held at one end only. Parts on the model's lowest level, figures, and flexible parts are left out.
pub(super) fn weak_connections(m: &Model, ground: &[bool]) -> Vec<Raw> {
    let mut held: Vec<Vec<&graph::Connection>> = vec![Vec::new(); m.placed.len()];
    for c in &m.links {
        held[c.a].push(c);
        held[c.b].push(c);
    }
    let pairs: HashSet<usize> = m.held.iter().flat_map(|&(a, b)| [a, b]).collect();
    let mut out = Vec::new();
    for (i, q) in m.placed.iter().enumerate() {
        let (Some(info), Some(body)) = (q.info.as_ref(), q.info.as_ref().and_then(|i| i.body)) else {
            continue;
        };
        let links = &held[i];
        if ground[i]
            || pairs.contains(&i)
            || links.is_empty()
            || links.len() > 2
            || links
                .iter()
                .any(|c| c.kind != Kind::Cyl || (c.radius - 6.0).abs() > 0.6)
            || holds::figure(&info.kind)
            || holds::flexible(&info.kind)
        {
            continue;
        }
        // Along the part's longer horizontal side.
        let size = body.max - body.min;
        let local = if size.x >= size.z { DVec3::X } else { DVec3::Z };
        let axis = (DMat3::from_mat4(q.m) * local).normalize_or_zero();
        if axis.y.abs() > 0.5 {
            continue;
        }
        let w = m.boxes[i].unwrap();
        let corners = [w.min, w.max];
        let (lo, hi) = corners.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), c| {
            let t = c.dot(axis);
            (lo.min(t), hi.max(t))
        });
        let ts: Vec<f64> = links.iter().map(|c| c.at.dot(axis)).collect();
        let (first, last) = ts
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &t| (a.min(t), b.max(t)));
        let reach = (first - lo).max(hi - last) - STUD / 2.0;
        if reach <= REACH + 0.5 {
            continue;
        }
        let studs = round(reach / STUD);
        let n = links.len();
        out.push(Raw::new(
            Rule::WeakConnection,
            vec![i],
            dsl_box(&w).min,
            format!(
                "{} ({}) is held by {} and reaches {} past {}.",
                q.p.id,
                name(q),
                plural(n as f64, "stud"),
                plural(studs, "stud"),
                if n == 1 { "it" } else { "them" }
            ),
            "Add a part under its far end, or hold it with more studs along its length.".into(),
        ));
    }
    out
}

/// How far off its seat a part can be for off_grid to say where it goes: half a stud.
const NEAR: f64 = STUD / 2.0;

/// off_grid: a part connected to nothing whose connector is a little off a mate (a typo in a position): sideways off the
/// stud grid, or up or down by part of a plate. The hint says how far to move it, in the DSL frame. Parts held some
/// other way (figures, stickers, flexible parts) and the ground are left out.
pub(super) fn off_grid(m: &Model, ground: &[bool]) -> Vec<Raw> {
    let mut linked = vec![false; m.placed.len()];
    for &(a, b) in &m.pairs() {
        linked[a] = true;
        linked[b] = true;
    }
    let mut by_part: Vec<Vec<usize>> = vec![Vec::new(); m.placed.len()];
    for (k, c) in m.conns.iter().enumerate() {
        by_part[c.part].push(k);
    }
    let mut out = Vec::new();
    for (i, q) in m.placed.iter().enumerate() {
        if linked[i] || ground[i] || q.info.is_none() || m.boxes[i].is_none() {
            continue;
        }
        // The smallest move that seats one of its connectors on a mate of another part's.
        let mut best: Option<(DVec3, usize)> = None;
        for &k in &by_part[i] {
            let c = &m.conns[k].c;
            for (j, o) in m.conns.iter().enumerate() {
                if o.part == i || !fits(c, &o.c) {
                    continue;
                }
                let d = seat_move(c, &o.c);
                // Off the grid: sideways off the stud's line, or along it by part of a plate (a whole plate's gap is
                // floating's to report).
                let along = d.dot(o.c.axis);
                let across = (d - o.c.axis * along).length();
                let plates = along.abs() / (PLATE / 2.0);
                let off = across > 0.5 || (plates - plates.round()).abs() * (PLATE / 2.0) > 0.5;
                if off && d.length() <= NEAR && best.is_none_or(|(b, _)| d.length() < b.length()) {
                    best = Some((d, j));
                }
            }
        }
        let Some((d, j)) = best else { continue };
        let other = &m.placed[m.conns[j].part];
        // The move in the DSL frame: x = −Z/20, y = X/20 studs, z = −Y/8 plates.
        let dsl = [round(-d.z / STUD), round(d.x / STUD), round(-d.y / PLATE)];
        out.push(Raw::new(
            Rule::OffGrid,
            vec![i],
            dsl_box(&m.boxes[i].unwrap()).min,
            format!(
                "{} ({}) is {} LDU off the grid: none of its connectors seats on a neighbor's.",
                q.p.id,
                name(q),
                num(round(d.length()))
            ),
            format!(
                "Move it by ({}, {}, {}) to seat it on {} ({}).",
                num(dsl[0]),
                num(dsl[1]),
                num(dsl[2]),
                other.p.id,
                name(other)
            ),
        ));
    }
    out
}

/// Whether two connectors could mate, by kind and gender.
fn fits(a: &super::connectors::Connector, b: &super::connectors::Connector) -> bool {
    a.kind == Kind::Cyl && b.kind == Kind::Cyl && a.gender != b.gender && a.axis.dot(b.axis).abs() > 0.99 && {
        let r = |c: &super::connectors::Connector| c.secs[0].radius;
        (r(a) - r(b)).abs() <= 0.6
    }
}

/// How far `a`'s part must move for `a` to start where `b` starts (a stud's base at its anti-stud's mouth).
fn seat_move(a: &super::connectors::Connector, b: &super::connectors::Connector) -> DVec3 {
    b.pos - a.pos
}
