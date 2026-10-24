// SPDX-License-Identifier: AGPL-3.0-or-later
//! LDraw, the parts index, the validator, the BOM, and steps. Only the version exists so far.

/// The Emmet version every front door reports.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
