// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `emmet` CLI (plan section 09). Every command calls one `emmet_core` operation; this file parses arguments and
//! prints the result, as the JSON envelope with `--json` or as text.

mod mcp;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use clap::{CommandFactory, Parser, Subcommand};
use emmet_core::data::DataDir;
use emmet_core::envelope::{Envelope, Error, Meta, Result};
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
    /// The app data folder: library, parts index, Python (default: EMMET_DATA_DIR, else the platform's).
    #[arg(long, global = true, value_name = "DIR")]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
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
    /// List colors, or the colors a part comes in.
    Colors {
        #[arg(long)]
        part: Option<String>,
    },
    /// Manage the LDraw library.
    Library {
        #[command(subcommand)]
        action: LibraryAction,
    },
    /// Run the stdio MCP server.
    Mcp {
        /// Set by the app only: turns on redirect notes.
        #[arg(long)]
        session: Option<String>,
        /// Write each tool's input and output schema to DIR and exit.
        #[arg(long, value_name = "DIR", hide = true)]
        write_schemas: Option<PathBuf>,
    },
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

/// What a command produced: its data and a human-readable rendering of it.
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

fn size_text(size: Option<emmet_core::index::Size>) -> String {
    let Some(s) = size else { return String::new() };
    let n = |v: f64| emmet_core::ldraw::num((v * 100.0).round() / 100.0);
    format!("  [{} x {} studs, {} plates]", n(s.x), n(s.y), n(s.z))
}

/// Prints the envelope (`--json`) or the text, and turns the result into the exit code (plan section 09).
fn finish<T: Serialize>(cli: &Cli, started: Instant, out: Output<T>, text: impl FnOnce(&T) -> String) -> ExitCode {
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
        let env = Envelope::from_result(out.result, vec![], meta);
        println!("{}", serde_json::to_string(&env).expect("envelope serializes"));
    } else {
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
