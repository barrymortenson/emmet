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
        })
    }

    pub fn path(&self, reference: &str) -> Option<&Path> {
        self.files.get(&normalize_ref(reference)).map(PathBuf::as_path)
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

/// Stud primitives (`stud*.dat`, and the `stug*.dat` stud groups) at any depth. The underside tubes share the prefix
/// and sit inside the body, so leaving them out doesn't change the box.
pub fn is_stud(reference: &str) -> bool {
    let name = normalize_ref(reference);
    let base = name.rsplit('/').next().unwrap_or(&name);
    base.starts_with("stud") || base.starts_with("stug")
}

/// Measures parts, caching each file's parsed lines and its own box.
pub struct Measurer<'a> {
    lib: &'a LibFiles,
    lines: HashMap<String, Option<std::rc::Rc<Vec<Line>>>>,
    boxes: HashMap<(String, bool), Option<Aabb>>,
}

const MAX_DEPTH: usize = 32;

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
            .and_then(|p| std::fs::read(p).ok())
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
    pub fn body_box(&mut self, reference: &str) -> Option<Aabb> {
        self.file_box(reference, true, 0)
            .or_else(|| self.file_box(reference, false, 0))
            .map(Aabb::rounded)
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
