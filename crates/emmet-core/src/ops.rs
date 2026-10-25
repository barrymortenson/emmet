// SPDX-License-Identifier: AGPL-3.0-or-later
//! The operations the CLI and the MCP server both expose (plan section 09). Each returns the envelope's `data`; the
//! front ends only shape it.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::colors::Color;
use crate::data::DataDir;
use crate::envelope::{Details, Error, ErrorCode, Result};
use crate::index::{Index, Part, SearchHit};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    pub query: String,
    pub parts: Vec<SearchHit>,
}

pub fn search_parts(data: &DataDir, query: &str, category: Option<&str>, limit: usize) -> Result<SearchResult> {
    let index = Index::ensure(data)?;
    let parts = index.search(query, category, limit.clamp(1, 100))?;
    Ok(SearchResult {
        query: query.into(),
        parts,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartInfo {
    #[serde(flatten)]
    pub part: Part,
    /// Connector summary, filled from 3A.
    pub connectors: Vec<String>,
    /// Colors the part was produced in, filled once the parts data pack is installed (3B).
    pub produced_colors: Vec<u32>,
    /// Alternate and mold-variant ids, filled from 3B.
    pub alternates: Vec<String>,
}

pub fn part_info(data: &DataDir, id: &str) -> Result<PartInfo> {
    let index = Index::ensure(data)?;
    match index.part(id)? {
        Some(part) => Ok(PartInfo {
            part,
            connectors: vec![],
            produced_colors: vec![],
            alternates: vec![],
        }),
        None => Err(unknown_part(&index, id)?),
    }
}

/// `UNKNOWN_PART` with the closest ids and the best search matches for `id`.
pub fn unknown_part(index: &Index, id: &str) -> Result<Error> {
    let mut suggestions: Vec<(String, String)> = index.similar_ids(id, 5)?;
    for hit in index.search(id, None, 5)? {
        if !suggestions.iter().any(|(s, _)| *s == hit.id) {
            suggestions.push((hit.id, hit.name));
        }
    }
    suggestions.truncate(5);
    let hint = if suggestions.is_empty() {
        "Search for the part with emmet search <words>, then use its id.".to_string()
    } else {
        let list = suggestions
            .iter()
            .map(|(id, name)| format!("{id} ({name})"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("Did you mean {list}? Or search with emmet search <words>.")
    };
    Ok(Error::new(
        ErrorCode::UnknownPart,
        format!("There's no part '{id}' in the LDraw library."),
        hint,
    )
    .with_details(Details {
        suggestions: suggestions.into_iter().map(|(id, _)| id).collect(),
        ..Default::default()
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ColorList {
    pub colors: Vec<Color>,
}

/// Every color, or with `part`, the colors that part was produced in (all colors until the parts data pack, 3B).
pub fn list_colors(data: &DataDir, part: Option<&str>) -> Result<ColorList> {
    let index = Index::ensure(data)?;
    if let Some(id) = part
        && index.part(id)?.is_none()
    {
        return Err(unknown_part(&index, id)?);
    }
    Ok(ColorList {
        colors: index.colors()?,
    })
}
