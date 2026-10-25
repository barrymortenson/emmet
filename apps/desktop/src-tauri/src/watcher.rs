// SPDX-License-Identifier: AGPL-3.0-or-later
//! The project watcher (plan section 07): sources only (model.py, models/**/*.py, params.json, emmet.toml), never
//! build/, debounced 150 ms, ignoring editor temp files.

use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher as _};

pub const DEBOUNCE: Duration = Duration::from_millis(150);

/// Watches a project until dropped.
pub struct Watcher {
    _inner: notify::RecommendedWatcher,
}

impl Watcher {
    /// Calls `on_change` once per burst of source changes, 150 ms after the last one.
    pub fn start(root: &Path, on_change: impl Fn() + Send + 'static) -> notify::Result<Self> {
        let root: PathBuf = emmet_core::fsx::canonical(root).unwrap_or_else(|_| root.to_path_buf());
        let (tx, rx) = mpsc::channel::<()>();
        let base = root.clone();
        let mut inner = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            // An error (Windows reports a full change buffer this way) or a rescan means changes may have been
            // missed: an extra run is better than a stale model shown as built.
            let Ok(event) = event else {
                let _ = tx.send(());
                return;
            };
            if event.need_rescan() || (!event.kind.is_access() && event.paths.iter().any(|p| is_source(&base, p))) {
                let _ = tx.send(());
            }
        })?;
        inner.watch(&root, RecursiveMode::Recursive)?;
        // The thread ends when the watcher (and with it the sender) is dropped.
        std::thread::spawn(move || {
            while rx.recv().is_ok() {
                loop {
                    match rx.recv_timeout(DEBOUNCE) {
                        Ok(()) => continue,
                        Err(mpsc::RecvTimeoutError::Timeout) => break,
                        Err(mpsc::RecvTimeoutError::Disconnected) => return,
                    }
                }
                on_change();
            }
        });
        Ok(Self { _inner: inner })
    }
}

/// Whether a changed path is one of the project's sources. Paths come from the OS canonical, like `root`.
pub fn is_source(root: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else { return false };
    let parts: Vec<String> = rel
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let Some(name) = parts.last() else { return false };
    if parts.iter().any(|p| p.starts_with('.') || p == "__pycache__") || is_temp(name) {
        return false;
    }
    match parts.as_slice() {
        [one] => matches!(one.as_str(), "model.py" | "params.json" | "emmet.toml"),
        [first, .., last] => first == "models" && last.ends_with(".py"),
        [] => false,
    }
}

/// Editor and tool temp files: vim's swap files and 4913 probe, Emacs's #name# and .#name, backups ending in ~,
/// and *.tmp (emmet's own atomic writes are hidden .tmp files).
fn is_temp(name: &str) -> bool {
    name == "4913"
        || name.ends_with('~')
        || name.starts_with('#')
        || [".swp", ".swx", ".swo", ".tmp", ".bak", ".orig"]
            .iter()
            .any(|x| name.ends_with(x))
}
