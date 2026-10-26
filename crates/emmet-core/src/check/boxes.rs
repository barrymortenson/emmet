// SPDX-License-Identifier: AGPL-3.0-or-later
//! Oriented boxes and their overlap (the separating axis test), for v0's collision rule: each part's body box (studs
//! left out) placed in the world, shrunk a little so parts that only touch never collide.

use glam::{DMat4, DVec3};

use crate::ldraw::geometry::Aabb;

/// A box in world space: its center, three unit axes, and the half size along each.
#[derive(Debug, Clone, Copy)]
pub struct Obb {
    pub center: DVec3,
    pub axes: [DVec3; 3],
    pub half: DVec3,
}

impl Obb {
    /// The part-local `body` box under the placement `m`, shrunk by `shrink` LDU on every side (never below zero).
    /// A mirrored or scaled matrix keeps its scale in the half sizes.
    pub fn new(body: &Aabb, m: &DMat4, shrink: f64) -> Self {
        let local_center = (body.min + body.max) / 2.0;
        let local_half = (body.max - body.min) / 2.0;
        let cols = [m.x_axis.truncate(), m.y_axis.truncate(), m.z_axis.truncate()];
        let mut axes = [DVec3::X, DVec3::Y, DVec3::Z];
        let mut half = [0.0; 3];
        for i in 0..3 {
            let len = cols[i].length();
            if len > 1e-9 {
                axes[i] = cols[i] / len;
            }
            half[i] = (local_half[i] * len - shrink).max(0.0);
        }
        Self {
            center: m.transform_point3(local_center),
            axes,
            half: DVec3::from_array(half),
        }
    }

    /// The world axis-aligned box around this one.
    pub fn aabb(&self) -> Aabb {
        let extent = DVec3::new(self.radius(DVec3::X), self.radius(DVec3::Y), self.radius(DVec3::Z));
        Aabb {
            min: self.center - extent,
            max: self.center + extent,
        }
    }

    /// Half the box's length along the unit axis `l`.
    fn radius(&self, l: DVec3) -> f64 {
        (0..3).map(|i| self.half[i] * self.axes[i].dot(l).abs()).sum()
    }

    /// How deep the two boxes overlap: the smallest overlap along any separating axis, and that axis. None when an
    /// axis separates them (they don't overlap, or only touch).
    pub fn overlap(&self, other: &Obb) -> Option<(f64, DVec3)> {
        let t = other.center - self.center;
        let mut best: Option<(f64, DVec3)> = None;
        let mut test = |l: DVec3| -> bool {
            let len = l.length();
            if len < 1e-6 {
                return true; // parallel edges: this cross product is no axis
            }
            let l = l / len;
            let depth = self.radius(l) + other.radius(l) - t.dot(l).abs();
            if depth <= 0.0 {
                return false;
            }
            if best.is_none_or(|(d, _)| depth < d) {
                best = Some((depth, l));
            }
            true
        };
        for a in self.axes {
            if !test(a) {
                return None;
            }
        }
        for b in other.axes {
            if !test(b) {
                return None;
            }
        }
        for a in self.axes {
            for b in other.axes {
                if !test(a.cross(b)) {
                    return None;
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brick(at: DVec3) -> Obb {
        // A 1 x 2 brick's body: 40 × 24 × 20 LDU.
        let body = Aabb {
            min: DVec3::new(-20.0, 0.0, -10.0),
            max: DVec3::new(20.0, 24.0, 10.0),
        };
        Obb::new(&body, &DMat4::from_translation(at), 1.0)
    }

    #[test]
    fn touching_parts_dont_overlap() {
        assert!(brick(DVec3::ZERO).overlap(&brick(DVec3::new(40.0, 0.0, 0.0))).is_none());
        assert!(
            brick(DVec3::ZERO)
                .overlap(&brick(DVec3::new(0.0, -24.0, 0.0)))
                .is_none()
        );
    }

    #[test]
    fn a_part_a_plate_too_low_overlaps_by_a_plate() {
        let (depth, axis) = brick(DVec3::ZERO)
            .overlap(&brick(DVec3::new(0.0, -16.0, 0.0)))
            .expect("overlap");
        // 8 LDU, less the 1 LDU shrink on each part.
        assert!((depth - 6.0).abs() < 1e-9, "{depth}");
        assert!(axis.y.abs() > 0.99);
    }

    #[test]
    fn a_turned_part_is_tested_as_turned() {
        // A 1 x 2 brick turned 45° about Y, off another's corner: the boxes' world bounds overlap, the bricks don't.
        let body = Aabb {
            min: DVec3::new(-20.0, 0.0, -10.0),
            max: DVec3::new(20.0, 24.0, 10.0),
        };
        let turned =
            DMat4::from_translation(DVec3::new(25.0, 0.0, 25.0)) * DMat4::from_rotation_y(std::f64::consts::FRAC_PI_4);
        let a = brick(DVec3::ZERO);
        let b = Obb::new(&body, &turned, 1.0);
        assert!(
            a.aabb().max.x > b.aabb().min.x && a.aabb().max.z > b.aabb().min.z,
            "the world bounds overlap"
        );
        assert!(a.overlap(&b).is_none());
    }
}
