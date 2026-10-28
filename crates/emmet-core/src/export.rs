// SPDX-License-Identifier: AGPL-3.0-or-later
//! LDraw exports (plan section 09's `emmet export`), from build/model.mpd: a flat `.ldr` (every part in one file, the
//! main model's steps kept), a clean `.mpd` (submodels as files), and an LPub3D-ready `.mpd` (each sub-assembly a
//! callout, so its steps print in a framed box beside the step that puts it on). Emmet's own `0 !EMMET` lines are left
//! out of all three; the PDF is 4E's.

use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::DataDir;
use crate::envelope::{Error, Result};
use crate::ldraw::{self, Document, File, Line, Meta};
use crate::project::Project;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Ldr,
    Mpd,
    Lpub3d,
}

impl Format {
    fn file_name(self, model: &str) -> String {
        match self {
            Format::Ldr => format!("{model}.ldr"),
            Format::Mpd => format!("{model}.mpd"),
            Format::Lpub3d => format!("{model}-lpub3d.mpd"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Exported {
    pub format: String,
    /// Where the file went.
    pub path: String,
    pub parts: usize,
    pub bytes: usize,
}

/// Writes the export to `out`, or to build/export/ (running the model first if build/ is stale).
pub fn export(data: &DataDir, project: &Project, format: Format, out: Option<&Path>) -> Result<Exported> {
    crate::ops::ensure_built(data, project)?;
    let path = project.root.join("build/model.mpd");
    let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    let doc = ldraw::parse(&text).map_err(|e| Error::internal(format!("build/model.mpd: {e}")))?;
    let parts = ldraw::placements::placements(&doc).len();
    let model = doc
        .files
        .first()
        .and_then(|f| f.name.as_deref())
        .map_or("model", |n| n.trim_end_matches(".ldr"))
        .to_string();
    let text = match format {
        Format::Ldr => flat(&doc, &model),
        Format::Mpd => ldraw::write(&clean(&doc, false)),
        Format::Lpub3d => ldraw::write(&clean(&doc, true)),
    };
    let target: PathBuf = match out {
        Some(p) => p.to_path_buf(),
        None => project.root.join("build/export").join(format.file_name(&model)),
    };
    crate::fsx::write_atomic(&target, text.as_bytes())?;
    Ok(Exported {
        format: match format {
            Format::Ldr => "ldr",
            Format::Mpd => "mpd",
            Format::Lpub3d => "lpub3d",
        }
        .into(),
        path: target.display().to_string(),
        parts,
        bytes: text.len(),
    })
}

fn emmet_line(line: &Line) -> bool {
    matches!(line, Line::Meta(Meta::EmmetPart { .. } | Meta::Emmet(_)))
}

/// The document without `0 !EMMET` lines; for LPub3D, each run of references to one submodel in a step wrapped in
/// `0 !LPUB CALLOUT BEGIN` / `END`.
fn clean(doc: &Document, lpub: bool) -> Document {
    let submodel = |name: &str| doc.files.len() > 1 && doc.file(name).is_some();
    let files = doc
        .files
        .iter()
        .map(|f| {
            let mut lines: Vec<Line> = Vec::new();
            let mut open: Option<String> = None;
            for line in f.lines.iter().filter(|l| !emmet_line(l)) {
                let target = match line {
                    Line::Sub { file, .. } if lpub && submodel(file) => Some(ldraw::normalize_ref(file)),
                    _ => None,
                };
                if open.is_some() && open != target {
                    lines.push(Line::Meta(Meta::Comment("!LPUB CALLOUT END".into())));
                    open = None;
                }
                if target.is_some() && open.is_none() {
                    lines.push(Line::Meta(Meta::Comment("!LPUB CALLOUT BEGIN".into())));
                    open = target;
                }
                lines.push(line.clone());
            }
            if open.is_some() {
                lines.push(Line::Meta(Meta::Comment("!LPUB CALLOUT END".into())));
            }
            File {
                name: f.name.clone(),
                lines,
            }
        })
        .collect();
    Document { files }
}

/// One file: the header, then every part in world coordinates, a part inside a submodel in the step that places its
/// copy.
fn flat(doc: &Document, model: &str) -> String {
    let mut placed = ldraw::placements::placements(doc);
    placed.sort_by_key(|p| p.copies.first().map_or(p.step, |c| c.step));
    let title = doc
        .files
        .first()
        .and_then(|f| {
            f.lines.iter().find_map(|l| match l {
                Line::Meta(Meta::Comment(c)) if !c.is_empty() && !c.starts_with('!') && !c.contains(':') => {
                    Some(c.clone())
                }
                _ => None,
            })
        })
        .unwrap_or_else(|| model.to_string());
    let mut out = format!("0 {title}\n0 Name: {model}.ldr\n\n");
    let mut step = placed
        .first()
        .map_or(1, |p| p.copies.first().map_or(p.step, |c| c.step));
    for p in &placed {
        let s = p.copies.first().map_or(p.step, |c| c.step);
        if s != step {
            out.push_str("0 STEP\n");
            step = s;
        }
        let m = glam::DMat4::from_cols_array(&p.matrix);
        let line = Line::Sub {
            color: p.color,
            pos: m.w_axis.truncate(),
            matrix: glam::DMat3::from_mat4(m),
            file: format!("{}.dat", p.part),
        };
        out.push_str(&ldraw::write_line(&line));
        out.push('\n');
    }
    out.push_str("0 STEP\n");
    out
}
