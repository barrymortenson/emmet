// SPDX-License-Identifier: AGPL-3.0-or-later
//! Parts search (plan section 12): dimensions first, then FTS5 with BM25 over the words left, with synonyms, then
//! boosts for plain, current parts. Budget: 20 ms a query.

use rusqlite::params_from_iter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Index, Size, db, dims, split_list};
use crate::envelope::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SearchHit {
    pub id: String,
    pub name: String,
    pub category: String,
    pub size: Option<Size>,
    /// Colors the part was produced in, or None until the parts data pack (3B) is installed.
    pub colors_count: Option<u32>,
    pub flags: Vec<String>,
}

const STOPWORDS: &[&str] = &["a", "an", "and", "the", "with", "of", "for", "x", "by", "in", "on"];

/// A word as FTS5 sees it, plurals folded: "bricks" → "brick", "glass" stays.
fn stem(w: &str) -> String {
    if w.len() > 3 && w.ends_with('s') && !w.ends_with("ss") {
        w[..w.len() - 1].to_string()
    } else {
        w.to_string()
    }
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .filter(|w| !STOPWORDS.contains(&w.as_str()))
        .map(|w| stem(&w))
        .collect()
}

fn term(w: &str) -> String {
    format!("\"{}\"*", w.replace('"', ""))
}

impl Index {
    pub fn search(&self, query: &str, category: Option<&str>, limit: usize) -> Result<Vec<SearchHit>> {
        let parsed = dims::parse(query);
        let tokens = words(&parsed.rest);
        if tokens.is_empty() && parsed.dims.is_empty() {
            return Ok(Vec::new());
        }
        let synonyms = self.part_synonyms()?;
        let group = |t: &String| {
            let mut alts = vec![term(t)];
            for (_, to) in synonyms.iter().filter(|(from, _)| from == t) {
                let ws = words(to);
                if !ws.is_empty() {
                    alts.push(format!(
                        "({})",
                        ws.iter().map(|w| term(w)).collect::<Vec<_>>().join(" AND ")
                    ));
                }
            }
            format!("({})", alts.join(" OR "))
        };
        let groups: Vec<String> = tokens.iter().map(group).collect();
        let mut hits = self.run(&groups.join(" AND "), &tokens, &parsed.dims, category)?;
        if hits.is_empty() && groups.len() > 1 {
            hits = self.run(&groups.join(" OR "), &tokens, &parsed.dims, category)?;
        }
        hits.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.id.cmp(&b.1.id)));
        let known_colors = self
            .conn()
            .query_row("SELECT count(*) > 0 FROM elements", [], |r| r.get::<_, bool>(0));
        let known_colors = known_colors.map_err(db)?;
        let mut out: Vec<SearchHit> = hits.into_iter().take(limit).map(|(_, h)| h).collect();
        if known_colors {
            for h in &mut out {
                let n = self.conn().query_row(
                    "SELECT count(DISTINCT color_code) FROM elements WHERE part_id = ?",
                    [&h.id],
                    |r| r.get::<_, u32>(0),
                );
                h.colors_count = n.ok();
            }
        }
        Ok(out)
    }

    fn part_synonyms(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT term, expands_to FROM synonyms")
            .map_err(db)?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(db)?;
        rows.collect::<std::result::Result<_, _>>().map_err(db)
    }

    /// Candidates with their score (lower is better).
    fn run(&self, fts: &str, tokens: &[String], dims: &[f64], category: Option<&str>) -> Result<Vec<(f64, SearchHit)>> {
        let mut sql = String::new();
        let mut args: Vec<rusqlite::types::Value> = Vec::new();
        if tokens.is_empty() {
            sql.push_str(
                "SELECT p.id, p.name, p.category, p.size_x, p.size_y, p.size_z, p.flags, 0.0 FROM parts p WHERE 1",
            );
        } else {
            sql.push_str(
                "SELECT p.id, p.name, p.category, p.size_x, p.size_y, p.size_z, p.flags, bm25(parts_fts, 10.0, 4.0, 1.0)
                 FROM parts_fts JOIN parts p ON p.rowid = parts_fts.rowid WHERE parts_fts MATCH ?",
            );
            args.push(fts.to_string().into());
        }
        if dims.len() >= 2 {
            let (a, b) = (dims[0], dims[1]);
            sql.push_str(
                " AND ((abs(p.size_x - ?) < 0.3 AND abs(p.size_y - ?) < 0.3) OR (abs(p.size_x - ?) < 0.3 AND abs(p.size_y - ?) < 0.3))",
            );
            args.extend([a, b, b, a].map(Into::into));
            if let Some(&h) = dims.get(2) {
                // The third number is a height in bricks (3 plates), as LDraw names write it.
                sql.push_str(" AND abs(p.size_z - ?) < 1.0");
                args.push((3.0 * h).into());
            }
        }
        if let Some(c) = category {
            sql.push_str(" AND lower(p.category) = lower(?)");
            args.push(c.to_string().into());
        }
        let mut stmt = self.conn().prepare_cached(&sql).map_err(db)?;
        let rows = stmt
            .query_map(params_from_iter(args), |r| {
                let size = match (
                    r.get::<_, Option<f64>>(3)?,
                    r.get::<_, Option<f64>>(4)?,
                    r.get::<_, Option<f64>>(5)?,
                ) {
                    (Some(x), Some(y), Some(z)) => Some(Size { x, y, z }),
                    _ => None,
                };
                let flags: String = r.get(6)?;
                let hit = SearchHit {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    category: r.get(2)?,
                    size,
                    colors_count: None,
                    flags: split_list(&flags),
                };
                Ok((r.get::<_, f64>(7)?, hit))
            })
            .map_err(db)?;
        let mut out = Vec::new();
        for row in rows {
            let (bm25, hit) = row.map_err(db)?;
            out.push((bm25 + penalty(&hit, tokens), hit));
        }
        Ok(out)
    }
}

/// Boosts plain, current parts: printed, obsolete, moved, hidden, and sticker parts sink unless the query asks for
/// them, a category named in the query rises, and shorter names win ties.
fn penalty(hit: &SearchHit, tokens: &[String]) -> f64 {
    let asked = |w: &str| tokens.iter().any(|t| t.starts_with(w));
    let mut p = 0.0;
    for f in &hit.flags {
        p += match f.as_str() {
            "alias" => 6.0,
            "obsolete" if !asked("obsolete") => 6.0,
            "hidden" => 4.0,
            "printed" if !asked("print") && !asked("pattern") => 5.0,
            "sticker" if !asked("sticker") => 8.0,
            "shortcut" => 1.0,
            _ => 0.0,
        };
    }
    let plain = hit
        .id
        .trim_end_matches(|c: char| c.is_ascii_lowercase())
        .chars()
        .all(|c| c.is_ascii_digit());
    if !plain {
        p += 0.5;
    }
    if tokens.iter().any(|t| t == &hit.category.to_lowercase()) {
        p -= 2.0;
    }
    p + 0.15 * hit.name.split_whitespace().count() as f64
}
