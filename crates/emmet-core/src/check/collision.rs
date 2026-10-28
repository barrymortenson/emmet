// SPDX-License-Identifier: AGPL-3.0-or-later
//! collision (plan section 11).

use std::collections::HashSet;

use glam::DVec3;

use super::boxes::Obb;
use super::{
    OVERLAP, OVERLAP_SIDE, PLATE, Placed, Raw, Rule, SHRINK, STUD, dsl_box, dsl_point, name, num, plural, round,
    src_key,
};
use crate::ldraw::geometry::Aabb;

pub(super) fn collisions(placed: &[Placed], world: &[Option<Obb>], held: &HashSet<(usize, usize)>) -> Vec<Raw> {
    let shrunk: Vec<Option<(Obb, Aabb)>> = placed
        .iter()
        .map(|q| {
            let o = Obb::new(q.info.as_ref()?.body.as_ref()?, &q.m, SHRINK);
            Some((o, o.aabb()))
        })
        .collect();
    // Sweep along x: each box is tested only against boxes that start before it ends.
    let mut order: Vec<usize> = (0..placed.len()).filter(|&i| shrunk[i].is_some()).collect();
    order.sort_by(|&a, &b| shrunk[a].unwrap().1.min.x.total_cmp(&shrunk[b].unwrap().1.min.x));
    let mut out = Vec::new();
    for (k, &a) in order.iter().enumerate() {
        let (oa, ba) = shrunk[a].unwrap();
        for &b in &order[k + 1..] {
            let (ob, bb) = shrunk[b].unwrap();
            if bb.min.x >= ba.max.x {
                break;
            }
            if bb.min.y >= ba.max.y || ba.min.y >= bb.max.y || bb.min.z >= ba.max.z || ba.min.z >= bb.max.z {
                continue;
            }
            let Some((depth, axis)) = oa.overlap(&ob) else { continue };
            let vertical = axis.y.abs() > 0.7;
            if depth <= if vertical { OVERLAP } else { OVERLAP_SIDE } {
                continue;
            }
            if held.contains(&(a.min(b), a.max(b))) {
                continue;
            }
            let (wa, wb) = (world[a].unwrap().aabb(), world[b].unwrap().aabb());
            out.push(collision(placed, a, b, depth + 2.0 * SHRINK, axis, &wa, &wb));
        }
    }
    out
}

/// A collision finding between placements `a` and `b`, ordered so the part placed later is the one to move.
fn collision(placed: &[Placed], a: usize, b: usize, depth: f64, axis: DVec3, wa: &Aabb, wb: &Aabb) -> Raw {
    let (a, b, wa, wb) = if src_key(placed[b].p) < src_key(placed[a].p) {
        (b, a, wb, wa)
    } else {
        (a, b, wa, wb)
    };
    let (qa, qb) = (&placed[a], &placed[b]);
    let both = Aabb {
        min: wa.min.max(wb.min),
        max: wa.max.min(wb.max),
    };
    let vertical = axis.y.abs() > 0.7;
    let amount = if vertical {
        plural(round(depth / PLATE), "plate")
    } else {
        plural(round(depth / STUD), "stud")
    };
    let message = format!(
        "{} ({}) and {} ({}) overlap by {amount}.",
        qa.p.id,
        name(qa),
        qb.p.id,
        name(qb)
    );
    let hint = if vertical {
        let (da, db) = (dsl_box(wa), dsl_box(wb));
        if db.min[2] + db.max[2] >= da.min[2] + da.max[2] {
            format!(
                "{} starts at z={} but {} ends at z={}: raise {} by {amount}.",
                qb.p.id,
                num(db.min[2]),
                qa.p.id,
                num(da.max[2]),
                qb.p.id
            )
        } else {
            format!(
                "{} ends at z={} but {} starts at z={}: lower {} by {amount}.",
                qb.p.id,
                num(db.max[2]),
                qa.p.id,
                num(da.min[2]),
                qb.p.id
            )
        }
    } else {
        format!(
            "They share {amount} side by side. Move {} by {amount}, or use a shorter part.",
            qb.p.id
        )
    };
    Raw {
        rule: Rule::Collision,
        parts: vec![a, b],
        at: dsl_point((both.min + both.max) / 2.0),
        message,
        hint,
    }
}
