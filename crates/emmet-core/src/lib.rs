// SPDX-License-Identifier: AGPL-3.0-or-later
//! LDraw, the parts index, the validator, the BOM, and steps. Only the version exists so far.

/// The workspace version, which the CLI reports. The desktop app and emmet_dsl keep their own copies until
/// `just release` (6A) bumps them together.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
