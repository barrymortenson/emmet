// SPDX-License-Identifier: AGPL-3.0-or-later
//! Path checks for everything the webview can name (plan section 15): a path is canonicalized, links included, and
//! must sit under the root it belongs to.

use std::path::{Path, PathBuf};

/// `path`, canonicalized, if it exists and sits under `root` (itself canonical). A link that leads out is refused.
pub fn inside(root: &Path, path: &Path) -> Option<PathBuf> {
    let real = emmet_core::fsx::canonical(path).ok()?;
    real.starts_with(root).then_some(real)
}
