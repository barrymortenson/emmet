// SPDX-License-Identifier: AGPL-3.0-or-later
//! Recent projects, kept in `<data>/recent.json`, newest first. `open_recent` takes a path from the webview, so it
//! opens only a path on this list (after both are canonicalized).

use std::path::{Path, PathBuf};

use emmet_core::data::DataDir;
use serde::{Deserialize, Serialize};

const MAX: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Recent {
    pub path: String,
    pub name: String,
}

fn file(data: &DataDir) -> PathBuf {
    data.root().join("recent.json")
}

pub fn list(data: &DataDir) -> Vec<Recent> {
    std::fs::read_to_string(file(data))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// Moves `root` to the top of the list.
pub fn touch(data: &DataDir, root: &Path, name: &str) {
    let path = root.display().to_string();
    let mut items = list(data);
    items.retain(|r| r.path != path);
    items.insert(
        0,
        Recent {
            path,
            name: name.to_string(),
        },
    );
    items.truncate(MAX);
    let text = serde_json::to_string_pretty(&items).expect("json") + "\n";
    let _ = emmet_core::fsx::write_atomic(&file(data), text.as_bytes());
}

/// The canonical project root for `path` if it's on the list. Anything else (another folder, a `..` walk out of a
/// listed one, a link to somewhere unlisted) is None.
pub fn find(data: &DataDir, path: &str) -> Option<PathBuf> {
    let want = emmet_core::fsx::canonical(Path::new(path)).ok()?;
    list(data)
        .iter()
        .filter_map(|r| emmet_core::fsx::canonical(Path::new(&r.path)).ok())
        .find(|p| *p == want)
}
