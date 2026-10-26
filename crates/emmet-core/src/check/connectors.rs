// SPDX-License-Identifier: AGPL-3.0-or-later
//! Stud connectors from LDraw's primitives (plan section 11, "Primitive scan"): a part's references to the stud
//! primitives in [`SYSTEM_STUDS`](crate::ldraw::geometry::SYSTEM_STUDS), at any depth, become studs at their
//! transformed positions. This is v0's table: top studs
//! only. The female side is the receiving part's underside (see `check::floating`), and 3A adds the anti-stud
//! primitives, the LDCad shadow data, and the overrides.

use std::collections::HashMap;
use std::rc::Rc;

use glam::{DMat3, DVec3};

use crate::ldraw::geometry::{Measurer, is_system_stud};
use crate::ldraw::{Line, normalize_ref};

/// One stud, in its part's LDraw frame: the center of its base, the unit direction it points (−Y for a stud on top of
/// an unrotated part), and how far it reaches (4 LDU, more for a stretched one whose base is sunk into its part).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stud {
    pub pos: DVec3,
    pub dir: DVec3,
    pub len: f64,
}

/// A stud primitive's height.
const STUD_LEN: f64 = 4.0;

const MAX_DEPTH: usize = 32;

/// Finds the studs of parts, caching each file's studs in its own frame.
pub struct Scanner<'a> {
    measurer: Measurer<'a>,
    files: HashMap<String, Rc<Vec<Stud>>>,
}

impl<'a> Scanner<'a> {
    pub fn new(measurer: Measurer<'a>) -> Self {
        Self {
            measurer,
            files: HashMap::new(),
        }
    }

    /// Every top stud of a part file (`3001.dat`), in the part's frame. Empty for a part with none or a file that's
    /// missing.
    pub fn studs(&mut self, reference: &str) -> Rc<Vec<Stud>> {
        self.file(reference, 0)
    }

    fn file(&mut self, reference: &str, depth: usize) -> Rc<Vec<Stud>> {
        let key = normalize_ref(reference);
        if let Some(found) = self.files.get(&key) {
            return found.clone();
        }
        let mut out = Vec::new();
        if depth <= MAX_DEPTH
            && let Some(lines) = self.measurer.lines(&key)
        {
            for line in lines.iter() {
                let Line::Sub { pos, matrix, file, .. } = line else {
                    continue;
                };
                if is_system_stud(file) {
                    out.push(along(*pos, *matrix * (DVec3::NEG_Y * STUD_LEN)));
                } else {
                    for s in self.file(file, depth + 1).iter() {
                        out.push(along(*matrix * s.pos + *pos, *matrix * (s.dir * s.len)));
                    }
                }
            }
        }
        let out = Rc::new(out);
        self.files.insert(key, out.clone());
        out
    }
}

/// A stud at `pos` reaching along `reach`; a degenerate one (a primitive scaled flat) points up and doesn't reach.
fn along(pos: DVec3, reach: DVec3) -> Stud {
    let len = reach.length();
    Stud {
        pos,
        dir: reach.try_normalize().unwrap_or(DVec3::NEG_Y),
        len: if len.is_finite() { len } else { 0.0 },
    }
}

/// The rotation part of a placement's matrix with its scale taken out, for directions.
pub fn rotation(m: DMat3) -> DMat3 {
    let n = |v: DVec3| v.try_normalize().unwrap_or(v);
    DMat3::from_cols(n(m.x_axis), n(m.y_axis), n(m.z_axis))
}
