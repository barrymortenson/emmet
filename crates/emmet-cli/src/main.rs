// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `emmet` CLI (plan section 09). Every command calls one `emmet_core` operation; this file parses arguments and
//! prints the result, as the JSON envelope with `--json` or as text.

mod eval;
mod host;
mod mcp;
mod render;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use clap::{CommandFactory, Parser, Subcommand};
use emmet_core::data::DataDir;
use emmet_core::envelope::{Envelope, Error, Meta, Result};
use emmet_core::project::{Project, Template};
use emmet_core::{library, ops};
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "emmet",
    about = "Emmet: brick models from real LDraw parts",
    disable_version_flag = true
)]
struct Cli {
    /// Print the version.
    #[arg(short = 'V', long)]
    version: bool,
    /// Print the result as the JSON envelope (plan section 09).
    #[arg(long, global = true)]
    json: bool,
    /// The project (default: the nearest folder with an emmet.toml, from the current directory up).
    #[arg(long, global = true, value_name = "DIR")]
    project: Option<PathBuf>,
    /// The app data folder: library, parts index, Python (default: EMMET_DATA_DIR, else .dev-data/ in a debug build,
    /// else the platform's).
    #[arg(long, global = true, value_name = "DIR")]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Create a project: emmet.toml, model.py, params.json, CARD.md, AGENTS.md and CLAUDE.md, .gitignore, and a
    /// trust marker.
    New {
        dir: PathBuf,
        #[arg(long, value_enum, default_value_t = TemplateArg::Cottage)]
        template: TemplateArg,
    },
    /// Run the model and write build/model.mpd, placements.json, and manifest.json.
    Run {
        /// Parameter values for this run only, on top of params.json.
        #[arg(long = "params", value_name = "K=V", num_args = 1..)]
        params: Vec<String>,
        /// Stop the run (and its Python) when stdin closes. The app's run queue cancels a run it no longer needs this
        /// way.
        #[arg(long, hide = true)]
        cancel_on_eof: bool,
    },
    /// Check the model for collisions, floating parts, and unknown or obsolete parts, and write build/validation.json.
    /// Runs the model first if build/ is out of date.
    Check {
        /// Exit 1 when there's an error-severity finding.
        #[arg(long)]
        strict: bool,
        /// Only these rules (repeatable): collision, floating, disconnected_submodel, step_order, step_size,
        /// weak_connection, off_grid, part_unavailable, color_unavailable, unknown_part, obsolete_part, alias_part.
        #[arg(long = "rule", value_name = "RULE")]
        rules: Vec<String>,
        /// Check the model run with these values, on top of params.json (which isn't changed).
        #[arg(long = "params", value_name = "K=V", num_args = 1..)]
        params: Vec<String>,
    },
    /// The parts list (part × color × quantity), written to build/bom.json. Runs the model first if build/ is out of
    /// date.
    Bom {
        /// json (the default), csv, rebrickable (Rebrickable's parts-list import), or bricklink-xml (a BrickLink
        /// wanted list).
        #[arg(long, value_enum, default_value_t = BomFormat::Json)]
        format: BomFormat,
        /// Color names as LDraw, Rebrickable, or BrickLink spell them (default: the app's setting, else ldraw).
        #[arg(long, value_enum)]
        color_names: Option<ColorNamesArg>,
    },
    /// Render the model to a PNG through the running app, or the app's hidden render mode when it isn't open. Runs
    /// the model first if build/ is out of date.
    Render {
        #[arg(long, default_value = "iso", value_parser = clap::builder::PossibleValuesParser::new(render::VIEWS))]
        view: String,
        #[arg(long, default_value_t = 1024)]
        size: u32,
        /// Emmet ids to highlight, comma-separated.
        #[arg(long, value_delimiter = ',')]
        highlight: Vec<String>,
        /// Where to write the PNG (default: build/renders/<view>.png).
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    /// Show the model's parameters and values, or set values in params.json.
    Params {
        /// Set a value (repeatable). A value equal to the default is dropped from params.json.
        #[arg(long = "set", value_name = "K=V", num_args = 1..)]
        set: Vec<String>,
    },
    /// Trust the project on this machine, so its model.py may run.
    Trust,
    /// Check the library, the parts index, and Python; --fix sets up what it can.
    Doctor {
        #[arg(long)]
        fix: bool,
    },
    /// Search parts by words and dimensions ("brick 2x4", "slope 45 2 x 2").
    Search {
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Show one part: name, size in studs and plates, flags.
    Part { id: String },
    /// List colors (--part: only the colors that part was made in, by the parts data pack).
    Colors {
        #[arg(long)]
        part: Option<String>,
    },
    /// Manage the LDraw library.
    Library {
        #[command(subcommand)]
        action: LibraryAction,
    },
    /// Print the doctrine: how an agent builds with Emmet (the same text as the MCP `rules` tool).
    Rules,
    /// Print the emmet_dsl vocabulary for model.py (the same text as the MCP `api` tool).
    Api,
    /// Run the stdio MCP server.
    Mcp {
        /// The app's chat session (set only by the app): every tool result carries the user's redirect notes.
        #[arg(long)]
        session: Option<String>,
        /// Write each tool's input and output schema to DIR and exit.
        #[arg(long, value_name = "DIR", hide = true)]
        write_schemas: Option<PathBuf>,
    },
    /// Live eval (plan section 16.1): one run of a ladder rung through a real agent CLI, on the eval API keys (D-37),
    /// scored and written to docs/evals/. It says what it bills and asks first (D-38). A dev tool.
    #[command(hide = !cfg!(debug_assertions))]
    Eval {
        #[arg(long, default_value = "1")]
        rung: String,
        #[arg(long, value_enum)]
        agent: AgentArg,
        /// For the fake agents (CI): no keychain, no question before it starts.
        #[arg(long)]
        dry_run: bool,
        /// Don't ask before a live run.
        #[arg(long)]
        yes: bool,
        /// Where the report and its render go (default: docs/evals/).
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
        /// Where to make the run's project (default: a new temp folder).
        #[arg(long, value_name = "DIR")]
        work: Option<PathBuf>,
    },
    /// The app-internal `emmet_host` MCP server for a chat session: approve and ask_user, over the bus.
    #[command(hide = true)]
    McpHost {
        #[arg(long)]
        session: String,
    },
    /// The parts data pack (plan section 12): build one from a Rebrickable catalog, install one, or roll back. For
    /// scripts/build-parts-data, dev, and tests.
    #[command(hide = true)]
    PartsData {
        #[command(subcommand)]
        action: PartsDataAction,
    },
}

#[derive(Subcommand)]
enum PartsDataAction {
    /// Build a pack from the catalog's .csv files: the SQLite file and parts-data.json in --out.
    Build {
        catalog: PathBuf,
        #[arg(long, value_name = "DIR")]
        out: PathBuf,
        /// The previous pack's parts-data.json: row counts must be within 10 % of it.
        #[arg(long, value_name = "FILE")]
        previous: Option<PathBuf>,
        /// ldraw-mini (tests/fixtures/ldraw-mini): every real part in its PARTS.txt must map.
        #[arg(long, value_name = "DIR")]
        ldraw_mini: Option<PathBuf>,
        /// The pack's date and version, YYYY-MM-DD (default: today, UTC).
        #[arg(long)]
        date: Option<String>,
        /// A PKCS#8 Ed25519 private key (DER) to sign parts-data.json with.
        #[arg(long, value_name = "FILE")]
        key: Option<PathBuf>,
    },
    /// Install a signed pack (a folder with parts-data.json, its .sig, and the SQLite file) after checking it.
    Install { dir: PathBuf },
    /// Go back to the pack installed before the current one.
    Rollback,
    /// The installed pack (after installing the bundled one when it's newer), or null.
    Status,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum AgentArg {
    Claude,
    Codex,
}

#[derive(Clone, Copy, PartialEq, clap::ValueEnum)]
enum BomFormat {
    Json,
    Csv,
    /// Rebrickable's parts-list import CSV.
    Rebrickable,
    /// A BrickLink wanted list.
    BricklinkXml,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum ColorNamesArg {
    Ldraw,
    Rebrickable,
    Bricklink,
}

impl From<ColorNamesArg> for emmet_core::bom::ColorNames {
    fn from(a: ColorNamesArg) -> Self {
        match a {
            ColorNamesArg::Ldraw => Self::Ldraw,
            ColorNamesArg::Rebrickable => Self::Rebrickable,
            ColorNamesArg::Bricklink => Self::Bricklink,
        }
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum TemplateArg {
    Cottage,
    Empty,
}

#[derive(Subcommand)]
enum LibraryAction {
    /// Show what's installed and what this Emmet pins.
    Status,
    /// Download and install the pinned library, then build the parts index.
    Install {
        /// Install from a complete.zip on disk instead of downloading.
        #[arg(long, value_name = "FILE")]
        zip: Option<PathBuf>,
        /// Download from this URL instead of the pinned one (with --sha256 and --release).
        #[arg(long, hide = true)]
        url: Option<String>,
        #[arg(long, hide = true)]
        sha256: Option<String>,
        #[arg(long, hide = true)]
        release: Option<String>,
    },
    /// Check every installed file against its checksum.
    Verify {
        /// Rebuild the parts index too.
        #[arg(long)]
        reindex: bool,
    },
}

/// What a command produced, and the facts its envelope's meta reports. `finish` renders it as JSON or text.
struct Output<T> {
    result: Result<T>,
    project: Option<String>,
    inputs_hash: Option<String>,
    generation: Option<u64>,
}

impl<T> From<Result<T>> for Output<T> {
    fn from(result: Result<T>) -> Self {
        Self {
            result,
            project: None,
            inputs_hash: None,
            generation: None,
        }
    }
}

fn main() -> ExitCode {
    let started = Instant::now();
    // A panic is a bug in Emmet, but a --json caller still gets an envelope it can read, not a stack trace.
    if std::env::args().any(|a| a == "--json") {
        std::panic::set_hook(Box::new(|info| {
            let env = Envelope::<()>::from_result(
                Err(Error::internal(format!("emmet crashed: {info}"))),
                vec![],
                Meta {
                    emmet: emmet_core::version().into(),
                    ..Default::default()
                },
            );
            println!("{}", serde_json::to_string(&env).expect("envelope serializes"));
            std::process::exit(3);
        }));
    }
    let cli = Cli::parse();
    if cli.version {
        return finish(
            &cli,
            started,
            Output::from(Ok(serde_json::json!({ "version": emmet_core::version() }))),
            |_| format!("emmet {}", emmet_core::version()),
        );
    }
    let Some(command) = &cli.command else {
        Cli::command().print_help().ok();
        return ExitCode::from(2);
    };
    let data = DataDir::resolve(cli.data_dir.as_deref());
    match command {
        Command::New { dir, template } => {
            let template = match template {
                TemplateArg::Cottage => Template::Cottage,
                TemplateArg::Empty => Template::Empty,
            };
            let out = data.and_then(|d| ops::new_project(&d, dir, template));
            let project = out.as_ref().ok().map(|c| c.project.clone());
            let mut out = Output::from(out);
            out.project = project;
            finish(&cli, started, out, |c| {
                format!(
                    "Created {} in {}. Next: emmet run --project {}",
                    c.name,
                    c.project,
                    dir.display()
                )
            })
        }
        Command::Run { params, cancel_on_eof } => {
            let mut meta = emmet_core::run::RunMeta::default();
            let mut root = None;
            let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            if *cancel_on_eof {
                let flag = cancel.clone();
                std::thread::spawn(move || {
                    let _ = std::io::copy(&mut std::io::stdin(), &mut std::io::sink());
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                });
            }
            let out = data.and_then(|d| {
                let overrides = key_values(params)?;
                let project = Project::find(cli.project.as_deref())?;
                root = Some(project.root.display().to_string());
                let (result, m) = ops::run_model_until(&d, &project, &overrides, &cancel);
                meta = m;
                result
            });
            let out = Output {
                result: out,
                project: root,
                inputs_hash: meta.inputs_hash,
                generation: meta.generation,
            };
            finish(&cli, started, out, |r| {
                format!(
                    "Built {} parts in {} steps ({:.2} s) → build/model.mpd",
                    r.parts,
                    r.steps,
                    r.ms as f64 / 1000.0
                )
            })
        }
        Command::Check { strict, rules, params } => {
            let mut root = None;
            let mut meta = (None, None);
            let out = data.and_then(|d| {
                let rules = rules
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
                let project = Project::find(cli.project.as_deref())?;
                root = Some(project.root.display().to_string());
                let v = ops::check_with(&d, &project, &rules, &key_values(params)?)?;
                meta = (v.inputs_hash.clone(), None);
                Ok(v)
            });
            let errors = out.as_ref().map_or(0, |v| v.errors());
            let out = Output {
                result: out,
                project: root,
                inputs_hash: meta.0,
                generation: meta.1,
            };
            let code = finish(&cli, started, out, check_text);
            // Findings aren't errors; --strict makes error-severity findings exit 1 (plan section 09).
            if code == ExitCode::SUCCESS && *strict && errors > 0 {
                ExitCode::from(1)
            } else {
                code
            }
        }
        Command::Bom { format, color_names } => {
            let mut root = None;
            let out = data.and_then(|d| {
                let project = Project::find(cli.project.as_deref())?;
                root = Some(project.root.display().to_string());
                let names = color_names.map_or_else(|| emmet_core::bom::ColorNames::preferred(&d), Into::into);
                let bom = ops::bom(&d, &project, names)?;
                let export = match format {
                    BomFormat::Json => None,
                    BomFormat::Csv => Some((emmet_core::bom::csv(&bom), vec![])),
                    BomFormat::Rebrickable => Some(emmet_core::bom::rebrickable_csv(&bom)?),
                    BomFormat::BricklinkXml => Some(emmet_core::bom::bricklink_xml(&bom)?),
                };
                Ok((bom, export))
            });
            let warnings = out
                .as_ref()
                .ok()
                .and_then(|(_, e)| e.as_ref().map(|(_, w)| w.clone()))
                .unwrap_or_default();
            let mut output = Output::from(out.as_ref().map(|(b, _)| b.clone()).map_err(Clone::clone));
            output.project = root;
            let export = out.ok().and_then(|(_, e)| e.map(|(text, _)| text));
            finish_with(&cli, started, output, warnings, |b| {
                if let Some(text) = export {
                    return text.trim_end().to_string();
                }
                let mut lines: Vec<String> = b
                    .lines
                    .iter()
                    .map(|l| {
                        let made = match l.availability.status {
                            emmet_core::parts_data::Status::ColorUnavailable => "  (not made in this color)",
                            emmet_core::parts_data::Status::PartUnavailable => "  (not made)",
                            _ => "",
                        };
                        format!("{:>4} × {:<10} {:<24} {}{made}", l.qty, l.part, l.color_name, l.name)
                    })
                    .collect();
                lines.push(format!(
                    "{} pieces: {} parts in {} part × color lines → build/bom.json",
                    b.totals.pieces, b.totals.parts, b.totals.lines
                ));
                if b.parts_data.is_some() {
                    lines.push(emmet_core::parts_data::CREDIT.to_string());
                }
                lines.join("\n")
            })
        }
        Command::Render {
            view,
            size,
            highlight,
            out,
        } => {
            let mut root = None;
            let result = data.and_then(|d| {
                let project = Project::find(cli.project.as_deref())?;
                root = Some(project.root.display().to_string());
                let png = render::render(&d, &project, view, *size, highlight)?;
                let path = match out {
                    Some(p) => std::path::absolute(p).unwrap_or_else(|_| p.clone()),
                    None => render::default_out(&project, view)?,
                };
                render::write_png(&path, &png)?;
                let mut info = png.info;
                info.path = Some(path.display().to_string());
                Ok(info)
            });
            let mut out = Output::from(result);
            out.project = root;
            finish(&cli, started, out, |r| {
                format!(
                    "Rendered the {} view at {} px ({}) → {}",
                    r.view,
                    r.size,
                    if r.via == "app" {
                        "in the open app"
                    } else {
                        "in render mode"
                    },
                    r.path.as_deref().unwrap_or("")
                )
            })
        }
        Command::Params { set } => {
            let mut root = None;
            let out = data.and_then(|d| {
                let values = key_values(set)?;
                let project = Project::find(cli.project.as_deref())?;
                root = Some(project.root.display().to_string());
                if values.is_empty() {
                    ops::get_params(&d, &project)
                } else {
                    ops::set_params(&d, &project, &values)
                }
            });
            let mut out = Output::from(out);
            out.project = root;
            finish(&cli, started, out, |p| {
                let mut lines = vec![format!("{} ({})", p.model, p.file)];
                for spec in &p.params {
                    let range = match (spec.min, spec.max) {
                        (Some(a), Some(b)) => format!(" [{}–{}]", emmet_core::ldraw::num(a), emmet_core::ldraw::num(b)),
                        _ => String::new(),
                    };
                    let mark = if p.changed.contains(&spec.name) {
                        " (changed)"
                    } else {
                        ""
                    };
                    lines.push(format!(
                        "  {} = {}{range}  {}{mark}",
                        spec.name, p.values[&spec.name], spec.help
                    ));
                }
                lines.join("\n")
            })
        }
        Command::Trust => {
            let out = data.and_then(|d| {
                let project = Project::find(cli.project.as_deref())?;
                ops::trust(&d, &project)
            });
            finish(&cli, started, out.into(), |t| {
                format!("Trusted {} on this machine.", t.project)
            })
        }
        Command::Doctor { fix } => {
            let out = data.map(|d| ops::doctor(&d, *fix));
            let ready = out.as_ref().is_ok_and(|d| d.ready);
            let code = finish(&cli, started, out.into(), |d| {
                d.checks
                    .iter()
                    .map(|c| {
                        let fix = c.fix.as_ref().map_or(String::new(), |f| format!("  → {f}"));
                        format!("{} {:<12} {}{fix}", if c.ok { "ok " } else { "!! " }, c.name, c.detail)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            });
            // Exit 4 (environment) when something isn't ready; the envelope still carries every check.
            if code == ExitCode::SUCCESS && !ready {
                ExitCode::from(4)
            } else {
                code
            }
        }
        Command::Search { query, category, limit } => {
            let q = query.join(" ");
            let out = data.and_then(|d| ops::search_parts(&d, &q, category.as_deref(), *limit));
            finish(&cli, started, out.into(), |r| {
                if r.parts.is_empty() {
                    return format!("No parts match '{}'.", r.query);
                }
                r.parts
                    .iter()
                    .map(|p| format!("{:<10} {}{}", p.id, p.name, size_text(p.size)))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        Command::Part { id } => {
            let out = data.and_then(|d| ops::part_info(&d, id));
            finish(&cli, started, out.into(), |p| {
                let mut s = format!(
                    "{}  {}\ncategory: {}{}",
                    p.part.id,
                    p.part.name,
                    p.part.category,
                    size_text(p.part.size)
                );
                if !p.part.flags.is_empty() {
                    s.push_str(&format!("\nflags: {}", p.part.flags.join(", ")));
                }
                if let Some(a) = &p.part.alias_of {
                    s.push_str(&format!("\nuse instead: {a}"));
                }
                s
            })
        }
        Command::Colors { part } => {
            let out = data.and_then(|d| ops::list_colors(&d, part.as_deref()));
            finish(&cli, started, out.into(), |r| {
                r.colors
                    .iter()
                    .map(|c| format!("{:>5}  {}  {}", c.code, c.rgb, c.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        Command::Library { action } => library_command(&cli, started, data, action),
        Command::Rules => finish(&cli, started, Output::from(Ok(mcp::DOCTRINE)), |d| {
            d.trim_end().to_string()
        }),
        Command::Api => finish(&cli, started, Output::from(Ok(mcp::API)), |d| d.trim_end().to_string()),
        Command::Eval {
            rung,
            agent,
            dry_run,
            yes,
            out,
            work,
        } => {
            let opts = eval::Options {
                rung: rung.clone(),
                agent: match agent {
                    AgentArg::Claude => emmet_agent::AgentKind::Claude,
                    AgentArg::Codex => emmet_agent::AgentKind::Codex,
                },
                dry_run: *dry_run,
                yes: *yes,
                out: out.clone(),
                work: work.clone(),
            };
            let result = data.and_then(|d| eval::eval(&d, &opts));
            let pass = result.as_ref().is_ok_and(|r| r.pass);
            let code = finish(&cli, started, result.into(), |r| {
                let mut lines = vec![format!(
                    "Rung 1 on {}: {} ({} min {} s, {} turns)",
                    r.run.agent,
                    if r.pass { "pass" } else { "fail" },
                    r.run.seconds / 60,
                    r.run.seconds % 60,
                    r.run.turns
                )];
                for s in &r.scores {
                    lines.push(format!(
                        "  {} {}: {}",
                        if s.pass { "ok " } else { "!! " },
                        s.name,
                        s.detail
                    ));
                }
                lines.push(format!("Project: {}", r.project));
                lines.join("\n")
            });
            // A run that finished but failed its scores exits 1, like a check under --strict.
            if code == ExitCode::SUCCESS && !pass {
                ExitCode::from(1)
            } else {
                code
            }
        }
        Command::PartsData { action } => parts_data_command(&cli, started, data, action),
        Command::McpHost { session } => {
            let out = data.and_then(|d| {
                let project = Project::find(cli.project.as_deref())?;
                host::serve(d, project.root, session.clone())
            });
            match out {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => finish::<()>(&cli, started, Err(e).into(), |_| String::new()),
            }
        }
        Command::Mcp { session, write_schemas } => {
            if let Some(dir) = write_schemas {
                let out = mcp::write_schemas(dir);
                return finish(&cli, started, out.into(), |n| {
                    format!("Wrote {n} schemas to {}.", dir.display())
                });
            }
            let out = data.and_then(|d| mcp::serve(d, cli.project.clone(), session.clone()));
            match out {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => finish::<()>(&cli, started, Err(e).into(), |_| String::new()),
            }
        }
    }
}

fn library_command(cli: &Cli, started: Instant, data: Result<DataDir>, action: &LibraryAction) -> ExitCode {
    match action {
        LibraryAction::Status => {
            let out = data.map(|d| library::status(&d));
            finish(cli, started, out.into(), |s| match &s.installed {
                Some(i) => format!(
                    "LDraw library {} ({} files) in {}\nparts index: {}\nthis Emmet pins {}",
                    i.release,
                    i.files,
                    s.path,
                    s.indexed_parts.map_or("not built".into(), |n| format!("{n} parts")),
                    s.pinned
                ),
                None => format!(
                    "No LDraw library in {}. Run emmet library install (release {}).",
                    s.path, s.pinned
                ),
            })
        }
        LibraryAction::Install {
            zip,
            url,
            sha256,
            release,
        } => {
            let out = data.and_then(|d| {
                let sha = sha256.as_deref().unwrap_or(library::SHA256);
                let release = release.as_deref().unwrap_or(library::RELEASE);
                let source = match zip {
                    Some(z) => library::Source::Zip(z),
                    None => library::Source::Url(url.as_deref().unwrap_or(library::URL)),
                };
                let mut last = 0u64;
                let quiet = cli.json;
                let mut progress = |done: u64, total: Option<u64>| {
                    if quiet || done < last + 8_000_000 && Some(done) != total {
                        return;
                    }
                    last = done;
                    match total {
                        Some(t) => eprintln!(
                            "downloading the LDraw library: {:.0} of {:.0} MB",
                            done as f64 / 1e6,
                            t as f64 / 1e6
                        ),
                        None => eprintln!("downloading the LDraw library: {:.0} MB", done as f64 / 1e6),
                    }
                };
                library::install(&d, release, sha, source, &mut progress)
            });
            finish(cli, started, out.into(), |s| {
                let i = s.installed.as_ref();
                format!(
                    "Installed LDraw library {} ({} files) and indexed {} parts.",
                    i.map_or("?", |i| i.release.as_str()),
                    i.map_or(0, |i| i.files),
                    s.indexed_parts.unwrap_or(0)
                )
            })
        }
        LibraryAction::Verify { reindex } => {
            let out = data.and_then(|d| library::verify(&d, *reindex));
            finish(cli, started, out.into(), |v| {
                let mut s = format!("LDraw library {}: all {} files match.", v.release, v.files);
                if let Some(n) = v.reindexed_parts {
                    s.push_str(&format!(" Rebuilt the parts index: {n} parts."));
                }
                s
            })
        }
    }
}

fn parts_data_command(cli: &Cli, started: Instant, data: Result<DataDir>, action: &PartsDataAction) -> ExitCode {
    use emmet_core::parts_data;
    let text = |m: &parts_data::Manifest| {
        let rows: Vec<String> = m.rows.iter().map(|(t, n)| format!("{t} {n}")).collect();
        format!(
            "Parts data {} ({} bytes, sha256 {}): {}.",
            m.version,
            m.size,
            m.sha256,
            rows.join(", ")
        )
    };
    match action {
        PartsDataAction::Build {
            catalog,
            out,
            previous,
            ldraw_mini,
            date,
            key,
        } => {
            let result = (|| {
                let key = match key {
                    Some(k) => Some(std::fs::read(k).map_err(|e| Error::io(k, e))?),
                    None => None,
                };
                let opts = parts_data::BuildOptions {
                    date: date.clone(),
                    previous: previous.clone(),
                    ldraw_mini: ldraw_mini.clone(),
                    key,
                };
                parts_data::build(catalog, out, &opts)
            })();
            finish(cli, started, result.into(), |m| format!("Built {}", text(m)))
        }
        PartsDataAction::Install { dir } => {
            let out = data.and_then(|d| parts_data::install(&d, dir, parts_data::TRUSTED_KEY));
            finish(cli, started, out.into(), |m| format!("Installed {}", text(m)))
        }
        PartsDataAction::Rollback => {
            let out = data.and_then(|d| parts_data::rollback(&d));
            finish(cli, started, out.into(), |m| format!("Back to {}", text(m)))
        }
        PartsDataAction::Status => {
            let out = data.and_then(|d| Ok(parts_data::PartsData::open(&d)?.map(|p| p.manifest().clone())));
            finish(cli, started, out.into(), |m| match m {
                Some(m) => format!("Installed {}", text(m)),
                None => "No parts data pack is installed.".into(),
            })
        }
    }
}

/// Findings as a compiler-style list: `error[collision] F-0001: …`, then `--> model.py:41` and `help:`.
fn check_text(v: &emmet_core::check::Validation) -> String {
    if v.findings.is_empty() {
        let allowed = if v.allowed.is_empty() {
            String::new()
        } else {
            format!(" ({} allowed in emmet.toml)", v.allowed.len())
        };
        return format!("No findings{allowed}. → build/validation.json");
    }
    let mut out = Vec::new();
    for f in &v.findings {
        let severity = serde_json::to_value(f.severity)
            .ok()
            .and_then(|s| s.as_str().map(String::from))
            .unwrap_or_default();
        out.push(format!("{severity}[{}] {}: {}", f.rule.name(), f.id, f.message));
        for s in &f.source {
            out.push(format!("  --> {}:{}", s.file, s.line));
        }
        out.push(format!("help: {}", f.hint));
    }
    let counts: Vec<String> = v.counts.iter().map(|(r, n)| format!("{n} {r}")).collect();
    out.push(format!(
        "{} findings ({}) → build/validation.json",
        v.findings.len(),
        counts.join(", ")
    ));
    out.join("\n")
}

/// `k=v` pairs as parameter values: JSON when the value parses as JSON (8, true, "Tan"), else the text itself.
fn key_values(pairs: &[String]) -> Result<std::collections::BTreeMap<String, serde_json::Value>> {
    let mut out = std::collections::BTreeMap::new();
    for pair in pairs {
        let Some((k, v)) = pair.split_once('=') else {
            return Err(Error::new(
                emmet_core::envelope::ErrorCode::ParamInvalid,
                format!("'{pair}' isn't name=value."),
                "Write each value as name=value, for example width=10 or wall_color=\"Dark Tan\".",
            ));
        };
        let value = serde_json::from_str(v).unwrap_or_else(|_| serde_json::Value::String(v.to_string()));
        out.insert(k.trim().to_string(), value);
    }
    Ok(out)
}

fn size_text(size: Option<emmet_core::index::Size>) -> String {
    let Some(s) = size else { return String::new() };
    let n = |v: f64| emmet_core::ldraw::num((v * 100.0).round() / 100.0);
    let plates = if n(s.z) == "1" { "plate" } else { "plates" };
    format!("  [{} x {} studs, {} {plates}]", n(s.x), n(s.y), n(s.z))
}

/// Prints the envelope (`--json`) or the text, and turns the result into the exit code (plan section 09).
fn finish<T: Serialize>(cli: &Cli, started: Instant, out: Output<T>, text: impl FnOnce(&T) -> String) -> ExitCode {
    finish_with(cli, started, out, vec![], text)
}

/// [`finish`] with the envelope's warnings, which text mode prints to stderr.
fn finish_with<T: Serialize>(
    cli: &Cli,
    started: Instant,
    out: Output<T>,
    warnings: Vec<emmet_core::envelope::Warning>,
    text: impl FnOnce(&T) -> String,
) -> ExitCode {
    let code = match &out.result {
        Ok(_) => 0,
        Err(e) => e.code.exit_code(),
    };
    if cli.json {
        let meta = Meta {
            emmet: emmet_core::version().into(),
            project: out.project,
            inputs_hash: out.inputs_hash,
            generation: out.generation,
            ms: started.elapsed().as_millis() as u64,
        };
        let env = Envelope::from_result(out.result, warnings, meta);
        println!("{}", serde_json::to_string(&env).expect("envelope serializes"));
    } else {
        for w in &warnings {
            eprintln!("warning[{}]: {}", w.code, w.message);
        }
        match &out.result {
            Ok(data) => {
                let t = text(data);
                if !t.is_empty() {
                    println!("{t}");
                }
            }
            Err(e) => print_error(e),
        }
    }
    ExitCode::from(code as u8)
}

/// A compiler-style error: `error[CODE]: message`, then `--> file:line`, the excerpt, and `help:`.
fn print_error(e: &Error) {
    let code = serde_json::to_value(e.code)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    eprintln!("error[{code}]: {}", e.message);
    if let Some(d) = &e.details {
        match (&d.file, d.line) {
            (Some(f), Some(l)) => eprintln!("  --> {f}:{l}"),
            (Some(f), None) => eprintln!("  --> {f}"),
            _ => {}
        }
        if let (Some(x), Some(l)) = (&d.excerpt, d.line) {
            eprintln!("{:>5} | {x}", l);
        }
    }
    eprintln!("help: {}", e.hint);
}
