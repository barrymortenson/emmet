// SPDX-License-Identifier: AGPL-3.0-or-later
//! Building instructions as a PDF (plan sections 08 and 14, session 4E, ADR 0014), in three parts:
//!
//! 1. [`jobs`]: what a book needs drawn, from build/steps.json and build/placements.json: the cover, every step of the
//!    main model and of each sub-assembly (in groups that share a camera), and every part × color in a parts box.
//! 2. The webview draws them through `viewer.render` (apps/desktop/src/instructions.ts) and returns [`Drawn`]: JPEGs,
//!    and what the layout needs to know about each picture.
//! 3. [`book`] lays the pages out by section 08's step rules as a display list, and [`pdf::write`] writes it.

pub mod font;
pub mod layout;
pub mod pdf;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::bom::Bom;
use crate::data::DataDir;
use crate::envelope::{Error, ErrorCode, Result};
use crate::ldraw::placements::Placement;
use crate::project::Project;
use crate::steps::Steps;

pub use layout::{Book, Item, Page, Paper, book};

/// A step's render, a part's, and the cover's, in pixels a side (before the crop).
pub const STEP_SIZE: u32 = 1000;
pub const PART_SIZE: u32 = 160;
pub const COVER_SIZE: u32 = 1600;

/// What the webview draws for one book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Jobs {
    pub step_size: u32,
    pub part_size: u32,
    pub cover_size: u32,
    /// The main model first, then each sub-assembly.
    pub runs: Vec<Run>,
    /// Each part × color a parts box or the parts list shows.
    pub parts: Vec<PartJob>,
    /// What's behind the pictures, so a JPEG sits on its box without an edge: the page (the cover and the main
    /// model's steps), a sub-assembly's box, and the parts box.
    pub page_background: [u8; 3],
    pub callout_background: [u8; 3],
    pub part_background: [u8; 3],
}

/// One scene's steps: the main model's, or one copy of a sub-assembly's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Run {
    /// "" for the main model, else the sub-assembly's file in model.mpd.
    pub file: String,
    /// The copy drawn: its parts are the placements whose `copies[depth]` is this id. "" for the main model.
    pub copy: String,
    /// How deep the sub-assembly sits: 0 for one the main model places.
    pub depth: u32,
    /// The steps in order, in groups that share one camera (a run of steps under one title).
    pub groups: Vec<Vec<StepJob>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct StepJob {
    pub n: u32,
    /// The ids of the parts the step adds, as the scene has them.
    pub new: Vec<String>,
    /// The sub-assembly copies it puts on: where each goes is the end of the line from its box.
    pub copies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PartJob {
    pub part: String,
    pub color: u32,
}

/// What the webview drew, in the order of [`Jobs`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Drawn {
    pub cover: Picture,
    /// One per run, each with its steps in order.
    pub runs: Vec<Vec<DrawnStep>>,
    pub parts: Vec<Picture>,
}

/// A JPEG, base64.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Picture {
    pub jpeg: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DrawnStep {
    pub n: u32,
    /// The build up to the step, its new parts outlined, cropped the same for every step of its group.
    pub picture: Picture,
    /// The picture's pixels per LDU at the model's center: how big a brick is, for one scale through the book.
    pub ldu: f64,
    /// Quarter turns of the model from the first step's view.
    pub turn: u32,
    /// Where each copy the step puts on is in the picture (pixels).
    pub copies: Vec<Spot>,
    /// Arrows for parts that don't go on from above, in the picture's pixels.
    pub arrows: Vec<Arrow>,
    /// A closer look at new parts that are hidden in the picture, and where they are in it.
    pub zoom: Option<Zoom>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Spot {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Arrow {
    pub from: [f64; 2],
    pub to: [f64; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Zoom {
    pub picture: Picture,
    pub x: f64,
    pub y: f64,
}

/// The jobs for a built project's book.
pub fn jobs(steps: &Steps, placements: &[Placement]) -> Jobs {
    let mut runs = vec![Run {
        file: String::new(),
        copy: String::new(),
        depth: 0,
        groups: groups(steps.steps.iter().map(|s| {
            (
                s.group.as_str(),
                StepJob {
                    n: s.n,
                    new: s.part_ids.clone(),
                    copies: s.submodels.iter().flat_map(|p| p.ids.clone()).collect(),
                },
            )
        })),
    }];
    for sub in &steps.submodels {
        // The first copy anywhere in the model: the placements whose submodel chain names this file.
        let Some((depth, copy)) = placements.iter().find_map(|p| {
            let d = p.submodel.iter().position(|f| f.eq_ignore_ascii_case(&sub.file))?;
            Some((d, p.copies.get(d)?.id.clone()))
        }) else {
            continue;
        };
        let full = |id: &str| format!("{copy}/{id}");
        runs.push(Run {
            file: sub.file.clone(),
            copy: copy.clone(),
            depth: depth as u32,
            // A sub-assembly's steps share one camera, so it stays put through its box.
            groups: vec![
                sub.steps
                    .iter()
                    .map(|s| StepJob {
                        n: s.n,
                        new: s.part_ids.iter().map(|i| full(i)).collect(),
                        copies: s.submodels.iter().flat_map(|p| p.ids.iter().map(|i| full(i))).collect(),
                    })
                    .collect(),
            ],
        });
    }
    let mut parts: Vec<PartJob> = placements
        .iter()
        .map(|p| PartJob {
            part: p.part.clone(),
            color: p.color,
        })
        .collect();
    parts.sort();
    parts.dedup();
    Jobs {
        step_size: STEP_SIZE,
        part_size: PART_SIZE,
        cover_size: COVER_SIZE,
        runs,
        parts,
        page_background: [255, 255, 255],
        callout_background: layout::CALLOUT_FILL,
        part_background: layout::BOX_FILL,
    }
}

/// Consecutive steps with the same title group share a camera.
fn groups<'a>(steps: impl Iterator<Item = (&'a str, StepJob)>) -> Vec<Vec<StepJob>> {
    let mut out: Vec<Vec<StepJob>> = Vec::new();
    let mut last: Option<&str> = None;
    for (group, step) in steps {
        match out.last_mut() {
            Some(g) if last == Some(group) => g.push(step),
            _ => out.push(vec![step]),
        }
        last = Some(group);
    }
    out
}

/// What a book is made from: the project's steps, parts, and name, and what the webview drew.
pub struct Sources<'a> {
    pub title: &'a str,
    pub age: Option<&'a str>,
    pub steps: &'a Steps,
    pub bom: &'a Bom,
    pub jobs: &'a Jobs,
    pub drawn: &'a Drawn,
}

/// What an export wrote.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Written {
    pub path: String,
    pub pages: usize,
    pub bytes: usize,
    pub paper: String,
}

/// The project's jobs, running the model first if build/ is stale.
pub fn project_jobs(data: &DataDir, project: &Project) -> Result<Jobs> {
    let steps = crate::steps::steps(data, project)?;
    let (placements, _) = crate::ops::built_placements(project)?;
    if placements.is_empty() {
        return Err(Error::new(
            ErrorCode::ParamInvalid,
            "The model has no parts, so there are no instructions to make.",
            "Build the model first.",
        ));
    }
    Ok(jobs(&steps, &placements))
}

/// Lays the book out from what was drawn and writes the PDF to `out`, or to build/export/<name> instructions.pdf.
pub fn write(
    data: &DataDir,
    project: &Project,
    jobs: &Jobs,
    drawn: &Drawn,
    paper: Paper,
    out: Option<&Path>,
) -> Result<(Written, Book)> {
    let steps = crate::steps::steps(data, project)?;
    let bom = crate::ops::bom_build(data, project, crate::bom::ColorNames::preferred(data))?;
    let book = book(
        &Sources {
            title: &project.config.name,
            age: project.config.build.age.as_deref(),
            steps: &steps,
            bom: &bom,
            jobs,
            drawn,
        },
        paper,
    )?;
    let bytes = pdf::write(&book, &project.config.name);
    let target: PathBuf = match out {
        Some(p) => p.to_path_buf(),
        None => project
            .root
            .join("build/export")
            .join(format!("{} instructions.pdf", project.config.name)),
    };
    crate::fsx::write_atomic(&target, &bytes)?;
    Ok((
        Written {
            path: target.display().to_string(),
            pages: book.pages.len(),
            bytes: bytes.len(),
            paper: paper.name().into(),
        },
        book,
    ))
}

/// The BOM's lines by part and color, for the parts boxes.
pub(crate) fn names(bom: &Bom) -> HashMap<(String, u32), (String, String)> {
    bom.lines
        .iter()
        .map(|l| ((l.part.clone(), l.color), (l.name.clone(), l.color_name.clone())))
        .collect()
}

/// Each part × color's picture index in [`Drawn::parts`].
pub(crate) fn part_index(jobs: &Jobs) -> BTreeMap<(String, u32), usize> {
    jobs.parts
        .iter()
        .enumerate()
        .map(|(i, p)| ((p.part.clone(), p.color), i))
        .collect()
}
