// SPDX-License-Identifier: AGPL-3.0-or-later
//! Collision proxies (plan section 11, ADR 0013): a part's solid body, studs and anti-studs left out, as a few boxes in
//! its own LDraw frame, built once per part and cached in the parts index.
//!
//! The body is the part's triangles without the top studs and the underside tubes and posts (which go into another
//! part when the two connect). It's filled by vertical rays, one per whole-LDU column through the middle of the column
//! (x + 0.5, z + 0.5), each crossing the surface an even number of times; a voxel is solid when its middle is inside.
//! A face on a whole LDU (most of a brick's) is exact, so parts that only touch never overlap. Columns with an odd
//! number of crossings (a gap in the mesh) are left empty. The voxels then merge into boxes: columns with the same
//! runs merge into rectangles.

use std::collections::{BTreeMap, HashMap};

use glam::{DMat3, DVec3};

use crate::ldraw::geometry::{Measurer, is_stud};
use crate::ldraw::{Line, normalize_ref};

/// A box in the part's frame, in whole LDU: [min x, min y, min z, max x, max y, max z].
pub type ProxyBox = [i16; 6];

/// The most voxel columns a part is filled at 1 LDU; a larger part is filled at a coarser step, with every box grown
/// to whole LDU around it.
const MAX_COLUMNS: f64 = 1_500_000.0;
const MAX_DEPTH: usize = 32;

/// The underside tubes and posts (anti-stud geometry), by base name: a stud enters them, so they're no body.
fn anti_stud(reference: &str) -> bool {
    let name = normalize_ref(reference);
    let base = name.rsplit('/').next().unwrap_or(&name);
    let base = base.strip_suffix(".dat").unwrap_or(base);
    [
        "stud3", "stud4", "stud12", "stud16", "stud18", "stud21", "stud22", "stud23", "stud2s",
    ]
    .iter()
    .any(|p| base.starts_with(p))
}

/// The part's body triangles in its frame.
fn triangles(m: &mut Measurer, reference: &str, r: DMat3, t: DVec3, depth: usize, out: &mut Vec<[DVec3; 3]>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Some(lines) = m.lines(reference) else { return };
    for line in lines.iter() {
        match line {
            Line::Sub { pos, matrix, file, .. } => {
                if !is_stud(file) && !anti_stud(file) {
                    triangles(m, file, r * *matrix, r * *pos + t, depth + 1, out);
                }
            }
            Line::Tri { points, .. } => out.push(points.map(|p| r * p + t)),
            Line::Quad { points, .. } => {
                let p = points.map(|p| r * p + t);
                out.push([p[0], p[1], p[2]]);
                out.push([p[2], p[3], p[0]]);
            }
            _ => {}
        }
    }
}

/// The y values where the vertical line through (x, z) crosses a triangle.
fn crossing(tri: &[DVec3; 3], x: f64, z: f64) -> Option<f64> {
    let [a, b, c] = *tri;
    let (v0x, v0z, v1x, v1z) = (b.x - a.x, b.z - a.z, c.x - a.x, c.z - a.z);
    let det = v0x * v1z - v1x * v0z;
    if det.abs() < 1e-12 {
        return None; // vertical, or degenerate
    }
    let (px, pz) = (x - a.x, z - a.z);
    let u = (px * v1z - v1x * pz) / det;
    let v = (v0x * pz - px * v0z) / det;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0).then_some(a.y + u * (b.y - a.y) + v * (c.y - a.y))
}

/// The most boxes a proxy keeps: a part with more (a large curved one) is filled again at a coarser step.
const MAX_BOXES: usize = 2048;

/// The proxy of a part file: its body's boxes. Empty when it has no closed body (a sticker, a flag, a missing file).
pub fn build(m: &mut Measurer, reference: &str) -> Vec<ProxyBox> {
    let mut tris = Vec::new();
    triangles(m, reference, DMat3::IDENTITY, DVec3::ZERO, 0, &mut tris);
    let mut step = 1.0;
    loop {
        let boxes = fill(&tris, step);
        if boxes.len() <= MAX_BOXES || step >= 64.0 {
            return boxes;
        }
        step *= 2.0;
    }
}

fn fill(tris: &[[DVec3; 3]], min_step: f64) -> Vec<ProxyBox> {
    if tris.is_empty() {
        return Vec::new();
    }
    let (mut lo, mut hi) = (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY));
    for t in tris {
        for p in t {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
    }
    if !lo.is_finite() || !hi.is_finite() || (hi - lo).max_element() > 30_000.0 {
        return Vec::new();
    }
    // The column step: 1 LDU, or coarser for a very large part.
    let (x0, z0) = (lo.x.floor(), lo.z.floor());
    let area = (hi.x.ceil() - x0) * (hi.z.ceil() - z0);
    let step = (area / MAX_COLUMNS).sqrt().ceil().max(min_step);
    let nx = ((hi.x.ceil() - x0) / step).ceil() as i64;
    let nz = ((hi.z.ceil() - z0) / step).ceil() as i64;
    // Each triangle into the columns its footprint covers.
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (k, t) in tris.iter().enumerate() {
        let (tx0, tx1) = (
            t.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
            t.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
        );
        let (tz0, tz1) = (
            t.iter().map(|p| p.z).fold(f64::INFINITY, f64::min),
            t.iter().map(|p| p.z).fold(f64::NEG_INFINITY, f64::max),
        );
        let col = |v: f64, o: f64| ((v - o) / step - 0.5).floor() as i64;
        for i in col(tx0, x0).max(0)..=(col(tx1, x0) + 1).min(nx - 1) {
            for j in col(tz0, z0).max(0)..=(col(tz1, z0) + 1).min(nz - 1) {
                grid.entry((i, j)).or_default().push(k);
            }
        }
    }
    // Each column's solid runs, in whole LDU (a voxel is solid when its middle is inside).
    let mut columns: BTreeMap<(i64, i64), Vec<(i16, i16)>> = BTreeMap::new();
    let mut ys: Vec<f64> = Vec::new();
    for (&(i, j), ks) in &grid {
        let (x, z) = (x0 + (i as f64 + 0.5) * step, z0 + (j as f64 + 0.5) * step);
        ys.clear();
        ys.extend(ks.iter().filter_map(|&k| crossing(&tris[k], x, z)));
        ys.sort_by(f64::total_cmp);
        ys.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        if ys.len() % 2 == 1 {
            continue;
        }
        let mut runs = Vec::new();
        for pair in ys.chunks(2) {
            let (a, b) = ((pair[0] - 0.5).ceil(), (pair[1] - 0.5).floor());
            if b >= a {
                runs.push((a as i16, (b + 1.0) as i16));
            }
        }
        if !runs.is_empty() {
            columns.insert((i, j), runs);
        }
    }
    merge(&columns, x0, z0, step)
}

/// Columns with the same runs, merged into rectangles (greedy: along z, then along x).
fn merge(columns: &BTreeMap<(i64, i64), Vec<(i16, i16)>>, x0: f64, z0: f64, step: f64) -> Vec<ProxyBox> {
    let mut by_run: BTreeMap<(i16, i16), Vec<(i64, i64)>> = BTreeMap::new();
    for (&(i, j), runs) in columns {
        for &r in runs {
            by_run.entry(r).or_default().push((i, j));
        }
    }
    let mut out = Vec::new();
    for ((y0, y1), cells) in by_run {
        let mut left: std::collections::BTreeSet<(i64, i64)> = cells.into_iter().collect();
        while let Some(&(i, j)) = left.iter().next() {
            // The longest run along z from (i, j), then as many rows along x as have it all.
            let mut j1 = j;
            while left.contains(&(i, j1 + 1)) {
                j1 += 1;
            }
            let mut i1 = i;
            while (j..=j1).all(|jj| left.contains(&(i1 + 1, jj))) {
                i1 += 1;
            }
            for ii in i..=i1 {
                for jj in j..=j1 {
                    left.remove(&(ii, jj));
                }
            }
            let x = |k: i64| (x0 + k as f64 * step).round() as i16;
            let z = |k: i64| (z0 + k as f64 * step).round() as i16;
            out.push([x(i), y0, z(j), x(i1 + 1), y1, z(j1 + 1)]);
        }
    }
    out
}

/// The boxes as bytes for the index (little-endian i16s).
pub fn to_blob(boxes: &[ProxyBox]) -> Vec<u8> {
    boxes.iter().flatten().flat_map(|v| v.to_le_bytes()).collect()
}

pub fn from_blob(blob: &[u8]) -> Vec<ProxyBox> {
    blob.as_chunks::<12>()
        .0
        .iter()
        .map(|c| std::array::from_fn(|k| i16::from_le_bytes([c[2 * k], c[2 * k + 1]])))
        .collect()
}
