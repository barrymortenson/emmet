// SPDX-License-Identifier: AGPL-3.0-or-later
//! The rules on the connection graph (plan section 11): floating.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use super::boxes::Obb;
use super::{GROUND, ON_THE_TABLE, Placed, Raw, Rule, SEAT, UnionFind, dsl_box, graph, name, num, plural, round};
use crate::ldraw::geometry::Aabb;

/// floating: the connection graph's components; every component with a part on the model's lowest level stands, and
/// every other component floats. A frame holds what fills it.
pub(super) fn floating(
    placed: &[Placed],
    world: &[Option<Obb>],
    links: &[graph::Connection],
    held: &HashSet<(usize, usize)>,
) -> Vec<Raw> {
    let n = placed.len();
    let boxes: Vec<Option<Aabb>> = world.iter().map(|o| o.map(|o| o.aabb())).collect();
    let mut sets = UnionFind::new(n);
    for c in links {
        sets.join(c.a, c.b);
    }
    for &(i, j) in held {
        sets.join(i, j);
    }
    // The ground: the model's lowest point (LDraw's −Y is up, so the largest y).
    let lowest = boxes
        .iter()
        .flatten()
        .map(|b| b.max.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut grounded = vec![false; n];
    for (i, b) in boxes.iter().enumerate() {
        let base = placed[i].info.as_ref().is_some_and(|i| i.kind.category == "Baseplate");
        if base || b.is_some_and(|b| b.max.y >= lowest - GROUND) {
            let r = sets.root(i);
            grounded[r] = true;
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, b) in boxes.iter().enumerate() {
        if b.is_some() && !grounded[sets.root(i)] {
            groups.entry(sets.root(i)).or_default().push(i);
        }
    }
    // Without a baseplate, a model can be several things on the table (a car, a figure beside it), each at its own
    // height off it: a group with nothing of the model under it, within two plates of the lowest point, stands.
    let baseplate = placed
        .iter()
        .any(|q| q.info.as_ref().is_some_and(|i| i.kind.category == "Baseplate"));
    if !baseplate {
        groups.retain(|_, parts| {
            let bottom = parts
                .iter()
                .map(|&i| boxes[i].unwrap().max.y)
                .fold(f64::NEG_INFINITY, f64::max);
            let members: HashSet<usize> = parts.iter().copied().collect();
            let over_something = parts.iter().any(|&i| {
                let a = boxes[i].unwrap();
                boxes.iter().enumerate().any(|(j, b)| {
                    !members.contains(&j)
                        && b.is_some_and(|b| {
                            b.min.y >= a.max.y - 1.0
                                && a.min.x + 2.0 < b.max.x
                                && b.min.x + 2.0 < a.max.x
                                && a.min.z + 2.0 < b.max.z
                                && b.min.z + 2.0 < a.max.z
                        })
                })
            });
            over_something || bottom < lowest - ON_THE_TABLE
        });
    }
    groups
        .into_values()
        .map(|parts| floating_finding(placed, &boxes, parts))
        .collect()
}

fn floating_finding(placed: &[Placed], boxes: &[Option<Aabb>], parts: Vec<usize>) -> Raw {
    let mut all = boxes[parts[0]].unwrap();
    for &i in &parts[1..] {
        let b = boxes[i].unwrap();
        all = Aabb {
            min: all.min.min(b.min),
            max: all.max.max(b.max),
        };
    }
    let here = dsl_box(&all);
    let z = num(here.min[2]);
    let message = if parts.len() == 1 {
        let q = &placed[parts[0]];
        format!(
            "{} ({}) at z={z} isn't held by any stud: it touches nothing below it.",
            q.p.id,
            name(q)
        )
    } else {
        format!(
            "{} parts at z={z} aren't held by any stud: they touch nothing below them.",
            parts.len()
        )
    };
    // The highest part under the group, by footprint.
    let members: BTreeSet<usize> = parts.iter().copied().collect();
    let below = boxes
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
        .max_by(|a, b| b.1.min.y.total_cmp(&a.1.min.y));
    let hint = match below {
        Some((i, b)) => {
            let top = dsl_box(&b).max[2];
            let gap = round(here.min[2] - top);
            let q = &placed[i];
            if gap > 0.0 {
                format!(
                    "They start {} above {} ({}), which ends at z={}: lower them by {}, or put a part under them.",
                    plural(gap, "plate"),
                    q.p.id,
                    name(q),
                    num(top),
                    plural(gap, "plate")
                )
            } else {
                format!(
                    "They rest on {} ({}) without a stud between them: put them on whole-stud positions, or on a part \
                     with studs.",
                    q.p.id,
                    name(q)
                )
            }
        }
        None => "Nothing is under them: give them a part to stand on, or move them down onto the model.".into(),
    };
    Raw {
        rule: Rule::Floating,
        at: here.min,
        parts,
        message,
        hint,
    }
}
