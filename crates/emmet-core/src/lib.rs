// SPDX-License-Identifier: AGPL-3.0-or-later
//! The engine the CLI, the MCP server, and the app share: LDraw files, the library and its parts index, colors,
//! projects, running a model, and the JSON envelope. The validator, BOM, and steps arrive in later sessions.

pub mod envelope;
pub mod fsx;
pub mod ldraw;

/// The workspace version, which the CLI reports. The desktop app and emmet_dsl keep their own copies until
/// `just release` (6A) bumps them together.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
