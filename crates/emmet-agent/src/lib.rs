// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Claude Code and Codex adapters (1C). Only the version exists so far.

/// This crate's version, which tracks the workspace.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
