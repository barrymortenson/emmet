// SPDX-License-Identifier: AGPL-3.0-or-later
//! Flattens an .mpd into the same rows as `build/placements.json` (plan section 07), which emmet_dsl writes, so
//! tests can check the two agree. Opening a bare .mpd (import, later) uses it too.

use glam::{DMat3, DMat4, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Document, Line, Meta, normalize_ref};

/// One placed part in LDraw's frame (LDU, −Y up).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Placement {
    /// The Emmet id (`0 !EMMET ID`), or `part/<n>` for a part without one.
    pub id: String,
    /// The LDraw part id, lowercased, without `.dat`.
    pub part: String,
    /// The LDraw color code.
    pub color: u32,
    /// The 4 × 4 transform, column-major (three.js `Matrix4.fromArray` order).
    pub matrix: [f64; 16],
    /// 1-based build step within its submodel.
    pub step: u32,
    /// The submodel names from the main model down to this part; empty for the main model.
    pub submodel: Vec<String>,
    /// `file:line` of the call that placed it, or empty.
    pub src: String,
}

/// Flattens the document's first file, following references to other files in the document as submodels. Color 16
/// inherits the submodel's color, as in LDraw.
pub fn placements(doc: &Document) -> Vec<Placement> {
    let mut out = Vec::new();
    if let Some(main) = doc.files.first() {
        walk(doc, &main.lines, DMat4::IDENTITY, 16, &mut Vec::new(), &mut out, 0);
    }
    out
}

fn walk(
    doc: &Document,
    lines: &[Line],
    parent: DMat4,
    parent_color: u32,
    path: &mut Vec<String>,
    out: &mut Vec<Placement>,
    depth: usize,
) {
    if depth > 16 {
        return;
    }
    let mut step = 1;
    let mut pending: Option<(String, String)> = None;
    for line in lines {
        match line {
            Line::Meta(Meta::Step) => step += 1,
            Line::Meta(Meta::EmmetPart { id, src }) => pending = Some((id.clone(), src.clone())),
            Line::Sub {
                color,
                pos,
                matrix,
                file,
            } => {
                let color = if *color == 16 { parent_color } else { *color };
                let local = to_mat4(*matrix, *pos);
                let world = parent * local;
                let (id, src) = pending
                    .take()
                    .unwrap_or_else(|| (format!("part/{}", out.len() + 1), String::new()));
                if let Some(sub) = doc.file(file).filter(|_| doc.files.len() > 1) {
                    path.push(sub.name.clone().unwrap_or_default());
                    walk(doc, &sub.lines, world, color, path, out, depth + 1);
                    path.pop();
                } else {
                    let part = normalize_ref(file);
                    let part = part.strip_suffix(".dat").unwrap_or(&part).to_string();
                    out.push(Placement {
                        id,
                        part,
                        color,
                        matrix: world.to_cols_array(),
                        step,
                        submodel: path.clone(),
                        src,
                    });
                }
            }
            _ => {}
        }
    }
}

pub fn to_mat4(m: DMat3, t: DVec3) -> DMat4 {
    DMat4::from_cols(
        m.x_axis.extend(0.0),
        m.y_axis.extend(0.0),
        m.z_axis.extend(0.0),
        t.extend(1.0),
    )
}
