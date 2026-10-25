// SPDX-License-Identifier: AGPL-3.0-or-later
//! The app data directory: the LDraw library, the parts index, and the Python runtime (plan sections 04 and 12).

use std::path::{Path, PathBuf};

use crate::envelope::{Error, ErrorCode, Result};

/// The bundle id the app data folder is named after.
pub const BUNDLE_ID: &str = "com.emmetbuilder.emmet";

#[derive(Debug, Clone)]
pub struct DataDir {
    root: PathBuf,
}

impl DataDir {
    /// `--data-dir`, else `EMMET_DATA_DIR`, else `.dev-data/` in the repo for a debug build, else the platform's app
    /// data folder (`~/Library/Application Support/<bundle id>`, `%APPDATA%\<bundle id>`).
    pub fn resolve(flag: Option<&Path>) -> Result<Self> {
        let root = match flag
            .map(Path::to_path_buf)
            .or_else(|| std::env::var_os("EMMET_DATA_DIR").map(PathBuf::from))
        {
            Some(dir) => dir,
            // crates/emmet-core → the repo root.
            None if cfg!(debug_assertions) => Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .expect("the crate sits two folders below the repo root")
                .join(".dev-data"),
            None => platform_dir().ok_or_else(|| {
                Error::new(
                    ErrorCode::IoError,
                    "Can't find the app data folder: HOME (APPDATA on Windows) isn't set.",
                    "Set HOME, or pass --data-dir <dir>.",
                )
            })?,
        };
        Ok(Self::at(root))
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let root = std::path::absolute(&root).unwrap_or(root);
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The installed library (what `complete.zip` holds under `ldraw/`).
    pub fn ldraw(&self) -> PathBuf {
        self.root.join("ldraw")
    }

    /// What `emmet library install` recorded: release, zip checksum, file count.
    pub fn library_json(&self) -> PathBuf {
        self.root.join("library.json")
    }

    /// The sha256 of every installed library file, for `emmet library verify`.
    pub fn library_manifest(&self) -> PathBuf {
        self.root.join("ldraw.sha256")
    }

    pub fn index(&self) -> PathBuf {
        self.root.join("parts.sqlite")
    }

    pub fn downloads(&self) -> PathBuf {
        self.root.join("downloads")
    }

    /// uv's managed Pythons, uv's cache, and the venv models run in.
    pub fn python(&self) -> PathBuf {
        self.root.join("python")
    }

    /// The random secret this install hashes into trust markers and `created_by` (D-27).
    pub fn machine_id(&self) -> PathBuf {
        self.root.join("machine-id")
    }
}

fn platform_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join(BUNDLE_ID))
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support").join(BUNDLE_ID))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .map(|d| d.join(BUNDLE_ID))
    }
}
