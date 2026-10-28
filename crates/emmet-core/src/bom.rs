// SPDX-License-Identifier: AGPL-3.0-or-later
//! bom v1 (plan sections 09 and 12): the parts list by part × color × quantity, from build/placements.json, with
//! Rebrickable's ids and names and each line's availability from the parts data pack, color names in the user's
//! preferred naming, and the exports: CSV, Rebrickable's parts-list import, and a BrickLink wanted list.

use std::collections::{BTreeMap, HashMap};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::DataDir;
use crate::envelope::{Error, ErrorCode, Result, Warning};
use crate::index::Index;
use crate::ldraw::placements::Placement;
use crate::parts_data::{self, Availability, PartsData, Status};

/// Whose color names the BOM uses (Settings → Data, `emmet bom --color-names`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ColorNames {
    #[default]
    Ldraw,
    Rebrickable,
    Bricklink,
}

impl ColorNames {
    /// The user's setting (`color_names` in the data dir's settings.json, which the app writes), else LDraw's.
    pub fn preferred(data: &DataDir) -> Self {
        std::fs::read_to_string(data.settings())
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .and_then(|v| serde_json::from_value(v["color_names"].clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BomLine {
    /// The LDraw part id.
    pub part: String,
    /// The LDraw part's name.
    pub name: String,
    /// The LDraw color code.
    pub color: u32,
    /// The color's name in the BOM's naming (LDraw's when the other has no name for it).
    pub color_name: String,
    pub qty: usize,
    /// Rebrickable's part id; null without the parts data pack or a mapping.
    pub rebrickable_id: Option<String>,
    /// Rebrickable's name for the part.
    pub rebrickable_name: Option<String>,
    /// Rebrickable's color id.
    pub rebrickable_color: Option<i64>,
    /// BrickLink's part id, for the wanted list.
    pub bricklink_id: Option<String>,
    /// BrickLink's color id.
    pub bricklink_color: Option<u32>,
    /// Whether the part was ever made in that color; `unknown` (never unavailable) without the pack or a mapping.
    pub availability: Availability,
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
    pub color_names: ColorNames,
    /// The parts data pack's version, or null when none is installed.
    pub parts_data: Option<String>,
}

pub fn bom(index: &Index, pack: Option<&PartsData>, placements: &[Placement], names: ColorNames) -> Result<Bom> {
    let colors: HashMap<u32, String> = index.colors()?.into_iter().map(|c| (c.code, c.name)).collect();
    let mut counts: BTreeMap<(&str, u32), usize> = BTreeMap::new();
    for p in placements {
        *counts.entry((p.part.as_str(), p.color)).or_insert(0) += 1;
    }
    let mut part_names: HashMap<&str, String> = HashMap::new();
    let mut lines = Vec::with_capacity(counts.len());
    for ((part, color), qty) in counts {
        let name = match part_names.get(part) {
            Some(n) => n.clone(),
            None => {
                let n = index
                    .part(part)?
                    .map_or_else(|| "(not in the library)".to_string(), |p| p.name);
                part_names.insert(part, n.clone());
                n
            }
        };
        let ldraw_name = colors.get(&color).cloned().unwrap_or_else(|| {
            if color >= 0x0200_0000 {
                format!("#{:06X}", color & 0xFF_FFFF)
            } else {
                format!("Color {color}")
            }
        });
        let mut line = BomLine {
            part: part.to_string(),
            name,
            color,
            color_name: ldraw_name,
            qty,
            rebrickable_id: None,
            rebrickable_name: None,
            rebrickable_color: None,
            bricklink_id: None,
            bricklink_color: None,
            availability: Availability::unknown(),
        };
        if let Some(pack) = pack {
            line.rebrickable_id = pack.rebrickable_part(index, part)?;
            if let Some(rb) = &line.rebrickable_id {
                line.rebrickable_name = pack.part_name(rb)?;
                line.bricklink_id = pack.bricklink_part(index, part)?;
            }
            if let Some(c) = pack.color(color)? {
                line.rebrickable_color = Some(c.rebrickable_id);
                line.bricklink_color = c.bricklink_id;
                match names {
                    ColorNames::Ldraw => {}
                    ColorNames::Rebrickable => line.color_name = c.rebrickable_name,
                    ColorNames::Bricklink => {
                        if let Some(n) = c.bricklink_name {
                            line.color_name = n;
                        }
                    }
                }
            }
            line.availability = pack.availability(index, part, color)?;
        }
        lines.push(line);
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
        parts: part_names.len(),
    };
    Ok(Bom {
        lines,
        totals,
        color_names: names,
        parts_data: pack.map(|p| p.version().to_string()),
    })
}

fn field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// The plain-words availability the CSV and the Parts list show.
pub fn availability_words(a: &Availability) -> &'static str {
    match a.status {
        Status::Available => "available",
        Status::ColorUnavailable => "not made in this color",
        Status::PartUnavailable => "not made",
        Status::Unknown => "unknown",
    }
}

/// The BOM as CSV: the Rebrickable credit as the first line when its data is in it, a header, then one row per line.
pub fn csv(bom: &Bom) -> String {
    let mut out = String::new();
    if bom.parts_data.is_some() {
        out.push_str(&field(parts_data::CREDIT));
        out.push('\n');
    }
    out.push_str("part,name,color,color_name,qty,rebrickable_id,rebrickable_name,availability,sets,years\n");
    for l in &bom.lines {
        let years = l.availability.years.map_or(String::new(), |[a, b]| format!("{a}-{b}"));
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            field(&l.part),
            field(&l.name),
            l.color,
            field(&l.color_name),
            l.qty,
            field(l.rebrickable_id.as_deref().unwrap_or("")),
            field(l.rebrickable_name.as_deref().unwrap_or("")),
            availability_words(&l.availability),
            l.availability.sets.map_or(String::new(), |n| n.to_string()),
            years,
        ));
    }
    out
}

fn no_pack() -> Error {
    Error::new(
        ErrorCode::LibraryMissing,
        "The parts data pack isn't installed, so there are no Rebrickable or BrickLink ids to export.",
        "Use --format csv or json, or install the parts data pack (the app installs the one it ships with).",
    )
}

/// Lines left out of an export, as a warning naming them.
fn left_out(code: &str, to: &str, missing: &[String]) -> Vec<Warning> {
    if missing.is_empty() {
        return vec![];
    }
    vec![Warning {
        code: code.into(),
        message: format!(
            "{} line{} with no {to} id left out: {}.",
            missing.len(),
            if missing.len() == 1 { "" } else { "s" },
            missing.join(", ")
        ),
    }]
}

/// Rebrickable's parts-list import CSV, exactly their format: `Part,Color,Quantity` with Rebrickable part and color ids.
/// It carries no credit line, because any other line makes the import fail; lines without Rebrickable ids are left
/// out and named in the warnings.
pub fn rebrickable_csv(bom: &Bom) -> Result<(String, Vec<Warning>)> {
    if bom.parts_data.is_none() {
        return Err(no_pack());
    }
    let mut out = String::from("Part,Color,Quantity\n");
    let mut missing = Vec::new();
    for l in &bom.lines {
        match (&l.rebrickable_id, l.rebrickable_color) {
            (Some(part), Some(color)) => out.push_str(&format!("{},{color},{}\n", field(part), l.qty)),
            _ => missing.push(format!("{} in {}", l.part, l.color_name)),
        }
    }
    Ok((out, left_out("REBRICKABLE_UNMAPPED", "Rebrickable", &missing)))
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A BrickLink wanted list (XML) with BrickLink part and color ids and the credit as a comment. Lines without
/// BrickLink ids are left out, counted in a comment and named in the warnings.
pub fn bricklink_xml(bom: &Bom) -> Result<(String, Vec<Warning>)> {
    if bom.parts_data.is_none() {
        return Err(no_pack());
    }
    let mut items = String::new();
    let mut missing = Vec::new();
    for l in &bom.lines {
        match (&l.bricklink_id, l.bricklink_color) {
            (Some(part), Some(color)) => items.push_str(&format!(
                "<ITEM><ITEMTYPE>P</ITEMTYPE><ITEMID>{}</ITEMID><COLOR>{color}</COLOR><MINQTY>{}</MINQTY></ITEM>\n",
                xml(part),
                l.qty
            )),
            _ => missing.push(format!("{} in {}", l.part, l.color_name)),
        }
    }
    // Only Emmet's own text goes in comments: a part id is the model's, and "--" would end a comment early. The
    // lines left out are named in the warnings.
    let mut out = format!("<!-- {} -->\n", parts_data::CREDIT);
    if !missing.is_empty() {
        out.push_str(&format!(
            "<!-- {} left out, with no BrickLink id; Emmet's warnings name {}. -->\n",
            if missing.len() == 1 {
                "1 line".to_string()
            } else {
                format!("{} lines", missing.len())
            },
            if missing.len() == 1 { "it" } else { "them" }
        ));
    }
    out.push_str("<INVENTORY>\n");
    out.push_str(&items);
    out.push_str("</INVENTORY>\n");
    Ok((out, left_out("BRICKLINK_UNMAPPED", "BrickLink", &missing)))
}
