// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pins and axles that miss their holes (session 4G): for a floating part or a collision, the hole a pin or axle was
//! meant for and how far off it is, so the hint can name the hole (part_info's name for it) instead of a direction.

use glam::DMat3;

use super::{Model, STUD};
use crate::technic::{self, Point};

/// How far to one side of a hole a pin can be and still be the one meant for it: a stud and a half.
const REACH: f64 = 1.5 * STUD;

/// A pin or axle that misses a hole.
pub(super) struct Miss {
    /// The placement with the pin or axle.
    pub pin: usize,
    /// The placement with the hole, and the hole's name.
    pub part: usize,
    pub hole: String,
    /// How far the pin's line is from the hole's middle, across it (LDU).
    pub across: f64,
    /// How far the pin reaches into the hole along it (LDU): negative when it stops short.
    pub depth: f64,
    /// A round pin in an axle hole.
    pub round_in_axle_hole: bool,
}

fn world(m: &Model, i: usize) -> Vec<Point> {
    let Some(info) = m.placed[i].info.as_ref() else {
        return Vec::new();
    };
    let mat = m.placed[i].m;
    let r = super::connectors::rotation(DMat3::from_mat4(mat));
    let t = mat.w_axis.truncate();
    technic::points(&info.connectors)
        .into_iter()
        .map(|p| Point {
            mid: r * p.mid + t,
            axis: (r * p.axis).normalize_or_zero(),
            ..p
        })
        .collect()
}

/// The hole that a pin or axle of placement `i` misses most narrowly, among the parts `outside` allows: off its line
/// by up to a stud and a half, or on its line but not far enough in. None when it has no pin or axle, or it's seated.
pub(super) fn missed(m: &Model, i: usize, outside: impl Fn(usize) -> bool) -> Option<Miss> {
    let pins: Vec<Point> = world(m, i).into_iter().filter(|p| !p.kind.is_hole()).collect();
    if pins.is_empty() {
        return None;
    }
    // A part the pin is already seated in isn't the one it misses.
    let seated = |j: usize| {
        m.links
            .iter()
            .any(|c| c.shaft.is_some() && (c.a, c.b) == (i, j) || c.shaft.is_some() && (c.a, c.b) == (j, i))
    };
    let mut best: Option<Miss> = None;
    for j in (0..m.placed.len()).filter(|&j| j != i && outside(j) && !seated(j)) {
        // Only parts near enough to hold a hole the pin could reach.
        let (Some(a), Some(b)) = (m.boxes[i], m.boxes[j]) else {
            continue;
        };
        if (0..3).any(|k| a.min[k] > b.max[k] + REACH || b.min[k] > a.max[k] + REACH) {
            continue;
        }
        for h in world(m, j).into_iter().filter(|h| h.kind.is_hole()) {
            for p in &pins {
                if p.axis.dot(h.axis).abs() < 0.99 {
                    continue;
                }
                let d = h.mid - p.mid;
                let t = d.dot(p.axis);
                let across = (d - p.axis * t).length();
                // The overlap of the pin and the hole along the pin's line.
                let depth = (p.len / 2.0).min(t + h.len / 2.0) - (-p.len / 2.0).max(t - h.len / 2.0);
                if across > REACH || depth < -STUD {
                    continue;
                }
                let worse = best.as_ref().is_some_and(|b| (b.across, -b.depth) <= (across, -depth));
                if !worse {
                    best = Some(Miss {
                        pin: i,
                        part: j,
                        hole: h.name.clone(),
                        across,
                        depth,
                        round_in_axle_hole: p.kind == technic::PointKind::Pin && h.kind == technic::PointKind::AxleHole,
                    });
                }
            }
        }
    }
    best
}

fn studs(v: f64) -> String {
    let n = super::round(v / STUD);
    if n == 1.0 {
        "1 stud".into()
    } else {
        format!("{} studs", super::num(n))
    }
}

impl Miss {
    /// "pin/high is 0.5 studs to one side of brace's hole 14", "… is only 0.15 studs into …", "… stops 0.5 studs
    /// short of …", "… only touches …", or in the hole but not seated (a pin's collar in it, a round pin in an axle hole).
    pub fn says(&self, m: &Model) -> String {
        let (pin, part) = (&m.placed[self.pin].p.id, &m.placed[self.part].p.id);
        let hole = &self.hole;
        if self.across > 1.0 {
            format!("{pin} is {} to one side of {part}'s hole {hole}", studs(self.across))
        } else if self.depth < -1.0 {
            format!("{pin} stops {} short of {part}'s hole {hole}", studs(-self.depth))
        } else if self.depth <= 1.0 {
            format!("{pin} only touches {part}'s hole {hole}")
        } else if self.depth < 4.0 {
            format!("{pin} is only {} into {part}'s hole {hole}", studs(self.depth))
        } else if self.round_in_axle_hole {
            format!("{pin} is a round pin in {part}'s hole {hole}, an axle hole, which takes only an axle")
        } else {
            format!("{pin} is in {part}'s hole {hole} but doesn't fit it: its collar or its wider end is inside")
        }
    }

    /// What to do: seat it with m.insert.
    pub fn fix(&self, m: &Model) -> String {
        format!(
            "Seat it with m.insert(part, color, <{}'s handle>.hole({})): it goes in the hole's middle, all the way in.",
            m.placed[self.part].p.id, self.hole
        )
    }

    /// What to do when the hole's part is the one that's off (the pin is held by the rest of the model).
    pub fn fix_part(&self, m: &Model) -> String {
        format!(
            "Place {} by its holes (m.beam, or m.add with hole={}) so its hole {} lands on {}.",
            m.placed[self.part].p.id, self.hole, self.hole, m.placed[self.pin].p.id
        )
    }
}

/// For a floating group: a pin or axle in it that misses a hole outside it ("seat the pin"), or a hole in it that
/// misses a pin or axle the rest of the model holds ("place the beam by its holes"), as the hint. `floating` is every
/// floating part.
pub(super) fn floating_hint(m: &Model, parts: &[usize], floating: &std::collections::HashSet<usize>) -> Option<String> {
    let inside: std::collections::HashSet<usize> = parts.iter().copied().collect();
    if let Some(miss) = parts.iter().find_map(|&i| missed(m, i, |j| !inside.contains(&j))) {
        return Some(format!("{}. {}", miss.says(m), miss.fix(m)));
    }
    let near = |j: usize| {
        // A pin that's floating itself is the part to move, not the beam.
        let (Some(b), false) = (m.boxes[j], inside.contains(&j) || floating.contains(&j)) else {
            return false;
        };
        parts.iter().any(|&i| {
            m.boxes[i].is_some_and(|a| (0..3).all(|k| a.min[k] <= b.max[k] + REACH && b.min[k] <= a.max[k] + REACH))
        })
    };
    let miss = (0..m.placed.len())
        .filter(|&j| near(j))
        .filter_map(|j| missed(m, j, |k| inside.contains(&k)))
        .min_by(|a, b| (a.across, -a.depth).partial_cmp(&(b.across, -b.depth)).unwrap())?;
    Some(format!("{}. {}", miss.says(m), miss.fix_part(m)))
}

/// For a collision of `a` and `b`: one is a pin or axle beside a hole of the other, rather than in it.
pub(super) fn collision_hint(m: &Model, a: usize, b: usize) -> Option<String> {
    let miss = missed(m, a, |j| j == b).or_else(|| missed(m, b, |j| j == a))?;
    (miss.across > 1.0).then(|| {
        format!(
            "{}, so it runs into {}'s body. {}",
            miss.says(m),
            m.placed[miss.part].p.id,
            miss.fix(m)
        )
    })
}
