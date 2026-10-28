// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet mcp`: the stdio MCP server named `emmet` (plan section 09). Each tool calls the same `emmet_core::ops`
//! function as its CLI command and returns the CLI's JSON envelope as structured content, plus a short text summary
//! sized for an agent's context. Nothing here decides anything; it only shapes output.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use emmet_core::data::DataDir;
use emmet_core::envelope::{Envelope, Error, Meta, Result};
use emmet_core::ops::{self, ColorList, Params, PartInfo, SearchResult};
use emmet_core::project::Project;
use emmet_core::run::{RunMeta, RunSummary};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::schema_for_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation, ServerCapabilities,
    ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// Words and dimensions, for example "brick 2x4" or "slope 45 2 x 2".
    pub query: String,
    /// Only parts in this category (Brick, Plate, Slope, Tile, …).
    #[serde(default)]
    pub category: Option<String>,
    /// How many parts to return (default 10, at most 100), per query.
    #[serde(default)]
    pub limit: Option<usize>,
    /// More queries to answer in the same call (up to 9): look up everything a stage needs at once.
    #[serde(default)]
    pub more: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PartArgs {
    /// An LDraw part id, like "3001".
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunArgs {
    /// Parameter values for this run only, on top of params.json.
    #[serde(default)]
    pub params: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NoArgs {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TechniqueArgs {
    /// A technique's name, like "gable_roof"; leave it out for the list.
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetParamsArgs {
    /// Parameter values to keep in params.json. A value equal to its default is dropped from the file.
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ColorsArgs {
    /// A part id: checks the part exists. Every color is returned until the parts data pack (3B) knows which colors
    /// each part came in.
    #[serde(default)]
    pub part: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenderArgs {
    /// iso (default), front, back, left, right, or top.
    #[serde(default)]
    pub view: Option<String>,
    /// Emmet ids to highlight; everything else is dimmed.
    #[serde(default)]
    pub highlight: Option<Vec<String>>,
    /// Width and height in pixels (default 768).
    #[serde(default)]
    pub size: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HighlightArgs {
    /// Emmet ids to highlight in the open app's viewer; an empty list clears the highlight.
    #[serde(default)]
    pub ids: Vec<String>,
    /// A finding id from validate ("F-0003"): highlights its parts.
    #[serde(default)]
    pub finding: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateArgs {
    /// Only these rules (default: all): collision, floating, disconnected_submodel, step_order, step_size,
    /// weak_connection, off_grid, unknown_part, obsolete_part, alias_part.
    #[serde(default)]
    pub rules: Option<Vec<String>>,
    /// Parameter values to check, on top of params.json, which isn't changed: runs the model with them first. Use it
    /// to check a slider's other values; params.json is the user's.
    #[serde(default)]
    pub params: Option<BTreeMap<String, Value>>,
}

/// What `validate` returns: the counts, and the first findings (all of them are in build/validation.json).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Checked {
    pub counts_by_rule: BTreeMap<String, usize>,
    /// The first 50, errors first, in source order.
    pub findings: Vec<emmet_core::check::Finding>,
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    /// Findings the project or the model allows (`[[check.allow]]`, `m.allow`), with their reasons.
    pub allowed: Vec<emmet_core::check::Allowed>,
    /// No errors and no warnings.
    pub clean: bool,
    /// The rules checked, when the call named some; empty when every rule ran.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,
}

/// The most findings `validate` returns; the rest are in build/validation.json.
const MAX_FINDINGS: usize = 50;
/// How many findings the text summary lists.
const SUMMARY_FINDINGS: usize = 10;

/// Whether the open app showed the highlight.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Highlighted {
    pub delivered: bool,
    /// Why not, when it wasn't delivered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// The doctrine (plan section 10), embedded in the binary: printed by `emmet rules` and the `rules` tool.
pub const DOCTRINE: &str = include_str!("../doctrine/EMMET.md");
/// The DSL vocabulary, generated from emmet_dsl's docstrings: printed by `emmet api` and the `api` tool.
pub const API: &str = include_str!("../doctrine/api.md");

#[derive(Clone)]
pub struct EmmetMcp {
    data: DataDir,
    project: Option<PathBuf>,
    /// The app's chat session, which turns on redirect notes (`--session`, set only by the app).
    session: Option<String>,
    tool_router: ToolRouter<Self>,
}

/// Runs a blocking core operation off the async runtime. A panic in it becomes `failed(INTERNAL)`, so the call still
/// gets an answer instead of the server going down with it.
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static, failed: impl FnOnce(Error) -> T) -> T {
    match tokio::task::spawn_blocking(f).await {
        Ok(t) => t,
        Err(e) => failed(Error::internal(format!("an emmet core task stopped: {e}"))),
    }
}

/// The envelope as structured content, plus `text` (or the error, with its file:line and hint) as the summary.
fn result<T: Serialize>(
    started: Instant,
    result: Result<T>,
    meta: RunMeta,
    project: Option<String>,
    text: impl FnOnce(&T) -> String,
) -> CallToolResult {
    let summary = match &result {
        Ok(data) => text(data),
        Err(e) => error_text(e),
    };
    let ok = result.is_ok();
    let env = Envelope::from_result(
        result,
        vec![],
        Meta {
            emmet: emmet_core::version().into(),
            project,
            inputs_hash: meta.inputs_hash,
            generation: meta.generation,
            ms: started.elapsed().as_millis() as u64,
        },
    );
    let value = serde_json::to_value(env).expect("envelope serializes");
    let mut out = if ok {
        CallToolResult::structured(value)
    } else {
        CallToolResult::structured_error(value)
    };
    out.content = vec![ContentBlock::text(summary)];
    out
}

fn error_text(e: &Error) -> String {
    let code = serde_json::to_value(e.code)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    let place = match e.details.as_ref().map(|d| (&d.file, d.line)) {
        Some((Some(f), Some(l))) => format!(" at {f}:{l}"),
        Some((Some(f), None)) => format!(" in {f}"),
        _ => String::new(),
    };
    format!("{code}{place}: {} {}", e.message, e.hint)
}

#[tool_router(router = tool_router)]
impl EmmetMcp {
    pub fn new(data: DataDir, project: Option<PathBuf>, session: Option<String>) -> Self {
        Self {
            data,
            project,
            session,
            tool_router: Self::tool_router(),
        }
    }

    fn find_project(&self) -> Result<Project> {
        Project::find(self.project.as_deref())
    }

    /// Search LDraw parts by words and dimensions; `more` asks several queries in one call. Returns ids, names,
    /// categories, and sizes (x and y in studs, z in plates), which is usually all you need to place a part.
    #[tool(output_schema = schema_for_output::<Envelope<SearchResult>>())]
    async fn search_parts(&self, Parameters(args): Parameters<SearchArgs>) -> CallToolResult {
        let started = Instant::now();
        let data = self.data.clone();
        let out = blocking(
            move || {
                ops::search_many(
                    &data,
                    &args.query,
                    &args.more.unwrap_or_default(),
                    args.category.as_deref(),
                    args.limit.unwrap_or(10),
                )
            },
            Err,
        )
        .await;
        result(started, out, RunMeta::default(), None, |r| {
            let one = |r: &SearchResult| {
                if r.parts.is_empty() {
                    return format!("No parts match '{}'.", r.query);
                }
                r.parts
                    .iter()
                    .map(|p| format!("{} {}{}", p.id, p.name, size(p.size)))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            if r.more.is_empty() {
                return one(r);
            }
            std::iter::once(r)
                .chain(r.more.iter())
                .map(|q| format!("## {}\n{}", q.query, one(q)))
                .collect::<Vec<_>>()
                .join("\n\n")
        })
    }

    /// One part: its name, size in studs and plates, flags (alias, obsolete, hidden, printed, sticker, shortcut),
    /// and body box.
    #[tool(output_schema = schema_for_output::<Envelope<PartInfo>>())]
    async fn part_info(&self, Parameters(args): Parameters<PartArgs>) -> CallToolResult {
        let started = Instant::now();
        let data = self.data.clone();
        let out = blocking(move || ops::part_info(&data, &args.id), Err).await;
        result(started, out, RunMeta::default(), None, |p| {
            let mut s = format!(
                "{} {} ({}){}",
                p.part.id,
                p.part.name,
                p.part.category,
                size(p.part.size)
            );
            if !p.part.flags.is_empty() {
                s.push_str(&format!(". Flags: {}", p.part.flags.join(", ")));
            }
            if let Some(a) = &p.part.alias_of {
                s.push_str(&format!(". Use {a} instead"));
            }
            s
        })
    }

    /// Run the project's model.py and write build/model.mpd, placements.json, and manifest.json. Errors name the
    /// file and line, with a hint.
    #[tool(output_schema = schema_for_output::<Envelope<RunSummary>>())]
    async fn run_model(&self, Parameters(args): Parameters<RunArgs>) -> CallToolResult {
        let started = Instant::now();
        let (data, me) = (self.data.clone(), self.clone());
        let (out, meta, root) = blocking(
            move || {
                let project = match me.find_project() {
                    Ok(p) => p,
                    Err(e) => return (Err(e), RunMeta::default(), None),
                };
                let root = Some(project.root.display().to_string());
                let (out, meta) = ops::run_model(&data, &project, &args.params.unwrap_or_default());
                (out, meta, root)
            },
            |e| (Err(e), RunMeta::default(), None),
        )
        .await;
        result(started, out, meta, root, |r| {
            format!(
                "Built {} parts in {} steps ({:.2} s), generation {}.",
                r.parts,
                r.steps,
                r.ms as f64 / 1000.0,
                r.generation
            )
        })
    }

    /// The model's parameters (type, default, range, help) and their current values.
    #[tool(output_schema = schema_for_output::<Envelope<Params>>())]
    async fn get_params(&self, Parameters(_): Parameters<NoArgs>) -> CallToolResult {
        let started = Instant::now();
        let (data, me) = (self.data.clone(), self.clone());
        let (out, root) = blocking(
            move || with_project(&me, |p| ops::get_params(&data, p)),
            |e| (Err(e), None),
        )
        .await;
        result(started, out, RunMeta::default(), root, |p| {
            p.params
                .iter()
                .map(|s| format!("{} = {}", s.name, p.values[&s.name]))
                .collect::<Vec<_>>()
                .join(", ")
        })
    }

    /// Set parameter values in params.json, checked against their types and ranges first.
    #[tool(output_schema = schema_for_output::<Envelope<Params>>())]
    async fn set_params(&self, Parameters(args): Parameters<SetParamsArgs>) -> CallToolResult {
        let started = Instant::now();
        let (data, me) = (self.data.clone(), self.clone());
        let (out, root) = blocking(
            move || with_project(&me, |p| ops::set_params(&data, p, &args.values)),
            |e| (Err(e), None),
        )
        .await;
        result(started, out, RunMeta::default(), root, |p| {
            if p.changed.is_empty() {
                return "Nothing changed.".into();
            }
            let changed: Vec<String> = p.changed.iter().map(|k| format!("{k} = {}", p.values[k])).collect();
            format!("Changed {}.", changed.join(", "))
        })
    }

    /// Render the model to a PNG (image content) through the viewer: the open Emmet app, or its hidden render mode.
    /// Runs the model first if build/ is out of date.
    #[tool(output_schema = schema_for_output::<Envelope<crate::render::Rendered>>())]
    async fn render(&self, Parameters(args): Parameters<RenderArgs>) -> CallToolResult {
        let started = Instant::now();
        let (data, me) = (self.data.clone(), self.clone());
        let view = args.view.unwrap_or_else(|| "iso".into());
        let (out, root) = blocking(
            move || {
                with_project(&me, |p| {
                    crate::render::render(
                        &data,
                        p,
                        &view,
                        args.size.unwrap_or(768),
                        &args.highlight.unwrap_or_default(),
                    )
                })
            },
            |e| (Err(e), None),
        )
        .await;
        let image = out.as_ref().ok().map(|png| emmet_core::base64::encode(&png.bytes));
        let mut call = result(started, out.map(|png| png.info), RunMeta::default(), root, |r| {
            let lit = if r.highlighted.is_empty() {
                String::new()
            } else {
                format!(", highlighting {}", r.highlighted.join(", "))
            };
            format!("The {} view at {} px{lit}.", r.view, r.size)
        });
        if let Some(png) = image {
            call.content.insert(0, ContentBlock::image(png, "image/png"));
        }
        call
    }

    /// Check the model: collisions, floating parts, submodels placed off the model, build steps (order and size),
    /// weak spans, parts a little off the grid, and unknown or obsolete parts. Runs the model first if build/ is out
    /// of date, or with `params` when given. Each finding has an id, the parts, the model.py line that placed them,
    /// and a hint that says what to change.
    #[tool(output_schema = schema_for_output::<Envelope<Checked>>())]
    async fn validate(&self, Parameters(args): Parameters<ValidateArgs>) -> CallToolResult {
        let started = Instant::now();
        let (data, me) = (self.data.clone(), self.clone());
        let (out, root) = blocking(
            move || {
                with_project(&me, |p| {
                    let rules = args
                        .rules
                        .unwrap_or_default()
                        .iter()
                        .map(|r| {
                            emmet_core::check::Rule::parse(r).ok_or_else(|| {
                                Error::new(
                                    emmet_core::envelope::ErrorCode::ParamInvalid,
                                    format!("There's no rule '{r}'."),
                                    format!("Use {}.", emmet_core::check::Rule::list()),
                                )
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let names: Vec<String> = rules.iter().map(|r| r.name().to_string()).collect();
                    ops::check_with(&data, p, &rules, &args.params.unwrap_or_default()).map(|v| (v, names))
                })
            },
            |e| (Err(e), None),
        )
        .await;
        let meta = RunMeta {
            inputs_hash: out.as_ref().ok().and_then(|(v, _)| v.inputs_hash.clone()),
            generation: None,
        };
        let out = out.map(|(v, only)| {
            let count = |s| v.findings.iter().filter(|f| f.severity == s).count();
            Checked {
                counts_by_rule: v.counts.clone(),
                total: v.findings.len(),
                errors: count(emmet_core::check::Severity::Error),
                warnings: count(emmet_core::check::Severity::Warning),
                allowed: v.allowed.clone(),
                clean: v.clean,
                findings: v.findings.into_iter().take(MAX_FINDINGS).collect(),
                only,
            }
        });
        result(started, out, meta, root, validate_text)
    }

    /// The parts list: part × color × quantity, with totals. Runs the model first if build/ is out of date.
    #[tool(output_schema = schema_for_output::<Envelope<emmet_core::bom::Bom>>())]
    async fn bom(&self, Parameters(_): Parameters<NoArgs>) -> CallToolResult {
        let started = Instant::now();
        let (data, me) = (self.data.clone(), self.clone());
        let (out, root) = blocking(move || with_project(&me, |p| ops::bom(&data, p)), |e| (Err(e), None)).await;
        result(started, out, RunMeta::default(), root, |b| {
            let top: Vec<String> = b
                .lines
                .iter()
                .take(8)
                .map(|l| format!("{} × {} {}", l.qty, l.color_name, l.name))
                .collect();
            let more = if b.lines.len() > top.len() { ", …" } else { "" };
            format!(
                "{} pieces: {} parts in {} part × color lines. Most used: {}{more}. Availability arrives with the parts \
                 data pack.",
                b.totals.pieces,
                b.totals.parts,
                b.totals.lines,
                top.join(", ")
            )
        })
    }

    /// Highlight parts in the open Emmet app's viewer, by Emmet id or by a finding id from validate. Says so when the
    /// app isn't open.
    #[tool(output_schema = schema_for_output::<Envelope<Highlighted>>())]
    async fn highlight(&self, Parameters(args): Parameters<HighlightArgs>) -> CallToolResult {
        let started = Instant::now();
        let (data, me) = (self.data.clone(), self.clone());
        let (out, root) = blocking(
            move || {
                with_project(&me, |p| {
                    let mut ids = args.ids;
                    ids.extend(args.finding);
                    let ids = crate::render::expand_findings(p, &ids)?;
                    let reply = emmet_bus::send(
                        &data,
                        emmet_bus::Message::Highlight {
                            project: p.root.display().to_string(),
                            ids,
                        },
                        std::time::Duration::from_secs(5),
                    );
                    Ok(Highlighted {
                        delivered: reply.delivered,
                        reason: reply.reason,
                    })
                })
            },
            |e| (Err(e), None),
        )
        .await;
        result(started, out, RunMeta::default(), root, |h| {
            if h.delivered {
                "Highlighted in the viewer.".into()
            } else {
                format!("Emmet app not open ({}).", h.reason.as_deref().unwrap_or("no answer"))
            }
        })
    }

    /// How to build with Emmet: the frame and units, the loop, building in stages, when to ask, and redirects. Read it
    /// once, before anything else.
    #[tool]
    async fn rules(&self, Parameters(_): Parameters<NoArgs>) -> CallToolResult {
        CallToolResult::success(vec![ContentBlock::text(DOCTRINE)])
    }

    /// The emmet_dsl vocabulary for model.py: @model, Model.add, Model.step, Handle, and the param types.
    #[tool]
    async fn api(&self, Parameters(_): Parameters<NoArgs>) -> CallToolResult {
        CallToolResult::success(vec![ContentBlock::text(API)])
    }

    /// The build steps, like an instruction book: each step's title, the ids of the parts it adds, its parts box
    /// (part, color, count), and the sub-assemblies it puts on; then each sub-assembly's own steps and how many copies
    /// the model has. Runs the model first if build/ is out of date.
    #[tool(output_schema = schema_for_output::<Envelope<emmet_core::steps::Steps>>())]
    async fn steps(&self, Parameters(_): Parameters<NoArgs>) -> CallToolResult {
        let started = Instant::now();
        let (data, me) = (self.data.clone(), self.clone());
        let (out, root) = blocking(
            move || with_project(&me, |p| emmet_core::steps::steps(&data, p)),
            |e| (Err(e), None),
        )
        .await;
        result(started, out, RunMeta::default(), root, |s| {
            let titles: Vec<String> = s
                .steps
                .iter()
                .map(|st| {
                    let title = if st.title.is_empty() {
                        "(untitled)"
                    } else {
                        st.title.as_str()
                    };
                    let n: usize = st.parts.iter().map(|b| b.count).sum::<usize>()
                        + st.submodels.iter().map(|m| m.count).sum::<usize>();
                    format!("{}. {title} ({n})", st.n)
                })
                .collect();
            format!("{}.\n{}", emmet_core::steps::summary(s), titles.join("\n"))
        })
    }

    /// The technique library: proven building blocks (gable roof, SNOT panel, window frame, track curve, …) that
    /// `m.use(name, …)` applies. With no name, the list; with a name, its doc, an example to copy, and its parts.
    #[tool(output_schema = schema_for_output::<Envelope<emmet_core::technique::Techniques>>())]
    async fn technique(&self, Parameters(args): Parameters<TechniqueArgs>) -> CallToolResult {
        let started = Instant::now();
        let out = emmet_core::technique::techniques(args.name.as_deref());
        result(started, out, RunMeta::default(), None, technique_text)
    }

    /// LDraw colors: code, name, RGB, finish, and other accepted spellings.
    #[tool(output_schema = schema_for_output::<Envelope<ColorList>>())]
    async fn list_colors(&self, Parameters(args): Parameters<ColorsArgs>) -> CallToolResult {
        let started = Instant::now();
        let data = self.data.clone();
        let out = blocking(move || ops::list_colors(&data, args.part.as_deref()), Err).await;
        result(started, out, RunMeta::default(), None, |r| {
            let solid: Vec<String> = r
                .colors
                .iter()
                .filter(|c| c.finish == "solid")
                .map(|c| format!("{} {}", c.code, c.name))
                .collect();
            format!("{} colors. Solid: {}.", r.colors.len(), solid.join(", "))
        })
    }
}

/// The list, one technique a line; or one technique's doc and example.
pub fn technique_text(t: &emmet_core::technique::Techniques) -> String {
    match &t.technique {
        Some(one) => format!(
            "{}\n\nExample:\n{}\n\nParts: {}",
            one.doc,
            one.snippet,
            one.parts.join(", ")
        ),
        None => t
            .techniques
            .iter()
            .map(|s| format!("{}: {}", s.name, s.title))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

/// The v1 summary, like a compiler's error list: the totals by severity and rule, then the first findings, errors
/// first and in source order, each `F-0001 error floating model.py:41 at (3, 4, 2): message Hint.`, and what to do
/// next.
fn validate_text(c: &Checked) -> String {
    let allowed = match c.allowed.len() {
        0 => String::new(),
        1 => " (1 allowed)".into(),
        n => format!(" ({n} allowed)"),
    };
    if c.total == 0 {
        if !c.only.is_empty() {
            return format!(
                "No {} findings{allowed}. Only those rules were checked: validate with no rules to check them all.",
                c.only.join(", ")
            );
        }
        return format!("No findings{allowed}: the model is clean.");
    }
    let n = |k: usize, one: &str, many: &str| format!("{k} {}", if k == 1 { one } else { many });
    let mut head = Vec::new();
    if c.errors > 0 {
        head.push(n(c.errors, "error", "errors"));
    }
    if c.warnings > 0 {
        head.push(n(c.warnings, "warning", "warnings"));
    }
    let infos = c.total - c.errors - c.warnings;
    if infos > 0 {
        head.push(n(infos, "note", "notes"));
    }
    let counts: Vec<String> = c.counts_by_rule.iter().map(|(r, n)| format!("{n} {r}")).collect();
    let mut lines = vec![format!("{}{allowed}: {}.", head.join(", "), counts.join(", "))];
    for f in c.findings.iter().take(SUMMARY_FINDINGS) {
        let severity = serde_json::to_value(f.severity)
            .ok()
            .and_then(|s| s.as_str().map(String::from))
            .unwrap_or_default();
        let at = f
            .source
            .first()
            .map_or(String::new(), |s| format!(" {}:{}", s.file, s.line));
        let n = |v: f64| emmet_core::ldraw::num(v);
        lines.push(format!(
            "{} {severity} {}{at} at ({}, {}, {}): {} {}",
            f.id,
            f.rule.name(),
            n(f.at[0]),
            n(f.at[1]),
            n(f.at[2]),
            f.message,
            f.hint
        ));
    }
    let more = if c.total > SUMMARY_FINDINGS {
        format!(
            "…and {} more; the full list is in build/validation.json.",
            c.total - SUMMARY_FINDINGS
        )
    } else {
        "The full list is in build/validation.json.".into()
    };
    let next = if c.errors > 0 {
        " Fix the errors first, top down (one fix often clears the findings after it), then validate again."
    } else {
        " No errors: fix the warnings that matter for this build."
    };
    lines.push(format!("{more}{next}"));
    lines.join("\n")
}

fn with_project<T>(me: &EmmetMcp, f: impl FnOnce(&Project) -> Result<T>) -> (Result<T>, Option<String>) {
    match me.find_project() {
        Ok(p) => (f(&p), Some(p.root.display().to_string())),
        Err(e) => (Err(e), None),
    }
}

fn size(s: Option<emmet_core::index::Size>) -> String {
    s.map(|s| {
        let n = |v: f64| emmet_core::ldraw::num((v * 100.0).round() / 100.0);
        let plates = if n(s.z) == "1" { "plate" } else { "plates" };
        format!(": {} x {} studs, {} {plates}", n(s.x), n(s.y), n(s.z))
    })
    .unwrap_or_default()
}

impl EmmetMcp {
    /// The session's pending redirect notes, taken from the app over the bus (D-33). None without `--session`, or
    /// when the app doesn't answer.
    fn take_notes(&self) -> Vec<emmet_core::envelope::UserNote> {
        let Some(session) = &self.session else {
            return vec![];
        };
        let reply = emmet_bus::send(
            &self.data,
            emmet_bus::Message::Notes {
                session: session.clone(),
            },
            std::time::Duration::from_secs(5),
        );
        reply
            .answer
            .filter(|_| reply.delivered)
            .and_then(|a| serde_json::from_value(a).ok())
            .unwrap_or_default()
    }
}

/// Puts the user's notes at the front of a tool result: in the envelope's `user_notes`, and as a first text line.
fn attach_notes(result: &mut CallToolResult, notes: Vec<emmet_core::envelope::UserNote>) {
    if notes.is_empty() {
        return;
    }
    let said: Vec<String> = notes.iter().map(|n| format!("\"{}\"", n.text)).collect();
    result.content.insert(
        0,
        ContentBlock::text(format!(
            "The user just said: {}. This overrides your current plan: acknowledge it in one line, then change course.",
            said.join(" and then ")
        )),
    );
    let value = serde_json::to_value(notes).expect("notes serialize");
    match result.structured_content.as_mut().and_then(Value::as_object_mut) {
        Some(env) => {
            env.insert("user_notes".into(), value);
        }
        None => result.structured_content = Some(serde_json::json!({ "user_notes": value })),
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for EmmetMcp {
    /// The router's call, plus the session's notes (the macro writes the rest of the handler).
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResponse, ErrorData> {
        let tcc = rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
        let mut response = self.tool_router.call(tcc).await?;
        if self.session.is_some()
            && let CallToolResponse::Complete(result) = &mut response
        {
            let me = self.clone();
            let notes = tokio::task::spawn_blocking(move || me.take_notes())
                .await
                .unwrap_or_default();
            attach_notes(result, notes);
        }
        Ok(response)
    }

    fn get_info(&self) -> ServerConfig {
        let mut info = ServerConfig::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("emmet", emmet_core::version());
        info.instructions = Some(
            "Emmet builds brick models from real LDraw parts. Edit model.py, call run_model, then validate, and fix \
             what they report. Find parts with search_parts and check one with part_info. Look at the result with \
             render. Units: x right and y back in studs, z up in plates."
                .into(),
        );
        info
    }
}

/// Serves MCP on stdin and stdout until the client closes them. `session` (the app's chat) turns on redirect notes.
pub fn serve(data: DataDir, project: Option<PathBuf>, session: Option<String>) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::internal(format!("starting the MCP runtime: {e}")))?;
    rt.block_on(async move {
        let service = EmmetMcp::new(data, project, session)
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| Error::internal(format!("starting the MCP server: {e}")))?;
        service
            .waiting()
            .await
            .map_err(|e| Error::internal(format!("the MCP server stopped: {e}")))?;
        Ok(())
    })
}

/// One tool's contract, as written to docs/spec/mcp/<tool>.schema.json.
#[derive(Serialize)]
struct ToolSchema {
    name: String,
    description: String,
    input: Value,
    output: Value,
}

/// Writes each tool's input and output schema to `dir`, from the same router the server serves. Returns the count.
pub fn write_schemas(dir: &Path) -> Result<usize> {
    let server = EmmetMcp::new(DataDir::at(dir), None, None);
    let tools = server.tool_router.list_all();
    std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    for t in &tools {
        let schema = ToolSchema {
            name: t.name.to_string(),
            description: t.description.as_deref().unwrap_or("").to_string(),
            input: Value::Object(Arc::unwrap_or_clone(t.input_schema.clone())),
            output: t
                .output_schema
                .clone()
                .map(|s| Value::Object(Arc::unwrap_or_clone(s)))
                .unwrap_or(Value::Null),
        };
        let text = serde_json::to_string_pretty(&schema).expect("schema serializes") + "\n";
        emmet_core::fsx::write_atomic(&dir.join(format!("{}.schema.json", t.name)), text.as_bytes())?;
    }
    Ok(tools.len())
}
