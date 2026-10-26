// SPDX-License-Identifier: AGPL-3.0-or-later
//! bom v0 (plan sections 09 and 12): the parts list by part × color × quantity, from build/placements.json. Rebrickable
//! ids and availability stay null until the parts data pack (3B).

use std::collections::{BTreeMap, HashMap};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::envelope::Result;
use crate::index::Index;
use crate::ldraw::placements::Placement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BomLine {
    /// The LDraw part id.
    pub part: String,
    pub name: String,
    /// The LDraw color code.
    pub color: u32,
    pub color_name: String,
    pub qty: usize,
    /// Rebrickable's part id; null until the parts data pack (3B).
    pub rebrickable_id: Option<String>,
    /// Whether the part exists in that color; null until the parts data pack (3B).
    pub availability: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Totals {
    /// Pieces in all.
    pub pieces: usize,
    /// Lines (part × color).
    pub lines: usize,
    /// Different parts, in any color.
    pub parts: usize,
}

/// `build/bom.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Bom {
    /// Most pieces first, then by part id and color.
    pub lines: Vec<BomLine>,
    pub totals: Totals,
}

pub fn bom(index: &Index, placements: &[Placement]) -> Result<Bom> {
    let colors: HashMap<u32, String> = index.colors()?.into_iter().map(|c| (c.code, c.name)).collect();
    let mut counts: BTreeMap<(&str, u32), usize> = BTreeMap::new();
    for p in placements {
        *counts.entry((p.part.as_str(), p.color)).or_insert(0) += 1;
    }
    let mut names: HashMap<&str, String> = HashMap::new();
    let mut lines = Vec::with_capacity(counts.len());
    for ((part, color), qty) in counts {
        let name = match names.get(part) {
            Some(n) => n.clone(),
            None => {
                let n = index
                    .part(part)?
                    .map_or_else(|| "(not in the library)".to_string(), |p| p.name);
                names.insert(part, n.clone());
                n
            }
        };
        let color_name = colors.get(&color).cloned().unwrap_or_else(|| {
            if color >= 0x0200_0000 {
                format!("#{:06X}", color & 0xFF_FFFF)
            } else {
                format!("Color {color}")
            }
        });
        lines.push(BomLine {
            part: part.to_string(),
            name,
            color,
            color_name,
            qty,
            rebrickable_id: None,
            availability: None,
        });
    }
    lines.sort_by(|a, b| {
        b.qty
            .cmp(&a.qty)
            .then_with(|| a.part.cmp(&b.part))
            .then(a.color.cmp(&b.color))
    });
    let totals = Totals {
        pieces: placements.len(),
        lines: lines.len(),
        parts: names.len(),
    };
    Ok(Bom { lines, totals })
}

/// The BOM as CSV: a header, then one row per line.
pub fn csv(bom: &Bom) -> String {
    let field = |s: &str| {
        if s.contains([',', '"', '\n']) {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    };
    let mut out = String::from("part,name,color,color_name,qty\n");
    for l in &bom.lines {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            field(&l.part),
            field(&l.name),
            l.color,
            field(&l.color_name),
            l.qty
        ));
    }
    out
}
