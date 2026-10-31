// SPDX-License-Identifier: AGPL-3.0-or-later
//! figure_clearance (D-53, `docs/spec/building-rules.md`): the spaces a figure needs, and the parts standing in them.
//! Spaces come from `m.keep_clear`: the techniques that place doors, seats, porches, and colonnades declare theirs, and
//! the agent declares the ones it builds by hand. Door frames found in the parts index make none: official models put a
//! counter, a stair, or a wall within a stud of their doors (Barry, 2026-10-31). A part is in a space when one of its
//! collision proxies is more than 1 LDU inside it on every axis (D-46).

use glam::DVec3;

use super::{KeepClear, Model, PLATE, Raw, Rule, STUD, dsl_point, num, plural};
use crate::ldraw::geometry::Aabb;

/// How deep a part must be inside a space to stand in it: the collision rule's bar (D-46).
const DEEP: f64 = 1.0;

/// One space from `m.keep_clear`: its boxes in LDraw's frame, what it's for, and where the model declared it.
struct Space {
    boxes: Vec<Aabb>,
    why: String,
    src: String,
}

pub(super) fn figure_clearance(m: &Model, keep: &[KeepClear]) -> Vec<Raw> {
    keep.iter()
        .flat_map(|k| {
            let space = Space {
                // Either corner first: the DSL writes min then max, and a hand-written placements.json may not.
                boxes: k
                    .boxes
                    .iter()
                    .map(|b| {
                        let (p, q) = (DVec3::new(b[0], b[1], b[2]), DVec3::new(b[3], b[4], b[5]));
                        Aabb {
                            min: p.min(q),
                            max: p.max(q),
                        }
                    })
                    .collect(),
                why: k.why.clone(),
                src: k.src.clone(),
            };
            findings(m, &space)
        })
        .collect()
}

/// Whether placement `j` is part of the building, which can block a figure: not a figure itself, its accessories, an
/// animal, a door leaf (it swings; a door frame is building), or a sticker.
fn building(m: &Model, j: usize) -> bool {
    let Some(info) = &m.placed[j].info else {
        return false;
    };
    let c = info.kind.category.as_str();
    let n = info.kind.name.trim_start_matches('~');
    !(n.starts_with("Minifig")
        || c.starts_with("Minifig")
        || c.starts_with("Figure")
        || (c == "Door" && !n.contains("Frame"))
        || matches!(c, "Animal" | "Sticker" | "Sticker Shortcut"))
}

/// How far `a` and `b` overlap on each axis (negative where they're apart).
fn overlap(a: &Aabb, b: &Aabb) -> DVec3 {
    a.max.min(b.max) - a.min.max(b.min)
}

/// Whether placement `j` stands in `space`: its world box first, then its collision proxies.
fn inside(m: &Model, j: usize, space: &Aabb) -> bool {
    let Some(wb) = m.boxes[j] else {
        return false;
    };
    if overlap(&wb, space).min_element() <= DEEP {
        return false;
    }
    // A part with no proxy (no closed body) is judged by its box.
    if m.proxies[j].is_empty() {
        return true;
    }
    let mj = m.placed[j].m;
    m.proxies[j].iter().any(|p| {
        let (lo, hi) = (
            DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64),
            DVec3::new(p[3] as f64, p[4] as f64, p[5] as f64),
        );
        let (mut wlo, mut whi) = (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY));
        for k in 0..8 {
            let c = DVec3::new(
                if k & 1 == 0 { lo.x } else { hi.x },
                if k & 2 == 0 { lo.y } else { hi.y },
                if k & 4 == 0 { lo.z } else { hi.z },
            );
            let w = mj.transform_point3(c);
            wlo = wlo.min(w);
            whi = whi.max(w);
        }
        overlap(&Aabb { min: wlo, max: whi }, space).min_element() > DEEP
    })
}

/// One finding for each of `space`'s boxes that parts stand in (a door's space has a box each side of its wall),
/// naming them all, with the move that clears them in a builder's words. A part in two boxes is named in the first.
fn findings(m: &Model, space: &Space) -> Vec<Raw> {
    let mut named = vec![false; m.placed.len()];
    let mut out = Vec::new();
    for s in &space.boxes {
        let parts: Vec<usize> = (0..m.placed.len())
            .filter(|&j| !named[j] && building(m, j) && inside(m, j, s))
            .collect();
        let Some(u) = parts.iter().filter_map(|&j| m.boxes[j]).reduce(|u, b| Aabb {
            min: u.min.min(b.min),
            max: u.max.max(b.max),
        }) else {
            continue;
        };
        for &j in &parts {
            named[j] = true;
        }
        out.push(finding(m, space, s, &u, parts));
    }
    out
}

/// The finding for `parts` (their box `u`) standing in the box `s` of `space`.
fn finding(m: &Model, space: &Space, s: &Aabb, u: &Aabb, parts: Vec<usize>) -> Raw {
    let what = name_of(m, &parts);
    let o = overlap(s, u);
    let at = dsl_point((s.min.max(u.min) + s.max.min(u.max)) / 2.0);
    let declared = if space.src.is_empty() {
        String::new()
    } else {
        format!(" (declared at {})", space.src)
    };
    let message = format!(
        "{} {} inside the space kept clear for {}{declared}.",
        capitalized(&what),
        if parts.len() == 1 || what.starts_with("the ") {
            "stands"
        } else {
            "stand"
        },
        space.why
    );
    let hint = hint(m, &parts, &what, &space.why, s, u, o);
    Raw::new(Rule::FigureClearance, parts, at, message, hint)
}

/// The smallest move that takes the parts (around `u`) out of the box `s`, preferring one that doesn't run them into
/// another part: up when they hang into it from above, otherwise sideways, in the DSL's words.
fn hint(m: &Model, parts: &[usize], what: &str, why: &str, s: &Aabb, u: &Aabb, o: DVec3) -> String {
    let allow = "or allow it with m.allow and a reason the user would accept.";
    // LDraw's −y is up: the parts hang in from above when their bottom is inside the space and their top above it.
    let from_above = u.max.y < s.max.y && u.min.y <= s.min.y + DEEP;
    if from_above {
        let plates = ((o.y - DEEP) / PLATE).ceil();
        return format!(
            "Raise {what} {} so it clears the space for {why}, {allow}",
            height(plates)
        );
    }
    // Sideways: LDraw +z is the DSL's −x, LDraw +x the DSL's +y.
    let mut moves = [
        (s.max.z - u.min.z, DVec3::Z, "left (−x)"),
        (u.max.z - s.min.z, -DVec3::Z, "right (+x)"),
        (s.max.x - u.min.x, DVec3::X, "back (+y)"),
        (u.max.x - s.min.x, -DVec3::X, "forward (−y)"),
    ];
    moves.sort_by(|a, b| a.0.total_cmp(&b.0));
    let studs = |d: f64| ((d - DEEP) / STUD).ceil();
    let fits = |&(d, dir, _): &(f64, DVec3, &str)| !runs_into(m, parts, dir * studs(d) * STUD);
    match moves.iter().copied().find(fits) {
        Some((d, _, dir)) => format!(
            "Move {what} {} {dir}, or move what needs the space the other way, so a figure fits; {allow}",
            plural(studs(d), "stud")
        ),
        None => format!(
            "Every way out for {what} runs into another part: move what needs the space instead, so a figure fits; \
             {allow}"
        ),
    }
}

/// Whether `parts`, moved by `by` (LDU), would stand in another part.
fn runs_into(m: &Model, parts: &[usize], by: DVec3) -> bool {
    parts.iter().filter_map(|&i| m.boxes[i]).any(|b| {
        let moved = Aabb {
            min: b.min + by,
            max: b.max + by,
        };
        m.boxes
            .iter()
            .enumerate()
            .filter(|(j, _)| !parts.contains(j))
            .filter_map(|(_, o)| o.as_ref())
            .any(|o| overlap(o, &moved).min_element() > DEEP)
    })
}

/// "a brick", "2 bricks", or "2 plates".
fn height(plates: f64) -> String {
    if plates > 0.0 && plates % 3.0 == 0.0 {
        if plates == 3.0 {
            "a brick".into()
        } else {
            format!("{} bricks", num(plates / 3.0))
        }
    } else {
        plural(plates, "plate")
    }
}

/// What a builder would call the parts: "the canopy (canopy-beams/2 and 9 more)" when their ids all start with that
/// word, else the first id and how many more.
fn name_of(m: &Model, parts: &[usize]) -> String {
    let ids: Vec<&str> = parts.iter().map(|&i| m.placed[i].p.id.as_str()).collect();
    let word = |id: &str| id.split(['-', '/', '_']).next().unwrap_or("").to_string();
    let first = word(ids[0]);
    let named =
        !first.is_empty() && first.chars().all(|c| c.is_ascii_alphabetic()) && ids.iter().all(|id| word(id) == first);
    let list = match ids.len() {
        1 => ids[0].to_string(),
        n => format!("{} and {} more", ids[0], n - 1),
    };
    if named { format!("the {first} ({list})") } else { list }
}

fn capitalized(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
