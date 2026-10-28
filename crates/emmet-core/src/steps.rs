// SPDX-License-Identifier: AGPL-3.0-or-later
//! Build steps (plan sections 08 and 09): `build/steps.json`, which emmet_dsl writes with the model, for
//! `emmet steps` and the MCP `steps` tool. Each step lists its new parts as a parts box (part, color, count, larger
//! parts first) and the sub-assemblies it places; each sub-assembly has its own steps and how many copies the model
//! has ("× 4").

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::DataDir;
use crate::envelope::{Error, Result};
use crate::project::Project;

/// One part × color in a step's parts box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BoxLine {
    /// The LDraw part id.
    pub part: String,
    /// The LDraw color code.
    pub color: u32,
    pub count: usize,
}

/// A sub-assembly a step places.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Placed {
    pub name: String,
    /// The submodel's file in model.mpd.
    pub file: String,
    /// Copies placed in this step.
    pub count: usize,
    /// Those copies' ids; each copy's parts are "<copy id>/<part id>".
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Step {
    /// From 1, within the main model or within its sub-assembly.
    pub n: u32,
    /// The step's title, or "" for an untitled one.
    pub title: String,
    /// The title before its colon, which groups steps in the steps list ("Walls, row 2").
    pub group: String,
    /// The ids of the parts this step adds (within a sub-assembly, the part's own id, the same in every copy).
    pub part_ids: Vec<String>,
    /// The parts box: each new part and color with its count, larger parts first.
    pub parts: Vec<BoxLine>,
    /// The sub-assemblies this step puts on.
    pub submodels: Vec<Placed>,
}

/// A unit built apart (a door in its frame, a lamp post), once, then placed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Submodel {
    pub name: String,
    pub file: String,
    /// How many copies the whole model has: the "× 4" on its box.
    pub count: usize,
    /// Parts in one copy.
    pub parts: usize,
    pub steps: Vec<Step>,
}

/// `build/steps.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Steps {
    pub steps: Vec<Step>,
    pub submodels: Vec<Submodel>,
}

/// The project's build steps (running the model first if build/ is stale).
pub fn steps(data: &DataDir, project: &Project) -> Result<Steps> {
    crate::ops::ensure_built(data, project)?;
    let path = project.root.join("build/steps.json");
    let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    serde_json::from_str(&text).map_err(|e| {
        Error::internal(format!("build/steps.json doesn't match what emmet expects: {e}")).at_file("build/steps.json")
    })
}

/// The one-line summary: the step count and the sub-assemblies ("18 steps; built apart: Window × 4, Door").
pub fn summary(s: &Steps) -> String {
    let mut out = format!("{} step{}", s.steps.len(), if s.steps.len() == 1 { "" } else { "s" });
    if !s.submodels.is_empty() {
        let subs: Vec<String> = s
            .submodels
            .iter()
            .map(|m| {
                if m.count > 1 {
                    format!("{} × {}", m.name, m.count)
                } else {
                    m.name.clone()
                }
            })
            .collect();
        out.push_str(&format!("; built apart: {}", subs.join(", ")));
    }
    out
}
