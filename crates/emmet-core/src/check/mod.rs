// SPDX-License-Identifier: AGPL-3.0-or-later
//! validate v0 (plan section 11): collisions by shrunk body boxes, floating parts by stud connections, and unknown,
//! obsolete, and alias parts. Findings carry stable ids, the parts' source lines, a position in the DSL frame, and a
//! hint, and go to `build/validation.json`. 3A replaces the boxes with collision proxies and the studs with the full
//! connection model.

mod boxes;
pub mod connectors;
pub mod graph;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;

use glam::{DMat4, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::envelope::Result;
use crate::index::{Index, Part};
use crate::ldraw::geometry::{Aabb, LibFiles, Measurer};
use crate::ldraw::placements::Placement;
use boxes::Obb;
use connectors::{Connector, Scanner};

/// What a finding means for the build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// The rules (plan section 11), in the order findings are listed within a severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    UnknownPart,
    Collision,
    Floating,
    DisconnectedSubmodel,
    StepOrder,
    StepSize,
    WeakConnection,
    OffGrid,
    ObsoletePart,
    AliasPart,
}

impl Rule {
    pub const ALL: [Rule; 10] = [
        Rule::UnknownPart,
        Rule::Collision,
        Rule::Floating,
        Rule::DisconnectedSubmodel,
        Rule::StepOrder,
        Rule::StepSize,
        Rule::WeakConnection,
        Rule::OffGrid,
        Rule::ObsoletePart,
        Rule::AliasPart,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Rule::UnknownPart => "unknown_part",
            Rule::Collision => "collision",
            Rule::Floating => "floating",
            Rule::DisconnectedSubmodel => "disconnected_submodel",
            Rule::StepOrder => "step_order",
            Rule::StepSize => "step_size",
            Rule::WeakConnection => "weak_connection",
            Rule::OffGrid => "off_grid",
            Rule::ObsoletePart => "obsolete_part",
            Rule::AliasPart => "alias_part",
        }
    }

    pub fn parse(name: &str) -> Option<Rule> {
        Rule::ALL.into_iter().find(|r| r.name() == name.trim())
    }

    pub fn severity(self) -> Severity {
        match self {
            Rule::UnknownPart | Rule::Collision | Rule::Floating | Rule::DisconnectedSubmodel | Rule::StepOrder => {
                Severity::Error
            }
            Rule::StepSize | Rule::WeakConnection | Rule::OffGrid | Rule::ObsoletePart => Severity::Warning,
            Rule::AliasPart => Severity::Info,
        }
    }
}

/// A line of the model's source that placed a part.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct Source {
    pub file: String,
    pub line: u32,
}

/// One problem with the model (plan section 11's finding shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Finding {
    /// `F-0001`…, stable for the same inputs.
    pub id: String,
    pub rule: Rule,
    pub severity: Severity,
    /// The Emmet ids of the parts involved.
    pub parts: Vec<String>,
    /// Where the model placed them, deduplicated, in source order.
    pub source: Vec<Source>,
    /// A point in the DSL frame: x and y in studs, z in plates.
    pub at: [f64; 3],
    pub message: String,
    pub hint: String,
}

/// A finding the project allows (`[[check.allow]]` in emmet.toml), with its reason.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Allowed {
    pub rule: String,
    pub parts: Vec<String>,
    pub reason: String,
}

/// `build/validation.json` (plan section 11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Validation {
    pub schema: u32,
    /// The inputs hash of the build these findings are for.
    pub inputs_hash: Option<String>,
    /// No errors and no warnings.
    pub clean: bool,
    /// Findings per rule, without the allowed ones.
    pub counts: BTreeMap<String, usize>,
    pub findings: Vec<Finding>,
    pub allowed: Vec<Allowed>,
}

impl Validation {
    pub fn errors(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Error).count()
    }

    pub fn finding(&self, id: &str) -> Option<&Finding> {
        self.findings.iter().find(|f| f.id.eq_ignore_ascii_case(id.trim()))
    }
}

/// An allow entry from emmet.toml.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Allow {
    pub rule: String,
    #[serde(default)]
    pub parts: Vec<String>,
    #[serde(default)]
    pub reason: String,
}

/// Shrunk this far on every side, body boxes that only touch never collide.
const SHRINK: f64 = 1.0;
/// Shrunk boxes that overlap by more than this collide (plan section 11)…
const OVERLAP: f64 = 0.5;
/// …and by more than this side by side, where body boxes take in pins, clips, and hinges that stick out of a part's
/// sides (a quarter stud). A part a half stud or more into its neighbor still collides.
const OVERLAP_SIDE: f64 = 5.0;
/// A stud enters a part through its underside when its base is at most this far inside it…
const SEAT: f64 = 1.5;
/// Parts whose bottom is this close to the model's lowest point stand on the ground.
const GROUND: f64 = 1.0;
/// Without a baseplate, a group with nothing under it whose bottom is this close to the lowest point stands on the
/// table beside the rest (two plates: models place a vehicle and the figures next to it a little apart in height).
const ON_THE_TABLE: f64 = 16.0;
const STUD: f64 = 20.0;
const PLATE: f64 = 8.0;

/// One placement with what the rules need to know about its part.
struct Placed<'p> {
    p: &'p Placement,
    m: DMat4,
    info: Option<Rc<Info>>,
}

/// A part's index row and its connectors.
struct Info {
    part: Part,
    /// What the part is, for rules that go by category or name: the part itself, or for a moved id (`~Moved to
    /// 3139b`) the part it moved to.
    kind: Part,
    body: Option<Aabb>,
    connectors: Rc<Vec<Connector>>,
}

/// The placements with what the rules need: each part's row, its box in the world, its connectors in the world, the
/// connections, and the pairs one part holds the other without a connector pair.
struct Model<'p> {
    placed: Vec<Placed<'p>>,
    world: Vec<Option<Obb>>,
    conns: Vec<graph::Placed>,
    links: Vec<graph::Connection>,
    held: HashSet<(usize, usize)>,
}

fn prepare<'p>(index: &Index, lib: &LibFiles, placements: &'p [Placement]) -> Result<Model<'p>> {
    let mut scanner = Scanner::new(Measurer::new(lib));
    let mut infos: HashMap<String, Option<Rc<Info>>> = HashMap::new();
    let mut placed = Vec::with_capacity(placements.len());
    for p in placements {
        let info = match infos.get(&p.part) {
            Some(i) => i.clone(),
            None => {
                let info = match index.part(&p.part)? {
                    Some(part) => {
                        let body = part.body_box.map(Aabb::from_array);
                        let connectors = scanner.connectors(&format!("{}.dat", part.id));
                        let kind = match (&part.alias_of, part.name.starts_with("~Moved to")) {
                            (Some(to), true) => index.part(to)?.unwrap_or_else(|| part.clone()),
                            _ => part.clone(),
                        };
                        Some(Rc::new(Info {
                            part,
                            kind,
                            body,
                            connectors,
                        }))
                    }
                    None => None,
                };
                infos.insert(p.part.clone(), info.clone());
                info
            }
        };
        placed.push(Placed {
            p,
            m: DMat4::from_cols_array(&p.matrix),
            info,
        });
    }
    let world: Vec<Option<Obb>> = placed
        .iter()
        .map(|q| Some(Obb::new(q.info.as_ref()?.body.as_ref()?, &q.m, 0.0)))
        .collect();
    let held = held_pairs(&placed, &world);
    let conns = graph::world(
        &placed
            .iter()
            .map(|q| {
                (
                    q.m,
                    q.info.as_ref().map_or_else(Default::default, |i| i.connectors.clone()),
                )
            })
            .collect::<Vec<_>>(),
    );
    let mut links = graph::connections(&conns);
    links.extend(underside_links(&placed, &world, &conns));
    Ok(Model {
        placed,
        world,
        conns,
        links,
        held,
    })
}

/// Checks `placements` (the rows of build/placements.json) with the rules in `rules`, against the parts index and the
/// library at `lib`.
pub fn validate(
    index: &Index,
    lib: &LibFiles,
    placements: &[Placement],
    rules: &[Rule],
    allow: &[Allow],
    inputs_hash: Option<String>,
) -> Result<Validation> {
    let model = prepare(index, lib, placements)?;
    let Model {
        placed,
        world,
        links,
        held,
        ..
    } = &model;
    let mut raw: Vec<Raw> = Vec::new();
    let on = |r: Rule| rules.contains(&r);
    if on(Rule::UnknownPart) || on(Rule::ObsoletePart) || on(Rule::AliasPart) {
        raw.extend(part_rules(placed, rules));
    }
    if on(Rule::Collision) {
        raw.extend(collisions(placed, world, held));
    }
    if on(Rule::Floating) {
        raw.extend(floating(placed, world, links, held));
    }
    Ok(finish(placed, raw, allow, inputs_hash))
}

/// For debugging the connection model: each named placement's connectors in the world, the near misses, every
/// connection that touches it, and what holds it without a connector pair.
#[doc(hidden)]
pub fn explain(index: &Index, lib: &LibFiles, placements: &[Placement], ids: &[String]) -> Result<String> {
    let model = prepare(index, lib, placements)?;
    let (world, links) = (&model.conns, &model.links);
    let mut out = String::new();
    for (i, p) in placements.iter().enumerate() {
        if !ids.iter().any(|id| id == &p.id) {
            continue;
        }
        out += &format!("{} {} at {:?}\n", p.id, p.part, &p.matrix[12..15]);
        for (k, w) in world.iter().enumerate().filter(|(_, w)| w.part == i) {
            let c = &w.c;
            out += &format!(
                "  {:?} {:?} {:?} pos {:.2?} axis {:.2?} secs {:?} slide {} group {:?}\n",
                c.source,
                c.kind,
                c.gender,
                c.pos.to_array(),
                c.axis.to_array(),
                c.secs.iter().map(|s| (s.radius, s.len)).collect::<Vec<_>>(),
                c.slide,
                c.group
            );
            for (j, why, off) in graph::near(&world, k, 4.0) {
                let q = &world[j];
                out += &format!(
                    "      near {} ({}) {:?} {:?} pos {:.2?} axis {:.2?} secs {:?} off {off:.2} why {why}\n",
                    placements[q.part].id,
                    placements[q.part].part,
                    q.c.kind,
                    q.c.gender,
                    q.c.pos.to_array(),
                    q.c.axis.to_array(),
                    q.c.secs.iter().map(|s| (s.radius, s.len)).collect::<Vec<_>>(),
                );
            }
        }
        for l in links.iter().filter(|l| l.a == i || l.b == i) {
            let other = if l.a == i { l.b } else { l.a };
            out += &format!(
                "  linked to {} ({}) by {:?} at {:.1?}\n",
                placements[other].id,
                placements[other].part,
                l.kind,
                l.at.to_array()
            );
        }
        for &(a, b) in model.held.iter().filter(|(a, b)| *a == i || *b == i) {
            let other = if a == i { b } else { a };
            out += &format!("  held with {} ({})\n", placements[other].id, placements[other].part);
        }
        if let Some(w) = model.world[i] {
            let b = w.aabb();
            out += &format!("  box {:.1?}..{:.1?}\n", b.min.to_array(), b.max.to_array());
        }
    }
    Ok(out)
}

/// A finding before it's numbered.
struct Raw {
    rule: Rule,
    /// Indexes into the placements.
    parts: Vec<usize>,
    at: [f64; 3],
    message: String,
    hint: String,
}

fn name(q: &Placed) -> String {
    q.info
        .as_ref()
        .map_or_else(|| q.p.part.clone(), |i| i.part.name.clone())
}

/// unknown_part, obsolete_part, and alias_part: one finding per part id, naming every placement of it.
fn part_rules(placed: &[Placed], rules: &[Rule]) -> Vec<Raw> {
    let mut by_part: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, q) in placed.iter().enumerate() {
        by_part.entry(q.p.part.as_str()).or_default().push(i);
    }
    let mut out = Vec::new();
    for (part, idx) in by_part {
        let q = &placed[idx[0]];
        let n = idx.len();
        let them = if n == 1 {
            "1 part uses".to_string()
        } else {
            format!("{n} parts use")
        };
        let (rule, message, hint) = match &q.info {
            None => (
                Rule::UnknownPart,
                format!("{them} '{part}', which isn't in the LDraw library."),
                "Find the part with search_parts (emmet search) and use its id.".to_string(),
            ),
            Some(info) => {
                let p = &info.part;
                let moved = p.name.starts_with("~Moved to");
                let to = p.alias_of.clone().unwrap_or_default();
                if moved || p.flags.iter().any(|f| f == "obsolete") {
                    let hint = if to.is_empty() {
                        "Use a current part instead; search_parts finds one.".to_string()
                    } else {
                        format!("Use {to} instead: it's the same part under its current id.")
                    };
                    (
                        Rule::ObsoletePart,
                        format!("{them} {part} ({}), which is obsolete.", p.name),
                        hint,
                    )
                } else if p.flags.iter().any(|f| f == "alias") && !to.is_empty() {
                    (
                        Rule::AliasPart,
                        format!("{them} {part} ({}), an alias of {to}.", p.name),
                        format!("Use {to}, the canonical id, so the parts list matches what you'd buy."),
                    )
                } else {
                    continue;
                }
            }
        };
        if !rules.contains(&rule) {
            continue;
        }
        out.push(Raw {
            rule,
            at: dsl_point(q.m.w_axis.truncate()),
            parts: idx,
            message,
            hint,
        });
    }
    out
}

/// The connector data's fallback: a part with no female connector at all (one the LDCad data doesn't cover, whose
/// underside isn't drawn with tube primitives) takes any stud that enters its underside, at least 4 LDU inside its
/// sides (validate v0's rule).
fn underside_links(placed: &[Placed], world: &[Option<Obb>], conns: &[graph::Placed]) -> Vec<graph::Connection> {
    use connectors::{Gender, Kind};
    const CELL: f64 = 40.0;
    let cell = |v: f64| (v / CELL).floor() as i64;
    let open: Vec<bool> = placed
        .iter()
        .map(|q| {
            q.info
                .as_ref()
                .is_some_and(|i| i.body.is_some() && !i.connectors.iter().any(|c| c.gender == Gender::Female))
        })
        .collect();
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, w) in world.iter().enumerate() {
        let (true, Some(w)) = (open[i], w) else { continue };
        let b = w.aabb();
        for x in cell(b.min.x)..=cell(b.max.x) {
            for z in cell(b.min.z)..=cell(b.max.z) {
                grid.entry((x, z)).or_default().push(i);
            }
        }
    }
    let mut out = Vec::new();
    if grid.is_empty() {
        return out;
    }
    let inverse: HashMap<usize, DMat4> = grid.values().flatten().map(|&j| (j, placed[j].m.inverse())).collect();
    for w in conns {
        let c = &w.c;
        if c.kind != Kind::Cyl || c.gender != Gender::Male || (c.secs[0].radius - 6.0).abs() > 0.6 || c.len() > 6.0 {
            continue;
        }
        let probe = c.pos + c.axis * (c.len() - 2.0).max(0.0);
        for &j in grid.get(&(cell(probe.x), cell(probe.z))).into_iter().flatten() {
            if j == w.part {
                continue;
            }
            let body = placed[j].info.as_ref().and_then(|i| i.body.as_ref());
            if receives(body, &inverse[&j], c.pos, c.axis, c.len()) {
                out.push(graph::Connection {
                    a: w.part,
                    b: j,
                    kind: Kind::Cyl,
                    at: c.pos + c.axis * c.len() / 2.0,
                    axis: c.axis,
                    radius: 6.0,
                    len: c.len(),
                });
            }
        }
    }
    out
}

/// Whether part `j` (its body box, and its inverse placement) takes a stud whose base is at `base` pointing along
/// `dir` and reaching `len`, in world space: the stud enters it through its underside, inside its sides.
fn receives(body: Option<&Aabb>, inv: &DMat4, base: DVec3, dir: DVec3, len: f64) -> bool {
    const ENTER: f64 = 2.0;
    const INSET: f64 = 4.0;
    let Some(body) = body else { return false };
    let local = inv.transform_point3(base);
    let local_dir = inv.transform_vector3(dir).normalize_or_zero();
    // Into the underside: the stud points up the part's own frame (−Y, so y shrinks along it), its base is at or
    // below the part's bottom, and its tip reaches into the part.
    let bottom = body.max.y;
    if local_dir.y > -0.9 || local.y < bottom - SEAT || local.y - len > bottom - ENTER {
        return false;
    }
    local.x >= body.min.x + INSET
        && local.x <= body.max.x - INSET
        && local.z >= body.min.z + INSET
        && local.z <= body.max.z - INSET
}

/// Window and door frames, and what fills them (glass, panes, shutters, doors): a filler inside its frame isn't a
/// collision.
fn frame(p: &Part) -> bool {
    matches!(p.category.as_str(), "Window" | "Door") && !filler(p) || p.name.contains("Frame")
}

fn filler(p: &Part) -> bool {
    p.category == "Glass"
        || ["Pane", "Shutter", "Glass"].iter().any(|w| p.name.contains(w))
        || p.category == "Door" && !p.name.contains("Frame")
}

fn inside(inner: &Aabb, outer: &Aabb, slack: f64) -> bool {
    (inner.min - outer.min).min_element() >= -slack && (outer.max - inner.max).min_element() >= -slack
}

/// True when one of placements `a` and `b` is a frame and the other fills it (glass in a window, a door in its frame):
/// they don't collide, and the frame holds the filler, which has no studs to hold it.
fn fills(pa: &Part, pb: &Part, wa: &Aabb, wb: &Aabb) -> bool {
    pa.id != pb.id && (frame(pa) && filler(pb) && inside(wb, wa, 2.0) || frame(pb) && filler(pa) && inside(wa, wb, 2.0))
}

fn tyre(p: &Part) -> bool {
    p.category == "Tyre" || p.name.trim_start_matches('~').starts_with("Tyre")
}

/// Parts that follow a path instead of snapping: hose and cable segments, chain links, strings.
pub(crate) fn flexible(p: &Part) -> bool {
    let n = p.name.trim_start_matches('~');
    matches!(p.category.as_str(), "Hose" | "String")
        || [
            "Hose Flexible",
            "Chain Link",
            "Chain Tread",
            "Minifig Chain",
            "Cable",
            "String",
        ]
        .iter()
        .any(|w| n.contains(w))
}

/// A minifigure's or figure's parts (and what they hold: headwear, accessories).
pub(crate) fn figure(p: &Part) -> bool {
    // By name too: an obsolete figure part's category is "Obsolete" (`~Minifig Hips (Obsolete)`).
    let name = p.name.trim_start_matches(['~', '_']);
    ["Minifig", "Figure"]
        .iter()
        .any(|w| p.category.starts_with(w) || name.starts_with(w))
}

pub(crate) fn sticker(p: &Part) -> bool {
    p.category.starts_with("Sticker")
}

/// One part holding another without a connector pair: a frame and its filler, a tyre and the wheel inside it, a
/// flexible part and what it touches along its path, a sticker and the surface it's on, and a figure and what it
/// touches (a figure is placed, not built: models pose figures by hand a millimeter off their joints, and seat or
/// stand them a millimeter off their studs).
fn holds(pa: &Part, pb: &Part, wa: &Aabb, wb: &Aabb) -> bool {
    let center = |b: &Aabb| (b.min + b.max) / 2.0;
    let contains = |outer: &Aabb, p: DVec3| {
        p.cmpge(outer.min - DVec3::splat(2.0)).all() && p.cmple(outer.max + DVec3::splat(2.0)).all()
    };
    // A wheel: a rim, or a part with wheels on it (122c01, Plate 2 x 2 with Red Wheels), whose box holds the tyre's
    // middle.
    let wheel = |p: &Part| !tyre(p) && (p.category == "Wheel" || p.name.contains("Wheel"));
    fills(pa, pb, wa, wb)
        || tyre(pa) && wheel(pb) && contains(wb, center(wa))
        || tyre(pb) && wheel(pa) && contains(wa, center(wb))
        || (flexible(pa) || flexible(pb) || sticker(pa) || sticker(pb) || figure(pa) || figure(pb))
            && touch(wa, wb, 1.0)
}

fn touch(a: &Aabb, b: &Aabb, slack: f64) -> bool {
    (a.min - b.max).max_element() <= slack && (b.min - a.max).max_element() <= slack
}

/// Every pair of placements one holds the other (see [`holds`]), found by a sweep of their world boxes along x.
fn held_pairs(placed: &[Placed], world: &[Option<Obb>]) -> HashSet<(usize, usize)> {
    let boxes: Vec<Option<Aabb>> = world.iter().map(|o| o.map(|o| o.aabb())).collect();
    let mut order: Vec<usize> = (0..placed.len())
        .filter(|&i| boxes[i].is_some() && placed[i].info.is_some())
        .collect();
    order.sort_by(|&a, &b| boxes[a].unwrap().min.x.total_cmp(&boxes[b].unwrap().min.x));
    let mut out = HashSet::new();
    for (k, &i) in order.iter().enumerate() {
        let bi = boxes[i].unwrap();
        for &j in &order[k + 1..] {
            let bj = boxes[j].unwrap();
            if bj.min.x > bi.max.x + 2.0 {
                break;
            }
            let (pi, pj) = (
                &placed[i].info.as_ref().unwrap().kind,
                &placed[j].info.as_ref().unwrap().kind,
            );
            if touch(&bi, &bj, 2.0) && holds(pi, pj, &bi, &bj) {
                out.insert((i.min(j), i.max(j)));
            }
        }
    }
    out
}

fn collisions(placed: &[Placed], world: &[Option<Obb>], held: &HashSet<(usize, usize)>) -> Vec<Raw> {
    let shrunk: Vec<Option<(Obb, Aabb)>> = placed
        .iter()
        .map(|q| {
            let o = Obb::new(q.info.as_ref()?.body.as_ref()?, &q.m, SHRINK);
            Some((o, o.aabb()))
        })
        .collect();
    // Sweep along x: each box is tested only against boxes that start before it ends.
    let mut order: Vec<usize> = (0..placed.len()).filter(|&i| shrunk[i].is_some()).collect();
    order.sort_by(|&a, &b| shrunk[a].unwrap().1.min.x.total_cmp(&shrunk[b].unwrap().1.min.x));
    let mut out = Vec::new();
    for (k, &a) in order.iter().enumerate() {
        let (oa, ba) = shrunk[a].unwrap();
        for &b in &order[k + 1..] {
            let (ob, bb) = shrunk[b].unwrap();
            if bb.min.x >= ba.max.x {
                break;
            }
            if bb.min.y >= ba.max.y || ba.min.y >= bb.max.y || bb.min.z >= ba.max.z || ba.min.z >= bb.max.z {
                continue;
            }
            let Some((depth, axis)) = oa.overlap(&ob) else { continue };
            let vertical = axis.y.abs() > 0.7;
            if depth <= if vertical { OVERLAP } else { OVERLAP_SIDE } {
                continue;
            }
            if held.contains(&(a.min(b), a.max(b))) {
                continue;
            }
            let (wa, wb) = (world[a].unwrap().aabb(), world[b].unwrap().aabb());
            out.push(collision(placed, a, b, depth + 2.0 * SHRINK, axis, &wa, &wb));
        }
    }
    out
}

/// A collision finding between placements `a` and `b`, ordered so the part placed later is the one to move.
fn collision(placed: &[Placed], a: usize, b: usize, depth: f64, axis: DVec3, wa: &Aabb, wb: &Aabb) -> Raw {
    let (a, b, wa, wb) = if src_key(placed[b].p) < src_key(placed[a].p) {
        (b, a, wb, wa)
    } else {
        (a, b, wa, wb)
    };
    let (qa, qb) = (&placed[a], &placed[b]);
    let both = Aabb {
        min: wa.min.max(wb.min),
        max: wa.max.min(wb.max),
    };
    let vertical = axis.y.abs() > 0.7;
    let amount = if vertical {
        plural(round(depth / PLATE), "plate")
    } else {
        plural(round(depth / STUD), "stud")
    };
    let message = format!(
        "{} ({}) and {} ({}) overlap by {amount}.",
        qa.p.id,
        name(qa),
        qb.p.id,
        name(qb)
    );
    let hint = if vertical {
        let (da, db) = (dsl_box(wa), dsl_box(wb));
        if db.min[2] + db.max[2] >= da.min[2] + da.max[2] {
            format!(
                "{} starts at z={} but {} ends at z={}: raise {} by {amount}.",
                qb.p.id,
                num(db.min[2]),
                qa.p.id,
                num(da.max[2]),
                qb.p.id
            )
        } else {
            format!(
                "{} ends at z={} but {} starts at z={}: lower {} by {amount}.",
                qb.p.id,
                num(db.max[2]),
                qa.p.id,
                num(da.min[2]),
                qb.p.id
            )
        }
    } else {
        format!(
            "They share {amount} side by side. Move {} by {amount}, or use a shorter part.",
            qb.p.id
        )
    };
    Raw {
        rule: Rule::Collision,
        parts: vec![a, b],
        at: dsl_point((both.min + both.max) / 2.0),
        message,
        hint,
    }
}

/// floating: the connection graph's components; every component with a part on the model's lowest level stands, and
/// every other component floats. A frame holds what fills it.
fn floating(
    placed: &[Placed],
    world: &[Option<Obb>],
    links: &[graph::Connection],
    held: &HashSet<(usize, usize)>,
) -> Vec<Raw> {
    let n = placed.len();
    let boxes: Vec<Option<Aabb>> = world.iter().map(|o| o.map(|o| o.aabb())).collect();
    let mut sets = UnionFind::new(n);
    for c in links {
        sets.join(c.a, c.b);
    }
    for &(i, j) in held {
        sets.join(i, j);
    }
    // The ground: the model's lowest point (LDraw's −Y is up, so the largest y).
    let lowest = boxes
        .iter()
        .flatten()
        .map(|b| b.max.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut grounded = vec![false; n];
    for (i, b) in boxes.iter().enumerate() {
        let base = placed[i].info.as_ref().is_some_and(|i| i.kind.category == "Baseplate");
        if base || b.is_some_and(|b| b.max.y >= lowest - GROUND) {
            let r = sets.root(i);
            grounded[r] = true;
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, b) in boxes.iter().enumerate() {
        if b.is_some() && !grounded[sets.root(i)] {
            groups.entry(sets.root(i)).or_default().push(i);
        }
    }
    // Without a baseplate, a model can be several things on the table (a car, a figure beside it), each at its own
    // height off it: a group with nothing of the model under it, within two plates of the lowest point, stands.
    let baseplate = placed
        .iter()
        .any(|q| q.info.as_ref().is_some_and(|i| i.kind.category == "Baseplate"));
    if !baseplate {
        groups.retain(|_, parts| {
            let bottom = parts
                .iter()
                .map(|&i| boxes[i].unwrap().max.y)
                .fold(f64::NEG_INFINITY, f64::max);
            let members: HashSet<usize> = parts.iter().copied().collect();
            let over_something = parts.iter().any(|&i| {
                let a = boxes[i].unwrap();
                boxes.iter().enumerate().any(|(j, b)| {
                    !members.contains(&j)
                        && b.is_some_and(|b| {
                            b.min.y >= a.max.y - 1.0
                                && a.min.x + 2.0 < b.max.x
                                && b.min.x + 2.0 < a.max.x
                                && a.min.z + 2.0 < b.max.z
                                && b.min.z + 2.0 < a.max.z
                        })
                })
            });
            over_something || bottom < lowest - ON_THE_TABLE
        });
    }
    groups
        .into_values()
        .map(|parts| floating_finding(placed, &boxes, parts))
        .collect()
}

fn floating_finding(placed: &[Placed], boxes: &[Option<Aabb>], parts: Vec<usize>) -> Raw {
    let mut all = boxes[parts[0]].unwrap();
    for &i in &parts[1..] {
        let b = boxes[i].unwrap();
        all = Aabb {
            min: all.min.min(b.min),
            max: all.max.max(b.max),
        };
    }
    let here = dsl_box(&all);
    let z = num(here.min[2]);
    let message = if parts.len() == 1 {
        let q = &placed[parts[0]];
        format!(
            "{} ({}) at z={z} isn't held by any stud: it touches nothing below it.",
            q.p.id,
            name(q)
        )
    } else {
        format!(
            "{} parts at z={z} aren't held by any stud: they touch nothing below them.",
            parts.len()
        )
    };
    // The highest part under the group, by footprint.
    let members: BTreeSet<usize> = parts.iter().copied().collect();
    let below = boxes
        .iter()
        .enumerate()
        .filter(|(i, b)| !members.contains(i) && b.is_some())
        .map(|(i, b)| (i, b.unwrap()))
        .filter(|(_, b)| {
            b.min.x < all.max.x
                && b.max.x > all.min.x
                && b.min.z < all.max.z
                && b.max.z > all.min.z
                && b.min.y >= all.max.y - SEAT
        })
        .max_by(|a, b| b.1.min.y.total_cmp(&a.1.min.y));
    let hint = match below {
        Some((i, b)) => {
            let top = dsl_box(&b).max[2];
            let gap = round(here.min[2] - top);
            let q = &placed[i];
            if gap > 0.0 {
                format!(
                    "They start {} above {} ({}), which ends at z={}: lower them by {}, or put a part under them.",
                    plural(gap, "plate"),
                    q.p.id,
                    name(q),
                    num(top),
                    plural(gap, "plate")
                )
            } else {
                format!(
                    "They rest on {} ({}) without a stud between them: put them on whole-stud positions, or on a part \
                     with studs.",
                    q.p.id,
                    name(q)
                )
            }
        }
        None => "Nothing is under them: give them a part to stand on, or move them down onto the model.".into(),
    };
    Raw {
        rule: Rule::Floating,
        at: here.min,
        parts,
        message,
        hint,
    }
}

/// Numbers the findings in a stable order (severity, then the first source line, then the parts) and applies the
/// project's allow entries.
fn finish(placed: &[Placed], raw: Vec<Raw>, allow: &[Allow], inputs_hash: Option<String>) -> Validation {
    let mut items: Vec<(Raw, Vec<Source>, Vec<String>)> = raw
        .into_iter()
        .map(|mut r| {
            r.parts.sort_by_key(|&i| (src_key(placed[i].p), placed[i].p.id.clone()));
            let mut source: Vec<Source> = Vec::new();
            for &i in &r.parts {
                if let Some(s) = parse_src(&placed[i].p.src)
                    && !source.contains(&s)
                {
                    source.push(s);
                }
            }
            let ids = r.parts.iter().map(|&i| placed[i].p.id.clone()).collect();
            (r, source, ids)
        })
        .collect();
    items.sort_by(|a, b| {
        (a.0.rule.severity(), a.1.first().cloned(), a.0.rule, &a.2).cmp(&(
            b.0.rule.severity(),
            b.1.first().cloned(),
            b.0.rule,
            &b.2,
        ))
    });
    let mut findings = Vec::new();
    let mut allowed = Vec::new();
    for (r, source, ids) in items {
        if let Some(a) = allow
            .iter()
            .find(|a| a.rule == r.rule.name() && !ids.is_empty() && ids.iter().all(|id| a.parts.contains(id)))
        {
            allowed.push(Allowed {
                rule: a.rule.clone(),
                parts: ids,
                reason: a.reason.clone(),
            });
            continue;
        }
        findings.push(Finding {
            id: format!("F-{:04}", findings.len() + 1),
            rule: r.rule,
            severity: r.rule.severity(),
            parts: ids,
            source,
            at: r.at,
            message: r.message,
            hint: r.hint,
        });
    }
    let mut counts = BTreeMap::new();
    for f in &findings {
        *counts.entry(f.rule.name().to_string()).or_insert(0) += 1;
    }
    Validation {
        schema: 1,
        inputs_hash,
        clean: !findings.iter().any(|f| f.severity != Severity::Info),
        counts,
        findings,
        allowed,
    }
}

/// `model.py:41` as a source; None for an empty or malformed one.
fn parse_src(src: &str) -> Option<Source> {
    let (file, line) = src.rsplit_once(':')?;
    Some(Source {
        file: file.to_string(),
        line: line.parse().ok()?,
    })
}

/// Sorts placements by where the model placed them; parts without a source come last.
fn src_key(p: &Placement) -> (bool, Option<Source>) {
    let s = parse_src(&p.src);
    (s.is_none(), s)
}

/// An LDraw point in the DSL frame (x = −Z/20, y = X/20, z = −Y/8).
fn dsl_point(v: DVec3) -> [f64; 3] {
    [round(-v.z / STUD), round(v.x / STUD), round(-v.y / PLATE)]
}

struct DslBox {
    min: [f64; 3],
    max: [f64; 3],
}

fn dsl_box(b: &Aabb) -> DslBox {
    DslBox {
        min: [round(-b.max.z / STUD), round(b.min.x / STUD), round(-b.max.y / PLATE)],
        max: [round(-b.min.z / STUD), round(b.max.x / STUD), round(-b.min.y / PLATE)],
    }
}

/// Rounded to 1/1000, so float noise from deep transforms stays out of messages and ids.
fn round(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0 + 0.0
}

fn num(v: f64) -> String {
    crate::ldraw::num(round(v))
}

fn plural(n: f64, unit: &str) -> String {
    let n = round(n);
    if n == 1.0 {
        format!("1 {unit}")
    } else {
        format!("{} {unit}s", num(n))
    }
}

struct UnionFind(Vec<usize>);

impl UnionFind {
    fn new(n: usize) -> Self {
        Self((0..n).collect())
    }

    fn root(&mut self, mut i: usize) -> usize {
        while self.0[i] != i {
            self.0[i] = self.0[self.0[i]];
            i = self.0[i];
        }
        i
    }

    fn join(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.root(a), self.root(b));
        if ra != rb {
            self.0[ra.max(rb)] = ra.min(rb);
        }
    }
}
