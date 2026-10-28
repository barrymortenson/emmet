// SPDX-License-Identifier: AGPL-3.0-or-later
//! The validator corpus and its seeded mutation generator (plan section 11): official models, which validate with no
//! errors, and faults injected into them, each with the rule that must catch it. A fault's ground truth comes from
//! the parts' body boxes alone (which part sits on which, what's above it), never from the validator's connection
//! graph, so recall measures the validator instead of agreeing with it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use emmet_core::check::{Rule, Validation};
use emmet_core::data::DataDir;
use emmet_core::index::Index;
use emmet_core::ldraw::{self, placements::Placement};
use glam::{DMat4, DVec3};

/// The placements of an .mpd or .ldr file.
pub fn placements_of(path: &Path) -> Vec<Placement> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let doc = ldraw::parse(&String::from_utf8_lossy(&bytes)).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let placements = ldraw::placements::placements(&doc);
    assert!(!placements.is_empty(), "{} places no parts", path.display());
    placements
}

/// The .mpd and .ldr files in `dir`, sorted.
pub fn models_in(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "mpd" || x == "ldr"))
        .collect();
    out.sort();
    out
}

pub fn validate(data: &Path, placements: &[Placement]) -> Validation {
    emmet_core::ops::validate_placements(&DataDir::at(data), placements, &Rule::ALL, &[], None)
        .unwrap_or_else(|e| panic!("validate: {e}"))
}

/// One line per finding, for failure messages.
pub fn report(v: &Validation) -> String {
    v.findings
        .iter()
        .map(|f| format!("  {} {} {:?}: {}", f.id, f.rule.name(), f.parts, f.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A placed part's world box (studs left out) and what the generator needs to know about it.
#[derive(Clone)]
struct Geo {
    min: DVec3,
    max: DVec3,
    /// Its own −Y points up the world's −Y: studs up.
    upright: bool,
    /// A brick, plate, tile, or slope: no clips, pins, or hinges to hold it some other way.
    basic: bool,
}

/// The kinds of fault, each with the rule that must report its part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fault {
    /// A part with nothing above it, lifted a plate off its supports: floating.
    Lift,
    /// A part removed from under one that stands on it alone: the one above floats.
    RemoveSupport,
    /// A part pushed a plate down into what it stands on: collision.
    Sink,
    /// A part pushed a stud sideways into the part beside it: collision.
    Slide,
    /// A long part turned a quarter turn about its middle: collision or floating.
    Turn,
    /// A part moved to a step before every part under it: step_order.
    Early,
    /// A step merged with the ones after it until it's too big: step_size.
    Lump,
    /// A submodel copy lifted a plate off what holds it: disconnected_submodel.
    OffsetCopy,
    /// A part nudged 4 LDU sideways, off the grid: off_grid.
    Nudge,
    /// A part id the library doesn't have: unknown_part.
    Unknown,
}

impl Fault {
    pub const ALL: [Fault; 10] = [
        Fault::Lift,
        Fault::RemoveSupport,
        Fault::Sink,
        Fault::Slide,
        Fault::Turn,
        Fault::Early,
        Fault::Lump,
        Fault::OffsetCopy,
        Fault::Nudge,
        Fault::Unknown,
    ];

    /// The rules that count as catching it.
    pub fn rules(self) -> &'static [Rule] {
        match self {
            Fault::Lift | Fault::RemoveSupport => &[Rule::Floating],
            Fault::Sink | Fault::Slide => &[Rule::Collision],
            Fault::Turn => &[Rule::Collision, Rule::Floating],
            Fault::Early => &[Rule::StepOrder],
            Fault::Lump => &[Rule::StepSize],
            Fault::OffsetCopy => &[Rule::DisconnectedSubmodel],
            Fault::Nudge => &[Rule::OffGrid],
            Fault::Unknown => &[Rule::UnknownPart],
        }
    }

    /// The rule its recall is reported under.
    pub fn rule(self) -> Rule {
        self.rules()[0]
    }
}

/// One injected fault: the changed placements and the part a finding must name.
pub struct Mutant {
    pub fault: Fault,
    pub placements: Vec<Placement>,
    pub part: String,
    pub what: String,
}

impl Mutant {
    /// True when a finding of one of the fault's rules names the part.
    pub fn caught(&self, v: &Validation) -> bool {
        v.findings
            .iter()
            .any(|f| self.fault.rules().contains(&f.rule) && f.parts.iter().any(|p| p == &self.part))
    }
}

/// A small seeded generator (xorshift64*), so a run is the same every time.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Up to `n` items of `from`, in a seeded order.
    pub fn pick<T: Clone>(&mut self, from: &[T], n: usize) -> Vec<T> {
        let mut v = from.to_vec();
        for i in (1..v.len()).rev() {
            v.swap(i, (self.next() % (i as u64 + 1)) as usize);
        }
        v.truncate(n);
        v
    }
}

const PLATE: f64 = 8.0;
const STUD: f64 = 20.0;
/// How far inside a part's sides another part must reach to be over or under it.
const INSET: f64 = 2.0;

fn geometry(index: &Index, placements: &[Placement]) -> Vec<Option<Geo>> {
    let mut parts = BTreeMap::new();
    placements
        .iter()
        .map(|p| {
            let part = parts
                .entry(p.part.clone())
                .or_insert_with(|| index.part(&p.part).ok().flatten())
                .clone()?;
            let b = part.body_box?;
            let m = DMat4::from_cols_array(&p.matrix);
            let (mut min, mut max) = (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY));
            for i in 0..8 {
                let c = DVec3::new(
                    if i & 1 == 0 { b[0] } else { b[3] },
                    if i & 2 == 0 { b[1] } else { b[4] },
                    if i & 4 == 0 { b[2] } else { b[5] },
                );
                let w = m.transform_point3(c);
                min = min.min(w);
                max = max.max(w);
            }
            Some(Geo {
                min,
                max,
                upright: m.y_axis.truncate().normalize_or_zero().y > 0.999,
                basic: ["Brick", "Plate", "Tile", "Slope"].contains(&part.category.as_str()),
            })
        })
        .collect()
}

/// The footprints of `a` and `b` overlap by more than INSET on both horizontal axes.
fn over(a: &Geo, b: &Geo) -> bool {
    a.min.x + INSET < b.max.x && b.min.x + INSET < a.max.x && a.min.z + INSET < b.max.z && b.min.z + INSET < a.max.z
}

/// The parts `i` stands on: their tops at its bottom (LDraw's −Y is up, so a top is a min.y), under its footprint.
fn supports(geo: &[Option<Geo>], i: usize) -> Vec<usize> {
    let Some(a) = &geo[i] else { return vec![] };
    (0..geo.len())
        .filter(|&j| {
            j != i
                && geo[j]
                    .as_ref()
                    .is_some_and(|b| (b.min.y - a.max.y).abs() < 1.0 && over(a, b))
        })
        .collect()
}

/// Nothing over `i`'s footprint within `h` LDU above its top (studs included).
fn clear_above(geo: &[Option<Geo>], i: usize, h: f64) -> bool {
    let Some(a) = &geo[i] else { return false };
    !(0..geo.len()).any(|j| {
        j != i
            && geo[j]
                .as_ref()
                .is_some_and(|b| over(a, b) && b.max.y > a.min.y - h && b.min.y < a.min.y + 1.0)
    })
}

fn moved(p: &Placement, by: DVec3) -> Placement {
    let mut q = p.clone();
    q.matrix[12] += by.x;
    q.matrix[13] += by.y;
    q.matrix[14] += by.z;
    q
}

/// The faults the generator can inject into `placements`, up to `per_fault` of each kind, chosen by `rng`.
pub fn mutants(index: &Index, placements: &[Placement], per_fault: usize, rng: &mut Rng) -> Vec<Mutant> {
    let geo = geometry(index, placements);
    let n = placements.len();
    let lowest = geo.iter().flatten().map(|g| g.max.y).fold(f64::NEG_INFINITY, f64::max);
    let plain = |i: usize| geo[i].as_ref().is_some_and(|g| g.upright && g.basic);
    let raised = |i: usize| geo[i].as_ref().is_some_and(|g| g.max.y < lowest - 1.0);
    let replace = |i: usize, q: Placement| {
        let mut v = placements.to_vec();
        v[i] = q;
        v
    };
    let id = |i: usize| placements[i].id.clone();
    let mut out = Vec::new();
    let mut add = |fault: Fault,
                   cands: Vec<usize>,
                   make: &dyn Fn(usize) -> Option<(Vec<Placement>, String, String)>,
                   rng: &mut Rng| {
        let mut made = 0;
        for i in rng.pick(&cands, cands.len()) {
            if made == per_fault {
                break;
            }
            if let Some((placements, part, what)) = make(i) {
                out.push(Mutant {
                    fault,
                    placements,
                    part,
                    what,
                });
                made += 1;
            }
        }
    };

    // Floating: lift a part with nothing above it, or take away the one part it stands on.
    let leaves: Vec<usize> = (0..n)
        .filter(|&i| plain(i) && raised(i) && !supports(&geo, i).is_empty() && clear_above(&geo, i, 2.0 * STUD))
        .collect();
    add(
        Fault::Lift,
        leaves.clone(),
        &|i| {
            Some((
                replace(i, moved(&placements[i], DVec3::new(0.0, -PLATE, 0.0))),
                id(i),
                format!("lift {} a plate", id(i)),
            ))
        },
        rng,
    );
    add(
        Fault::RemoveSupport,
        leaves.clone(),
        &|i| {
            let s = supports(&geo, i);
            (s.len() == 1).then(|| {
                let mut v = placements.to_vec();
                v.remove(s[0]);
                (v, id(i), format!("remove {} from under {}", id(s[0]), id(i)))
            })
        },
        rng,
    );

    // Collision: push a part down into what it stands on, or sideways into its neighbor.
    let stacked: Vec<usize> = (0..n).filter(|&i| plain(i) && !supports(&geo, i).is_empty()).collect();
    add(
        Fault::Sink,
        stacked.clone(),
        &|i| {
            Some((
                replace(i, moved(&placements[i], DVec3::new(0.0, PLATE, 0.0))),
                id(i),
                format!("sink {} a plate", id(i)),
            ))
        },
        rng,
    );
    add(
        Fault::Slide,
        (0..n).filter(|&i| plain(i)).collect(),
        &|i| {
            let a = geo[i].as_ref()?;
            // A neighbor touching one side, sharing at least a plate of height and a stud of that side.
            let dirs = [DVec3::X, DVec3::NEG_X, DVec3::Z, DVec3::NEG_Z];
            let shares = |b: &Geo| a.min.y + PLATE <= b.max.y && b.min.y + PLATE <= a.max.y;
            for d in dirs {
                let touching = geo.iter().enumerate().any(|(j, b)| {
                    let Some(b) = b else { return false };
                    if j == i || !shares(b) {
                        return false;
                    }
                    let (face_a, face_b) = if d.x > 0.0 {
                        (a.max.x, b.min.x)
                    } else if d.x < 0.0 {
                        (-a.min.x, -b.max.x)
                    } else if d.z > 0.0 {
                        (a.max.z, b.min.z)
                    } else {
                        (-a.min.z, -b.max.z)
                    };
                    let across = if d.x != 0.0 {
                        a.min.z + STUD <= b.max.z && b.min.z + STUD <= a.max.z
                    } else {
                        a.min.x + STUD <= b.max.x && b.min.x + STUD <= a.max.x
                    };
                    (face_a - face_b).abs() < 1.0 && across
                });
                if touching {
                    return Some((
                        replace(i, moved(&placements[i], d * STUD)),
                        id(i),
                        format!("slide {} a stud", id(i)),
                    ));
                }
            }
            None
        },
        rng,
    );
    add(
        Fault::Turn,
        stacked.clone(),
        &|i| {
            let a = geo[i].as_ref()?;
            let (w, d) = (a.max.x - a.min.x, a.max.z - a.min.z);
            if (w - d).abs() < 2.0 * STUD - 1.0 {
                return None;
            }
            let center = (a.min + a.max) / 2.0;
            let m = DMat4::from_cols_array(&placements[i].matrix);
            let turned = DMat4::from_translation(center)
                * DMat4::from_rotation_y(std::f64::consts::FRAC_PI_2)
                * DMat4::from_translation(-center)
                * m;
            let mut q = placements[i].clone();
            q.matrix = turned.to_cols_array();
            Some((replace(i, q), id(i), format!("turn {} a quarter turn", id(i))))
        },
        rng,
    );

    // step_order: a main-model part moved to a step before every part under it.
    add(
        Fault::Early,
        leaves.clone(),
        &|i| {
            let p = &placements[i];
            if !p.submodel.is_empty() {
                return None;
            }
            let s = supports(&geo, i);
            let first = s.iter().map(|&j| placements[j].step).min()?;
            if first < 2 || s.iter().any(|&j| !placements[j].submodel.is_empty()) {
                return None;
            }
            let mut q = p.clone();
            q.step = first - 1;
            Some((
                replace(i, q),
                id(i),
                format!("move {} from step {} to {}", id(i), p.step, first - 1),
            ))
        },
        rng,
    );

    // step_size: a main-model step merged with the steps after it until it has more than 12 parts.
    let mut steps: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (i, p) in placements.iter().enumerate() {
        if p.submodel.is_empty() {
            steps.entry(p.step).or_default().push(i);
        }
    }
    let step_list: Vec<u32> = steps.keys().copied().collect();
    add(
        Fault::Lump,
        (0..step_list.len()).collect(),
        &|k| {
            let first = step_list[k];
            let mut count = 0;
            let mut v = placements.to_vec();
            for &s in &step_list[k..] {
                for &i in &steps[&s] {
                    v[i].step = first;
                    count += 1;
                }
                if count > 12 {
                    let i = steps[&first][0];
                    return Some((v, id(i), format!("merge steps {first}–{s} into one of {count} parts")));
                }
            }
            None
        },
        rng,
    );

    // disconnected_submodel: a top-level submodel copy, lifted a plate, with nothing above it.
    let mut copies: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, p) in placements.iter().enumerate() {
        if let Some(c) = p.copies.first() {
            copies.entry(c.id.clone()).or_default().push(i);
        }
    }
    let copy_list: Vec<String> = copies.keys().cloned().collect();
    add(
        Fault::OffsetCopy,
        (0..copy_list.len()).collect(),
        &|k| {
            let members = &copies[&copy_list[k]];
            if members.len() == placements.len() || !members.iter().all(|&i| raised(i)) {
                return None;
            }
            let inside = |j: usize| members.contains(&j);
            // Nothing outside the copy is over any of its parts.
            let blocked = members.iter().any(|&i| {
                let Some(a) = &geo[i] else { return true };
                geo.iter().enumerate().any(|(j, b)| {
                    !inside(j)
                        && b.as_ref()
                            .is_some_and(|b| over(a, b) && b.max.y > a.min.y - 2.0 * STUD && b.min.y < a.min.y + 1.0)
                })
            });
            if blocked {
                return None;
            }
            let mut v = placements.to_vec();
            for &i in members {
                v[i] = moved(&placements[i], DVec3::new(0.0, -PLATE, 0.0));
            }
            Some((
                v,
                id(members[0]),
                format!("lift submodel copy {} a plate", copy_list[k]),
            ))
        },
        rng,
    );

    add(
        Fault::Nudge,
        stacked.clone(),
        &|i| {
            Some((
                replace(i, moved(&placements[i], DVec3::new(4.0, 0.0, 0.0))),
                id(i),
                format!("nudge {} 4 LDU", id(i)),
            ))
        },
        rng,
    );
    add(
        Fault::Unknown,
        (0..n).collect(),
        &|i| {
            let mut q = placements[i].clone();
            q.part = "x-emmet-missing".into();
            Some((replace(i, q), id(i), format!("give {} an id the library lacks", id(i))))
        },
        rng,
    );
    out
}

/// Runs up to `per_fault` faults of each kind the generator makes for each model and returns the recall, printing each miss.
pub fn recall(data: &Path, models: &[(String, Vec<Placement>)], per_fault: usize) -> Recall {
    let index = Index::ensure(&DataDir::at(data)).unwrap();
    let mut recall = Recall::default();
    for (name, placements) in models {
        let seed = name
            .bytes()
            .fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));
        for m in mutants(&index, placements, per_fault, &mut Rng::new(seed)) {
            let v = validate(data, &m.placements);
            let caught = m.caught(&v);
            if !caught {
                println!("missed {:?} in {name}: {}\n{}", m.fault, m.what, report(&v));
            }
            recall.add(m.fault, caught);
        }
    }
    recall
}

/// Caught and total faults per rule.
#[derive(Default)]
pub struct Recall(pub BTreeMap<Rule, (usize, usize)>);

impl Recall {
    pub fn add(&mut self, fault: Fault, caught: bool) {
        let e = self.0.entry(fault.rule()).or_default();
        e.0 += caught as usize;
        e.1 += 1;
    }

    /// Fails under the gates (plan section 11): 95 % overall, 90 % for each error rule.
    pub fn gate(&self) {
        let (c, t) = self.overall();
        assert!(
            t > 0 && c * 100 >= t * 95,
            "overall recall under 95 %:\n{}",
            self.table()
        );
        for (rule, &(c, t)) in &self.0 {
            if rule.severity() == emmet_core::check::Severity::Error {
                assert!(
                    c * 100 >= t * 90,
                    "{} recall under 90 %:\n{}",
                    rule.name(),
                    self.table()
                );
            }
        }
    }

    pub fn overall(&self) -> (usize, usize) {
        self.0.values().fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
    }

    /// A Markdown table, for the test output and the PR.
    pub fn table(&self) -> String {
        let pct = |(c, t): (usize, usize)| {
            if t == 0 {
                "n/a".into()
            } else {
                format!("{:.1} %", 100.0 * c as f64 / t as f64)
            }
        };
        let mut s = String::from("| Rule | Severity | Caught | Recall |\n|---|---|---|---|\n");
        for (rule, &(c, t)) in &self.0 {
            s += &format!(
                "| {} | {:?} | {c} / {t} | {} |\n",
                rule.name(),
                rule.severity(),
                pct((c, t))
            );
        }
        let o = self.overall();
        s += &format!("| **overall** | | {} / {} | {} |\n", o.0, o.1, pct(o));
        s
    }
}
