// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet mcp`: the stdio MCP server (plan section 09).

use std::path::{Path, PathBuf};

use emmet_core::data::DataDir;
use emmet_core::envelope::{Error, Result};

pub fn serve(_data: DataDir, _project: Option<PathBuf>, _session: Option<String>) -> Result<()> {
    Err(Error::internal("emmet mcp isn't built yet"))
}

pub fn write_schemas(_dir: &Path) -> Result<usize> {
    Ok(0)
}
