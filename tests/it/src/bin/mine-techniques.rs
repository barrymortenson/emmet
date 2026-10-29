// SPDX-License-Identifier: AGPL-3.0-or-later
//! The technique miner (session 3C, plan section 21): which small sub-assemblies recur across official models, found
//! on the validator's connection graph (3A). It ranks ideas for the technique library; techniques are written by hand
//! and aren't derived from the models (D-48, ADR 0003).
//!
//!   cargo run -p emmet-it --bin mine-techniques -- [--query <candidates.json>] <data dir> <out dir> <models>…
//!
//! The data dir needs the full LDraw library and its parts index; models are .mpd or .ldr files or folders of them. A
//! motif is a connected set of 2–12 parts with at least one part or joint beyond plain stacking (a part on its side or
//! upside down, a clip, a hinge, a bar, a part that isn't a plain brick, plate, or tile). Motifs are keyed by shape:
//! each part's id and its place relative to an anchor part, colors ignored. They're grown a part at a time from the
//! motifs found in at least MIN_SETS sets (a set's alternate models count once), and ranked by how many sets have one,
//! then size, then how many times they appear. Writes `<out dir>/motifs.json` (the top TOP) with an example of each of
//! the first EXAMPLES as `<out dir>/examples/<rank>.ldr`, for a render. With --query, a list of candidates, each
//! `{"name", "signature": [part ids]}`, it also writes `matches.json`: per candidate, the widest motif that has every
//! signature part, and its example as `matches/<name>.ldr`. The examples are never committed (D-48).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use emmet_core::data::DataDir;
use emmet_core::index::Index;
use emmet_core::ldraw::geometry::LibFiles;
use emmet_it::corpus::{models_in, placements_of};
use glam::DMat4;
use serde_json::json;

/// A motif counts once it's in this many sets.
const MIN_SETS: usize = 3;
const MAX_PARTS: usize = 12;
/// How many instances of a motif per set grow into the next size: enough to find every larger motif, few enough
/// that a wall of a hundred brackets doesn't swamp the run.
const GROW_PER_SET: usize = 40;
/// How many motifs of each size go on to grow, the most widespread first.
const KEEP_PER_SIZE: usize = 4000;
const EXAMPLES: usize = 400;
/// How many parts a candidate's picture shows at most.
const CROP: usize = 60;
/// How many motifs motifs.json lists.
const TOP: usize = 3000;

/// Plain stacking: parts in these categories, upright, joined by studs straight up, are no technique on their own.
const PLAIN: [&str; 3] = ["Brick", "Plate", "Tile"];
/// Words in a plain category's part name that make it more than a plain part.
const SPECIAL: [&str; 14] = [
    "Jumper",
    "Clip",
    "Handle",
    "Bar",
    "Hinge",
    "Side",
    "Headlight",
    "Rail",
    "Groove",
    "Click",
    "Pin",
    "Hole",
    "Bracket",
    "Lock",
];

struct Model {
    file: String,
    /// The set it's from, an index into the run's sets.
    set: usize,
    /// Placements whose part the library has (current or moved), which a picture may show.
    known: Vec<bool>,
    title: String,
    author: String,
    license: String,
    parts: Vec<String>,
    colors: Vec<u32>,
    matrices: Vec<DMat4>,
    adj: Vec<Vec<usize>>,
    /// Placements that make a motif worth having on their own (see SPECIAL), and joints that do.
    special: Vec<bool>,
    special_edges: HashSet<(usize, usize)>,
}

fn header(text: &str, key: &str) -> String {
    text.lines()
        .find_map(|l| l.trim().strip_prefix(key).map(|v| v.trim().to_string()))
        .unwrap_or_default()
}

fn load(index: &Index, lib: &LibFiles, path: &Path, names: &mut HashMap<String, (String, String)>) -> Model {
    let text = String::from_utf8_lossy(&std::fs::read(path).unwrap()).into_owned();
    let placements = placements_of(path);
    let joints = emmet_core::check::joints(index, lib, &placements).unwrap();
    let n = placements.len();
    let mut keep = vec![false; n];
    let mut special = vec![false; n];
    let mut matrices = Vec::with_capacity(n);
    let mut parts = Vec::with_capacity(n);
    let mut known = vec![false; n];
    for (i, p) in placements.iter().enumerate() {
        let m = DMat4::from_cols_array(&p.matrix);
        matrices.push(m);
        // A moved id counts as the part it moved to, so old and new files share motifs.
        let mut id = p.part.clone();
        if let Some(part) = index.part(&id).unwrap()
            && part.name.starts_with("~Moved to")
            && let Some(to) = part.alias_of
        {
            id = to;
        }
        parts.push(id.clone());
        let (name, category) = names
            .entry(id.clone())
            .or_insert_with(|| match index.part(&id).unwrap() {
                Some(part) => (part.name, part.category),
                None => (String::new(), String::new()),
            })
            .clone();
        known[i] = !name.is_empty();
        // Figures, stickers, and baseplates join everything to everything; unknown and obsolete parts (a `~` name)
        // can't be judged.
        if name.is_empty()
            || name.starts_with('~')
            || category.starts_with("Minifig")
            || ["Sticker", "Baseplate"].contains(&category.as_str())
        {
            continue;
        }
        keep[i] = true;
        // Upright means the part's own up (LDraw −Y) is the world's.
        let upright = (m.y_axis.y - 1.0).abs() < 1e-3;
        let plain = PLAIN.contains(&category.as_str()) && !SPECIAL.iter().any(|w| name.contains(w));
        special[i] = !upright || !plain;
    }
    let mut adj = vec![BTreeSet::new(); n];
    let mut special_edges = HashSet::new();
    for j in &joints {
        if !keep[j.a] || !keep[j.b] || j.a == j.b {
            continue;
        }
        adj[j.a].insert(j.b);
        adj[j.b].insert(j.a);
        let vertical = j.axis.is_some_and(|a| a[1].abs() > 0.99);
        if j.kind != "held" && (j.kind != "cyl" || !vertical) {
            special_edges.insert((j.a.min(j.b), j.a.max(j.b)));
        }
    }
    Model {
        set: 0,
        known,
        file: path.file_name().unwrap().to_string_lossy().into_owned(),
        title: text
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .trim_start_matches('0')
            .trim()
            .to_string(),
        author: header(&text, "0 Author:"),
        license: header(&text, "0 !LICENSE"),
        parts,
        colors: placements.iter().map(|p| p.color).collect(),
        matrices,
        adj: adj.into_iter().map(|s| s.into_iter().collect()).collect(),
        special,
        special_edges,
    }
}

/// A number rounded for a key: positions to 2 LDU (official files are often 1 LDU off), rotations to 0.01.
fn q(v: f64, step: f64) -> i64 {
    (v / step).round() as i64
}

/// The motif's key: the smallest, over anchors with the first part id, of every other part's id and transform in the
/// anchor's frame, sorted.
fn key(m: &Model, set: &[usize]) -> String {
    let first = set.iter().map(|&i| &m.parts[i]).min().unwrap();
    let mut best: Option<String> = None;
    for &a in set.iter().filter(|&&a| &m.parts[a] == first) {
        let inv = m.matrices[a].inverse();
        let mut items: Vec<String> = set
            .iter()
            .filter(|&&i| i != a)
            .map(|&i| {
                let r = inv * m.matrices[i];
                let c = r.to_cols_array();
                let mut s = format!("{}@{},{},{}", m.parts[i], q(c[12], 2.0), q(c[13], 2.0), q(c[14], 2.0));
                for v in [c[0], c[1], c[2], c[4], c[5], c[6], c[8], c[9], c[10]] {
                    let _ = write!(s, ",{}", q(v, 0.01));
                }
                s
            })
            .collect();
        items.sort();
        let k = format!("{first}|{}", items.join(";"));
        if best.as_ref().is_none_or(|b| &k < b) {
            best = Some(k);
        }
    }
    best.unwrap()
}

fn is_special(m: &Model, set: &[usize]) -> bool {
    set.iter().any(|&i| m.special[i])
        || set
            .iter()
            .any(|&a| set.iter().any(|&b| a < b && m.special_edges.contains(&(a, b))))
}

#[derive(Default)]
struct Motif {
    /// Per set (a set's main and alternate models count once), its instances: (model, sorted placement indices).
    by_set: BTreeMap<usize, Vec<(usize, Vec<usize>)>>,
}

impl Motif {
    fn sets(&self) -> usize {
        self.by_set.len()
    }

    fn instances(&self) -> usize {
        self.by_set.values().map(Vec::len).sum()
    }

    /// Wider first (in more sets), then larger (a bigger motif in as many sets says more), then more often.
    fn rank(&self, size: usize) -> (usize, usize, usize) {
        (self.sets(), size, self.instances())
    }
}

fn add(
    motifs: &mut HashMap<String, Motif>,
    seen: &mut HashSet<(usize, Vec<usize>)>,
    models: &[Model],
    mi: usize,
    set: Vec<usize>,
) {
    if !seen.insert((mi, set.clone())) {
        return;
    }
    let k = key(&models[mi], &set);
    motifs
        .entry(k)
        .or_default()
        .by_set
        .entry(models[mi].set)
        .or_default()
        .push((mi, set));
}

/// A motif's example as LDraw lines, for a render (never committed: D-48).
fn example_ldr(models: &[Model], (mi, set): &(usize, Vec<usize>), title: &str) -> String {
    let m = &models[*mi];
    let mut ldr = format!("0 {title} (from {})\n", m.file);
    for &i in set {
        let c = m.matrices[i].to_cols_array();
        let _ = writeln!(
            ldr,
            "1 {} {} {} {} {} {} {} {} {} {} {} {} {} {}.dat",
            m.colors[i], c[12], c[13], c[14], c[0], c[4], c[8], c[1], c[5], c[9], c[2], c[6], c[10], m.parts[i]
        );
    }
    ldr
}

/// The group and every part whose origin is within two studs of it across and a brick above or below, the nearest
/// first, up to CROP parts: the technique in place, for a picture.
fn around(m: &Model, group: &[usize]) -> Vec<usize> {
    let at = |i: usize| m.matrices[i].w_axis.truncate();
    let (mut lo, mut hi) = (at(group[0]), at(group[0]));
    for &i in group {
        lo = lo.min(at(i));
        hi = hi.max(at(i));
    }
    let pad = glam::DVec3::new(40.0, 24.0, 40.0);
    let (lo, hi) = (lo - pad, hi + pad);
    let mid = (lo + hi) / 2.0;
    let mut rest: Vec<usize> = (0..m.parts.len())
        .filter(|&i| m.known[i] && !group.contains(&i) && at(i).cmpge(lo).all() && at(i).cmple(hi).all())
        .collect();
    rest.sort_by(|&a, &b| at(a).distance(mid).total_cmp(&at(b).distance(mid)));
    let mut out = group.to_vec();
    out.extend(rest.into_iter().take(CROP.saturating_sub(group.len())));
    out
}

fn row(models: &[Model], names: &HashMap<String, (String, String)>, size: usize, motif: &Motif) -> serde_json::Value {
    let (_, example) = motif.by_set.values().max_by_key(|s| s.len()).unwrap().first().unwrap();
    let m = &models[motif.by_set.values().max_by_key(|s| s.len()).unwrap()[0].0];
    let mut parts: BTreeMap<&str, usize> = BTreeMap::new();
    for &i in example {
        *parts.entry(&m.parts[i]).or_default() += 1;
    }
    json!({
        "size": size,
        "sets": motif.sets(),
        "instances": motif.instances(),
        "parts": parts.iter().map(|(id, n)| json!({
            "id": id, "count": n, "name": names.get(*id).map(|(n, _)| n.as_str()).unwrap_or(""),
        })).collect::<Vec<_>>(),
        "found_in": motif.by_set.values().map(|s| json!({
            "file": models[s[0].0].file, "count": s.len(),
        })).collect::<Vec<_>>(),
    })
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // --query <file>: [{"name", "signature": [part ids]}]: for each, the widest motif with every signature part.
    let query = args.iter().position(|a| a == "--query").map(|i| {
        let q = args.remove(i + 1);
        args.remove(i);
        q
    });
    if args.len() < 3 {
        eprintln!("usage: mine-techniques [--query <candidates.json>] <data dir> <out dir> <model files or dirs>…");
        std::process::exit(2);
    }
    let data = DataDir::at(Path::new(&args[0]));
    let out = PathBuf::from(&args[1]);
    let index = Index::ensure(&data).unwrap();
    let lib = LibFiles::probe(&data.ldraw());
    let mut files = Vec::new();
    for a in &args[2..] {
        let p = PathBuf::from(a);
        if p.is_dir() {
            files.extend(models_in(&p));
        } else {
            files.push(p);
        }
    }
    let mut names = HashMap::new();
    let mut set_ids: Vec<String> = Vec::new();
    let models: Vec<Model> = files
        .iter()
        .map(|f| {
            eprintln!("mine-techniques: reading {}", f.display());
            let mut m = load(&index, &lib, f, &mut names);
            // 10001-1_B-Model.mpd and 10001-1.mpd are one set.
            let set = m
                .file
                .trim_end_matches(".mpd")
                .trim_end_matches(".ldr")
                .split('_')
                .next()
                .unwrap()
                .to_string();
            m.set = set_ids.iter().position(|s| *s == set).unwrap_or_else(|| {
                set_ids.push(set);
                set_ids.len() - 1
            });
            m
        })
        .collect();

    // Size 2: every joint that's more than plain stacking.
    let mut level: HashMap<String, Motif> = HashMap::new();
    let mut seen = HashSet::new();
    for (mi, m) in models.iter().enumerate() {
        for a in 0..m.parts.len() {
            for &b in &m.adj[a] {
                if a < b && is_special(m, &[a, b]) {
                    add(&mut level, &mut seen, &models, mi, vec![a, b]);
                }
            }
        }
    }
    let mut all: Vec<(usize, Motif)> = Vec::new();
    for size in 2..=MAX_PARTS {
        let mut ranked: Vec<Motif> = std::mem::take(&mut level)
            .into_values()
            .filter(|m| m.sets() >= MIN_SETS)
            .collect();
        ranked.sort_by_key(|m| std::cmp::Reverse(m.rank(size)));
        eprintln!(
            "mine-techniques: {size} parts: {} motifs in {MIN_SETS}+ sets",
            ranked.len()
        );
        if ranked.is_empty() {
            break;
        }
        if size < MAX_PARTS {
            let mut seen = HashSet::new();
            for motif in ranked.iter().take(KEEP_PER_SIZE) {
                for found in motif.by_set.values() {
                    for (mi, set) in found.iter().take(GROW_PER_SET) {
                        let m = &models[*mi];
                        for &i in set {
                            for &j in &m.adj[i] {
                                if set.contains(&j) {
                                    continue;
                                }
                                let mut grown = set.clone();
                                grown.push(j);
                                grown.sort_unstable();
                                add(&mut level, &mut seen, &models, *mi, grown);
                            }
                        }
                    }
                }
            }
        }
        all.extend(ranked.into_iter().map(|m| (size, m)));
    }
    all.sort_by_key(|(size, m)| std::cmp::Reverse(m.rank(*size)));

    std::fs::create_dir_all(out.join("examples")).unwrap();
    let mut rows = Vec::new();
    for (rank, (size, motif)) in all.iter().take(TOP).enumerate() {
        let mut r = row(&models, &names, *size, motif);
        r["rank"] = json!(rank + 1);
        if rank < EXAMPLES {
            let name = format!("{:04}.ldr", rank + 1);
            let example = &motif.by_set.values().max_by_key(|s| s.len()).unwrap()[0];
            std::fs::write(
                out.join("examples").join(&name),
                example_ldr(&models, example, &format!("Motif {}", rank + 1)),
            )
            .unwrap();
            r["example"] = json!(name);
        }
        rows.push(r);
    }
    let doc = json!({
        "models": models.iter().map(|m| json!({
            "file": m.file, "set": set_ids[m.set], "title": m.title, "author": m.author, "license": m.license,
            "parts": m.parts.len(),
        })).collect::<Vec<_>>(),
        "sets": set_ids.len(),
        "motifs": rows,
    });
    std::fs::write(out.join("motifs.json"), serde_json::to_string_pretty(&doc).unwrap()).unwrap();
    eprintln!(
        "mine-techniques: {} motifs, the top {TOP} in {}",
        all.len(),
        out.join("motifs.json").display()
    );

    let Some(query) = query else { return };
    let wanted: Vec<serde_json::Value> = serde_json::from_slice(&std::fs::read(&query).unwrap()).unwrap();
    std::fs::create_dir_all(out.join("matches")).unwrap();
    let mut matches = serde_json::Map::new();
    for w in &wanted {
        let name = w["name"].as_str().unwrap();
        let signature: Vec<String> = w["signature"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let with: Vec<&(usize, Motif)> = all
            .iter()
            .filter(|(_, motif)| {
                let (mi, set) = &motif.by_set.values().next().unwrap()[0];
                signature
                    .iter()
                    .all(|id| set.iter().any(|&i| &models[*mi].parts[i] == id))
            })
            .collect();
        // How widely: the widest motif with the signature (the smallest, usually). What it looks like in a build: the
        // largest one still in a quarter as many sets.
        let best = with.iter().max_by_key(|(size, m)| m.rank(*size));
        let mut value = json!({});
        // The picture: one joined group, from the set that has the most, and the parts around it.
        let mut firsts: BTreeMap<usize, (usize, Vec<usize>)> = BTreeMap::new();
        if let Some((size, motif)) = best {
            value["motif"] = row(&models, &names, *size, motif);
            let floor = MIN_SETS.max(motif.sets().div_ceil(4));
            let (big_size, big) = with
                .iter()
                .filter(|(_, m)| m.sets() >= floor)
                .max_by_key(|(size, m)| (*size, m.sets(), m.instances()))
                .unwrap();
            value["context"] = row(&models, &names, *big_size, big);
        }
        // However each set shapes it: the sets with the signature parts joined to each other (a group of them joined
        // directly or through one part), and the sets that use them all at all. No minimum, so a rare one counts.
        let (mut used, mut joined, mut times) = (BTreeSet::new(), BTreeMap::new(), 0);
        for (mi, m) in models.iter().enumerate() {
            let nodes: Vec<usize> = (0..m.parts.len())
                .filter(|&i| signature.contains(&m.parts[i]) && !m.adj[i].is_empty())
                .collect();
            if !signature.iter().all(|id| nodes.iter().any(|&i| &m.parts[i] == id)) {
                continue;
            }
            used.insert(m.set);
            let mut done = HashSet::new();
            for &start in &nodes {
                if !done.insert(start) {
                    continue;
                }
                let mut group = vec![start];
                let mut k = 0;
                // Joined directly or through one part between (two tiles on one plate).
                while k < group.len() {
                    for &j in &m.adj[group[k]] {
                        for &h in std::iter::once(&j).chain(&m.adj[j]) {
                            if signature.contains(&m.parts[h]) && done.insert(h) {
                                group.push(h);
                            }
                        }
                    }
                    k += 1;
                }
                if signature.iter().all(|id| group.iter().any(|&i| &m.parts[i] == id)) {
                    times += 1;
                    *joined.entry(m.set).or_insert(0) += 1;
                    firsts.entry(m.set).or_insert((mi, group));
                }
            }
        }
        value["used_sets"] = json!(used.len());
        value["joined_sets"] = json!(joined.len());
        value["joined"] = json!(times);
        value["found_in"] = json!(
            joined
                .keys()
                .map(|&s| models.iter().find(|m| m.set == s).unwrap().file.clone())
                .collect::<Vec<_>>()
        );
        let example = joined
            .iter()
            .max_by_key(|(_, n)| **n)
            .map(|(s, _)| firsts.remove(s).unwrap());
        if let Some((mi, group)) = example {
            let crop = around(&models[mi], &group);
            std::fs::write(
                out.join("matches").join(format!("{name}.ldr")),
                example_ldr(&models, &(mi, crop), name),
            )
            .unwrap();
            value["example"] = json!(format!("{name}.ldr"));
            // The example's first lines are the group, for a render that highlights it.
            value["highlight"] = json!(group.len());
        }
        matches.insert(name.to_string(), value);
    }
    std::fs::write(
        out.join("matches.json"),
        serde_json::to_string_pretty(&matches).unwrap(),
    )
    .unwrap();
    eprintln!(
        "mine-techniques: {} queries in {}",
        wanted.len(),
        out.join("matches.json").display()
    );
}
