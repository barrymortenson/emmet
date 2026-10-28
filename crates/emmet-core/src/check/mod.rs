// SPDX-License-Identifier: AGPL-3.0-or-later
//! The validator (plan section 11, `docs/spec/validator.md`): the connection model (connectors, the pairs they seat,
//! and the pairs that hold without them) and the rules over it, collisions by collision proxies, and unknown,
//! obsolete, and alias parts. Findings carry stable ids, the parts' source lines, a position in the DSL frame, and a
//! hint, and go to `build/validation.json`.

mod boxes;
mod collision;
pub mod connectors;
pub mod graph;
mod holds;
pub mod proxy;
mod steps;
mod structure;

use std::collections::{BTreeMap, HashMap, HashSet};
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

/// The rules (plan section 11), in the order findings with the same severity and source line are listed.
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

    /// Every rule's name, for a hint: "collision, floating, …, or alias_part".
    pub fn list() -> String {
        let names: Vec<&str> = Rule::ALL.iter().map(|r| r.name()).collect();
        format!("{}, or {}", names[..names.len() - 1].join(", "), names[names.len() - 1])
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

/// A finding the project or the model allows (`[[check.allow]]` in emmet.toml, `m.allow`), with its reason.
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

/// What the validator reads besides the placements.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Options {
    /// Findings the project allows: emmet.toml's `[[check.allow]]` and the model's `m.allow`.
    pub allow: Vec<Allow>,
    /// The parts the model stands on (`m.ground`), besides its lowest level and any baseplate.
    pub ground: Vec<String>,
    /// How many parts a step may add before step_size warns: the Age build setting's target (0 means 8, the 6+
    /// target).
    pub step_target: usize,
}

/// An allow entry from emmet.toml or the model (`m.allow`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Allow {
    pub rule: String,
    #[serde(default)]
    pub parts: Vec<String>,
    #[serde(default)]
    pub reason: String,
}

/// How far inside a part a stud's base may be and still enter it through its underside, and how far a part's top may
/// be inside another's bottom and still be under it.
const SEAT: f64 = 1.5;
/// Parts whose bottom is this close to the model's lowest point stand on the ground.
const GROUND: f64 = 1.0;
/// Without a baseplate or ground parts, a group with nothing under it whose bottom is this close to the lowest point stands on the
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
    /// The world boxes around `world`.
    boxes: Vec<Option<Aabb>>,
    /// The model's lowest point (LDraw's −Y is up, so the largest y).
    lowest: f64,
    conns: Vec<graph::Placed>,
    links: Vec<graph::Connection>,
    held: HashSet<(usize, usize)>,
    /// Each placement's collision proxy (empty for an unknown part).
    proxies: Vec<Rc<Vec<proxy::ProxyBox>>>,
}

impl Model<'_> {
    /// Every pair of placements joined by a connection or held one by the other, each once.
    fn pairs(&self) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = self
            .links
            .iter()
            .map(|c| (c.a.min(c.b), c.a.max(c.b)))
            .chain(self.held.iter().copied())
            .collect();
        out.sort();
        out.dedup();
        out
    }

    fn kind(&self, i: usize) -> Option<&Part> {
        self.placed[i].info.as_ref().map(|i| &i.kind)
    }
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
    let held = holds::held_pairs(&placed, &world);
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
    links.extend(holds::underside_links(&placed, &world, &conns));
    let mut held = held;
    held.extend(holds::wedged_pairs(&world, &conns));
    let ids: Vec<String> = infos
        .iter()
        .filter(|(_, i)| i.is_some())
        .map(|(id, _)| id.clone())
        .collect();
    let cache = index.proxies(lib, &ids)?;
    let proxies = placed
        .iter()
        .map(|q| cache.get(&q.p.part).cloned().unwrap_or_default())
        .collect();
    let boxes: Vec<Option<Aabb>> = world.iter().map(|o| o.map(|o| o.aabb())).collect();
    let lowest = boxes
        .iter()
        .flatten()
        .map(|b| b.max.y)
        .fold(f64::NEG_INFINITY, f64::max);
    Ok(Model {
        placed,
        world,
        boxes,
        lowest,
        conns,
        links,
        held,
        proxies,
    })
}

/// Checks `placements` (the rows of build/placements.json) with the rules in `rules`, against the parts index and the
/// library at `lib`.
pub fn validate(
    index: &Index,
    lib: &LibFiles,
    placements: &[Placement],
    rules: &[Rule],
    options: &Options,
    inputs_hash: Option<String>,
) -> Result<Validation> {
    let model = prepare(index, lib, placements)?;
    let placed = &model.placed;
    let mut raw: Vec<Raw> = Vec::new();
    let on = |r: Rule| rules.contains(&r);
    if on(Rule::UnknownPart) || on(Rule::ObsoletePart) || on(Rule::AliasPart) {
        raw.extend(part_rules(placed, rules));
    }
    if on(Rule::Collision) {
        raw.extend(collision::collisions(&model));
    }
    let ground = structure::ground(&model, options);
    let groups = structure::floating_groups(&model, options);
    let (disconnected, named) = structure::disconnected_submodels(&model);
    if on(Rule::DisconnectedSubmodel) {
        raw.extend(disconnected);
    }
    let skip: HashSet<usize> = if on(Rule::DisconnectedSubmodel) {
        named
    } else {
        HashSet::new()
    };
    if on(Rule::Floating) {
        raw.extend(structure::floating(&model, &groups, &skip));
    }
    if on(Rule::StepOrder) {
        let mut stuck: HashSet<usize> = groups.iter().flatten().copied().collect();
        stuck.extend(skip.iter().copied());
        raw.extend(steps::step_order(&model, options, &ground, &stuck));
    }
    if on(Rule::StepSize) {
        raw.extend(steps::step_size(&model, options.step_target));
    }
    if on(Rule::WeakConnection) {
        raw.extend(structure::weak_connections(&model, &ground));
    }
    if on(Rule::OffGrid) {
        raw.extend(structure::off_grid(&model, &ground));
    }
    Ok(finish(placed, raw, &options.allow, inputs_hash))
}

/// Every connection between placements, as (id, id, kind), one per mating pair of connectors (a joint of two pairs
/// is listed twice), and every pair one holds the other (kind "held"): for tests that pin the connector data (track,
/// the overrides).
#[doc(hidden)]
pub fn links(index: &Index, lib: &LibFiles, placements: &[Placement]) -> Result<Vec<(String, String, String)>> {
    let model = prepare(index, lib, placements)?;
    let id = |i: usize| placements[i].id.clone();
    let mut out: Vec<(String, String, String)> = model
        .links
        .iter()
        .map(|c| (id(c.a), id(c.b), format!("{:?}", c.kind).to_lowercase()))
        .chain(model.held.iter().map(|&(a, b)| (id(a), id(b), "held".to_string())))
        .collect();
    out.sort();
    Ok(out)
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
            for (j, why, off) in graph::near(world, k, 4.0) {
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
    /// The finding's severity when it isn't the rule's (step_order's insertion-path check is a warning in v1).
    severity: Option<Severity>,
    /// Indexes into the placements.
    parts: Vec<usize>,
    at: [f64; 3],
    message: String,
    hint: String,
}

impl Raw {
    fn new(rule: Rule, parts: Vec<usize>, at: [f64; 3], message: String, hint: String) -> Self {
        Self {
            rule,
            severity: None,
            parts,
            at,
            message,
            hint,
        }
    }

    fn severity(&self) -> Severity {
        self.severity.unwrap_or(self.rule.severity())
    }
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
        out.push(Raw::new(rule, idx, dsl_point(q.m.w_axis.truncate()), message, hint));
    }
    out
}

/// Numbers the findings in a stable order (severity, then the first source line, then the rule, then the parts) and applies the
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
        (a.0.severity(), a.1.first().cloned(), a.0.rule, &a.2).cmp(&(
            b.0.severity(),
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
            severity: r.severity(),
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
