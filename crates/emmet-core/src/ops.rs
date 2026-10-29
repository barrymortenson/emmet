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
use crate::parts_data::PartsData;
use crate::project::Project;
use crate::run::{RunMeta, RunSummary};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    pub query: String,
    pub parts: Vec<SearchHit>,
    /// The results of the other queries asked in the same call, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub more: Vec<SearchResult>,
}

/// The most queries one `search_parts` call runs.
pub const MAX_QUERIES: usize = 10;

/// [`search_parts`] for `query` and each of `more` (at most [`MAX_QUERIES`] in all), in one call.
pub fn search_many(
    data: &DataDir,
    query: &str,
    more: &[String],
    category: Option<&str>,
    limit: usize,
) -> Result<SearchResult> {
    if more.len() + 1 > MAX_QUERIES {
        return Err(Error::new(
            ErrorCode::ParamInvalid,
            format!(
                "That's {} queries; one call runs at most {MAX_QUERIES}.",
                more.len() + 1
            ),
            "Split them over two calls.",
        ));
    }
    let mut first = search_parts(data, query, category, limit)?;
    for q in more {
        first.more.push(search_parts(data, q, category, limit)?);
    }
    Ok(first)
}

pub fn search_parts(data: &DataDir, query: &str, category: Option<&str>, limit: usize) -> Result<SearchResult> {
    let index = Index::ensure(data)?;
    let mut parts = index.search(query, category, limit.clamp(1, 100))?;
    if let Some(pack) = PartsData::open(data)? {
        for hit in &mut parts {
            hit.colors_count = pack.part_colors(&index, &hit.id)?.map(|c| c.len() as u32);
        }
    }
    Ok(SearchResult {
        query: query.into(),
        parts,
        more: vec![],
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartInfo {
    #[serde(flatten)]
    pub part: Part,
    /// Connector summary; null until connectors arrive (3A), so an empty list always means "none".
    pub connectors: Option<Vec<String>>,
    /// LDraw codes of the colors the part was made in (the parts data pack); null without the pack or a Rebrickable
    /// mapping, so an empty list always means "none".
    pub produced_colors: Option<Vec<u32>>,
    /// Alternate and mold-variant part ids, as LDraw ids; null without the pack or a Rebrickable mapping.
    pub alternates: Option<Vec<String>>,
}

pub fn part_info(data: &DataDir, id: &str) -> Result<PartInfo> {
    let index = Index::ensure(data)?;
    match index.part(id)? {
        Some(part) => {
            let pack = PartsData::open(data)?;
            let (produced_colors, alternates) = match &pack {
                Some(p) => (
                    p.part_colors(&index, &part.id)?.map(|c| c.into_iter().collect()),
                    p.alternates(&index, &part.id)?,
                ),
                None => (None, None),
            };
            Ok(PartInfo {
                part,
                connectors: None,
                produced_colors,
                alternates,
            })
        }
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

/// Every color, or with `part`, the colors that part was made in. Without the parts data pack, or for a part with no
/// Rebrickable mapping, `part` only checks the part exists and every color is listed.
pub fn list_colors(data: &DataDir, part: Option<&str>) -> Result<ColorList> {
    let index = Index::ensure(data)?;
    let pack = PartsData::open(data)?;
    let mut colors = index.colors()?;
    crate::parts_data::annotate_colors(pack.as_ref(), &mut colors)?;
    if let Some(id) = part {
        if index.part(id)?.is_none() {
            return Err(unknown_part(&index, id)?);
        }
        if let Some(made) = pack.as_ref().map(|p| p.part_colors(&index, id)).transpose()?.flatten() {
            colors.retain(|c| made.contains(&c.code));
        }
    }
    Ok(ColorList { colors })
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
    /// One of the knobs `@model(knobs=...)` names: the app's panel shows only these.
    #[serde(default)]
    pub knob: bool,
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
    params_from(crate::run::schema(data, project, &BTreeMap::new())?)
}

/// The DSL's schema reply, checked to hold a value for every parameter it lists.
fn params_from(v: serde_json::Value) -> Result<Params> {
    let p: Params = serde_json::from_value(v).map_err(|e| Error::internal(format!("emmet_dsl's schema reply: {e}")))?;
    if let Some(missing) = p.params.iter().find(|s| !p.values.contains_key(&s.name)) {
        return Err(Error::internal(format!(
            "emmet_dsl's schema reply has no value for '{}'.",
            missing.name
        )));
    }
    Ok(p)
}

/// Checks `values` against the schema and writes params.json: overrides only, so a value equal to its default is
/// dropped from the file.
pub fn set_params(data: &DataDir, project: &Project, values: &BTreeMap<String, serde_json::Value>) -> Result<Params> {
    let before = get_params(data, project)?;
    let mut after = params_from(crate::run::schema(data, project, values)?)?;
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

/// "Make this the default": writes a parameter's current value into model.py as its default (one literal rewritten,
/// the rest of the file byte for byte), and drops it from params.json.
pub fn write_default(data: &DataDir, project: &Project, name: &str) -> Result<Params> {
    let before = get_params(data, project)?;
    let Some(spec) = before.params.iter().find(|p| p.name == name) else {
        let known = before
            .params
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(Error::new(
            ErrorCode::ParamInvalid,
            format!("There's no parameter '{name}'."),
            format!("The model's parameters are: {known}."),
        ));
    };
    let value = before.values[name].clone();
    if value != spec.default {
        crate::run::write_default(data, project, name, &value, &spec.kind)?;
    }
    let mut overrides = project.params()?;
    overrides.remove(name);
    project.write_params(&overrides)?;
    let mut after = get_params(data, project)?;
    after.changed = vec![name.to_string()];
    Ok(after)
}

pub fn run_model(
    data: &DataDir,
    project: &Project,
    overrides: &BTreeMap<String, serde_json::Value>,
) -> (Result<RunSummary>, RunMeta) {
    crate::run::run_model(data, project, overrides)
}

/// [`run_model`] that stops when `cancel` is set (the app's run queue, through `emmet run --cancel-on-eof`).
pub fn run_model_until(
    data: &DataDir,
    project: &Project,
    overrides: &BTreeMap<String, serde_json::Value>,
    cancel: &std::sync::atomic::AtomicBool,
) -> (Result<RunSummary>, RunMeta) {
    crate::run::run_model_until(data, project, overrides, cancel)
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
    // require, not installed: a damaged library says so, with its own hint, instead of "not installed".
    let lib = crate::library::require(data);
    checks.push(match &lib {
        Ok(i) => Check {
            name: "library".into(),
            ok: true,
            detail: format!("LDraw {} ({} files)", i.release, i.files),
            fix: None,
        },
        Err(e) => Check {
            name: "library".into(),
            ok: false,
            detail: e.message.clone(),
            fix: Some(e.hint.clone()),
        },
    });
    let index = if fix && lib.is_ok() {
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
            problem = Some(e);
        }
        python = crate::python::status(data);
    }
    let uv_ok = python.uv.is_some() || python.ready;
    checks.push(Check {
        name: "uv".into(),
        ok: uv_ok,
        detail: python
            .uv
            .clone()
            .unwrap_or_else(|| "not found next to emmet or on PATH".into()),
        fix: (!uv_ok).then(|| "install uv (https://docs.astral.sh/uv/)".into()),
    });
    // The setup error's own hint, not "run emmet doctor --fix" again after it just failed.
    let (detail, fix_hint) = match (&python.python, &python.emmet_dsl, problem) {
        (_, _, Some(e)) => (e.message, Some(e.hint)),
        (Some(p), Some(v), None) => (
            format!("Python {} with emmet_dsl {v}: {p}", crate::python::PYTHON_VERSION),
            None,
        ),
        _ => ("not set up".into(), Some("emmet doctor --fix".into())),
    };
    checks.push(Check {
        name: "python".into(),
        ok: python.ready,
        detail,
        fix: if python.ready { None } else { fix_hint },
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

/// Runs the model when build/placements.json is missing or manifest.json's inputs hash isn't the current one.
pub fn ensure_built(data: &DataDir, project: &Project) -> Result<()> {
    let manifest = read_json(&project.root.join("build/manifest.json"));
    let library = crate::library::require(data)?;
    let parts_data = crate::parts_data::version(data)?;
    let current = project.inputs_hash(&BTreeMap::new(), &library.release, parts_data.as_deref())?;
    let fresh = project.root.join("build/placements.json").is_file()
        && manifest.as_ref().and_then(|m| m["inputs_hash"].as_str()) == Some(current.as_str());
    if !fresh {
        run_model(data, project, &BTreeMap::new()).0?;
    }
    Ok(())
}

fn read_json(path: &std::path::Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
}

/// The placements of the project's current build, and the inputs hash they were built from.
pub fn built_placements(project: &Project) -> Result<(Vec<crate::ldraw::placements::Placement>, Option<String>)> {
    let path = project.root.join("build/placements.json");
    let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    let doc: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        Error::internal(format!("build/placements.json doesn't parse: {e}")).at_file("build/placements.json")
    })?;
    let rows = serde_json::from_value(doc["parts"].clone()).map_err(|e| {
        Error::internal(format!("build/placements.json isn't a list of placements: {e}"))
            .at_file("build/placements.json")
    })?;
    let hash =
        read_json(&project.root.join("build/manifest.json")).and_then(|m| m["inputs_hash"].as_str().map(String::from));
    Ok((rows, hash))
}

/// What the model wrote into build/placements.json besides its parts: `m.allow` and `m.ground` (DSL v1, 3B).
#[derive(Debug, Clone, Default, serde::Deserialize)]
struct BuildExtras {
    #[serde(default)]
    allow: Vec<crate::check::Allow>,
    #[serde(default)]
    ground: Vec<String>,
}

/// The validator's options for the project's current build: the allow entries from emmet.toml and the model, the
/// model's ground parts, and the step target from the Age build setting.
pub fn check_options(project: &Project) -> Result<crate::check::Options> {
    let path = project.root.join("build/placements.json");
    let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    let extras: BuildExtras = serde_json::from_str(&text).map_err(|e| {
        Error::internal(format!("build/placements.json's allow or ground doesn't parse: {e}"))
            .at_file("build/placements.json")
    })?;
    let mut allow = project.config.check.allow.clone();
    allow.extend(extras.allow);
    Ok(crate::check::Options {
        allow,
        ground: extras.ground,
        step_target: project.config.build.step_target(),
    })
}

/// Validates the project's current build (running the model first if build/ is stale) and, when every rule runs,
/// writes build/validation.json. `rules` empty means every rule.
pub fn check(data: &DataDir, project: &Project, rules: &[crate::check::Rule]) -> Result<crate::check::Validation> {
    check_with(data, project, rules, &BTreeMap::new())
}

/// [`check`] of the model run with `overrides` on top of params.json (params.json isn't changed): how an agent
/// checks a slider's other values in one call.
pub fn check_with(
    data: &DataDir,
    project: &Project,
    rules: &[crate::check::Rule],
    overrides: &BTreeMap<String, serde_json::Value>,
) -> Result<crate::check::Validation> {
    if overrides.is_empty() {
        ensure_built(data, project)?;
    } else {
        run_model(data, project, overrides).0?;
    }
    check_build(data, project, rules)
}

/// [`check`] on build/ as it is, without running the model: for the app, right after its own run.
pub fn check_build(
    data: &DataDir,
    project: &Project,
    rules: &[crate::check::Rule],
) -> Result<crate::check::Validation> {
    let (placements, hash) = built_placements(project)?;
    let all = rules.is_empty();
    let rules = if all {
        crate::check::Rule::ALL.to_vec()
    } else {
        rules.to_vec()
    };
    let validation = validate_placements(data, &placements, &rules, &check_options(project)?, hash)?;
    if all {
        let text = serde_json::to_string_pretty(&validation).expect("validation serializes") + "\n";
        crate::fsx::write_atomic(&project.root.join("build/validation.json"), text.as_bytes())?;
    }
    Ok(validation)
}

/// The validator over any placements: a project's build, or an .mpd's (`ldraw::placements`).
pub fn validate_placements(
    data: &DataDir,
    placements: &[crate::ldraw::placements::Placement],
    rules: &[crate::check::Rule],
    options: &crate::check::Options,
    inputs_hash: Option<String>,
) -> Result<crate::check::Validation> {
    let index = Index::ensure(data)?;
    let lib = crate::ldraw::geometry::LibFiles::probe(&data.ldraw());
    let pack = PartsData::open(data)?;
    crate::check::validate(&index, &lib, pack.as_ref(), placements, rules, options, inputs_hash)
}

/// The parts list of the project's current build (running the model first if build/ is stale), with color names in
/// `names`; writes build/bom.json.
pub fn bom(data: &DataDir, project: &Project, names: crate::bom::ColorNames) -> Result<crate::bom::Bom> {
    ensure_built(data, project)?;
    bom_build(data, project, names)
}

/// [`bom`] on build/ as it is, without running the model.
pub fn bom_build(data: &DataDir, project: &Project, names: crate::bom::ColorNames) -> Result<crate::bom::Bom> {
    let (placements, _) = built_placements(project)?;
    let index = Index::ensure(data)?;
    let pack = PartsData::open(data)?;
    let bom = crate::bom::bom(&index, pack.as_ref(), &placements, names)?;
    let text = serde_json::to_string_pretty(&bom).expect("bom serializes") + "\n";
    crate::fsx::write_atomic(&project.root.join("build/bom.json"), text.as_bytes())?;
    Ok(bom)
}
