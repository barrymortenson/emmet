// SPDX-License-Identifier: AGPL-3.0-or-later
//! Finding library files and measuring parts: the body box, studs excluded, in the part's own LDU frame.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{Line, normalize_ref, parse};
use glam::{DMat3, DVec3};

/// Every file in an LDraw library, by normalized reference name. `parts/3001.dat` is `3001.dat`,
/// `parts/s/3001s01.dat` is `s/3001s01.dat`, `p/48/1-4cyli.dat` is `48/1-4cyli.dat`. When a name is in both,
/// `parts/` wins over `p/`, as in LDraw's search order.
#[derive(Debug, Default)]
pub struct LibFiles {
    pub root: PathBuf,
    files: HashMap<String, PathBuf>,
    /// Look files up on disk as they're asked for instead of from a scan (see [`LibFiles::probe`]).
    probe: bool,
}

impl LibFiles {
    pub fn scan(root: &Path) -> std::io::Result<Self> {
        let mut files = HashMap::new();
        for dir in ["p", "parts"] {
            let base = root.join(dir);
            if base.is_dir() {
                walk(&base, &base, &mut files)?;
            }
        }
        Ok(Self {
            root: root.to_path_buf(),
            files,
            probe: false,
        })
    }

    /// Finds files on disk when asked, in the same order, without scanning the library first: for reading a few
    /// parts (the validator) rather than all of them (the index). The library's files are lowercase, as LDraw
    /// publishes them.
    pub fn probe(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            files: HashMap::new(),
            probe: true,
        }
    }

    pub fn path(&self, reference: &str) -> Option<PathBuf> {
        let key = normalize_ref(reference);
        if !self.probe {
            return self.files.get(&key).cloned();
        }
        // A reference is a relative name inside the library; one that climbs out of it, or is absolute, is no file.
        if key.split('/').any(|c| c.is_empty() || c == "." || c == "..") || key.contains(':') {
            return None;
        }
        ["parts", "p"]
            .iter()
            .map(|dir| self.root.join(dir).join(&key))
            .find(|p| p.is_file())
    }

    /// The parts in `parts/` (not subparts), as (reference name, path), sorted by name.
    pub fn parts(&self) -> Vec<(&str, &Path)> {
        let parts_dir = self.root.join("parts");
        let mut out: Vec<_> = self
            .files
            .iter()
            .filter(|(name, path)| !name.contains('/') && path.parent() == Some(parts_dir.as_path()))
            .map(|(n, p)| (n.as_str(), p.as_path()))
            .collect();
        out.sort();
        out
    }
}

fn walk(base: &Path, dir: &Path, files: &mut HashMap<String, PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            walk(base, &path, files)?;
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("dat")) {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy();
            // Later directories in the scan order (parts/ after p/) take precedence.
            files.insert(normalize_ref(&rel), path.clone());
        }
    }
    Ok(())
}

/// An axis-aligned box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: DVec3,
    pub max: DVec3,
}

impl Aabb {
    fn point(p: DVec3) -> Self {
        Self { min: p, max: p }
    }

    fn add(acc: &mut Option<Aabb>, b: Aabb) {
        *acc = Some(match acc {
            Some(a) => Aabb {
                min: a.min.min(b.min),
                max: a.max.max(b.max),
            },
            None => b,
        });
    }

    /// The box around this one under `m · p + t`. Exact when `m` is axis-aligned.
    fn transformed(&self, m: DMat3, t: DVec3) -> Aabb {
        let mut out: Option<Aabb> = None;
        for i in 0..8 {
            let c = DVec3::new(
                if i & 1 == 0 { self.min.x } else { self.max.x },
                if i & 2 == 0 { self.min.y } else { self.max.y },
                if i & 4 == 0 { self.min.z } else { self.max.z },
            );
            Aabb::add(&mut out, Aabb::point(m * c + t));
        }
        out.expect("8 corners")
    }

    /// Rounded to 1/1000 LDU, so float noise from deep transforms doesn't leak into sizes and hashes.
    pub fn rounded(self) -> Aabb {
        let r = |v: DVec3| (v * 1000.0).round() / 1000.0 + DVec3::ZERO; // + 0 turns -0 into 0
        Aabb {
            min: r(self.min),
            max: r(self.max),
        }
    }

    pub fn to_array(self) -> [f64; 6] {
        [self.min.x, self.min.y, self.min.z, self.max.x, self.max.y, self.max.z]
    }

    pub fn from_array(a: [f64; 6]) -> Self {
        Self {
            min: DVec3::new(a[0], a[1], a[2]),
            max: DVec3::new(a[3], a[4], a[5]),
        }
    }
}

/// The System stud primitives a part's top studs are drawn with, by base name (plan section 11's primitive table,
/// started in 1D). Each is a stud whose base is its origin and which points along its own −Y. Not here: the underside
/// tubes and posts (`stud3`, `stud4*`, `stud12`, `stud16*`, `stud18a`, `stud21a`, `stud22a`, `stud23*`, `stud2s*`),
/// the Duplo, Quatro, and Scala studs, and the `studline` placeholder. `stug-*` stud groups aren't listed: they're made
/// of these.
pub const SYSTEM_STUDS: &[&str] = &[
    "stud",    // Stud
    "studa",   // Stud without Base Edges
    "stud2",   // Stud Open
    "stud2a",  // Stud Open without Base Edges
    "stud6",   // Stud Open For Round 2x2 Parts
    "stud6a",  // Stud Open For Round 2x2 Parts without Base Edges
    "stud9",   // Stud Open with Small Holes
    "stud10",  // Stud Truncated Laterally Curved 40D for Round 2 x 2 Parts
    "stud13",  // Stud Truncated Laterally Straight 38D for Electric Light & Sound Bricks
    "stud15",  // Stud Truncated Laterally Straight 40D for Round 2 x 2 Parts
    "stud17",  // Stud Open For Octagonal Parts
    "stud17a", // Stud Open For Octagonal Parts without Base Edges
    "stud26",  // Stud without Logo
    "studel",  // Stud with Electric Contact
    "studh",   // Stud with Hole at Outer Edge (Holder for Window/Door Bump)
    "studhl",  // Stud with Hole at Outer Edge with LEGO Logo - Left
    "studhr",  // Stud with Hole at Outer Edge with LEGO Logo - Right
    "studp01", // Stud with Dot Pattern
    "studx",   // Stud Cross
    "studxa",  // Stud Cross without Base Edges
    "stud-logo",
    "stud-logo2",
    "stud-logo3",
    "stud-logo4",
    "stud-logo5", // Stud with LEGO Logo
    "stud2-logo",
    "stud2-logo2",
    "stud2-logo3",
    "stud2-logo4",
    "stud2-logo5", // Stud Open with LEGO Logo
];

/// Top studs of the other systems (Duplo, Quatro, Scala): left out of body boxes like System studs, but no connectors.
const OTHER_TOP_STUDS: &[&str] = &["stud5", "stud7", "stud7a", "stud14", "stud19", "stud20", "stud24"];

fn base_name(reference: &str) -> String {
    let name = normalize_ref(reference);
    let base = name.rsplit('/').next().unwrap_or(&name);
    base.strip_suffix(".dat").unwrap_or(base).to_string()
}

/// True when `reference` names one of [`SYSTEM_STUDS`] (in `p/`, `p/8/`, or `p/48/`, any case, either separator).
pub fn is_system_stud(reference: &str) -> bool {
    SYSTEM_STUDS.contains(&base_name(reference).as_str())
}

/// The stud primitives body boxes leave out: top studs of every system and the `stug*` stud groups, at any depth.
/// Underside tubes and posts stay in: most sit inside the body, and some are its outside (a 1 x 1 round plate's lower
/// half is a `stud4`).
pub fn is_stud(reference: &str) -> bool {
    let base = base_name(reference);
    base.starts_with("stug") || SYSTEM_STUDS.contains(&base.as_str()) || OTHER_TOP_STUDS.contains(&base.as_str())
}

/// Measures parts, caching each file's parsed lines and its own box.
pub struct Measurer<'a> {
    lib: &'a LibFiles,
    lines: HashMap<String, Option<std::rc::Rc<Vec<Line>>>>,
    boxes: HashMap<(String, bool), Option<Aabb>>,
}

const MAX_DEPTH: usize = 32;
/// A stud's height, and its diameter with its edge lines, in LDU.
const STUD_HEIGHT: f64 = 4.0;
const STUD_WIDTH: f64 = 13.0;
const STUD_PITCH: f64 = 20.0;

impl<'a> Measurer<'a> {
    pub fn new(lib: &'a LibFiles) -> Self {
        Self {
            lib,
            lines: HashMap::new(),
            boxes: HashMap::new(),
        }
    }

    /// The parsed lines of a library file (every file of the document, one after another), or None when it's missing or unreadable.
    pub fn lines(&mut self, reference: &str) -> Option<std::rc::Rc<Vec<Line>>> {
        let key = normalize_ref(reference);
        if let Some(cached) = self.lines.get(&key) {
            return cached.clone();
        }
        let parsed = self
            .lib
            .path(&key)
            .and_then(|p| std::fs::read(&p).ok())
            .and_then(|bytes| {
                let text = String::from_utf8_lossy(&bytes);
                parse(&text)
                    .ok()
                    .map(|doc| std::rc::Rc::new(doc.files.into_iter().flat_map(|f| f.lines).collect::<Vec<_>>()))
            });
        self.lines.insert(key, parsed.clone());
        parsed
    }

    /// The body box: every vertex of every line type 2–4, recursively, leaving out studs. Falls back to the box with
    /// studs for a part that is nothing but studs. None when the file is missing or has no geometry.
    ///
    /// Some parts draw a few studs by hand instead of with stud primitives (door and window frames draw their cut
    /// corner studs that way), which would make the body a stud taller than the part stacks. When the box's top is a
    /// stud's height above the part's origin plane and everything above that plane is stud-sized islands no taller
    /// than a stud, the top is the origin plane.
    pub fn body_box(&mut self, reference: &str) -> Option<Aabb> {
        let found = self
            .file_box(reference, true, 0)
            .or_else(|| self.file_box(reference, false, 0))?;
        let mut b = found.rounded();
        if (b.min.y + STUD_HEIGHT).abs() < 0.01 && self.hand_drawn_studs(reference) {
            b.min.y = 0.0;
        }
        Some(b)
    }

    /// True when every piece of geometry above the part's origin plane (y < 0, LDraw's up) rises at most a stud's
    /// height above it, in islands no wider than a stud, on a body at least a stud wide each way (a brick-system top, not
    /// a hose segment or a pin).
    fn hand_drawn_studs(&mut self, reference: &str) -> bool {
        let mut pieces = Vec::new();
        self.pieces(reference, DMat3::IDENTITY, DVec3::ZERO, 0, &mut pieces);
        let (above, rest): (Vec<Aabb>, Vec<Aabb>) = pieces.into_iter().partition(|p| p.min.y < -0.01);
        // A piece may run down into the body (a cut stud's sides do); none may rise more than a stud above the plane.
        if above.is_empty() || above.iter().any(|p| p.min.y < -STUD_HEIGHT - 0.01) {
            return false;
        }
        let mut body = None;
        for p in rest {
            Aabb::add(&mut body, p);
        }
        let Some(body) = body else { return false };
        if body.max.x - body.min.x < STUD_PITCH - 0.5 || body.max.z - body.min.z < STUD_PITCH - 0.5 {
            return false;
        }
        // Islands: pieces whose footprints touch, joined.
        let mut islands: Vec<Aabb> = Vec::new();
        for p in above {
            let mut merged = p;
            islands.retain(|i| {
                let touch = i.min.x <= merged.max.x + 0.5
                    && merged.min.x <= i.max.x + 0.5
                    && i.min.z <= merged.max.z + 0.5
                    && merged.min.z <= i.max.z + 0.5;
                if touch {
                    merged = Aabb {
                        min: merged.min.min(i.min),
                        max: merged.max.max(i.max),
                    };
                }
                !touch
            });
            islands.push(merged);
        }
        islands
            .iter()
            .all(|i| i.max.x - i.min.x <= STUD_WIDTH && i.max.z - i.min.z <= STUD_WIDTH)
    }

    /// The box of every line type 2–4 and every stud-free subfile, in the part's frame (studs left out).
    fn pieces(&mut self, reference: &str, m: DMat3, t: DVec3, depth: usize, out: &mut Vec<Aabb>) {
        if depth > MAX_DEPTH {
            return;
        }
        let Some(lines) = self.lines(reference) else { return };
        for line in lines.iter() {
            let points: &[DVec3] = match line {
                Line::Sub { pos, matrix, file, .. } => {
                    if !is_stud(file) {
                        self.pieces(file, m * *matrix, m * *pos + t, depth + 1, out);
                    }
                    continue;
                }
                Line::Edge { points, .. } => points,
                Line::Tri { points, .. } => points,
                Line::Quad { points, .. } => points,
                Line::Optional { .. } | Line::Meta(_) | Line::Blank => continue,
            };
            let mut acc = None;
            for p in points {
                Aabb::add(&mut acc, Aabb::point(m * *p + t));
            }
            out.extend(acc);
        }
    }

    fn file_box(&mut self, reference: &str, skip_studs: bool, depth: usize) -> Option<Aabb> {
        let key = (normalize_ref(reference), skip_studs);
        if let Some(cached) = self.boxes.get(&key) {
            return *cached;
        }
        let mut acc = None;
        self.accumulate(reference, DMat3::IDENTITY, DVec3::ZERO, skip_studs, depth, &mut acc);
        self.boxes.insert(key, acc);
        acc
    }

    fn accumulate(
        &mut self,
        reference: &str,
        m: DMat3,
        t: DVec3,
        skip_studs: bool,
        depth: usize,
        acc: &mut Option<Aabb>,
    ) {
        if depth > MAX_DEPTH {
            return;
        }
        let Some(lines) = self.lines(reference) else { return };
        for line in lines.iter() {
            match line {
                Line::Sub { pos, matrix, file, .. } => {
                    if skip_studs && is_stud(file) {
                        continue;
                    }
                    let (cm, ct) = (m * *matrix, m * *pos + t);
                    if axis_aligned(cm) {
                        if let Some(b) = self.file_box(file, skip_studs, depth + 1) {
                            Aabb::add(acc, b.transformed(cm, ct));
                        }
                    } else {
                        self.accumulate(file, cm, ct, skip_studs, depth + 1, acc);
                    }
                }
                Line::Edge { points, .. } => points.iter().for_each(|p| Aabb::add(acc, Aabb::point(m * *p + t))),
                Line::Tri { points, .. } => points.iter().for_each(|p| Aabb::add(acc, Aabb::point(m * *p + t))),
                Line::Quad { points, .. } => points.iter().for_each(|p| Aabb::add(acc, Aabb::point(m * *p + t))),
                Line::Optional { .. } | Line::Meta(_) | Line::Blank => {}
            }
        }
    }
}

/// True when every row and column of `m` has one nonzero entry (a scaled permutation), so boxes map to boxes exactly.
fn axis_aligned(m: DMat3) -> bool {
    let a = m.to_cols_array();
    let nz = |v: f64| v.abs() > 1e-9;
    (0..3).all(|c| (0..3).filter(|&r| nz(a[c * 3 + r])).count() == 1)
        && (0..3).all(|r| (0..3).filter(|&c| nz(a[c * 3 + r])).count() == 1)
}
