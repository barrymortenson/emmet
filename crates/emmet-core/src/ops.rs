// SPDX-License-Identifier: AGPL-3.0-or-later
//! The operations the CLI and the MCP server both expose (plan section 09). Each returns the envelope's `data`; the
//! front ends only shape it.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::colors::Color;
use crate::data::DataDir;
use crate::envelope::{Details, Error, ErrorCode, Result};
use crate::index::{Index, Part, SearchHit};
use crate::project::Project;
use crate::run::{RunMeta, RunSummary};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    pub query: String,
    pub parts: Vec<SearchHit>,
}

pub fn search_parts(data: &DataDir, query: &str, category: Option<&str>, limit: usize) -> Result<SearchResult> {
    let index = Index::ensure(data)?;
    let parts = index.search(query, category, limit.clamp(1, 100))?;
    Ok(SearchResult {
        query: query.into(),
        parts,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartInfo {
    #[serde(flatten)]
    pub part: Part,
    /// Connector summary, filled from 3A.
    pub connectors: Vec<String>,
    /// Colors the part was produced in, filled once the parts data pack is installed (3B).
    pub produced_colors: Vec<u32>,
    /// Alternate and mold-variant ids, filled from 3B.
    pub alternates: Vec<String>,
}

pub fn part_info(data: &DataDir, id: &str) -> Result<PartInfo> {
    let index = Index::ensure(data)?;
    match index.part(id)? {
        Some(part) => Ok(PartInfo {
            part,
            connectors: vec![],
            produced_colors: vec![],
            alternates: vec![],
        }),
        None => Err(unknown_part(&index, id)?),
    }
}

/// `UNKNOWN_PART` with the closest ids and the best search matches for `id`.
pub fn unknown_part(index: &Index, id: &str) -> Result<Error> {
    let mut suggestions: Vec<(String, String)> = index.similar_ids(id, 5)?;
    for hit in index.search(id, None, 5)? {
        if !suggestions.iter().any(|(s, _)| *s == hit.id) {
            suggestions.push((hit.id, hit.name));
        }
    }
    suggestions.truncate(5);
    let hint = if suggestions.is_empty() {
        "Search for the part with emmet search <words>, then use its id.".to_string()
    } else {
        let list = suggestions
            .iter()
            .map(|(id, name)| format!("{id} ({name})"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("Did you mean {list}? Or search with emmet search <words>.")
    };
    Ok(Error::new(
        ErrorCode::UnknownPart,
        format!("There's no part '{id}' in the LDraw library."),
        hint,
    )
    .with_details(Details {
        suggestions: suggestions.into_iter().map(|(id, _)| id).collect(),
        ..Default::default()
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ColorList {
    pub colors: Vec<Color>,
}

/// Every color, or with `part`, the colors that part was produced in (all colors until the parts data pack, 3B).
pub fn list_colors(data: &DataDir, part: Option<&str>) -> Result<ColorList> {
    let index = Index::ensure(data)?;
    if let Some(id) = part
        && index.part(id)?.is_none()
    {
        return Err(unknown_part(&index, id)?);
    }
    Ok(ColorList {
        colors: index.colors()?,
    })
}

/// One parameter of the model's schema.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParamSpec {
    pub name: String,
    /// int, float, bool, choice, or color.
    #[serde(rename = "type")]
    pub kind: String,
    pub default: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<serde_json::Value>>,
    /// From the model's docstring.
    pub help: String,
    /// The line in `file` that declares it.
    pub line: u32,
}

/// The model's parameters and their current values.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Params {
    /// The @model function's name.
    pub model: String,
    /// The first line of its docstring.
    pub doc: String,
    pub file: String,
    pub params: Vec<ParamSpec>,
    /// Every parameter's current value: the default, or the override in params.json.
    pub values: BTreeMap<String, serde_json::Value>,
    /// What this call changed (set_params only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<String>,
}

pub fn get_params(data: &DataDir, project: &Project) -> Result<Params> {
    let v = crate::run::schema(data, project, &BTreeMap::new())?;
    serde_json::from_value(v).map_err(|e| Error::internal(format!("emmet_dsl's schema reply: {e}")))
}

/// Checks `values` against the schema and writes params.json: overrides only, so a value equal to its default is
/// dropped from the file.
pub fn set_params(data: &DataDir, project: &Project, values: &BTreeMap<String, serde_json::Value>) -> Result<Params> {
    let before = get_params(data, project)?;
    let v = crate::run::schema(data, project, values)?;
    let mut after: Params =
        serde_json::from_value(v).map_err(|e| Error::internal(format!("emmet_dsl's schema reply: {e}")))?;
    let mut overrides = BTreeMap::new();
    for p in &after.params {
        let value = &after.values[&p.name];
        if *value != p.default {
            overrides.insert(p.name.clone(), value.clone());
        }
    }
    project.write_params(&overrides)?;
    after.changed = after
        .values
        .iter()
        .filter(|(k, v)| before.values.get(*k) != Some(v))
        .map(|(k, _)| k.clone())
        .collect();
    Ok(after)
}

pub fn run_model(
    data: &DataDir,
    project: &Project,
    overrides: &BTreeMap<String, serde_json::Value>,
) -> (Result<RunSummary>, RunMeta) {
    crate::run::run_model(data, project, overrides)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Trusted {
    pub project: String,
    pub trusted: bool,
}

pub fn trust(data: &DataDir, project: &Project) -> Result<Trusted> {
    project.trust(data)?;
    Ok(Trusted {
        project: project.root.display().to_string(),
        trusted: true,
    })
}

/// One line of `emmet doctor`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Check {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    /// What to run when it isn't ok.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Doctor {
    pub ready: bool,
    pub checks: Vec<Check>,
}

/// Checks the library, the parts index, uv, and Python; with `fix`, builds the index and sets Python up.
pub fn doctor(data: &DataDir, fix: bool) -> Doctor {
    let mut checks = Vec::new();
    let lib = crate::library::installed(data);
    checks.push(Check {
        name: "library".into(),
        ok: lib.is_some(),
        detail: lib.as_ref().map_or_else(
            || format!("not installed in {}", data.ldraw().display()),
            |i| format!("LDraw {} ({} files)", i.release, i.files),
        ),
        fix: lib.is_none().then(|| "emmet library install".into()),
    });
    let index = if fix && lib.is_some() {
        Index::ensure(data)
    } else {
        Index::open(data)
    };
    checks.push(match index.and_then(|i| i.part_count()) {
        Ok(n) => Check {
            name: "parts index".into(),
            ok: true,
            detail: format!("{n} parts"),
            fix: None,
        },
        Err(e) => Check {
            name: "parts index".into(),
            ok: false,
            detail: e.message,
            fix: Some("emmet library verify --reindex".into()),
        },
    });
    let mut python = crate::python::status(data);
    let mut problem = None;
    if fix && !python.ready {
        if let Err(e) = crate::python::ensure(data) {
            problem = Some(e.message);
        }
        python = crate::python::status(data);
    }
    checks.push(Check {
        name: "uv".into(),
        ok: python.uv.is_some() || python.ready,
        detail: python
            .uv
            .clone()
            .unwrap_or_else(|| "not found next to emmet or on PATH".into()),
        fix: python
            .uv
            .is_none()
            .then(|| "install uv (https://docs.astral.sh/uv/)".into()),
    });
    checks.push(Check {
        name: "python".into(),
        ok: python.ready,
        detail: match (&python.python, &python.emmet_dsl, problem) {
            (_, _, Some(p)) => p,
            (Some(p), Some(v), None) => format!("Python {} with emmet_dsl {v}: {p}", crate::python::PYTHON_VERSION),
            _ => "not set up".into(),
        },
        fix: (!python.ready).then(|| "emmet doctor --fix".into()),
    });
    Doctor {
        ready: checks.iter().all(|c| c.ok),
        checks,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Created {
    pub project: String,
    pub name: String,
    pub files: Vec<String>,
}

pub fn new_project(data: &DataDir, dir: &std::path::Path, template: crate::project::Template) -> Result<Created> {
    let project = Project::create(data, dir, template)?;
    let mut files: Vec<String> = std::fs::read_dir(&project.root)
        .map_err(|e| Error::io(&project.root, e))?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();
    Ok(Created {
        project: project.root.display().to_string(),
        name: project.config.name,
        files,
    })
}
