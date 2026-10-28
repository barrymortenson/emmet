// SPDX-License-Identifier: AGPL-3.0-or-later
//! Oriented boxes: a part's body box (studs left out) placed in the world, for its world bounds.

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
}
