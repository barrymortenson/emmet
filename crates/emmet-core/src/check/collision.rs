// SPDX-License-Identifier: AGPL-3.0-or-later
//! collision (plan section 11, ADR 0013): the collision proxies, an AABB tree, and parry3d's narrow phase.

use std::collections::HashMap;

use glam::{DMat3, DVec3};
use parry3d_f64::bounding_volume::Aabb as ParryAabb;
use parry3d_f64::partitioning::{Bvh, BvhBuildStrategy};
use parry3d_f64::shape::Cuboid;

use super::proxy::ProxyBox;
use super::{Model, PLATE, Placed, Raw, Rule, STUD, dsl_box, dsl_point, graph, name, num, plural, round, src_key};
use crate::ldraw::geometry::Aabb;

/// How deep two parts' proxies must overlap to collide: more than their 1 LDU voxels can blur (Barry, 2026-10-28; a
/// sloped face can stick out half a voxel, so touching slopes read as a 1 LDU overlap).
const DEEP: f64 = 1.0;
/// How far around a connected pair's seat an overlap is exempt: a stud in its anti-stud, a pin in its hole.
const SEAT_ROOM: f64 = 2.0;

/// collision: pairs of parts whose proxies overlap by more than 1 LDU. Candidates come from an AABB tree over the
/// parts' world boxes; each pair's proxy boxes are compared in the first part's frame (exactly when the second is turned
/// by quarter turns, by parry's cuboid contact otherwise). Pairs one part holds the other are exempt, and so is an
/// overlap inside a connected pair's seat.
pub(super) fn collisions(m: &Model) -> Vec<Raw> {
    let n = m.placed.len();
    let solid: Vec<bool> = (0..n)
        .map(|i| !m.proxies[i].is_empty() && m.boxes[i].is_some())
        .collect();
    let leaves: Vec<(usize, ParryAabb)> = (0..n)
        .filter(|&i| solid[i])
        .map(|i| {
            let b = m.boxes[i].unwrap();
            (i, ParryAabb::new(pv(b.min), pv(b.max)))
        })
        .collect();
    if leaves.len() < 2 {
        return Vec::new();
    }
    let bvh = Bvh::from_iter(BvhBuildStrategy::Binned, leaves.iter().cloned());
    let mut seats: HashMap<(usize, usize), Vec<&graph::Connection>> = HashMap::new();
    for c in &m.links {
        seats.entry((c.a.min(c.b), c.a.max(c.b))).or_default().push(c);
    }
    let mut out = Vec::new();
    for (i, aabb) in &leaves {
        for j in bvh.intersect_aabb(aabb) {
            let j = j as usize;
            if j <= *i || m.held.contains(&(*i, j)) {
                continue;
            }
            let none = Vec::new();
            let at_seats = seats.get(&(*i, j)).unwrap_or(&none);
            if let Some((depth, axis)) = deepest(m, *i, j, at_seats)
                && depth > DEEP + 1e-6
            {
                // The proxies say whether they collide; the body boxes say by how much, and which way to move.
                let (depth, axis) = body_overlap(m, *i, j).unwrap_or((depth, axis));
                let (wa, wb) = (m.boxes[*i].unwrap(), m.boxes[j].unwrap());
                out.push(collision(&m.placed, *i, j, depth, axis, &wa, &wb));
            }
        }
    }
    out
}

/// How far placements `a` and `b`'s body boxes (oriented, in the world) overlap, and the axis to separate them along:
/// the move that undoes the collision.
fn body_overlap(m: &Model, a: usize, b: usize) -> Option<(f64, DVec3)> {
    use parry3d_f64::math::{Matrix, Pose, Rotation};
    let (oa, ob) = (m.world[a]?, m.world[b]?);
    let pose = |o: &super::boxes::Obb| {
        let mut ax = o.axes;
        if ax[0].cross(ax[1]).dot(ax[2]) < 0.0 {
            ax[2] = -ax[2];
        }
        Pose::from_parts(
            pv(o.center),
            Rotation::from_mat3(&Matrix::from_cols(pv(ax[0]), pv(ax[1]), pv(ax[2]))),
        )
    };
    let (pa, pb) = (pose(&oa), pose(&ob));
    let c = parry3d_f64::query::contact(&pa, &Cuboid::new(pv(oa.half)), &pb, &Cuboid::new(pv(ob.half)), 0.0).ok()??;
    (c.dist < 0.0).then(|| (-c.dist, DVec3::new(c.normal1.x, c.normal1.y, c.normal1.z)))
}

fn pv(v: DVec3) -> parry3d_f64::math::Vector {
    parry3d_f64::math::Vector::new(v.x, v.y, v.z)
}

fn box_min(b: &ProxyBox) -> DVec3 {
    DVec3::new(b[0] as f64, b[1] as f64, b[2] as f64)
}

fn box_max(b: &ProxyBox) -> DVec3 {
    DVec3::new(b[3] as f64, b[4] as f64, b[5] as f64)
}

/// The deepest overlap of placements `a` and `b`'s proxies outside their seats: (depth, the world axis of least
/// overlap).
fn deepest(m: &Model, a: usize, b: usize, seats: &[&graph::Connection]) -> Option<(f64, DVec3)> {
    let (ma, mb) = (m.placed[a].m, m.placed[b].m);
    let rel = ma.inverse() * mb;
    let r = DMat3::from_mat4(rel);
    let t = rel.w_axis.truncate();
    let quarter = (0..3).all(|c| {
        let col = r.col(c);
        let big = col.abs().max_element();
        (big - 1.0).abs() < 1e-6 && (col.abs().element_sum() - big) < 1e-6
    });
    let ra = DMat3::from_mat4(ma);
    let in_seat = |p_local: DVec3| {
        let p = ma.transform_point3(p_local);
        seats.iter().any(|c| {
            let v = p - c.at;
            let along = v.dot(c.axis);
            along.abs() <= c.len / 2.0 + SEAT_ROOM && (v - c.axis * along).length() <= c.radius + SEAT_ROOM
        })
    };
    let pa = &m.proxies[a];
    let mut best: Option<(f64, DVec3)> = None;
    for bb in m.proxies[b].iter() {
        let (lo, hi) = (box_min(bb), box_max(bb));
        // b's box in a's frame: its corners, and the box around them.
        let (mut blo, mut bhi) = (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY));
        for k in 0..8 {
            let c = DVec3::new(
                if k & 1 == 0 { lo.x } else { hi.x },
                if k & 2 == 0 { lo.y } else { hi.y },
                if k & 4 == 0 { lo.z } else { hi.z },
            );
            let w = r * c + t;
            blo = blo.min(w);
            bhi = bhi.max(w);
        }
        for ab in pa.iter() {
            let (alo, ahi) = (box_min(ab), box_max(ab));
            let over = ahi.min(bhi) - alo.max(blo);
            if over.min_element() <= DEEP {
                continue;
            }
            let (depth, axis_local, center) = if quarter {
                let k = if over.x <= over.y && over.x <= over.z {
                    0
                } else if over.y <= over.z {
                    1
                } else {
                    2
                };
                let mut axis = DVec3::ZERO;
                axis[k] = 1.0;
                (over[k], axis, (ahi.min(bhi) + alo.max(blo)) / 2.0)
            } else {
                match turned(alo, ahi, lo, hi, r, t) {
                    Some(x) => x,
                    None => continue,
                }
            };
            if depth <= DEEP || in_seat(center) {
                continue;
            }
            if best.is_none_or(|(d, _)| depth > d) {
                best = Some((depth, (ra * axis_local).normalize_or_zero()));
            }
        }
    }
    best
}

/// The overlap of an axis-aligned box (`alo`..`ahi`) and a box `lo`..`hi` under `r`, `t` (a turned part), by parry's
/// cuboid contact: (depth, the contact normal, the contact's middle), all in the first box's frame.
fn turned(alo: DVec3, ahi: DVec3, lo: DVec3, hi: DVec3, r: DMat3, t: DVec3) -> Option<(f64, DVec3, DVec3)> {
    use parry3d_f64::math::{Matrix, Pose, Rotation};
    let ca = (alo + ahi) / 2.0;
    let cb = r * ((lo + hi) / 2.0) + t;
    // b's axes in a's frame, unit length, made right-handed (a mirrored box is the same box).
    let mut axes = [r.col(0), r.col(1), r.col(2)];
    let half_b = (hi - lo) / 2.0 * DVec3::new(axes[0].length(), axes[1].length(), axes[2].length());
    for v in axes.iter_mut() {
        *v = v.normalize_or_zero();
    }
    if axes[0].cross(axes[1]).dot(axes[2]) < 0.0 {
        axes[2] = -axes[2];
    }
    let rot = Rotation::from_mat3(&Matrix::from_cols(pv(axes[0]), pv(axes[1]), pv(axes[2])));
    let pos12 = Pose::from_parts(pv(cb - ca), rot);
    let ca_box = Cuboid::new(pv((ahi - alo) / 2.0));
    let cb_box = Cuboid::new(pv(half_b));
    let c = parry3d_f64::query::contact::contact_cuboid_cuboid(&pos12, &ca_box, &cb_box, 0.0)?;
    if c.dist >= 0.0 {
        return None;
    }
    let n = DVec3::new(c.normal1.x, c.normal1.y, c.normal1.z);
    let p1 = DVec3::new(c.point1.x, c.point1.y, c.point1.z) + ca;
    let p2 = pos12_point(&pos12, c.point2) + ca;
    Some((-c.dist, n, (p1 + p2) / 2.0))
}

fn pos12_point(pos12: &parry3d_f64::math::Pose, p: parry3d_f64::math::Vector) -> DVec3 {
    let w = *pos12 * p;
    DVec3::new(w.x, w.y, w.z)
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
    Raw::new(
        Rule::Collision,
        vec![a, b],
        dsl_point((both.min + both.max) / 2.0),
        message,
        hint,
    )
}
