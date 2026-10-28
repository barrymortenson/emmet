// SPDX-License-Identifier: AGPL-3.0-or-later
//! The parts index (`parts.sqlite`, plan section 12): parts with their sizes and body boxes, FTS5 over names,
//! categories, and keywords, colors, and search synonyms. `connectors` and `elements` stay empty until 3A and 3B.

pub mod dims;
mod helpers;
mod search;

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use helpers::Resolved;
pub use search::SearchHit;

use crate::colors::{self, Color};
use crate::data::DataDir;
use crate::envelope::{Error, ErrorCode, Result};
use crate::ldraw::geometry::{Aabb, LibFiles, Measurer};
use crate::ldraw::{Line, Meta};

/// Bumped when the schema or what goes into it changes; an older index is rebuilt.
const SCHEMA: &str = "3";

const SYNONYMS: &str = include_str!("../../../../data/synonyms.toml");

const DDL: &str = "
CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE parts(id TEXT PRIMARY KEY, name TEXT NOT NULL, category TEXT NOT NULL, keywords TEXT NOT NULL,
  size_x REAL, size_y REAL, size_z REAL, studs_top INTEGER, flags TEXT NOT NULL, alias_of TEXT, body_box BLOB,
  file_rel TEXT NOT NULL);
CREATE VIRTUAL TABLE parts_fts USING fts5(name, category, keywords, content='parts', content_rowid='rowid',
  tokenize='unicode61', prefix='1 2 3');
CREATE TABLE connectors(part_id TEXT NOT NULL, kind TEXT NOT NULL, pos BLOB, dir BLOB, gender TEXT, radius REAL,
  source TEXT);
CREATE TABLE colors(code INTEGER PRIMARY KEY, name TEXT NOT NULL, rgb TEXT NOT NULL, edge_rgb TEXT NOT NULL,
  finish TEXT NOT NULL, rebrickable_id INTEGER, bricklink_name TEXT, aliases TEXT NOT NULL);
CREATE TABLE elements(part_id TEXT NOT NULL, color_code INTEGER NOT NULL, rebrickable_part TEXT, years TEXT,
  set_count INTEGER);
CREATE TABLE synonyms(term TEXT NOT NULL, expands_to TEXT NOT NULL);
CREATE TABLE proxies(part_id TEXT PRIMARY KEY, boxes BLOB NOT NULL);
";

/// Size in DSL units at `rot=0`: x and y in studs, z in plates (plan section 08).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Size {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Size {
    /// From a body box in LDraw's frame: DSL x runs along −Z, y along X, z along −Y (`frame` in docs/spec/dsl.md).
    pub fn of(b: &Aabb) -> Self {
        let r = |v: f64| (v * 1000.0).round() / 1000.0;
        Self {
            x: r((b.max.z - b.min.z) / 20.0),
            y: r((b.max.x - b.min.x) / 20.0),
            z: r((b.max.y - b.min.y) / 8.0),
        }
    }
}

/// One part's row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub category: String,
    pub keywords: Vec<String>,
    pub size: Option<Size>,
    /// alias, obsolete, hidden, printed, sticker, shortcut.
    pub flags: Vec<String>,
    /// The part this one moved to or stands for.
    pub alias_of: Option<String>,
    /// The body box in the part's LDraw frame, LDU: [min x, min y, min z, max x, max y, max z].
    pub body_box: Option<[f64; 6]>,
    /// The file, relative to the library root.
    pub file: String,
}

pub struct Index {
    conn: Connection,
    path: std::path::PathBuf,
}

impl Index {
    /// Opens the index read-only. Fails with `LIBRARY_MISSING` when there isn't one.
    pub fn open(data: &DataDir) -> Result<Self> {
        let path = data.index();
        let conn =
            Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|_| missing_index(data))?;
        let schema: Option<String> = conn
            .query_row("SELECT value FROM meta WHERE key = 'schema'", [], |r| r.get(0))
            .optional()
            .map_err(|_| missing_index(data))?;
        if schema.as_deref() != Some(SCHEMA) {
            return Err(missing_index(data));
        }
        Ok(Self { conn, path })
    }

    /// Opens the index, building it first when it's missing or its schema version isn't this Emmet's.
    pub fn ensure(data: &DataDir) -> Result<Self> {
        match Self::open(data) {
            Ok(index) => Ok(index),
            Err(_) => {
                crate::library::require(data)?;
                build(data)?;
                Self::open(data)
            }
        }
    }

    pub fn part_count(&self) -> Result<usize> {
        self.conn
            .query_row("SELECT count(*) FROM parts", [], |r| r.get::<_, i64>(0))
            .map(|n| n as usize)
            .map_err(db)
    }

    pub fn meta(&self, key: &str) -> Option<String> {
        self.conn
            .query_row("SELECT value FROM meta WHERE key = ?", [key], |r| r.get(0))
            .ok()
    }

    pub fn part(&self, id: &str) -> Result<Option<Part>> {
        let id = normalize_id(id);
        self.conn
            .query_row(
                "SELECT id, name, category, keywords, size_x, size_y, size_z, flags, alias_of, body_box, file_rel
                 FROM parts WHERE id = ?",
                [&id],
                |r| {
                    let size = match (
                        r.get::<_, Option<f64>>(4)?,
                        r.get::<_, Option<f64>>(5)?,
                        r.get::<_, Option<f64>>(6)?,
                    ) {
                        (Some(x), Some(y), Some(z)) => Some(Size { x, y, z }),
                        _ => None,
                    };
                    let keywords: String = r.get(3)?;
                    let flags: String = r.get(7)?;
                    let blob: Option<Vec<u8>> = r.get(9)?;
                    Ok(Part {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        category: r.get(2)?,
                        keywords: split_list(&keywords),
                        size,
                        flags: split_list(&flags),
                        alias_of: r.get(8)?,
                        body_box: blob.as_deref().and_then(box_from_blob),
                        file: r.get(10)?,
                    })
                },
            )
            .optional()
            .map_err(db)
    }

    pub fn colors(&self) -> Result<Vec<Color>> {
        let mut stmt = self
            .conn
            .prepare("SELECT code, name, rgb, edge_rgb, finish, aliases FROM colors ORDER BY code")
            .map_err(db)?;
        let rows = stmt
            .query_map([], |r| {
                let aliases: String = r.get(5)?;
                Ok(Color {
                    code: r.get(0)?,
                    name: r.get(1)?,
                    rgb: r.get(2)?,
                    edge_rgb: r.get(3)?,
                    finish: r.get(4)?,
                    aliases: serde_json::from_str(&aliases).unwrap_or_default(),
                    produced: None,
                })
            })
            .map_err(db)?;
        rows.collect::<std::result::Result<_, _>>().map_err(db)
    }

    /// Color abbreviations from `data/synonyms.toml`, as (abbreviation, full name).
    pub fn color_abbreviations(&self) -> Vec<(String, String)> {
        synonyms_toml().1
    }

    /// Every `~Moved to` part as (old id, new id), for the parts data pack's mapping rule 2 (ADR 0004).
    pub fn moved(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, alias_of FROM parts WHERE alias_of IS NOT NULL AND name LIKE '~Moved to %'")
            .map_err(db)?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(db)?;
        rows.collect::<std::result::Result<_, _>>().map_err(db)
    }

    /// Up to `n` part ids close to an id that doesn't exist: same prefix first, then by edit distance.
    pub fn similar_ids(&self, id: &str, n: usize) -> Result<Vec<(String, String)>> {
        let id = normalize_id(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM parts WHERE flags NOT LIKE '%alias%'")
            .map_err(db)?;
        let all: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(db)?
            .filter_map(|r| r.ok())
            .collect();
        let digits: String = id.chars().take_while(char::is_ascii_alphanumeric).collect();
        let mut scored: Vec<(usize, usize, String, String)> = all
            .into_iter()
            .map(|(pid, name)| {
                let common = pid.chars().zip(digits.chars()).take_while(|(a, b)| a == b).count();
                (usize::MAX - common, crate::text::edit_distance(&id, &pid), pid, name)
            })
            .filter(|(common, dist, ..)| *dist <= 2 || usize::MAX - *common >= 3)
            .collect();
        scored.sort();
        Ok(scored.into_iter().take(n).map(|(_, _, id, name)| (id, name)).collect())
    }

    /// The collision proxies of parts (by id), from the index's cache, building and caching the missing ones from the
    /// library (ADR 0013). The cache fills as models use parts; a cache that can't be written (a read-only data dir)
    /// only costs the build again next time. A proxy built without all of its part's files isn't cached.
    pub fn proxies(
        &self,
        lib: &LibFiles,
        ids: &[String],
    ) -> Result<HashMap<String, std::rc::Rc<Vec<crate::check::proxy::ProxyBox>>>> {
        use crate::check::proxy;
        let mut out = HashMap::new();
        let mut missing = Vec::new();
        {
            let mut stmt = self
                .conn
                .prepare_cached("SELECT boxes FROM proxies WHERE part_id = ?")
                .map_err(db)?;
            for id in ids {
                let blob: Option<Vec<u8>> = stmt.query_row([id], |r| r.get(0)).optional().map_err(db)?;
                match blob.as_deref().and_then(proxy::from_blob) {
                    Some(boxes) => {
                        out.insert(id.clone(), std::rc::Rc::new(boxes));
                    }
                    None => missing.push(id.clone()),
                }
            }
        }
        if missing.is_empty() {
            return Ok(out);
        }
        let mut m = Measurer::new(lib);
        let built: Vec<(String, (Vec<proxy::ProxyBox>, bool))> = missing
            .into_iter()
            .map(|id| {
                let boxes = proxy::build(&mut m, &format!("{id}.dat"));
                (id, boxes)
            })
            .collect();
        if let Ok(mut w) = Connection::open(&self.path) {
            let _ = w.busy_timeout(std::time::Duration::from_secs(2));
            if let Ok(tx) = w.transaction() {
                // A part whose files couldn't be read isn't cached: the next run tries again.
                for (id, (boxes, _)) in built.iter().filter(|(_, (_, complete))| *complete) {
                    let _ = tx.execute(
                        "INSERT OR REPLACE INTO proxies(part_id, boxes) VALUES (?, ?)",
                        params![id, proxy::to_blob(boxes)],
                    );
                }
                let _ = tx.commit();
            }
        }
        for (id, boxes) in built {
            out.insert(id, std::rc::Rc::new(boxes.0));
        }
        Ok(out)
    }

    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

fn missing_index(data: &DataDir) -> Error {
    Error::new(
        ErrorCode::LibraryMissing,
        format!(
            "The parts index in {} is missing or out of date.",
            data.root().display()
        ),
        "Run emmet library verify --reindex (or emmet library install if the library is missing too).",
    )
}

pub(crate) fn db(e: rusqlite::Error) -> Error {
    Error::internal(format!("parts index: {e}"))
}

/// `3001`, `3001.dat`, and ` 3001.DAT ` are all `3001`.
pub fn normalize_id(id: &str) -> String {
    let id = crate::ldraw::normalize_ref(id);
    id.strip_suffix(".dat").unwrap_or(&id).to_string()
}

fn split_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

fn box_to_blob(b: &Aabb) -> Vec<u8> {
    b.to_array().iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn box_from_blob(blob: &[u8]) -> Option<[f64; 6]> {
    (blob.len() == 48).then(|| std::array::from_fn(|i| f64::from_le_bytes(blob[i * 8..i * 8 + 8].try_into().unwrap())))
}

/// Pairs of words: (term, expansion) for parts, (abbreviation, full name) for colors.
type Pairs = Vec<(String, String)>;

/// The part synonyms and color abbreviations in data/synonyms.toml.
fn synonyms_toml() -> (Pairs, Pairs) {
    let doc: toml::Table = SYNONYMS.parse().expect("data/synonyms.toml parses (checked by tests)");
    let mut parts = Vec::new();
    if let Some(t) = doc.get("parts").and_then(|v| v.as_table()) {
        for (term, list) in t {
            for e in list.as_array().into_iter().flatten().filter_map(|v| v.as_str()) {
                parts.push((term.clone(), e.to_string()));
            }
        }
    }
    let mut colors = Vec::new();
    if let Some(t) = doc.get("colors").and_then(|v| v.as_table()) {
        for (abbr, full) in t {
            if let Some(full) = full.as_str() {
                colors.push((abbr.clone(), full.to_string()));
            }
        }
    }
    (parts, colors)
}

/// What a part file's header says.
struct Header {
    name: String,
    category: String,
    keywords: Vec<String>,
    kind: String,
}

fn header(lines: &[Line]) -> Header {
    let mut name = None;
    let mut category = None;
    let mut keywords = Vec::new();
    let mut kind = String::new();
    for line in lines {
        let Line::Meta(Meta::Comment(text)) = line else {
            continue;
        };
        if name.is_none() {
            name = Some(text.trim().to_string());
            continue;
        }
        if let Some(c) = text.strip_prefix("!CATEGORY ") {
            category = Some(c.trim().to_string());
        } else if let Some(k) = text.strip_prefix("!KEYWORDS ") {
            keywords.extend(k.split(',').map(|w| w.trim().to_string()).filter(|w| !w.is_empty()));
        } else if let Some(o) = text.strip_prefix("!LDRAW_ORG ") {
            kind = o.split_whitespace().next().unwrap_or("").to_string();
        }
    }
    let name = name
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let category = category.unwrap_or_else(|| {
        name.trim_start_matches(['~', '=', '_', '|'])
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    });
    Header {
        name,
        category,
        keywords,
        kind,
    }
}

struct Row {
    id: String,
    header: Header,
    body: Option<Aabb>,
    file_rel: String,
    flags: Vec<&'static str>,
    alias_of: Option<String>,
}

fn row(lib: &LibFiles, m: &mut Measurer, reference: &str, path: &Path) -> Row {
    let lines = m.lines(reference).unwrap_or_default();
    let header = header(&lines);
    let id = reference.strip_suffix(".dat").unwrap_or(reference).to_string();
    let mut flags = Vec::new();
    let mut alias_of = None;
    let n = &header.name;
    if let Some(target) = n.strip_prefix("~Moved to ") {
        flags.push("alias");
        alias_of = Some(normalize_id(target));
    } else if n.starts_with('=') || header.kind.contains("Alias") {
        flags.push("alias");
        alias_of = lines.iter().find_map(|l| match l {
            Line::Sub { file, .. } => Some(normalize_id(file)),
            _ => None,
        });
    }
    if n.contains("(Obsolete)") {
        flags.push("obsolete");
    }
    if n.starts_with('~') && alias_of.is_none() {
        flags.push("hidden");
    }
    let printed_id = id
        .find('p')
        .is_some_and(|i| i > 0 && id[..i].chars().all(|c| c.is_ascii_digit()));
    if printed_id || n.contains("Pattern") || n.contains(" Print") {
        flags.push("printed");
    }
    if header.category == "Sticker" || n.starts_with("Sticker") {
        flags.push("sticker");
    }
    if header.kind.starts_with("Shortcut") {
        flags.push("shortcut");
    }
    let file_rel = path
        .strip_prefix(&lib.root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    Row {
        body: m.body_box(reference),
        id,
        header,
        file_rel,
        flags,
        alias_of,
    }
}

/// Builds `parts.sqlite` from the installed library, into a temp file that replaces the old index when complete.
/// Returns the number of parts.
pub fn build(data: &DataDir) -> Result<usize> {
    let info = crate::library::require(data)?;
    let root = data.ldraw();
    let lib = LibFiles::scan(&root).map_err(|e| Error::io(&root, e))?;
    let parts = lib.parts();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(16);
    let chunk = parts.len().div_ceil(threads).max(1);
    let mut rows: Vec<Row> = std::thread::scope(|s| {
        let handles: Vec<_> = parts
            .chunks(chunk)
            .map(|chunk| {
                let lib = &lib;
                s.spawn(move || {
                    let mut m = Measurer::new(lib);
                    chunk.iter().map(|(r, p)| row(lib, &mut m, r, p)).collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("index thread"))
            .collect()
    });
    rows.sort_by(|a, b| a.id.cmp(&b.id));

    let config = root.join("LDConfig.ldr");
    let ldconfig = std::fs::read_to_string(&config).map_err(|e| {
        Error::new(
            ErrorCode::LibraryCorrupt,
            format!("The library has no readable LDConfig.ldr (the color table): {e}"),
            "Run emmet library install to put the library back.",
        )
        .at_file(config.display().to_string())
    })?;
    let mut color_rows = colors::parse_ldconfig(&ldconfig);
    let (part_synonyms, color_abbrs) = synonyms_toml();
    for (abbr, full) in &color_abbrs {
        if let Some(c) = color_rows
            .iter_mut()
            .find(|c| colors::key(&c.name) == colors::key(full))
        {
            c.aliases.push(abbr.clone());
        }
    }

    let final_path = data.index();
    let tmp = data.root().join(format!("parts.sqlite.{}.tmp", std::process::id()));
    let _ = std::fs::remove_file(&tmp);
    let mut conn = Connection::open(&tmp).map_err(db)?;
    conn.execute_batch("PRAGMA journal_mode = OFF; PRAGMA synchronous = OFF;")
        .map_err(db)?;
    conn.execute_batch(DDL).map_err(db)?;
    let tx = conn.transaction().map_err(db)?;
    {
        let mut ins = tx
            .prepare(
                "INSERT INTO parts(id, name, category, keywords, size_x, size_y, size_z, studs_top, flags, alias_of,
                 body_box, file_rel) VALUES (?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, ?)",
            )
            .map_err(db)?;
        for r in &rows {
            let size = r.body.as_ref().map(Size::of);
            ins.execute(params![
                r.id,
                r.header.name,
                r.header.category,
                r.header.keywords.join(", "),
                size.map(|s| s.x),
                size.map(|s| s.y),
                size.map(|s| s.z),
                r.flags.join(","),
                r.alias_of,
                r.body.as_ref().map(box_to_blob),
                r.file_rel,
            ])
            .map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT INTO colors(code, name, rgb, edge_rgb, finish, aliases) VALUES (?, ?, ?, ?, ?, ?)")
            .map_err(db)?;
        for c in &color_rows {
            let aliases = serde_json::to_string(&c.aliases).expect("json");
            ins.execute(params![c.code, c.name, c.rgb, c.edge_rgb, c.finish, aliases])
                .map_err(db)?;
        }
        let mut ins = tx
            .prepare("INSERT INTO synonyms(term, expands_to) VALUES (?, ?)")
            .map_err(db)?;
        for (term, to) in part_synonyms.iter().chain(color_abbrs.iter()) {
            ins.execute(params![term, to]).map_err(db)?;
        }
        let mut ins = tx.prepare("INSERT INTO meta(key, value) VALUES (?, ?)").map_err(db)?;
        for (k, v) in [
            ("schema", SCHEMA),
            ("library", info.release.as_str()),
            ("emmet", crate::version()),
        ] {
            ins.execute(params![k, v]).map_err(db)?;
        }
    }
    tx.execute_batch("INSERT INTO parts_fts(parts_fts) VALUES ('rebuild');")
        .map_err(db)?;
    tx.commit().map_err(db)?;
    conn.execute_batch("VACUUM;").map_err(db)?;
    drop(conn);
    std::fs::rename(&tmp, &final_path).map_err(|e| Error::io(&final_path, e))?;
    Ok(rows.len())
}
