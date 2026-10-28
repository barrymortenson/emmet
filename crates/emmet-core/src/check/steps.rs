// SPDX-License-Identifier: AGPL-3.0-or-later
//! The build-step rules (plan sections 08 and 11): step_order and step_size.
//!
//! Steps are read per scope: the main model, and each submodel copy with its own steps. A scope's elements are its own
//! parts and its submodel copies, each copy one element placed at the parent's step that places it (built in its own
//! box first, like an instruction book's sub-assembly).

use std::collections::{BTreeMap, HashMap, HashSet};

use glam::DVec3;

use super::connectors::Kind;
use super::structure::copy_path;
use super::{Model, ON_THE_TABLE, Options, Raw, Rule, Severity, UnionFind, dsl_box, name};
use crate::ldraw::geometry::Aabb;

/// One element of a scope: its parts, and the step that adds it.
struct Element {
    parts: Vec<usize>,
    step: u32,
    /// A submodel copy, not a part.
    copy: bool,
}

struct Scope {
    /// The copy path (empty for the main model).
    key: Vec<String>,
    elements: Vec<Element>,
    /// Each placement in the scope, by index, to its element.
    of: HashMap<usize, usize>,
}

fn scopes(m: &Model) -> Vec<Scope> {
    let mut by_key: BTreeMap<Vec<String>, Scope> = BTreeMap::new();
    // Each scope's element for each of its submodel copies, by the copy's path.
    let mut copies: HashMap<Vec<String>, usize> = HashMap::new();
    for (i, q) in m.placed.iter().enumerate() {
        let path = copy_path(q.p);
        for d in 0..=path.len() {
            let key = path[..d].to_vec();
            let scope = by_key.entry(key.clone()).or_insert_with(|| Scope {
                key,
                elements: Vec::new(),
                of: HashMap::new(),
            });
            let e = if d == path.len() {
                scope.elements.push(Element {
                    parts: Vec::new(),
                    step: q.p.step,
                    copy: false,
                });
                scope.elements.len() - 1
            } else {
                let step = q.p.copies.get(d).map_or(1, |c| c.step);
                *copies.entry(path[..d + 1].to_vec()).or_insert_with(|| {
                    scope.elements.push(Element {
                        parts: Vec::new(),
                        step,
                        copy: true,
                    });
                    scope.elements.len() - 1
                })
            };
            scope.elements[e].parts.push(i);
            scope.of.insert(i, e);
        }
    }
    by_key.into_values().collect()
}

fn scope_name(m: &Model, scope: &Scope) -> String {
    match scope.of.keys().next() {
        Some(&i) if !scope.key.is_empty() => {
            let sub = m.placed[i]
                .p
                .submodel
                .get(scope.key.len() - 1)
                .cloned()
                .unwrap_or_default();
            format!(" of submodel '{sub}'")
        }
        _ => String::new(),
    }
}

fn label(m: &Model, e: &Element) -> String {
    let q = &m.placed[e.parts[0]];
    if e.copy {
        let d = q.p.submodel.len().min(q.p.copies.len()).saturating_sub(1);
        format!("submodel '{}'", q.p.submodel.get(d).cloned().unwrap_or_default())
    } else {
        format!("{} ({})", q.p.id, name(q))
    }
}

/// step_order: each step's elements connect to elements from earlier steps of their scope (or stand on the ground in
/// the main model). An element placed under earlier parts with no free way in is a warning (v1). Parts in `skip`
/// (already floating or disconnected) are left out.
pub(super) fn step_order(m: &Model, options: &Options, ground: &[bool], skip: &HashSet<usize>) -> Vec<Raw> {
    let pairs = m.pairs();
    let loose =
        options.ground.is_empty() && !(0..m.placed.len()).any(|i| m.kind(i).is_some_and(|k| k.category == "Baseplate"));
    let mut out = Vec::new();
    for scope in scopes(m) {
        let es = &scope.elements;
        let Some(first) = es.iter().map(|e| e.step).min() else {
            continue;
        };
        let mut edges: Vec<(u32, usize, usize)> = pairs
            .iter()
            .filter_map(|&(a, b)| {
                let (&ea, &eb) = (scope.of.get(&a)?, scope.of.get(&b)?);
                (ea != eb).then(|| (es[ea].step.max(es[eb].step), ea, eb))
            })
            .collect();
        edges.sort();
        // In a submodel, an element joined to a part outside it is built onto the model around it (files often use
        // submodels to organize, not as sub-assemblies built in hand): it counts as joined to what came before.
        let mut outside = vec![false; es.len()];
        if !scope.key.is_empty() {
            for &(a, b) in &pairs {
                match (scope.of.get(&a), scope.of.get(&b)) {
                    (Some(&e), None) | (None, Some(&e)) => outside[e] = true,
                    _ => {}
                }
            }
        }
        let mut sets = UnionFind::new(es.len());
        let mut earliest: Vec<u32> = es.iter().map(|e| e.step).collect();
        let mut steps: Vec<u32> = es.iter().map(|e| e.step).collect();
        steps.sort();
        steps.dedup();
        let mut k = 0;
        for &s in &steps {
            while k < edges.len() && edges[k].0 <= s {
                let (ra, rb) = (sets.root(edges[k].1), sets.root(edges[k].2));
                if ra != rb {
                    let e = earliest[ra].min(earliest[rb]);
                    sets.join(ra, rb);
                    let r = sets.root(ra);
                    earliest[r] = e;
                }
                k += 1;
            }
            // Which groups stand on the ground (the model's ground, wherever the scope is).
            let mut stands = vec![false; es.len()];
            {
                for (ei, e) in es.iter().enumerate().filter(|(_, e)| e.step <= s) {
                    let on = e.parts.iter().any(|&i| ground[i])
                        || loose
                            && e.parts
                                .iter()
                                .all(|&i| m.boxes[i].is_none_or(|b| b.max.y >= m.lowest - ON_THE_TABLE));
                    if on {
                        let r = sets.root(ei);
                        stands[r] = true;
                    }
                }
            }
            for (ei, e) in es.iter().enumerate().filter(|(_, e)| e.step == s) {
                if e.parts.iter().any(|i| skip.contains(i)) {
                    continue;
                }
                let r = sets.root(ei);
                let connected = earliest[r] < s || outside[ei];
                // The first step needs nothing before it: its pieces may be laid out apart, or held in hand and built
                // under later (a body first, its legs last).
                if s == first {
                    continue;
                }
                if !connected && !stands[r] {
                    // Laid out and joined later by parts put on it (an instruction book bridges two plates this way):
                    // fine. Resting on parts that come later: it hangs in the air at its step.
                    if !rests_on_later(m, &scope, ei) {
                        continue;
                    }
                    out.push(unconnected(m, &scope, ei, &edges));
                } else if connected && !e.copy {
                    out.extend(no_way_in(m, &scope, ei));
                }
            }
        }
    }
    out
}

fn unconnected(m: &Model, scope: &Scope, ei: usize, edges: &[(u32, usize, usize)]) -> Raw {
    let es = &scope.elements;
    let e = &es[ei];
    // The earliest later element it connects to: what holds it.
    let later = edges
        .iter()
        .filter(|(_, a, b)| *a == ei || *b == ei)
        .map(|&(_, a, b)| if a == ei { b } else { a })
        .filter(|&o| es[o].step > e.step)
        .min_by_key(|&o| es[o].step);
    let what = label(m, e);
    let message = format!(
        "{what} in step {}{} connects to nothing from earlier steps.",
        e.step,
        scope_name(m, scope)
    );
    let hint = match later {
        Some(o) => format!(
            "It's held only by {} from step {}: move it to step {} or later, or move {} earlier.",
            label(m, &es[o]),
            es[o].step,
            es[o].step,
            label(m, &es[o])
        ),
        None => "Move it to a step after the parts that hold it.".into(),
    };
    let at = dsl_box(&bounds(m, &e.parts)).min;
    Raw::new(Rule::StepOrder, e.parts.clone(), at, message, hint)
}

/// Whether element `ei` sits on a part a later step adds (that part holds it up), or is held only by later parts some
/// way the connectors don't tell.
fn rests_on_later(m: &Model, scope: &Scope, ei: usize) -> bool {
    let es = &scope.elements;
    let e = &es[ei];
    let mine: HashSet<usize> = e.parts.iter().copied().collect();
    let later = |j: usize| scope.of.get(&j).is_some_and(|&o| es[o].step > e.step);
    let mut joined = false;
    for c in &m.links {
        let (me, other, toward) = if mine.contains(&c.a) {
            (c.a, c.b, c.toward_a)
        } else if mine.contains(&c.b) {
            (c.b, c.a, c.toward_b)
        } else {
            continue;
        };
        if me == other || mine.contains(&other) || !later(other) {
            continue;
        }
        joined = true;
        // Its side of the seat is up: it sits on the later part. Unknown: assume so.
        if toward.is_none_or(|t| t.y < -0.5) {
            return true;
        }
    }
    !joined
}

fn bounds(m: &Model, parts: &[usize]) -> Aabb {
    let mut it = parts.iter().filter_map(|&i| m.boxes[i]);
    let first = it.next().unwrap_or(Aabb {
        min: DVec3::ZERO,
        max: DVec3::ZERO,
    });
    it.fold(first, |a, b| Aabb {
        min: a.min.min(b.min),
        max: a.max.max(b.max),
    })
}

/// The insertion-path check (a warning in v1): a part that seats on earlier parts from one side, with an earlier part
/// in the way on that side, can't be put in (furniture added after the ceiling closed the room in).
fn no_way_in(m: &Model, scope: &Scope, ei: usize) -> Option<Raw> {
    let es = &scope.elements;
    let e = &es[ei];
    let i = e.parts[0];
    let earlier = |j: usize| scope.of.get(&j).is_some_and(|&o| es[o].step < e.step);
    // The way it comes in, from each connection to an earlier part: from the side its mate isn't on.
    let mut ways: Vec<DVec3> = Vec::new();
    let mut mates = HashSet::new();
    for c in m.links.iter().filter(|c| c.kind == Kind::Cyl && (c.a == i || c.b == i)) {
        let other = if c.a == i { c.b } else { c.a };
        if !earlier(other) {
            continue;
        }
        mates.insert(other);
        if let Some(w) = if c.a == i { c.toward_a } else { c.toward_b } {
            ways.push(w);
        }
    }
    let way = *ways.first()?;
    let squeezed = ways.iter().any(|w| w.dot(way) < -0.5);
    let blocker = if squeezed {
        None
    } else {
        if way.abs().max_element() < 0.99 {
            return None;
        }
        let b = m.boxes[i]?;
        // The box swept from the part out along its way in, to the model's edge.
        let far = 1.0e6;
        let swept = Aabb {
            min: b.min + DVec3::splat(1.0) + way.min(DVec3::ZERO) * far,
            max: b.max - DVec3::splat(1.0) + way.max(DVec3::ZERO) * far,
        };
        let beyond = |o: &Aabb| (o.max + o.min).dot(way) / 2.0 > (b.max + b.min).dot(way) / 2.0;
        // In the swept box: across the way in by at least 4 LDU each way (not a neighbor it only brushes), along it at
        // all.
        let inside = |o: &Aabb| {
            let d = o.max.min(swept.max) - o.min.max(swept.min);
            (0..3).all(|k| d[k] >= if way[k].abs() > 0.5 { 1.0 } else { 4.0 })
        };
        let center = (b.min + b.max) / 2.0;
        (0..m.placed.len())
            .filter(|&j| j != i && earlier(j) && !mates.contains(&j))
            .filter_map(|j| Some((j, m.boxes[j]?)))
            .filter(|(_, o)| beyond(o) && inside(o))
            .min_by(|a, b| {
                let d = |o: &Aabb| ((o.max + o.min) / 2.0 - center).length();
                d(&a.1).total_cmp(&d(&b.1))
            })
    };
    if !squeezed && blocker.is_none() {
        return None;
    }
    let what = label(m, e);
    let from = if way.y < -0.5 {
        "above"
    } else if way.y > 0.5 {
        "below"
    } else {
        "the side"
    };
    let (message, hint) = match blocker {
        Some((j, _)) => {
            let t = scope.of.get(&j).map_or(0, |&o| es[o].step);
            (
                format!(
                    "{what} goes in at step {}, after {} from step {t} closed the way in from {from}.",
                    e.step,
                    label(m, &es[scope.of[&j]])
                ),
                format!(
                    "Move it to a step before {t}, while there's room to put it in, or build that part as a section \
                     closed after."
                ),
            )
        }
        None => (
            format!(
                "{what} goes in at step {} between earlier parts above and below it.",
                e.step
            ),
            "Move it to a step before the parts on one side of it.".to_string(),
        ),
    };
    let mut raw = Raw::new(Rule::StepOrder, vec![i], dsl_box(&m.boxes[i]?).min, message, hint);
    raw.severity = Some(Severity::Warning);
    Some(raw)
}

/// step_size: a step adds more than the build's target (each submodel copy counts as one).
pub(super) fn step_size(m: &Model, target: usize) -> Vec<Raw> {
    let target = if target == 0 { 8 } else { target };
    let mut out = Vec::new();
    for scope in scopes(m) {
        let mut by_step: BTreeMap<u32, Vec<&Element>> = BTreeMap::new();
        for e in &scope.elements {
            by_step.entry(e.step).or_default().push(e);
        }
        for (s, es) in by_step {
            if es.len() <= target {
                continue;
            }
            let parts: Vec<usize> = es.iter().flat_map(|e| e.parts.iter().copied()).collect();
            let copies = es.iter().filter(|e| e.copy).count();
            let counted = match copies {
                0 => String::new(),
                1 => ", counting its sub-assembly as one".into(),
                n => format!(", counting each of its {n} sub-assemblies as one"),
            };
            out.push(Raw::new(
                Rule::StepSize,
                parts.clone(),
                dsl_box(&bounds(m, &parts)).min,
                format!(
                    "Step {s}{} adds {} parts{counted}: the build's target is {target} a step.",
                    scope_name(m, &scope),
                    es.len()
                ),
                format!("Split it into steps of about {target}: by wall, side, or row, the far side first."),
            ));
        }
    }
    out
}
