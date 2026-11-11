// SPDX-License-Identifier: AGPL-3.0-or-later
//! What rungs 2 and 3 are scored on beyond rung 1's checks (plan section 16.1, docs/spec/ladder.md, session 5A):
//! rung 2's track loop, closed through its connectors; rung 3's parts list against the set's Rebrickable inventory
//! (F1 over part × color) and its placements against the OMR model (the share of the set's parts that have a part of
//! the same id within a stud, once the two models are lined up). Each is a plain function of two lists, so the eval,
//! a dry run, and a hand session's score all read the same numbers.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use emmet_core::data::DataDir;
use emmet_core::envelope::{Error, Result};
use emmet_core::index::Index;
use emmet_core::ldraw::geometry::LibFiles;
use emmet_core::ldraw::placements::Placement;

/// A stud, in LDU: how far a part may sit from its place in the set and still match.
const STUD: f64 = 20.0;

/// Plastic train track and the 9V switch points rung 2 builds its loop from (docs/spec/ladder.md).
fn is_track(part: &str) -> bool {
    part == "53401" || part == "53400" || part.starts_with("75541") || part.starts_with("75542")
}

/// Rung 2's track.
#[derive(Debug, PartialEq)]
pub struct Track {
    pub pieces: usize,
    pub switches: usize,
    /// Some run of track comes back to where it started, joined connector to connector all the way round.
    pub closes: bool,
}

/// The model's track, and whether it closes: a ring in the graph of track pieces joined by their connectors. A piece
/// laid a stud short of the next joins nothing, so a loop with a gap doesn't close.
pub fn track(data: &DataDir, placements: &[Placement]) -> Result<Track> {
    let pieces: Vec<Placement> = placements.iter().filter(|p| is_track(&p.part)).cloned().collect();
    let switches = pieces.iter().filter(|p| p.part.starts_with("7554")).count();
    if pieces.len() < 3 {
        return Ok(Track {
            pieces: pieces.len(),
            switches,
            closes: false,
        });
    }
    let index = Index::open(data)?;
    let lib = LibFiles::probe(&data.ldraw());
    let at: HashMap<&str, usize> = pieces.iter().enumerate().map(|(i, p)| (p.id.as_str(), i)).collect();
    // One edge per pair of pieces, however many connectors join them (a rail joint is two).
    let mut edges: Vec<(usize, usize)> = emmet_core::check::links(&index, &lib, &pieces)?
        .iter()
        .filter(|(_, _, kind)| kind != "held")
        .filter_map(|(a, b, _)| Some((*at.get(a.as_str())?, *at.get(b.as_str())?)))
        .filter(|(a, b)| a != b)
        .map(|(a, b)| (a.min(b), a.max(b)))
        .collect();
    edges.sort_unstable();
    edges.dedup();
    // An edge between two pieces already joined some other way closes a ring.
    let mut group: Vec<usize> = (0..pieces.len()).collect();
    fn root(group: &mut [usize], mut i: usize) -> usize {
        while group[i] != i {
            group[i] = group[group[i]];
            i = group[i];
        }
        i
    }
    let mut closes = false;
    for (a, b) in edges {
        let (ra, rb) = (root(&mut group, a), root(&mut group, b));
        if ra == rb {
            closes = true;
        } else {
            group[ra] = rb;
        }
    }
    Ok(Track {
        pieces: pieces.len(),
        switches,
        closes,
    })
}

/// Whether the model names a `feature` ("platform", "canopy"): in a part's id, a sub-assembly's name, or a step's
/// title. The agent names its own groups and steps, and a station whose builder never says "canopy" has none a
/// person reading its steps would find.
pub fn names(placements: &[Placement], step_titles: &[String], feature: &str) -> bool {
    let has = |s: &str| s.to_lowercase().contains(feature);
    placements
        .iter()
        .any(|p| has(&p.id) || p.submodel.iter().any(|s| has(s)))
        || step_titles.iter().any(|t| has(t))
}

/// A set's parts list: (Rebrickable part, Rebrickable color) → how many, spares left out.
#[derive(Debug, Default)]
pub struct Inventory {
    pub lines: BTreeMap<(String, i64), usize>,
}

impl Inventory {
    pub fn pieces(&self) -> usize {
        self.lines.values().sum()
    }
}

fn csv_rows(path: &Path) -> Result<Vec<Vec<String>>> {
    let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
    // Rebrickable's id, number, and flag columns hold no commas or quotes; these files have no free text.
    Ok(text
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|c| c.trim().to_string()).collect())
        .collect())
}

/// `set`'s inventory from a Rebrickable catalog folder (`inventories.csv`, `inventory_parts.csv`): its lowest
/// version, without spares.
pub fn inventory(catalog: &Path, set: &str) -> Result<Inventory> {
    let id = csv_rows(&catalog.join("inventories.csv"))?
        .into_iter()
        .filter(|r| r.len() >= 3 && r[2] == set)
        .min_by_key(|r| r[1].parse::<u32>().unwrap_or(u32::MAX))
        .map(|r| r[0].clone())
        .ok_or_else(|| Error::internal(format!("{} has no inventory for {set}", catalog.display())))?;
    let mut inv = Inventory::default();
    for r in csv_rows(&catalog.join("inventory_parts.csv"))? {
        if r.len() < 5 || r[0] != id || r[4].eq_ignore_ascii_case("true") {
            continue;
        }
        let (Ok(color), Ok(qty)) = (r[2].parse::<i64>(), r[3].parse::<usize>()) else {
            return Err(Error::internal(format!(
                "{}: a row of inventory {id} isn't part, color, quantity: {r:?}",
                catalog.display()
            )));
        };
        *inv.lines.entry((r[1].clone(), color)).or_default() += qty;
    }
    if inv.lines.is_empty() {
        return Err(Error::internal(format!(
            "{}: inventory {id} ({set}) lists no parts",
            catalog.display()
        )));
    }
    Ok(inv)
}

/// Molds of one part count as that part: Rebrickable's mold relationships (`M` rows of `part_relationships.csv`),
/// each part named by the first id of its group. A set's inventory names the mold that year's boxes held; a builder
/// ordering parts gets whichever mold is made now.
pub fn molds(catalog: &Path) -> Result<HashMap<String, String>> {
    let mut of: HashMap<String, String> = HashMap::new();
    fn root(of: &HashMap<String, String>, id: &str) -> String {
        let mut at = id.to_string();
        while let Some(next) = of.get(&at).filter(|n| **n != at) {
            at = next.clone();
        }
        at
    }
    for r in csv_rows(&catalog.join("part_relationships.csv"))? {
        if r.len() < 3 || r[0] != "M" {
            continue;
        }
        let (a, b) = (root(&of, &r[1]), root(&of, &r[2]));
        if a != b {
            let (keep, fold) = if a < b { (a, b) } else { (b, a) };
            of.insert(fold, keep);
        }
    }
    let ids: Vec<String> = of.keys().cloned().collect();
    Ok(ids.into_iter().map(|id| (id.clone(), root(&of, &id))).collect())
}

/// How close a parts list is to a set's.
#[derive(Debug, PartialEq)]
pub struct F1 {
    pub f1: f64,
    /// Of the model's pieces, the share the set has.
    pub precision: f64,
    /// Of the set's pieces, the share the model has.
    pub recall: f64,
    pub matched: usize,
    pub model: usize,
    pub set: usize,
}

/// F1 over part × color, counting pieces: a line of 4 where the set has 6 matches 4. `model` is each line's
/// Rebrickable part and color, `None` for a part or color Rebrickable doesn't know, which matches nothing.
pub fn bom_f1(model: &[(Option<String>, Option<i64>, usize)], set: &Inventory, molds: &HashMap<String, String>) -> F1 {
    let canon = |p: &str| molds.get(p).cloned().unwrap_or_else(|| p.to_string());
    let mut want: BTreeMap<(String, i64), usize> = BTreeMap::new();
    for ((part, color), qty) in &set.lines {
        *want.entry((canon(part), *color)).or_default() += qty;
    }
    let mut have: BTreeMap<(String, i64), usize> = BTreeMap::new();
    let mut pieces = 0;
    for (part, color, qty) in model {
        pieces += qty;
        if let (Some(part), Some(color)) = (part, color) {
            *have.entry((canon(part), *color)).or_default() += qty;
        }
    }
    let matched: usize = have
        .iter()
        .map(|(k, n)| (*n).min(want.get(k).copied().unwrap_or(0)))
        .sum();
    let total = set.pieces();
    let share = |of: usize| if of == 0 { 0.0 } else { matched as f64 / of as f64 };
    let (precision, recall) = (share(pieces), share(total));
    let f1 = if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    };
    F1 {
        f1,
        precision,
        recall,
        matched,
        model: pieces,
        set: total,
    }
}

/// How many of the set's parts the model has in place.
#[derive(Debug, PartialEq)]
pub struct Placed {
    pub matched: usize,
    pub set: usize,
}

impl Placed {
    pub fn share(&self) -> f64 {
        if self.set == 0 {
            0.0
        } else {
            self.matched as f64 / self.set as f64
        }
    }
}

/// A part's id for matching (an alias and the part it stands for are one part), and where its origin is, in LDU.
pub fn located(index: &Index, placements: &[Placement]) -> Vec<(String, [f64; 3])> {
    let mut canon: HashMap<String, String> = HashMap::new();
    placements
        .iter()
        .map(|p| {
            let id = canon
                .entry(p.part.clone())
                .or_insert_with(|| {
                    index
                        .part(&p.part)
                        .ok()
                        .flatten()
                        .and_then(|part| part.alias_of)
                        .unwrap_or_else(|| p.part.clone())
                        .to_lowercase()
                })
                .clone();
            (id, [p.matrix[12], p.matrix[13], p.matrix[14]])
        })
        .collect()
}

/// The share of the set's parts with a model part of the same id within a stud, each model part matching once. The
/// model may stand anywhere and face any of the four ways: the set is turned in quarter turns about the vertical and
/// slid to wherever the most parts line up, and the best of those counts.
pub fn placement_match(set: &[(String, [f64; 3])], model: &[(String, [f64; 3])]) -> Placed {
    let mut by_id: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, (id, _)) in model.iter().enumerate() {
        by_id.entry(id.as_str()).or_default().push(i);
    }
    let mut best = 0;
    for turn in 0..4 {
        // LDraw's vertical is y: a quarter turn swaps x and z.
        let turned: Vec<[f64; 3]> = set
            .iter()
            .map(|(_, [x, y, z])| match turn {
                0 => [*x, *y, *z],
                1 => [-*z, *y, *x],
                2 => [-*x, *y, -*z],
                _ => [*z, *y, -*x],
            })
            .collect();
        // Every pair of one set part and one model part of its id votes for the slide that would put them together,
        // to the half stud; the slides with the most votes are tried exactly.
        let mut votes: HashMap<[i64; 3], (usize, [f64; 3])> = HashMap::new();
        for ((id, _), at) in set.iter().zip(&turned) {
            for &m in by_id.get(id.as_str()).map(Vec::as_slice).unwrap_or(&[]) {
                let p = model[m].1;
                let slide = [p[0] - at[0], p[1] - at[1], p[2] - at[2]];
                let bin = slide.map(|d| (d / (STUD / 2.0)).round() as i64);
                votes.entry(bin).or_insert((0, slide)).0 += 1;
            }
        }
        let mut slides: Vec<(usize, [f64; 3])> = votes.into_values().collect();
        slides.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        for (_, slide) in slides.into_iter().take(8) {
            let mut used = vec![false; model.len()];
            let mut matched = 0;
            for ((id, _), at) in set.iter().zip(&turned) {
                let want = [at[0] + slide[0], at[1] + slide[1], at[2] + slide[2]];
                let nearest = by_id
                    .get(id.as_str())
                    .into_iter()
                    .flatten()
                    .filter(|&&m| !used[m])
                    .map(|&m| {
                        let p = model[m].1;
                        let d = ((p[0] - want[0]).powi(2) + (p[1] - want[1]).powi(2) + (p[2] - want[2]).powi(2)).sqrt();
                        (d, m)
                    })
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                if let Some((d, m)) = nearest
                    && d <= STUD + 0.01
                {
                    used[m] = true;
                    matched += 1;
                }
            }
            best = best.max(matched);
        }
    }
    Placed {
        matched: best,
        set: set.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inv(lines: &[(&str, i64, usize)]) -> Inventory {
        Inventory {
            lines: lines.iter().map(|(p, c, n)| ((p.to_string(), *c), *n)).collect(),
        }
    }

    #[test]
    fn f1_counts_pieces_by_part_and_color() {
        let set = inv(&[("3001", 4, 6), ("3020", 0, 2), ("3023", 15, 2)]);
        let none = HashMap::new();
        // The set itself: 1.0.
        let same: Vec<_> = set
            .lines
            .iter()
            .map(|((p, c), n)| (Some(p.clone()), Some(*c), *n))
            .collect();
        assert_eq!(bom_f1(&same, &set, &none).f1, 1.0);
        // 4 of the 6 red bricks, the plates in the wrong color, and a part Rebrickable doesn't know: 4 of 7 pieces are
        // the set's, and 4 of the set's 10 are here.
        let model = [
            (Some("3001".to_string()), Some(4), 4),
            (Some("3020".to_string()), Some(1), 2),
            (None, Some(0), 1),
        ];
        let f = bom_f1(&model, &set, &none);
        assert_eq!((f.matched, f.model, f.set), (4, 7, 10));
        assert!((f.f1 - 2.0 * (4.0 / 7.0) * 0.4 / (4.0 / 7.0 + 0.4)).abs() < 1e-12);
        // Nothing built: 0, not a division by zero.
        assert_eq!(bom_f1(&[], &set, &none).f1, 0.0);
        // Another mold of a part is that part.
        let molds = HashMap::from([("3023b".to_string(), "3023".to_string())]);
        let mold = [(Some("3023b".to_string()), Some(15), 2)];
        assert_eq!(bom_f1(&mold, &set, &molds).matched, 2);
        assert_eq!(bom_f1(&mold, &set, &none).matched, 0);
    }

    #[test]
    fn placements_match_wherever_the_model_stands_and_whichever_way_it_faces() {
        let set: Vec<(String, [f64; 3])> = vec![
            ("3001".into(), [0.0, 0.0, 0.0]),
            ("3001".into(), [80.0, 0.0, 0.0]),
            ("3020".into(), [40.0, -24.0, 20.0]),
            ("3023".into(), [0.0, -24.0, 60.0]),
        ];
        assert_eq!(placement_match(&set, &set).matched, 4);
        // Turned a quarter and moved 7 studs along and 2 bricks up: still the same model.
        let moved: Vec<(String, [f64; 3])> = set
            .iter()
            .map(|(id, [x, y, z])| (id.clone(), [-z + 140.0, y - 48.0, x + 60.0]))
            .collect();
        assert_eq!(placement_match(&set, &moved).matched, 4);
        // One part a stud off still matches; two studs off doesn't; another part in the right place doesn't either.
        let mut off = set.clone();
        off[2].1[0] += 20.0;
        assert_eq!(placement_match(&set, &off).matched, 4);
        off[2].1[0] += 20.0;
        assert_eq!(placement_match(&set, &off).matched, 3);
        let mut wrong = set.clone();
        wrong[3].0 = "3024".into();
        assert_eq!(placement_match(&set, &wrong).matched, 3);
        // One model part can't stand in for two of the set's.
        let one = vec![set[0].clone()];
        assert_eq!(placement_match(&set, &one), Placed { matched: 1, set: 4 });
        assert_eq!(placement_match(&set, &[]).share(), 0.0);
    }
}
