// SPDX-License-Identifier: AGPL-3.0-or-later
//! The color table from the library's `LDConfig.ldr`, and name lookup that takes LDraw, Rebrickable, or BrickLink
//! spellings ("Light Bluish Gray", "Light_Bluish_Grey", "LBG") or an LDraw code (plan section 08).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Color {
    /// The LDraw color code.
    pub code: u32,
    /// The display name, in US spelling ("Light Bluish Gray").
    pub name: String,
    /// `#RRGGBB`
    pub rgb: String,
    pub edge_rgb: String,
    /// solid, transparent, chrome, pearlescent, metal, matte_metallic, rubber, glitter, speckle, or glow.
    pub finish: String,
    /// Other names that resolve to this color: LDraw's own name, plus abbreviations from `data/synonyms.toml`.
    pub aliases: Vec<String>,
}

/// The key a color name is looked up by: lowercase letters and digits only, with "grey" spelled "gray", so
/// "Light_Bluish_Grey", "light bluish gray", and "LightBluishGray" are one key.
pub fn key(name: &str) -> String {
    let k: String = name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    k.replace("grey", "gray")
}

/// Parses the `0 !COLOUR` lines of an `LDConfig.ldr`.
pub fn parse_ldconfig(text: &str) -> Vec<Color> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut w = line.split_whitespace();
        if w.next() != Some("0") || w.next() != Some("!COLOUR") {
            continue;
        }
        let Some(raw_name) = w.next() else { continue };
        let words: Vec<&str> = w.collect();
        let field = |k: &str| {
            words
                .iter()
                .position(|x| *x == k)
                .and_then(|i| words.get(i + 1))
                .copied()
        };
        let Some(code) = field("CODE").and_then(|c| c.parse().ok()) else {
            continue;
        };
        let alpha: u32 = field("ALPHA").and_then(|a| a.parse().ok()).unwrap_or(255);
        let finish = ["CHROME", "PEARLESCENT", "METAL", "MATTE_METALLIC", "RUBBER"]
            .iter()
            .find(|f| words.contains(f))
            .map(|f| f.to_lowercase())
            .or_else(|| {
                let material = field("MATERIAL").map(str::to_lowercase);
                material.filter(|m| m == "glitter" || m == "speckle")
            })
            .unwrap_or_else(|| {
                if words.contains(&"LUMINANCE") {
                    "glow".into()
                } else {
                    "solid".into()
                }
            });
        let finish = if finish == "solid" && alpha < 255 {
            "transparent".to_string()
        } else {
            finish
        };
        out.push(Color {
            code,
            name: display_name(raw_name),
            rgb: field("VALUE").unwrap_or("#000000").to_uppercase(),
            edge_rgb: field("EDGE").unwrap_or("#000000").to_uppercase(),
            finish,
            aliases: vec![raw_name.to_string()],
        });
    }
    out
}

/// `Light_Bluish_Grey` → `Light Bluish Gray`.
fn display_name(raw: &str) -> String {
    raw.split('_')
        .map(|w| if w == "Grey" { "Gray" } else { w })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Finds a color by code or by any spelling; `abbreviations` maps keys like "lbg" to a full name.
pub fn find<'a>(colors: &'a [Color], query: &str, abbreviations: &[(String, String)]) -> Option<&'a Color> {
    let q = query.trim();
    if let Ok(code) = q.parse::<u32>() {
        return colors.iter().find(|c| c.code == code);
    }
    let mut k = key(q);
    if let Some((_, full)) = abbreviations.iter().find(|(abbr, _)| key(abbr) == k) {
        k = key(full);
    }
    colors
        .iter()
        .find(|c| key(&c.name) == k || c.aliases.iter().any(|a| key(a) == k))
}

/// The closest color names to `query`, best first.
pub fn nearest(colors: &[Color], query: &str, n: usize) -> Vec<String> {
    let k = key(query);
    let mut scored: Vec<(usize, &str)> = colors
        .iter()
        .map(|c| (crate::text::edit_distance(&k, &key(&c.name)), c.name.as_str()))
        .collect();
    scored.sort();
    scored.into_iter().take(n).map(|(_, name)| name.to_string()).collect()
}
