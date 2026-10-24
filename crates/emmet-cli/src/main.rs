// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `emmet` CLI. Only `--version` exists so far.

use clap::{CommandFactory, Parser};
use serde_json::json;

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
}

fn main() {
    let cli = Cli::parse();
    let version = emmet_core::version();
    if !cli.version {
        // Commands arrive in 1A; until then anything else is a usage error (exit 2, plan section 09).
        Cli::command().print_help().ok();
        std::process::exit(2);
    }
    if cli.json {
        let envelope =
            json!({ "ok": true, "data": { "version": version }, "warnings": [], "meta": { "emmet": version } });
        println!("{envelope}");
    } else {
        println!("emmet {version}");
    }
}
