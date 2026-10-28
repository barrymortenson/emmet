// SPDX-License-Identifier: AGPL-3.0-or-later
//! The engine the CLI, the MCP server, and the app share: LDraw files, the library and its parts index, colors,
//! projects, running a model, the validator, the BOM, the parts data pack, build steps, the technique library, and
//! the JSON envelope.

pub mod base64;
pub mod bom;
pub mod check;
pub mod colors;
pub mod data;
pub mod envelope;
pub mod export;
pub mod fsx;
pub mod index;
pub mod ldraw;
pub mod library;
pub mod ops;
pub mod parts_data;
pub mod project;
pub mod python;
pub mod run;
pub mod steps;
pub mod technique;
pub mod text;

/// The workspace version, which the CLI reports. The desktop app and emmet_dsl keep their own copies until
/// `just release` (6A) bumps them together.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
