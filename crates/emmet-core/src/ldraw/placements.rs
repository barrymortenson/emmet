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
    /// One entry per `submodel` name: that copy of the submodel and the parent step that places it. Empty when the
    /// writer doesn't say, and then each submodel name counts as one copy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copies: Vec<SubmodelCopy>,
    /// `file:line` of the call that placed it, or empty.
    pub src: String,
}

/// A copy of a submodel: its id (the placement's id, `towers/2`) and the step of the parent model that places it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SubmodelCopy {
    pub id: String,
    pub step: u32,
}

/// Flattens the document's first file, following references to other files in the document as submodels. Color 16
/// inherits the submodel's color, as in LDraw.
pub fn placements(doc: &Document) -> Vec<Placement> {
    let mut out = Vec::new();
    if let Some(main) = doc.files.first() {
        walk(
            doc,
            &main.lines,
            DMat4::IDENTITY,
            16,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut out,
        );
    }
    out
}

/// True for a file of the document that is a part, not a submodel: an embedded part, subpart, primitive, or
/// shortcut (`0 !LDRAW_ORG Unofficial_Part`, …), which is placed as one part like a library file.
fn embedded_part(file: &super::File) -> bool {
    file.lines.iter().any(|l| match l {
        Line::Meta(Meta::Comment(c)) => c.strip_prefix("!LDRAW_ORG").is_some_and(|rest| {
            let kind = rest.split_whitespace().next().unwrap_or("");
            let kind = kind.strip_prefix("Unofficial_").unwrap_or(kind);
            [
                "Part",
                "Subpart",
                "Primitive",
                "8_Primitive",
                "48_Primitive",
                "Shortcut",
            ]
            .contains(&kind)
        }),
        _ => false,
    })
}

fn walk(
    doc: &Document,
    lines: &[Line],
    parent: DMat4,
    parent_color: u32,
    path: &mut Vec<String>,
    copies: &mut Vec<SubmodelCopy>,
    out: &mut Vec<Placement>,
) {
    // The submodel path is the depth: a document whose submodels reference each other in a loop stops here.
    if path.len() > 16 {
        return;
    }
    let mut step = 1;
    let mut pending: Option<(String, String)> = None;
    for (n, line) in lines.iter().enumerate() {
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
                if let Some(sub) = doc.file(file).filter(|f| doc.files.len() > 1 && !embedded_part(f)) {
                    path.push(sub.name.clone().unwrap_or_default());
                    // A copy without an Emmet id is named by where it's placed: the parent copy and the line.
                    let id = if src.is_empty() && id.starts_with("part/") {
                        let parent = copies.last().map_or("main", |c| c.id.as_str());
                        format!("{parent}/{}", n + 1)
                    } else if let Some(parent) = copies.last() {
                        format!("{}/{id}", parent.id)
                    } else {
                        id
                    };
                    copies.push(SubmodelCopy { id, step });
                    walk(doc, &sub.lines, world, color, path, copies, out);
                    copies.pop();
                    path.pop();
                } else {
                    let part = normalize_ref(file);
                    let part = part.strip_suffix(".dat").unwrap_or(&part).to_string();
                    // Inside a submodel copy, an Emmet id is the copy's id plus the part's own (towers/2/wall/3), as
                    // emmet_dsl writes placements.json.
                    let id = match copies.last() {
                        Some(copy) if !src.is_empty() => format!("{}/{id}", copy.id),
                        _ => id,
                    };
                    out.push(Placement {
                        id,
                        part,
                        color,
                        matrix: world.to_cols_array(),
                        step,
                        submodel: path.clone(),
                        copies: copies.clone(),
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
