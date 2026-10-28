// SPDX-License-Identifier: AGPL-3.0-or-later
//! The parts data pack (plan section 12, ADR 0004): Rebrickable's catalog cut down to a derived SQLite file with a
//! signed manifest. [`build`] makes one from a catalog download and checks it before writing; [`install`] and the
//! bundled pack put one in the data dir with an atomic swap and keep the previous one for [`rollback`]; [`PartsData`]
//! answers the engine's questions: the LDraw → Rebrickable and BrickLink ids, color names, and whether a part was ever
//! made in a color. Unknown is never unavailable: an unmapped part or color, or no pack at all, reads `unknown`.

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::data::DataDir;
use crate::envelope::{Error, ErrorCode, Result};
use crate::fsx;
use crate::index::Index;

/// The credit Rebrickable asks for, wherever its data appears (plan section 12).
pub const CREDIT: &str = "Part, color, and set data from Rebrickable (rebrickable.com)";

/// Bumped when the pack's tables change; a pack of another schema isn't read.
pub const SCHEMA: u32 = 1;

/// The manifest's file name, next to the SQLite file; its signature is `parts-data.json.sig`.
pub const MANIFEST: &str = "parts-data.json";

/// Where tests and dev point at a bundled pack (a folder with `parts-data.json` and the SQLite file it names).
pub const BUNDLE_ENV: &str = "EMMET_BUNDLED_PARTS_DATA";

/// The public key a signed pack's manifest must be signed with. Until session 6A makes the real key there's none in a
/// release build, so a release refuses every signed pack (fail closed: the test key's private half is in the repo).
/// Debug builds, which the tests run, trust the throwaway test key in `tests/fixtures/parts-data/`.
pub const TRUSTED_KEY: Option<&[u8]> = if cfg!(debug_assertions) {
    Some(include_bytes!(
        "../../../tests/fixtures/parts-data/test-signing-key.pub"
    ))
} else {
    None
};

const DDL: &str = "
CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE colors(id INTEGER PRIMARY KEY, name TEXT NOT NULL, rgb TEXT NOT NULL, is_trans INTEGER NOT NULL,
  year_from INTEGER, year_to INTEGER);
CREATE TABLE color_mappings(color_id INTEGER NOT NULL, system TEXT NOT NULL, ext_id TEXT NOT NULL, names TEXT NOT NULL,
  is_export INTEGER NOT NULL, is_import INTEGER NOT NULL);
CREATE TABLE part_categories(id INTEGER PRIMARY KEY, name TEXT NOT NULL);
CREATE TABLE parts(part_num TEXT PRIMARY KEY COLLATE NOCASE, name TEXT NOT NULL, category_id INTEGER, material TEXT);
CREATE TABLE part_mappings(part_num TEXT NOT NULL, system TEXT NOT NULL, ext_id TEXT NOT NULL COLLATE NOCASE,
  is_export INTEGER NOT NULL, is_import INTEGER NOT NULL);
CREATE TABLE part_relationships(rel_type TEXT NOT NULL, child TEXT NOT NULL, parent TEXT NOT NULL);
CREATE TABLE elements(element_id TEXT NOT NULL, part_num TEXT NOT NULL, color_id INTEGER NOT NULL);
CREATE TABLE set_parts(part_num TEXT NOT NULL, color_id INTEGER NOT NULL, set_count INTEGER NOT NULL,
  year_from INTEGER, year_to INTEGER, PRIMARY KEY(part_num, color_id)) WITHOUT ROWID;
CREATE INDEX color_mappings_ext ON color_mappings(system, ext_id);
CREATE INDEX color_mappings_color ON color_mappings(color_id, system);
CREATE INDEX part_mappings_ext ON part_mappings(system, ext_id);
CREATE INDEX part_mappings_part ON part_mappings(part_num, system);
CREATE INDEX part_relationships_child ON part_relationships(child);
CREATE INDEX part_relationships_parent ON part_relationships(parent);
CREATE INDEX elements_part ON elements(part_num, color_id);
";

/// `parts-data.json`: what a pack holds, and the sha256 its SQLite file must have.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Manifest {
    pub schema: u32,
    /// The pack's version: the date it was built, `YYYY-MM-DD`. A later date is a newer pack.
    pub version: String,
    pub date: String,
    /// The SQLite file's name, in the same folder.
    pub file: String,
    pub sha256: String,
    pub size: u64,
    /// Rows per table.
    pub rows: BTreeMap<String, u64>,
    pub credit: String,
}

/// Which of ADR 0004's four rules mapped an LDraw part to Rebrickable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MappingRule {
    /// 1. An LD row names the LDraw id.
    LdRow,
    /// 2. An LD row names an old id that LDraw moved to this one.
    MovedLdRow,
    /// 3. Rebrickable has a part with the same id.
    SameId,
    /// 4. The base of a formed variant (`75541-f1` → `75541`) maps by rules 1–3.
    FormedVariant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PartMapping {
    pub rebrickable_id: String,
    pub rule: MappingRule,
}

/// Whether a part was ever made in a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Rebrickable has an element or a set with the part in that color.
    Available,
    /// The part was made, but never in that color.
    ColorUnavailable,
    /// Rebrickable knows the part but has no element or set with it in any color.
    PartUnavailable,
    /// No pack, or the part or color has no Rebrickable mapping. Never read as unavailable.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Availability {
    pub status: Status,
    /// Sets with the part in that color (Rebrickable's inventories); null when unknown.
    pub sets: Option<u32>,
    /// The first and last year of those sets; null when there are none.
    pub years: Option<[u16; 2]>,
}

impl Availability {
    pub fn unknown() -> Self {
        Self {
            status: Status::Unknown,
            sets: None,
            years: None,
        }
    }
}

/// An LDraw color as Rebrickable and BrickLink know it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ColorIds {
    pub rebrickable_id: i64,
    pub rebrickable_name: String,
    pub bricklink_id: Option<u32>,
    pub bricklink_name: Option<String>,
}

// ---------------------------------------------------------------------------------------------------------------
// The mapping rules, shared by the build's coverage check and the installed pack.

/// What the mapping rules ask of a catalog.
trait Catalog {
    /// The Rebrickable part an LD row names for `ldraw` (lowercase), the `is_import_id` row first.
    fn ld_row(&self, ldraw: &str) -> Result<Option<String>>;
    /// Rebrickable's spelling of `id` when it has that part.
    fn part(&self, id: &str) -> Result<Option<String>>;
}

/// ADR 0004's four rules, in order. `moved_from` gives the old LDraw ids whose `~Moved to` file names `id`.
fn map_part(c: &dyn Catalog, moved_from: &dyn Fn(&str) -> Vec<String>, id: &str) -> Result<Option<PartMapping>> {
    let id = crate::index::normalize_id(id);
    let found = |p: String, rule| {
        Some(PartMapping {
            rebrickable_id: p,
            rule,
        })
    };
    if let Some(p) = c.ld_row(&id)? {
        return Ok(found(p, MappingRule::LdRow));
    }
    for old in moved_from(&id) {
        if let Some(p) = c.ld_row(&old)? {
            return Ok(found(p, MappingRule::MovedLdRow));
        }
    }
    if let Some(p) = c.part(&id)? {
        return Ok(found(p, MappingRule::SameId));
    }
    if let Some(base) = formed_base(&id)
        && let Some(m) = map_part(c, moved_from, base)?
    {
        return Ok(found(m.rebrickable_id, MappingRule::FormedVariant));
    }
    Ok(None)
}

/// `75541-f1` → `75541`.
fn formed_base(id: &str) -> Option<&str> {
    let (base, n) = id.rsplit_once("-f")?;
    (!base.is_empty() && !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())).then_some(base)
}

// ---------------------------------------------------------------------------------------------------------------
// Building a pack from a catalog download.

/// What [`build`] needs besides the catalog.
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    /// The pack's date and version, `YYYY-MM-DD` (default: today, UTC).
    pub date: Option<String>,
    /// A previous pack's manifest: every table's row count must be within 10 % of it.
    pub previous: Option<PathBuf>,
    /// `tests/fixtures/ldraw-mini`: every real part in its PARTS.txt must map.
    pub ldraw_mini: Option<PathBuf>,
    /// A PKCS#8 Ed25519 private key: writes `parts-data.json.sig`.
    pub key: Option<Vec<u8>>,
}

/// The columns each catalog file must have (the schema check).
const COLUMNS: &[(&str, &[&str])] = &[
    ("colors", &["id", "name", "rgb", "is_trans", "y1", "y2"]),
    (
        "color_mappings",
        &[
            "id",
            "ext_system",
            "ext_id",
            "ext_descr",
            "is_export_id",
            "is_import_id",
        ],
    ),
    ("part_categories", &["id", "name"]),
    ("parts", &["part_num", "name", "part_cat_id", "part_material"]),
    (
        "part_mappings",
        &["part_num", "ext_system", "ext_id", "is_export_id", "is_import_id"],
    ),
    ("part_relationships", &["rel_type", "child_part_num", "parent_part_num"]),
    ("elements", &["element_id", "part_num", "color_id"]),
    ("sets", &["set_num", "year"]),
    ("inventories", &["id", "set_num"]),
    ("inventory_parts", &["inventory_id", "part_num", "color_id"]),
];

/// A failed pre-publish check: names the check, and nothing was written.
fn check_failed(check: &str, what: impl std::fmt::Display) -> Error {
    Error::new(
        ErrorCode::ParamInvalid,
        format!("The parts data pack failed its {check} check: {what}. Nothing was written."),
        "Check the catalog download (and the previous manifest), fix what the check names, and build again.",
    )
}

/// One catalog file, read whole, with its header.
struct Table {
    name: &'static str,
    text: String,
    cols: Vec<String>,
}

impl Table {
    fn read(dir: &Path, name: &'static str, required: &[&str]) -> Result<Self> {
        let path = dir.join(format!("{name}.csv"));
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let gz = if dir.join(format!("{name}.csv.gz")).is_file() {
                    " (there's a .csv.gz: scripts/build-parts-data unpacks it first)"
                } else {
                    ""
                };
                return Err(check_failed("schema", format!("{name}.csv is missing{gz}")));
            }
            Err(e) => return Err(Error::io(&path, e)),
        };
        let text = text.strip_prefix('\u{feff}').map(String::from).unwrap_or(text);
        let cols: Vec<String> = Rows::new(&text)
            .next()
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.trim().to_string())
            .collect();
        for c in required {
            if !cols.iter().any(|h| h == c) {
                return Err(check_failed("schema", format!("{name}.csv has no {c} column")));
            }
        }
        Ok(Self { name, text, cols })
    }

    fn col(&self, name: &str) -> usize {
        self.cols
            .iter()
            .position(|c| c == name)
            .expect("required columns are checked on read")
    }

    /// Calls `f` with each row after the header. A row with fewer fields than the header is a schema failure.
    fn each(&self, mut f: impl FnMut(&[std::borrow::Cow<str>]) -> Result<()>) -> Result<()> {
        for (i, row) in Rows::new(&self.text).enumerate().skip(1) {
            if row.len() == 1 && row[0].is_empty() {
                continue;
            }
            if row.len() < self.cols.len() {
                return Err(check_failed(
                    "schema",
                    format!(
                        "{}.csv row {} has {} fields, not {}",
                        self.name,
                        i + 1,
                        row.len(),
                        self.cols.len()
                    ),
                ));
            }
            f(&row)?;
        }
        Ok(())
    }
}

/// RFC 4180 rows: comma-separated, `"…"` quoting with `""` for a quote, LF or CRLF line ends.
struct Rows<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> Rows<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }
}

impl<'a> Iterator for Rows<'a> {
    type Item = Vec<std::borrow::Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::borrow::Cow;
        let bytes = self.text.as_bytes();
        if self.pos >= bytes.len() {
            return None;
        }
        let mut fields = Vec::new();
        let mut i = self.pos;
        loop {
            if i < bytes.len() && bytes[i] == b'"' {
                let mut out = String::new();
                let mut start = i + 1;
                i += 1;
                loop {
                    match bytes.get(i) {
                        None => {
                            out.push_str(&self.text[start..i]);
                            break;
                        }
                        Some(b'"') if bytes.get(i + 1) == Some(&b'"') => {
                            out.push_str(&self.text[start..=i]);
                            i += 2;
                            start = i;
                        }
                        Some(b'"') => {
                            out.push_str(&self.text[start..i]);
                            i += 1;
                            break;
                        }
                        Some(_) => i += 1,
                    }
                }
                fields.push(Cow::Owned(out));
            } else {
                let start = i;
                while i < bytes.len() && !matches!(bytes[i], b',' | b'\n' | b'\r') {
                    i += 1;
                }
                fields.push(Cow::Borrowed(&self.text[start..i]));
            }
            match bytes.get(i) {
                Some(b',') => i += 1,
                Some(b'\r') if bytes.get(i + 1) == Some(&b'\n') => {
                    i += 2;
                    break;
                }
                Some(b'\n' | b'\r') => {
                    i += 1;
                    break;
                }
                // A stray character after a closing quote joins the field's line; skip to the next separator.
                Some(_) => {
                    while i < bytes.len() && !matches!(bytes[i], b',' | b'\n') {
                        i += 1;
                    }
                }
                None => break,
            }
        }
        self.pos = i;
        Some(fields)
    }
}

fn truthy(s: &str) -> bool {
    matches!(s.trim().to_ascii_lowercase().as_str(), "true" | "t" | "1")
}

fn int(table: &str, s: &str) -> Result<i64> {
    s.trim()
        .parse()
        .map_err(|_| check_failed("schema", format!("{table}.csv has '{s}' where a number belongs")))
}

fn opt_int(s: &str) -> Option<i64> {
    s.trim().parse().ok()
}

/// `["Light Bluish Gray", "Light_Bluish_Gray"]` (the JSON list in `ext_descr`), or the text itself.
fn names(descr: &str) -> Vec<String> {
    match serde_json::from_str::<Vec<String>>(descr) {
        Ok(list) => list,
        Err(_) if descr.trim().is_empty() => vec![],
        Err(_) => vec![descr.trim().to_string()],
    }
}

struct ColorRow {
    id: i64,
    name: String,
    rgb: String,
    is_trans: bool,
    y1: Option<i64>,
    y2: Option<i64>,
}

struct MapRow {
    rebrickable: String,
    system: String,
    ext_id: String,
    names: Vec<String>,
    export: bool,
    import: bool,
}

/// A Rebrickable part and color.
type PartColor = (String, i64);
/// How many sets have a part × color, and their first and last year.
type SetCount = (u32, Option<i64>, Option<i64>);
/// New LDraw id → the old ids moved to it.
type Moves = HashMap<String, Vec<String>>;

/// Every table parsed, before anything is written.
struct Catalogue {
    colors: Vec<ColorRow>,
    color_mappings: Vec<MapRow>,
    categories: Vec<(i64, String)>,
    parts: Vec<(String, String, Option<i64>, String)>,
    part_mappings: Vec<MapRow>,
    relationships: Vec<(String, String, String)>,
    elements: Vec<(String, String, i64)>,
    /// (part, color) → (sets, first year, last year), sorted.
    set_parts: Vec<(PartColor, SetCount)>,
}

impl Catalogue {
    fn read(dir: &Path) -> Result<Self> {
        let mut tables: HashMap<&str, Table> = HashMap::new();
        for (name, cols) in COLUMNS {
            tables.insert(name, Table::read(dir, name, cols)?);
        }
        let t = |n: &str| &tables[n];

        let mut colors = Vec::new();
        let c = t("colors");
        let (id, name, rgb, trans, y1, y2) = (
            c.col("id"),
            c.col("name"),
            c.col("rgb"),
            c.col("is_trans"),
            c.col("y1"),
            c.col("y2"),
        );
        c.each(|r| {
            colors.push(ColorRow {
                id: int("colors", &r[id])?,
                name: r[name].to_string(),
                rgb: r[rgb].to_string(),
                is_trans: truthy(&r[trans]),
                y1: opt_int(&r[y1]),
                y2: opt_int(&r[y2]),
            });
            Ok(())
        })?;

        let mapping = |name: &str, key: &str| -> Result<Vec<MapRow>> {
            let m = t(name);
            let (k, sys, ext, exp, imp) = (
                m.col(key),
                m.col("ext_system"),
                m.col("ext_id"),
                m.col("is_export_id"),
                m.col("is_import_id"),
            );
            let descr = m.cols.iter().position(|c| c == "ext_descr");
            let mut out = Vec::new();
            m.each(|r| {
                // Only LDraw and BrickLink rows (plan section 12): BrickOwl and LEGO rows are left out.
                if r[sys] == "LD" || r[sys] == "BL" {
                    out.push(MapRow {
                        rebrickable: r[k].to_string(),
                        system: r[sys].to_string(),
                        ext_id: r[ext].to_string(),
                        names: descr.map(|d| names(&r[d])).unwrap_or_default(),
                        export: truthy(&r[exp]),
                        import: truthy(&r[imp]),
                    });
                }
                Ok(())
            })?;
            Ok(out)
        };
        let color_mappings = mapping("color_mappings", "id")?;
        for m in &color_mappings {
            int("color_mappings", &m.rebrickable)?;
        }
        let part_mappings = mapping("part_mappings", "part_num")?;

        let mut categories = Vec::new();
        let c = t("part_categories");
        let (id, name) = (c.col("id"), c.col("name"));
        c.each(|r| {
            categories.push((int("part_categories", &r[id])?, r[name].to_string()));
            Ok(())
        })?;

        let mut parts = Vec::new();
        let p = t("parts");
        let (num, name, cat, mat) = (
            p.col("part_num"),
            p.col("name"),
            p.col("part_cat_id"),
            p.col("part_material"),
        );
        p.each(|r| {
            parts.push((
                r[num].to_string(),
                r[name].to_string(),
                opt_int(&r[cat]),
                r[mat].to_string(),
            ));
            Ok(())
        })?;

        let mut relationships = Vec::new();
        let p = t("part_relationships");
        let (rel, child, parent) = (p.col("rel_type"), p.col("child_part_num"), p.col("parent_part_num"));
        p.each(|r| {
            relationships.push((r[rel].to_string(), r[child].to_string(), r[parent].to_string()));
            Ok(())
        })?;

        let mut elements = Vec::new();
        let e = t("elements");
        let (eid, part, color) = (e.col("element_id"), e.col("part_num"), e.col("color_id"));
        e.each(|r| {
            elements.push((r[eid].to_string(), r[part].to_string(), int("elements", &r[color])?));
            Ok(())
        })?;

        // inventory_parts + inventories + sets → one row per part × color: how many sets, and their years. Minifig
        // inventories (not in sets.csv) still count as made, with no set or year.
        let mut years: HashMap<String, Option<i64>> = HashMap::new();
        let s = t("sets");
        let (num, year) = (s.col("set_num"), s.col("year"));
        s.each(|r| {
            years.insert(r[num].to_string(), opt_int(&r[year]));
            Ok(())
        })?;
        let mut inventories: HashMap<String, String> = HashMap::new();
        let inv = t("inventories");
        let (id, num) = (inv.col("id"), inv.col("set_num"));
        inv.each(|r| {
            inventories.insert(r[id].to_string(), r[num].to_string());
            Ok(())
        })?;
        #[derive(Default)]
        struct Agg {
            sets: HashSet<String>,
            from: Option<i64>,
            to: Option<i64>,
        }
        let mut agg: HashMap<(String, i64), Agg> = HashMap::new();
        let ip = t("inventory_parts");
        let (iid, part, color) = (ip.col("inventory_id"), ip.col("part_num"), ip.col("color_id"));
        ip.each(|r| {
            let Some(set) = inventories.get(r[iid].as_ref()) else {
                return Ok(());
            };
            let a = agg
                .entry((r[part].to_string(), int("inventory_parts", &r[color])?))
                .or_default();
            if let Some(year) = years.get(set) {
                a.sets.insert(set.clone());
                if let Some(y) = *year {
                    a.from = Some(a.from.map_or(y, |f| f.min(y)));
                    a.to = Some(a.to.map_or(y, |t| t.max(y)));
                }
            }
            Ok(())
        })?;
        let mut set_parts: Vec<_> = agg
            .into_iter()
            .map(|(k, a)| (k, (a.sets.len() as u32, a.from, a.to)))
            .collect();
        set_parts.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(Self {
            colors,
            color_mappings,
            categories,
            parts,
            part_mappings,
            relationships,
            elements,
            set_parts,
        })
    }

    fn rows(&self) -> BTreeMap<String, u64> {
        [
            ("colors", self.colors.len()),
            ("color_mappings", self.color_mappings.len()),
            ("part_categories", self.categories.len()),
            ("parts", self.parts.len()),
            ("part_mappings", self.part_mappings.len()),
            ("part_relationships", self.relationships.len()),
            ("elements", self.elements.len()),
            ("set_parts", self.set_parts.len()),
        ]
        .into_iter()
        .map(|(k, n)| (k.to_string(), n as u64))
        .collect()
    }
}

/// The in-memory catalog the coverage check maps through.
struct Memory {
    ld: HashMap<String, (String, bool)>,
    parts: HashMap<String, String>,
}

impl Catalog for Memory {
    fn ld_row(&self, ldraw: &str) -> Result<Option<String>> {
        Ok(self.ld.get(ldraw).map(|(p, _)| p.clone()))
    }
    fn part(&self, id: &str) -> Result<Option<String>> {
        Ok(self.parts.get(id).cloned())
    }
}

/// ldraw-mini parts Rebrickable has no part for, which the mapping check lets pass (their availability reads unknown):
/// 3A's OMR fixtures brought them, a wheel assembly with tires and a sticker, the kinds ADR 0004 found unmapped.
const KNOWN_UNMAPPED: &[&str] = &["3137c05", "4586822a"];

/// ldraw-mini's real parts (PARTS.txt, without `~`, `=`, and `_` titles) and its `~Moved to` files as new → old ids.
fn mini_parts(dir: &Path) -> Result<(Vec<String>, Moves)> {
    let title = |id: &str| -> Option<String> {
        let bytes = std::fs::read(dir.join("parts").join(format!("{id}.dat"))).ok()?;
        let text = String::from_utf8_lossy(&bytes);
        let first = text.lines().next()?.trim();
        Some(first.strip_prefix('0').unwrap_or(first).trim().to_string())
    };
    let list = dir.join("PARTS.txt");
    let text = std::fs::read_to_string(&list).map_err(|e| Error::io(&list, e))?;
    let mut real = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("");
        for id in line.split_whitespace() {
            let id = id.to_lowercase();
            match title(&id) {
                Some(t) if t.starts_with(['~', '=', '_']) => {}
                Some(_) => real.push(id),
                None => {
                    return Err(check_failed(
                        "ldraw-mini",
                        format!("PARTS.txt lists {id}, which has no file"),
                    ));
                }
            }
        }
    }
    let mut moves = Moves::new();
    let parts = dir.join("parts");
    for entry in std::fs::read_dir(&parts).map_err(|e| Error::io(&parts, e))?.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        let Some(id) = name.strip_suffix(".dat") else { continue };
        if let Some(to) = title(id).as_deref().and_then(|t| t.strip_prefix("~Moved to ")) {
            moves
                .entry(crate::index::normalize_id(to))
                .or_default()
                .push(id.to_string());
        }
    }
    for olds in moves.values_mut() {
        olds.sort();
    }
    Ok((real, moves))
}

/// The pre-publish checks (plan section 12): row counts against the previous pack, the ldraw-mini parts' mapping,
/// and the color table's round trip. The schema check runs as the files are read.
fn check(cat: &Catalogue, rows: &BTreeMap<String, u64>, opts: &BuildOptions) -> Result<()> {
    if let Some(prev) = &opts.previous {
        let text = std::fs::read_to_string(prev).map_err(|e| Error::io(prev, e))?;
        let prev: Manifest = serde_json::from_str(&text)
            .map_err(|e| check_failed("row count", format!("the previous manifest doesn't parse: {e}")))?;
        for (table, &was) in &prev.rows {
            let now = rows.get(table).copied().unwrap_or(0);
            let off = (now as f64 - was as f64).abs();
            if off > was as f64 * 0.10 {
                return Err(check_failed(
                    "row count",
                    format!("{table} has {now} rows, more than 10 % off the previous pack's {was}"),
                ));
            }
        }
    }

    if let Some(mini) = &opts.ldraw_mini {
        let mut mem = Memory {
            ld: HashMap::new(),
            parts: cat.parts.iter().map(|p| (p.0.to_lowercase(), p.0.clone())).collect(),
        };
        for m in cat.part_mappings.iter().filter(|m| m.system == "LD") {
            let key = m.ext_id.to_lowercase();
            if !mem.ld.get(&key).is_some_and(|(_, import)| *import) {
                mem.ld.insert(key, (m.rebrickable.clone(), m.import));
            }
        }
        let (real, moves) = mini_parts(mini)?;
        let moved_from = |id: &str| moves.get(id).cloned().unwrap_or_default();
        let mut unmapped = Vec::new();
        for id in &real {
            if map_part(&mem, &moved_from, id)?.is_none() && !KNOWN_UNMAPPED.contains(&id.as_str()) {
                unmapped.push(id.clone());
            }
        }
        if !unmapped.is_empty() {
            return Err(check_failed(
                "ldraw-mini mapping",
                format!(
                    "{} of its {} real parts don't map: {}",
                    unmapped.len(),
                    real.len(),
                    unmapped.join(", ")
                ),
            ));
        }
    }

    // LDraw → Rebrickable takes the best LD row (import and export, then import, then any; the first of equals), as
    // [`PartsData::color`] does; Rebrickable → LDraw the export one. Every LDraw code some color exports must come
    // back to itself. An import-only code (256 Rubber_Black → 0 Black) needn't: it isn't any color's LDraw name.
    let mut to_rb: HashMap<&str, (i64, (bool, bool))> = HashMap::new();
    let mut to_ld: HashMap<i64, &str> = HashMap::new();
    for m in cat.color_mappings.iter().filter(|m| m.system == "LD") {
        let rb = int("color_mappings", &m.rebrickable)?;
        let rank = (m.import, m.export);
        if to_rb.get(m.ext_id.as_str()).is_none_or(|(_, best)| rank > *best) {
            to_rb.insert(&m.ext_id, (rb, rank));
        }
        if m.export {
            to_ld.entry(rb).or_insert(&m.ext_id);
        }
    }
    if to_ld.is_empty() {
        return Err(check_failed(
            "color round trip",
            "color_mappings.csv has no LDraw export rows",
        ));
    }
    let exported: BTreeSet<&str> = to_ld.values().copied().collect();
    let mut broken: Vec<String> = Vec::new();
    for ld in exported {
        let rb = to_rb[ld].0;
        let back = to_ld.get(&rb).copied().unwrap_or("nothing");
        if back != ld {
            broken.push(format!("LDraw {ld} → Rebrickable {rb} → LDraw {back}"));
        }
    }
    broken.sort();
    if !broken.is_empty() {
        return Err(check_failed("color round trip", broken.join("; ")));
    }
    Ok(())
}

/// Builds a pack from the Rebrickable catalog in `catalog` (the `.csv` files) into `out`: the SQLite file,
/// `parts-data.json`, and with a key, its signature. Every check runs first; a failed one writes nothing.
pub fn build(catalog: &Path, out: &Path, opts: &BuildOptions) -> Result<Manifest> {
    let date = match &opts.date {
        Some(d) => {
            if !is_date(d) {
                return Err(Error::new(
                    ErrorCode::ParamInvalid,
                    format!("'{d}' isn't a date."),
                    "Write it as YYYY-MM-DD.",
                ));
            }
            d.clone()
        }
        None => today(),
    };
    let key = match &opts.key {
        Some(k) => Some(signing_key(k)?),
        None => None,
    };
    let cat = Catalogue::read(catalog)?;
    let rows = cat.rows();
    check(&cat, &rows, opts)?;

    std::fs::create_dir_all(out).map_err(|e| Error::io(out, e))?;
    let file = format!("parts-data-{date}.sqlite");
    let path = out.join(&file);
    let tmp = out.join(format!(".{file}.{}.tmp", std::process::id()));
    let _ = std::fs::remove_file(&tmp);
    let written = write_sqlite(&tmp, &cat, &date);
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::rename(&tmp, &path).map_err(|e| Error::io(&path, e))?;
    let manifest = Manifest {
        schema: SCHEMA,
        version: date.clone(),
        date,
        file,
        sha256: fsx::sha256_file(&path)?,
        size: std::fs::metadata(&path).map_err(|e| Error::io(&path, e))?.len(),
        rows,
        credit: CREDIT.into(),
    };
    let text = serde_json::to_string_pretty(&manifest).expect("manifest serializes") + "\n";
    fsx::write_atomic(&out.join(MANIFEST), text.as_bytes())?;
    if let Some(key) = key {
        let sig = key.sign(text.as_bytes());
        let b64 = crate::base64::encode(sig.as_ref()) + "\n";
        fsx::write_atomic(&out.join(format!("{MANIFEST}.sig")), b64.as_bytes())?;
    }
    Ok(manifest)
}

fn db(e: rusqlite::Error) -> Error {
    Error::internal(format!("parts data pack: {e}"))
}

fn write_sqlite(path: &Path, cat: &Catalogue, date: &str) -> Result<()> {
    let mut conn = Connection::open(path).map_err(db)?;
    conn.execute_batch("PRAGMA journal_mode = OFF; PRAGMA synchronous = OFF;")
        .map_err(db)?;
    conn.execute_batch(DDL).map_err(db)?;
    let tx = conn.transaction().map_err(db)?;
    {
        let mut ins = tx.prepare("INSERT INTO meta(key, value) VALUES (?, ?)").map_err(db)?;
        for (k, v) in [
            ("schema", SCHEMA.to_string()),
            ("version", date.to_string()),
            ("credit", CREDIT.to_string()),
        ] {
            ins.execute(params![k, v]).map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT INTO colors(id, name, rgb, is_trans, year_from, year_to) VALUES (?, ?, ?, ?, ?, ?)")
            .map_err(db)?;
        for c in &cat.colors {
            ins.execute(params![c.id, c.name, c.rgb, c.is_trans, c.y1, c.y2])
                .map_err(db)?;
        }
        let mut ins = tx
            .prepare(
                "INSERT INTO color_mappings(color_id, system, ext_id, names, is_export, is_import)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .map_err(db)?;
        for m in &cat.color_mappings {
            let names = serde_json::to_string(&m.names).expect("json");
            ins.execute(params![
                int("color_mappings", &m.rebrickable)?,
                m.system,
                m.ext_id,
                names,
                m.export,
                m.import
            ])
            .map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT INTO part_categories(id, name) VALUES (?, ?)")
            .map_err(db)?;
        for (id, name) in &cat.categories {
            ins.execute(params![id, name]).map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT OR IGNORE INTO parts(part_num, name, category_id, material) VALUES (?, ?, ?, ?)")
            .map_err(db)?;
        for (num, name, cat_id, material) in &cat.parts {
            ins.execute(params![num, name, cat_id, material]).map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT INTO part_mappings(part_num, system, ext_id, is_export, is_import) VALUES (?, ?, ?, ?, ?)")
            .map_err(db)?;
        for m in &cat.part_mappings {
            ins.execute(params![m.rebrickable, m.system, m.ext_id, m.export, m.import])
                .map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT INTO part_relationships(rel_type, child, parent) VALUES (?, ?, ?)")
            .map_err(db)?;
        for (rel, child, parent) in &cat.relationships {
            ins.execute(params![rel, child, parent]).map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT INTO elements(element_id, part_num, color_id) VALUES (?, ?, ?)")
            .map_err(db)?;
        for (id, part, color) in &cat.elements {
            ins.execute(params![id, part, color]).map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT INTO set_parts(part_num, color_id, set_count, year_from, year_to) VALUES (?, ?, ?, ?, ?)")
            .map_err(db)?;
        for ((part, color), (sets, from, to)) in &cat.set_parts {
            ins.execute(params![part, color, sets, from, to]).map_err(db)?;
        }
    }
    tx.commit().map_err(db)?;
    conn.execute_batch("VACUUM;").map_err(db)?;
    Ok(())
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// Today's date in UTC, `YYYY-MM-DD` (days since 1970 to a civil date, after Howard Hinnant's `civil_from_days`).
fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs()) as i64;
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

// ---------------------------------------------------------------------------------------------------------------
// Signatures and installing.

fn signing_key(pkcs8: &[u8]) -> Result<ring::signature::Ed25519KeyPair> {
    ring::signature::Ed25519KeyPair::from_pkcs8_maybe_unchecked(pkcs8).map_err(|_| {
        Error::new(
            ErrorCode::ParamInvalid,
            "The signing key isn't a PKCS#8 Ed25519 private key.",
            "Pass the key file in DER form (openssl genpkey -algorithm ed25519 -outform DER).",
        )
    })
}

fn corrupt(dir: &Path, message: impl Into<String>) -> Error {
    Error::new(
        ErrorCode::LibraryCorrupt,
        message,
        "The pack is damaged or wasn't made by Emmet's release workflow; the installed pack wasn't changed. Get the \
         pack again.",
    )
    .at_file(dir.display().to_string())
}

/// How a pack is checked before it's installed.
enum Trust<'a> {
    /// A downloaded pack: the manifest's Ed25519 signature against this public key, then the sha256.
    Signature(&'a [u8]),
    /// The pack bundled with the app: the sha256 only (plan section 12).
    Bundled,
}

/// The pointer file in the data dir's pack folder: which pack is in use, and the one before it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Pointer {
    current: Option<String>,
    previous: Option<String>,
}

fn pointer_path(data: &DataDir) -> PathBuf {
    data.parts_data().join("current.json")
}

fn read_pointer(data: &DataDir) -> Result<Pointer> {
    let path = pointer_path(data);
    match std::fs::read_to_string(&path) {
        Ok(t) => serde_json::from_str(&t).map_err(|e| corrupt(&path, format!("{} is damaged: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Pointer::default()),
        Err(e) => Err(Error::io(&path, e)),
    }
}

fn write_pointer(data: &DataDir, p: &Pointer) -> Result<()> {
    let text = serde_json::to_string_pretty(p).expect("json") + "\n";
    fsx::write_atomic(&pointer_path(data), text.as_bytes())
}

/// The manifest of an installed pack by its name in the pointer.
fn installed_manifest(data: &DataDir, stem: &str) -> Result<Manifest> {
    let path = data.parts_data().join(format!("{stem}.json"));
    let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    serde_json::from_str(&text).map_err(|e| corrupt(&path, format!("{} is damaged: {e}", path.display())))
}

/// Installs the signed pack in `dir` (`parts-data.json`, its `.sig`, and the SQLite file it names) into the data
/// dir: the signature against `trusted_key`, then the sha256, then an atomic swap. The previous pack stays for
/// [`rollback`]. A pack that fails a check leaves the installed one as it was.
pub fn install(data: &DataDir, dir: &Path, trusted_key: &[u8]) -> Result<Manifest> {
    install_from(data, dir, Trust::Signature(trusted_key))
}

/// Installs the pack bundled with the app, checked by its sha256 only.
pub fn install_bundled(data: &DataDir, dir: &Path) -> Result<Manifest> {
    install_from(data, dir, Trust::Bundled)
}

fn install_from(data: &DataDir, dir: &Path, trust: Trust) -> Result<Manifest> {
    let manifest_path = dir.join(MANIFEST);
    let bytes = std::fs::read(&manifest_path).map_err(|e| Error::io(&manifest_path, e))?;
    if let Trust::Signature(key) = trust {
        let sig_path = dir.join(format!("{MANIFEST}.sig"));
        let sig = std::fs::read_to_string(&sig_path)
            .ok()
            .and_then(|t| crate::base64::decode(t.trim()))
            .ok_or_else(|| {
                corrupt(
                    dir,
                    format!("{} has no readable signature ({MANIFEST}.sig).", dir.display()),
                )
            })?;
        ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key)
            .verify(&bytes, &sig)
            .map_err(|_| {
                corrupt(
                    dir,
                    format!(
                        "The parts data pack in {} isn't signed by a key Emmet trusts.",
                        dir.display()
                    ),
                )
            })?;
    }
    let manifest: Manifest = serde_json::from_slice(&bytes)
        .map_err(|e| corrupt(dir, format!("{} doesn't parse: {e}", manifest_path.display())))?;
    if manifest.schema != SCHEMA {
        return Err(corrupt(
            dir,
            format!(
                "The parts data pack in {} has schema {}; this Emmet reads schema {SCHEMA}.",
                dir.display(),
                manifest.schema
            ),
        ));
    }
    // The manifest names a file next to it, never a path.
    let plain = Path::new(&manifest.file)
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)))
        && !manifest.file.contains(['/', '\\']);
    if !plain || manifest.file.is_empty() {
        return Err(corrupt(
            dir,
            format!("The manifest names '{}', which isn't a file name.", manifest.file),
        ));
    }
    let stem = format!(
        "parts-data-{}-{}",
        manifest
            .version
            .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "_"),
        manifest.sha256.get(..12).unwrap_or("x")
    );
    let folder = data.parts_data();
    std::fs::create_dir_all(&folder).map_err(|e| Error::io(&folder, e))?;
    let dest = folder.join(format!("{stem}.sqlite"));
    let present = dest.is_file() && fsx::sha256_file(&dest).ok().as_deref() == Some(manifest.sha256.as_str());
    if !present {
        copy_checked(&dir.join(&manifest.file), &dest, &manifest)?;
    }
    // The file must be a pack this Emmet reads, with the manifest's version.
    let version: Option<String> = Connection::open_with_flags(&dest, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .ok()
        .and_then(|c| {
            c.query_row("SELECT value FROM meta WHERE key = 'version'", [], |r| r.get(0))
                .ok()
        });
    if version.as_deref() != Some(manifest.version.as_str()) {
        let _ = std::fs::remove_file(&dest);
        return Err(corrupt(
            dir,
            "The pack's SQLite file isn't the version its manifest names.",
        ));
    }
    fsx::write_atomic(&folder.join(format!("{stem}.json")), &bytes)?;
    let mut pointer = read_pointer(data).unwrap_or_default();
    if pointer.current.as_deref() != Some(stem.as_str()) {
        pointer.previous = pointer.current.take();
        pointer.current = Some(stem);
    }
    write_pointer(data, &pointer)?;
    prune(data, &pointer);
    Ok(manifest)
}

/// Copies `src` to `dest` through a temp file in `dest`'s folder, hashing as it goes; renames it into place only when
/// the sha256 and size match the manifest.
fn copy_checked(src: &Path, dest: &Path, manifest: &Manifest) -> Result<()> {
    let dir = dest.parent().expect("dest has a folder");
    let tmp = dir.join(format!(
        ".{}.{}-{}.tmp",
        dest.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id(),
        fsx::random_hex(1)
    ));
    let copied = (|| -> Result<()> {
        let mut from = std::fs::File::open(src).map_err(|e| Error::io(src, e))?;
        let mut to = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| Error::io(&tmp, e))?;
        let mut h = Sha256::new();
        let mut size = 0u64;
        let mut buf = vec![0u8; 1 << 16];
        loop {
            let n = from.read(&mut buf).map_err(|e| Error::io(src, e))?;
            if n == 0 {
                break;
            }
            h.update(&buf[..n]);
            size += n as u64;
            to.write_all(&buf[..n]).map_err(|e| Error::io(&tmp, e))?;
        }
        let got = fsx::hex(&h.finalize());
        if got != manifest.sha256 || size != manifest.size {
            return Err(corrupt(
                src,
                format!(
                    "{} has sha256 {got} ({size} bytes), but its manifest says {} ({} bytes).",
                    src.display(),
                    manifest.sha256,
                    manifest.size
                ),
            ));
        }
        to.sync_all().map_err(|e| Error::io(&tmp, e))?;
        drop(to);
        std::fs::rename(&tmp, dest).map_err(|e| Error::io(dest, e))
    })();
    if copied.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    copied
}

/// Removes packs that are neither current nor previous. A file another process still has open stays until next time.
fn prune(data: &DataDir, pointer: &Pointer) {
    let keep: Vec<&str> = [&pointer.current, &pointer.previous]
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect();
    let Ok(entries) = std::fs::read_dir(data.parts_data()) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let stem = name
            .strip_suffix(".sqlite")
            .or_else(|| name.strip_suffix(".json"))
            .filter(|s| s.starts_with("parts-data-"));
        if let Some(stem) = stem
            && !keep.contains(&stem)
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Goes back to the pack installed before the current one. The current one becomes the previous, so a second
/// rollback undoes the first.
pub fn rollback(data: &DataDir) -> Result<Manifest> {
    let pointer = read_pointer(data)?;
    let Some(previous) = pointer.previous.clone() else {
        return Err(Error::new(
            ErrorCode::LibraryMissing,
            "There's no previous parts data pack to go back to.",
            "Nothing to undo: the current pack is the only one installed.",
        ));
    };
    let manifest = installed_manifest(data, &previous)?;
    write_pointer(
        data,
        &Pointer {
            current: Some(previous),
            previous: pointer.current,
        },
    )?;
    Ok(manifest)
}

/// The installed pack's manifest, or None.
pub fn installed(data: &DataDir) -> Option<Manifest> {
    let stem = read_pointer(data).ok()?.current?;
    installed_manifest(data, &stem).ok()
}

/// The pack bundled with the app: `EMMET_BUNDLED_PARTS_DATA`, else next to the executable (`parts-data/` beside it on
/// Windows, `Contents/Resources/parts-data/` in the macOS app). None when there's no `parts-data.json` there.
pub fn bundled_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(BUNDLE_ENV) {
        let dir = PathBuf::from(dir);
        return dir.join(MANIFEST).is_file().then_some(dir);
    }
    let exe = std::env::current_exe().ok()?;
    let here = exe.parent()?;
    [here.join("parts-data"), here.join("../Resources/parts-data")]
        .into_iter()
        .find(|d| d.join(MANIFEST).is_file())
}

/// Installs the bundled pack when the data dir has none, an unreadable one, or an older one. Returns the bundled
/// manifest when it installed it.
pub fn ensure_bundled(data: &DataDir) -> Result<Option<Manifest>> {
    let Some(dir) = bundled_dir() else {
        return Ok(None);
    };
    let bundled: Manifest = match std::fs::read(dir.join(MANIFEST))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
    {
        Some(m) => m,
        None => return Ok(None),
    };
    if bundled.schema != SCHEMA {
        return Ok(None);
    }
    let older = match installed(data) {
        Some(m) => m.schema != SCHEMA || m.version < bundled.version,
        None => true,
    };
    if !older {
        return Ok(None);
    }
    install_bundled(data, &dir).map(Some)
}

// ---------------------------------------------------------------------------------------------------------------
// The installed pack.

/// The installed pack's version (installing the bundled one first when it's newer), or None: for the run manifest
/// and the inputs hash.
pub fn version(data: &DataDir) -> Result<Option<String>> {
    Ok(PartsData::open(data)?.map(|p| p.version().to_string()))
}

/// An open parts data pack.
pub struct PartsData {
    conn: Connection,
    manifest: Manifest,
    /// New LDraw id → the old ids moved to it, from the parts index (rule 2), built on first use.
    moves: OnceCell<Moves>,
}

impl Catalog for PartsData {
    fn ld_row(&self, ldraw: &str) -> Result<Option<String>> {
        self.conn
            .prepare_cached(
                "SELECT part_num FROM part_mappings WHERE system = 'LD' AND ext_id = ?
                 ORDER BY is_import DESC, rowid LIMIT 1",
            )
            .and_then(|mut s| s.query_row([ldraw], |r| r.get(0)).optional())
            .map_err(db)
    }
    fn part(&self, id: &str) -> Result<Option<String>> {
        self.conn
            .prepare_cached("SELECT part_num FROM parts WHERE part_num = ?")
            .and_then(|mut s| s.query_row([id], |r| r.get(0)).optional())
            .map_err(db)
    }
}

impl PartsData {
    /// The installed pack, installing the bundled one first when it's newer; None when there's no pack at all.
    pub fn open(data: &DataDir) -> Result<Option<Self>> {
        ensure_bundled(data)?;
        let Some(stem) = read_pointer(data)?.current else {
            return Ok(None);
        };
        let manifest = installed_manifest(data, &stem)?;
        if manifest.schema != SCHEMA {
            return Ok(None);
        }
        let path = data.parts_data().join(format!("{stem}.sqlite"));
        let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| {
            corrupt(
                &path,
                format!("The parts data pack {} doesn't open: {e}", path.display()),
            )
        })?;
        Ok(Some(Self {
            conn,
            manifest,
            moves: OnceCell::new(),
        }))
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The pack's version, for the run manifest and the inputs hash.
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    fn moved_from(&self, index: &Index) -> Result<&Moves> {
        if let Some(m) = self.moves.get() {
            return Ok(m);
        }
        let mut moves = Moves::new();
        for (old, new) in index.moved()? {
            moves.entry(new).or_default().push(old);
        }
        Ok(self.moves.get_or_init(|| moves))
    }

    /// The Rebrickable part for an LDraw id, by ADR 0004's four rules in order, and the rule that found it.
    pub fn mapping(&self, index: &Index, ldraw: &str) -> Result<Option<PartMapping>> {
        let moves = self.moved_from(index)?;
        map_part(self, &|id| moves.get(id).cloned().unwrap_or_default(), ldraw)
    }

    /// Rebrickable's id for an LDraw part, or None when it has no mapping.
    pub fn rebrickable_part(&self, index: &Index, ldraw: &str) -> Result<Option<String>> {
        Ok(self.mapping(index, ldraw)?.map(|m| m.rebrickable_id))
    }

    /// Rebrickable's name for one of its parts.
    pub fn part_name(&self, rebrickable: &str) -> Result<Option<String>> {
        self.conn
            .prepare_cached("SELECT name FROM parts WHERE part_num = ?")
            .and_then(|mut s| s.query_row([rebrickable], |r| r.get(0)).optional())
            .map_err(db)
    }

    /// BrickLink's id for an LDraw part (through its Rebrickable part's BL row), or None.
    pub fn bricklink_part(&self, index: &Index, ldraw: &str) -> Result<Option<String>> {
        let Some(rb) = self.rebrickable_part(index, ldraw)? else {
            return Ok(None);
        };
        self.conn
            .prepare_cached(
                "SELECT ext_id FROM part_mappings WHERE system = 'BL' AND part_num = ?
                 ORDER BY is_export DESC, rowid LIMIT 1",
            )
            .and_then(|mut s| s.query_row([rb], |r| r.get(0)).optional())
            .map_err(db)
    }

    fn rebrickable_color(&self, ldraw: u32) -> Result<Option<i64>> {
        self.conn
            .prepare_cached(
                "SELECT color_id FROM color_mappings WHERE system = 'LD' AND ext_id = ?
                 ORDER BY is_import DESC, is_export DESC, rowid LIMIT 1",
            )
            .and_then(|mut s| s.query_row([ldraw.to_string()], |r| r.get(0)).optional())
            .map_err(db)
    }

    /// An LDraw color's Rebrickable and BrickLink ids and names, or None when Rebrickable has no mapping for it.
    pub fn color(&self, ldraw: u32) -> Result<Option<ColorIds>> {
        let Some(rb) = self.rebrickable_color(ldraw)? else {
            return Ok(None);
        };
        let name: String = self
            .conn
            .prepare_cached("SELECT name FROM colors WHERE id = ?")
            .and_then(|mut s| s.query_row([rb], |r| r.get(0)).optional())
            .map_err(db)?
            .unwrap_or_default();
        let bl: Option<(String, String)> = self
            .conn
            .prepare_cached(
                "SELECT ext_id, names FROM color_mappings WHERE system = 'BL' AND color_id = ?
                 ORDER BY is_export DESC, rowid LIMIT 1",
            )
            .and_then(|mut s| s.query_row([rb], |r| Ok((r.get(0)?, r.get(1)?))).optional())
            .map_err(db)?;
        let (bricklink_id, bricklink_name) = match bl {
            Some((id, names)) => (
                id.parse().ok(),
                serde_json::from_str::<Vec<String>>(&names)
                    .ok()
                    .and_then(|n| n.into_iter().next()),
            ),
            None => (None, None),
        };
        Ok(Some(ColorIds {
            rebrickable_id: rb,
            rebrickable_name: name,
            bricklink_id,
            bricklink_name,
        }))
    }

    /// Every name Rebrickable and BrickLink give an LDraw color (Rebrickable's, then the LD and BL rows'), for
    /// lookups in any spelling.
    pub fn color_aliases(&self, ldraw: u32) -> Result<Vec<String>> {
        let Some(rb) = self.rebrickable_color(ldraw)? else {
            return Ok(vec![]);
        };
        let mut out: Vec<String> = Vec::new();
        let mut push = |n: String| {
            if !n.is_empty() && !out.contains(&n) {
                out.push(n);
            }
        };
        if let Some(name) = self
            .conn
            .prepare_cached("SELECT name FROM colors WHERE id = ?")
            .and_then(|mut s| s.query_row([rb], |r| r.get::<_, String>(0)).optional())
            .map_err(db)?
        {
            push(name);
        }
        let mut stmt = self
            .conn
            .prepare_cached("SELECT names FROM color_mappings WHERE color_id = ? ORDER BY system DESC, rowid")
            .map_err(db)?;
        let rows: Vec<String> = stmt
            .query_map([rb], |r| r.get(0))
            .map_err(db)?
            .collect::<std::result::Result<_, _>>()
            .map_err(db)?;
        for names in rows {
            for n in serde_json::from_str::<Vec<String>>(&names).unwrap_or_default() {
                push(n);
            }
        }
        Ok(out)
    }

    /// The LDraw codes of every color some part was made in (an element or a set).
    pub fn produced_colors(&self) -> Result<BTreeSet<u32>> {
        self.ldraw_codes(
            "SELECT color_id FROM elements UNION SELECT color_id FROM set_parts",
            rusqlite::params![],
        )
    }

    /// The LDraw codes of the colors an LDraw part was made in; None when the part has no Rebrickable mapping.
    pub fn part_colors(&self, index: &Index, ldraw: &str) -> Result<Option<BTreeSet<u32>>> {
        let Some(rb) = self.rebrickable_part(index, ldraw)? else {
            return Ok(None);
        };
        self.ldraw_codes(
            "SELECT color_id FROM elements WHERE part_num = ?1 UNION SELECT color_id FROM set_parts WHERE part_num = ?1",
            rusqlite::params![rb],
        )
        .map(Some)
    }

    /// LDraw codes for the Rebrickable colors `colors_sql` selects: every LD row of each.
    fn ldraw_codes(&self, colors_sql: &str, args: &[&dyn rusqlite::ToSql]) -> Result<BTreeSet<u32>> {
        let sql =
            format!("SELECT DISTINCT ext_id FROM color_mappings WHERE system = 'LD' AND color_id IN ({colors_sql})");
        let mut stmt = self.conn.prepare_cached(&sql).map_err(db)?;
        let rows: Vec<String> = stmt
            .query_map(args, |r| r.get(0))
            .map_err(db)?
            .collect::<std::result::Result<_, _>>()
            .map_err(db)?;
        Ok(rows.into_iter().filter_map(|c| c.parse().ok()).collect())
    }

    /// Alternate and mold-variant parts (Rebrickable's `A` and `M` relationships, either way round), as LDraw ids
    /// the library has; None when the part has no Rebrickable mapping.
    pub fn alternates(&self, index: &Index, ldraw: &str) -> Result<Option<Vec<String>>> {
        let Some(rb) = self.rebrickable_part(index, ldraw)? else {
            return Ok(None);
        };
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT parent FROM part_relationships WHERE child = ?1 AND rel_type IN ('A', 'M')
                 UNION SELECT child FROM part_relationships WHERE parent = ?1 AND rel_type IN ('A', 'M')",
            )
            .map_err(db)?;
        let others: Vec<String> = stmt
            .query_map([&rb], |r| r.get(0))
            .map_err(db)?
            .collect::<std::result::Result<_, _>>()
            .map_err(db)?;
        let mut out = Vec::new();
        let me = crate::index::normalize_id(ldraw);
        for other in others {
            if let Some(id) = self.to_ldraw(index, &other)?
                && id != me
                && !out.contains(&id)
            {
                out.push(id);
            }
        }
        out.sort();
        Ok(Some(out))
    }

    /// Rebrickable → LDraw (ADR 0004's reverse): the LD row marked `is_export_id`, else the identical id, when the
    /// library has that part.
    fn to_ldraw(&self, index: &Index, rebrickable: &str) -> Result<Option<String>> {
        let exported: Option<String> = self
            .conn
            .prepare_cached(
                "SELECT ext_id FROM part_mappings WHERE system = 'LD' AND part_num = ? AND is_export = 1 LIMIT 1",
            )
            .and_then(|mut s| s.query_row([rebrickable], |r| r.get(0)).optional())
            .map_err(db)?;
        let id = crate::index::normalize_id(&exported.unwrap_or_else(|| rebrickable.to_string()));
        Ok(index.part(&id)?.map(|p| p.id))
    }

    /// Whether an LDraw part was ever made in an LDraw color. Unknown, never unavailable, when the part or the color
    /// has no Rebrickable mapping.
    pub fn availability(&self, index: &Index, ldraw: &str, color: u32) -> Result<Availability> {
        let Some(rb) = self.rebrickable_part(index, ldraw)? else {
            return Ok(Availability::unknown());
        };
        let made: bool = self
            .conn
            .prepare_cached(
                "SELECT EXISTS(SELECT 1 FROM elements WHERE part_num = ?1)
                     OR EXISTS(SELECT 1 FROM set_parts WHERE part_num = ?1)",
            )
            .and_then(|mut s| s.query_row([&rb], |r| r.get(0)))
            .map_err(db)?;
        if !made {
            return Ok(Availability {
                status: Status::PartUnavailable,
                sets: Some(0),
                years: None,
            });
        }
        let Some(rb_color) = self.rebrickable_color(color)? else {
            return Ok(Availability::unknown());
        };
        let element: bool = self
            .conn
            .prepare_cached("SELECT EXISTS(SELECT 1 FROM elements WHERE part_num = ? AND color_id = ?)")
            .and_then(|mut s| s.query_row(params![rb, rb_color], |r| r.get(0)))
            .map_err(db)?;
        let sets: Option<(u32, Option<u16>, Option<u16>)> = self
            .conn
            .prepare_cached("SELECT set_count, year_from, year_to FROM set_parts WHERE part_num = ? AND color_id = ?")
            .and_then(|mut s| {
                s.query_row(params![rb, rb_color], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                    .optional()
            })
            .map_err(db)?;
        let (count, years) = match sets {
            Some((n, Some(a), Some(b))) => (n, Some([a, b])),
            Some((n, ..)) => (n, None),
            None => (0, None),
        };
        let status = if element || sets.is_some() {
            Status::Available
        } else {
            Status::ColorUnavailable
        };
        Ok(Availability {
            status,
            sets: Some(count),
            years,
        })
    }
}

/// Adds what the pack knows to the library's colors: whether each was ever made (`produced`), and Rebrickable's and
/// BrickLink's names as aliases, except a name that's already another color's (BrickLink's "Light Gray" stays
/// LDraw's Light Gray). Without a pack the colors stay as they are.
pub fn annotate_colors(pack: Option<&PartsData>, colors: &mut [crate::colors::Color]) -> Result<()> {
    use crate::colors::key;
    let Some(pack) = pack else { return Ok(()) };
    let produced = pack.produced_colors()?;
    let mut owner: HashMap<String, u32> = HashMap::new();
    for c in colors.iter() {
        for n in std::iter::once(&c.name).chain(c.aliases.iter()) {
            owner.entry(key(n)).or_insert(c.code);
        }
    }
    for c in colors.iter_mut() {
        c.produced = Some(produced.contains(&c.code));
        for n in pack.color_aliases(c.code)? {
            let k = key(&n);
            if owner.get(&k).is_none_or(|o| *o == c.code) && !c.aliases.contains(&n) && n != c.name {
                owner.insert(k, c.code);
                c.aliases.push(n);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_split_quoted_fields() {
        let text = "a,b,c\r\n1,\"x, \"\"y\"\"\",\n\"multi\nline\",2,3";
        let rows: Vec<Vec<String>> = Rows::new(text)
            .map(|r| r.into_iter().map(|c| c.into_owned()).collect())
            .collect();
        assert_eq!(
            rows,
            [
                vec!["a", "b", "c"],
                vec!["1", "x, \"y\"", ""],
                vec!["multi\nline", "2", "3"]
            ]
        );
    }

    #[test]
    fn formed_variants_strip_to_their_base() {
        assert_eq!(formed_base("75541-f1"), Some("75541"));
        assert_eq!(formed_base("75541-f12"), Some("75541"));
        assert_eq!(formed_base("75541-fx"), None);
        assert_eq!(formed_base("-f1"), None);
        assert_eq!(formed_base("3001"), None);
    }
}
