// SPDX-License-Identifier: AGPL-3.0-or-later
//! Part helpers (plan section 08): `Brick(2, 4)`, `Plate(1, 6)`, `Tile(2, 2)`, and `Slope(45, 2, 1)` resolve to the
//! library's plain part of that size, or to nothing with the sizes the library does have.

use super::{Index, db};
use crate::envelope::Result;

/// What a helper resolved to.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    pub id: Option<String>,
    /// When `id` is None: the nearest sizes of that kind, as "Brick 1 x 4 (3010)".
    pub near: Vec<String>,
}

impl Index {
    /// The part for a helper: `kind` is brick, plate, tile, or slope; `dims` are (w, l, bricks tall) for the first
    /// three, with w ≤ l, and (depth down the slope, width, angle) for a slope. The plain part wins ("Brick 2 x 4"
    /// over "Brick 2 x 4 with Holes"), then a tile "with Groove", then the shortest name; aliases, moved, obsolete,
    /// printed, and sticker parts never count.
    pub fn resolve_helper(&self, kind: &str, dims: &[u32]) -> Result<Resolved> {
        let (category, title) = match kind {
            "brick" => ("Brick", "Brick"),
            "plate" => ("Plate", "Plate"),
            "tile" => ("Tile", "Tile"),
            "slope" => ("Slope", "Slope Brick"),
            _ => return Ok(Resolved { id: None, near: vec![] }),
        };
        let [a, b, c] = match dims {
            [a, b, c] => [*a, *b, *c],
            [a, b] => [*a, *b, 1],
            _ => return Ok(Resolved { id: None, near: vec![] }),
        };
        let wanted = if kind == "slope" {
            format!("{title} {c} {a} x {b}")
        } else if kind == "brick" && c != 1 {
            format!("{title} {a} x {b} x {c}")
        } else {
            format!("{title} {a} x {b}")
        };
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, size_x, size_y, size_z FROM parts
                 WHERE category = ? AND alias_of IS NULL AND flags NOT LIKE '%alias%' AND flags NOT LIKE '%obsolete%'
                   AND flags NOT LIKE '%printed%' AND flags NOT LIKE '%sticker%' AND size_x IS NOT NULL",
            )
            .map_err(db)?;
        let rows: Vec<(String, String, f64, f64, f64)> = stmt
            .query_map([category], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .map_err(db)?
            .collect::<std::result::Result<_, _>>()
            .map_err(db)?;
        let height = |z: f64| if kind == "brick" { (z / 3.0).round() as u32 } else { 1 };
        let fits = |x: f64, y: f64, z: f64| {
            let (x, y) = (x.round() as u32, y.round() as u32);
            let size = if kind == "slope" {
                (x, y) == (a, b)
            } else {
                (x.min(y), x.max(y)) == (a, b)
            };
            size && (kind != "brick" || height(z) == c)
        };
        let plain = |name: &str| name.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut best: Option<(u8, usize, String)> = None;
        for (id, name, x, y, z) in &rows {
            let name = plain(name);
            if !fits(*x, *y, *z) || !name.starts_with(&wanted) {
                continue;
            }
            let rest = &name[wanted.len()..];
            // "Brick 1 x 2 x 5" isn't a Brick 1 x 2: the name must end or go on with words, not another size.
            if rest.starts_with(" x") || rest.starts_with(|ch: char| ch.is_ascii_digit()) {
                continue;
            }
            let rank = if rest.is_empty() {
                0
            } else if rest == " with Groove" {
                1
            } else {
                2
            };
            let key = (rank, name.len(), id.clone());
            if best.as_ref().is_none_or(|b| key < *b) {
                best = Some(key);
            }
        }
        if let Some((_, _, id)) = best {
            return Ok(Resolved {
                id: Some(id),
                near: vec![],
            });
        }
        // The nearest plain sizes of the same kind, for the error.
        let mut near: Vec<(u32, String)> = rows
            .iter()
            .filter(|(_, name, ..)| {
                let name = plain(name);
                name.starts_with(title)
                    && name[title.len()..]
                        .trim_start()
                        .chars()
                        .next()
                        .is_some_and(|ch| ch.is_ascii_digit())
            })
            .filter_map(|(id, name, x, y, z)| {
                let (x, y) = (x.round() as u32, y.round() as u32);
                let (w, l) = if kind == "slope" { (x, y) } else { (x.min(y), x.max(y)) };
                let off = w.abs_diff(a) * 4 + l.abs_diff(b) + height(*z).abs_diff(c) * 2;
                let name = plain(name);
                let short = name.split(" with ").next().unwrap_or(&name).to_string();
                Some((off, format!("{short} ({id})")))
            })
            .collect();
        near.sort();
        near.dedup_by(|x, y| x.1.split(" (").next() == y.1.split(" (").next());
        Ok(Resolved {
            id: None,
            near: near.into_iter().take(4).map(|(_, n)| n).collect(),
        })
    }
}
