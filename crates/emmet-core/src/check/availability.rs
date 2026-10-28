// SPDX-License-Identifier: AGPL-3.0-or-later
//! part_unavailable and color_unavailable (plan section 11): a part Rebrickable has no element or set for, or a part in
//! a color it was never made in, from the parts data pack. One finding per part × color, naming every placement.
//! Unknown is never unavailable: an unmapped part or color, or no pack, gives no finding.

use std::collections::{BTreeMap, HashMap};

use super::{Placed, Raw, Rule, dsl_point};
use crate::envelope::Result;
use crate::index::Index;
use crate::parts_data::{PartsData, Status};

/// How many colors a color_unavailable hint offers.
const SUGGESTIONS: usize = 3;

pub(super) fn findings(index: &Index, pack: &PartsData, placed: &[Placed], rules: &[Rule]) -> Result<Vec<Raw>> {
    let mut lines: BTreeMap<(&str, u32), Vec<usize>> = BTreeMap::new();
    for (i, q) in placed.iter().enumerate() {
        // An unknown part is unknown_part's finding.
        if q.info.is_some() {
            lines.entry((q.p.part.as_str(), q.p.color)).or_default().push(i);
        }
    }
    let colors: HashMap<u32, (String, Option<[f64; 3]>)> = index
        .colors()?
        .into_iter()
        .map(|c| (c.code, (c.name, rgb(&c.rgb))))
        .collect();
    let color_name = |code: u32| {
        colors
            .get(&code)
            .map_or_else(|| format!("color {code}"), |c| c.0.clone())
    };
    let mut out = Vec::new();
    for ((part, color), idx) in lines {
        let a = pack.availability(index, part, color)?;
        let rule = match a.status {
            Status::PartUnavailable => Rule::PartUnavailable,
            Status::ColorUnavailable => Rule::ColorUnavailable,
            Status::Available | Status::Unknown => continue,
        };
        if !rules.contains(&rule) {
            continue;
        }
        let q = &placed[idx[0]];
        let name = q.info.as_ref().map_or(part, |i| i.part.name.as_str());
        let them = if idx.len() == 1 {
            "1 part uses".to_string()
        } else {
            format!("{} parts use", idx.len())
        };
        let (message, hint) = if rule == Rule::PartUnavailable {
            let alternates = pack.alternates(index, part)?.unwrap_or_default();
            let hint = if alternates.is_empty() {
                "Use a part that was made; search_parts finds one.".to_string()
            } else {
                format!(
                    "Use a part that was made, such as its alternate {}.",
                    or_list(&alternates)
                )
            };
            (
                format!("{them} {part} ({name}), which Rebrickable has in no set or element: it was never made."),
                hint,
            )
        } else {
            let made = pack.part_colors(index, part)?.unwrap_or_default();
            let mut near: Vec<(f64, u32)> = made
                .iter()
                .filter_map(|&c| Some((distance(colors.get(&color)?.1?, colors.get(&c)?.1?), c)))
                .collect();
            near.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            let names: Vec<String> = near.iter().take(SUGGESTIONS).map(|(_, c)| color_name(*c)).collect();
            let hint = if names.is_empty() {
                format!("Use a color {part} was made in; list_colors with part {part} lists them.")
            } else {
                format!(
                    "Use a color {part} was made in, such as {} (closest first); list_colors with part {part} lists \
                     all {}.",
                    or_list(&names),
                    made.len()
                )
            };
            (
                format!(
                    "{them} {part} ({name}) in {}, a color it was never made in.",
                    color_name(color)
                ),
                hint,
            )
        };
        out.push(Raw::new(rule, idx, dsl_point(q.m.w_axis.truncate()), message, hint));
    }
    Ok(out)
}

/// "a", "a or b", "a, b, or c".
fn or_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [a] => a.clone(),
        [a, b] => format!("{a} or {b}"),
        [rest @ .., last] => format!("{}, or {last}", rest.join(", ")),
    }
}

/// `#RRGGBB` → RGB in 0–255.
fn rgb(hex: &str) -> Option<[f64; 3]> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let c = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok().map(f64::from);
    Some([c(0)?, c(2)?, c(4)?])
}

/// A perceptual-ish distance between two colors: RGB weighted by how much each channel shows ("redmean").
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let r = (a[0] + b[0]) / 2.0;
    let (dr, dg, db) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    ((2.0 + r / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - r) / 256.0) * db * db).sqrt()
}
