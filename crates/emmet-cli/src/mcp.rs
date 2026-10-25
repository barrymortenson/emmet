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
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router};
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
    /// How many parts to return (default 10, at most 100).
    #[serde(default)]
    pub limit: Option<usize>,
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
pub struct SetParamsArgs {
    /// Parameter values to keep in params.json. A value equal to its default is dropped from the file.
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ColorsArgs {
    /// Only the colors this part comes in.
    #[serde(default)]
    pub part: Option<String>,
}

#[derive(Clone)]
pub struct EmmetMcp {
    data: DataDir,
    project: Option<PathBuf>,
    tool_router: ToolRouter<Self>,
}

/// Runs a blocking core operation off the async runtime.
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(f).await.expect("emmet core task panicked")
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
    pub fn new(data: DataDir, project: Option<PathBuf>) -> Self {
        Self {
            data,
            project,
            tool_router: Self::tool_router(),
        }
    }

    fn find_project(&self) -> Result<Project> {
        Project::find(self.project.as_deref())
    }

    /// Search LDraw parts by words and dimensions. Returns ids, names, categories, and sizes (x and y in studs, z in
    /// plates).
    #[tool(output_schema = schema_for_output::<Envelope<SearchResult>>())]
    async fn search_parts(&self, Parameters(args): Parameters<SearchArgs>) -> CallToolResult {
        let started = Instant::now();
        let data = self.data.clone();
        let out =
            blocking(move || ops::search_parts(&data, &args.query, args.category.as_deref(), args.limit.unwrap_or(10)))
                .await;
        result(started, out, RunMeta::default(), None, |r| {
            if r.parts.is_empty() {
                return format!("No parts match '{}'.", r.query);
            }
            r.parts
                .iter()
                .map(|p| format!("{} {}{}", p.id, p.name, size(p.size)))
                .collect::<Vec<_>>()
                .join("\n")
        })
    }

    /// One part: its name, size in studs and plates, flags (alias, obsolete, printed), and body box.
    #[tool(output_schema = schema_for_output::<Envelope<PartInfo>>())]
    async fn part_info(&self, Parameters(args): Parameters<PartArgs>) -> CallToolResult {
        let started = Instant::now();
        let data = self.data.clone();
        let out = blocking(move || ops::part_info(&data, &args.id)).await;
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
        let (out, meta, root) = blocking(move || {
            let project = match me.find_project() {
                Ok(p) => p,
                Err(e) => return (Err(e), RunMeta::default(), None),
            };
            let root = Some(project.root.display().to_string());
            let (out, meta) = ops::run_model(&data, &project, &args.params.unwrap_or_default());
            (out, meta, root)
        })
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
        let (out, root) = blocking(move || with_project(&me, |p| ops::get_params(&data, p))).await;
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
        let (out, root) = blocking(move || with_project(&me, |p| ops::set_params(&data, p, &args.values))).await;
        result(started, out, RunMeta::default(), root, |p| {
            if p.changed.is_empty() {
                return "Nothing changed.".into();
            }
            let changed: Vec<String> = p.changed.iter().map(|k| format!("{k} = {}", p.values[k])).collect();
            format!("Changed {}.", changed.join(", "))
        })
    }

    /// LDraw colors: code, name, RGB, finish, and other accepted spellings.
    #[tool(output_schema = schema_for_output::<Envelope<ColorList>>())]
    async fn list_colors(&self, Parameters(args): Parameters<ColorsArgs>) -> CallToolResult {
        let started = Instant::now();
        let data = self.data.clone();
        let out = blocking(move || ops::list_colors(&data, args.part.as_deref())).await;
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

fn with_project<T>(me: &EmmetMcp, f: impl FnOnce(&Project) -> Result<T>) -> (Result<T>, Option<String>) {
    match me.find_project() {
        Ok(p) => (f(&p), Some(p.root.display().to_string())),
        Err(e) => (Err(e), None),
    }
}

fn size(s: Option<emmet_core::index::Size>) -> String {
    s.map(|s| {
        let n = |v: f64| emmet_core::ldraw::num((v * 100.0).round() / 100.0);
        format!(": {} x {} studs, {} plates", n(s.x), n(s.y), n(s.z))
    })
    .unwrap_or_default()
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for EmmetMcp {
    fn get_info(&self) -> ServerConfig {
        let mut info = ServerConfig::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("emmet", emmet_core::version());
        info.instructions = Some(
            "Emmet builds brick models from real LDraw parts. Edit model.py, call run_model, and fix what it reports. \
             Find parts with search_parts and check one with part_info. Units: x right and y back in studs, z up in \
             plates."
                .into(),
        );
        info
    }
}

/// Serves MCP on stdin and stdout until the client closes them. `session` turns on redirect notes once the bus
/// exists (1C); until then it's accepted and unused.
pub fn serve(data: DataDir, project: Option<PathBuf>, _session: Option<String>) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::internal(format!("starting the MCP runtime: {e}")))?;
    rt.block_on(async move {
        let service = EmmetMcp::new(data, project)
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
    let server = EmmetMcp::new(DataDir::at(dir), None);
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
