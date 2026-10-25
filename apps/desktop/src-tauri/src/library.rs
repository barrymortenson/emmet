// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `emmet-lib://` protocol (plan section 13): LDraw library files for the viewer, served from app data with an
//! in-memory cache. `/pack?parts=3001,3004` returns every file those parts use in one reply (ADR 0005 found one
//! request per file is what makes WebView2 slow). Names resolve only through an index of the files inside the
//! library, built without following links, so no request can reach outside it.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;

pub struct Reply {
    pub status: u16,
    pub mime: &'static str,
    pub body: Vec<u8>,
}

impl Reply {
    fn not_found(why: &str) -> Self {
        Self {
            status: 404,
            mime: "text/plain",
            body: why.as_bytes().to_vec(),
        }
    }
}

/// What `/pack` returns. Each part lists its own file first, then every file it uses; `files` holds each file once.
#[derive(Debug, Default, Serialize)]
pub struct Pack {
    pub parts: BTreeMap<String, Vec<String>>,
    pub files: BTreeMap<String, String>,
    /// Files a part refers to that the library doesn't have, by part.
    pub missing: BTreeMap<String, Vec<String>>,
}

pub struct Library {
    root: PathBuf,
    /// Lowercase `/`-separated path inside the library → the file.
    index: OnceLock<HashMap<String, PathBuf>>,
    cache: Mutex<HashMap<String, Arc<String>>>,
}

impl Library {
    /// The library folder (`<data>/ldraw`). It may not exist yet; every request then answers 404.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            index: OnceLock::new(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Answers a request for `path` (percent-encoded, as in the URL) and its query.
    pub fn respond(&self, path: &str, query: Option<&str>) -> Reply {
        let Some(path) = decode(path) else {
            return Reply::not_found("bad path");
        };
        let path = path.trim_start_matches('/');
        if path == "pack" {
            let ids = query
                .and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("parts=")))
                .and_then(decode)
                .unwrap_or_default();
            let ids: Vec<&str> = ids
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .take(2000)
                .collect();
            let body = serde_json::to_vec(&self.pack(&ids)).expect("pack serializes");
            return Reply {
                status: 200,
                mime: "application/json",
                body,
            };
        }
        match self.file(path) {
            Some(text) => Reply {
                status: 200,
                mime: "text/plain; charset=utf-8",
                body: text.as_bytes().to_vec(),
            },
            None => Reply::not_found("not in the library"),
        }
    }

    /// Every file the given parts use, each part's own file first.
    pub fn pack(&self, ids: &[&str]) -> Pack {
        let mut pack = Pack::default();
        for &id in ids {
            let id = id.trim_end_matches(".dat");
            let main = format!("{}.dat", id.to_lowercase());
            let mut order = Vec::new();
            let mut missing = BTreeSet::new();
            let mut seen = BTreeSet::new();
            let mut todo = vec![main.clone()];
            while let Some(name) = todo.pop() {
                if !seen.insert(name.clone()) {
                    continue;
                }
                let Some(text) = self.reference(&name) else {
                    missing.insert(name);
                    continue;
                };
                for r in references(&text) {
                    todo.push(r);
                }
                pack.files.entry(name.clone()).or_insert_with(|| (*text).clone());
                order.push(name);
            }
            if !missing.is_empty() {
                pack.missing.insert(id.to_string(), missing.into_iter().collect());
            }
            pack.parts.insert(id.to_string(), order);
        }
        pack
    }

    /// A reference as LDrawLoader names it (`parts/s/x.dat`, `p/48/x.dat`, or a bare name), looked up the way the
    /// loader searches: as named, then under parts/ and p/.
    fn reference(&self, name: &str) -> Option<Arc<String>> {
        if name.starts_with("parts/") || name.starts_with("p/") {
            return self.file(name);
        }
        ["parts/", "p/", ""]
            .iter()
            .find_map(|dir| self.file(&format!("{dir}{name}")))
    }

    /// One library file by its path inside the library, any case. `..`, `.`, empty segments, backslashes, and drive
    /// or root prefixes never match: names come only from the index.
    pub fn file(&self, rel: &str) -> Option<Arc<String>> {
        let key = rel.to_lowercase();
        if key.split('/').any(|s| s.is_empty() || s == "." || s == "..") || key.contains(['\\', ':']) {
            return None;
        }
        if let Some(hit) = self.cache.lock().expect("library cache").get(&key) {
            return Some(hit.clone());
        }
        let path = self.index().get(&key)?;
        let bytes = std::fs::read(path).ok()?;
        let text = Arc::new(String::from_utf8_lossy(&bytes).into_owned());
        self.cache.lock().expect("library cache").insert(key, text.clone());
        Some(text)
    }

    fn index(&self) -> &HashMap<String, PathBuf> {
        self.index.get_or_init(|| {
            let mut map = HashMap::new();
            if let Ok(root) = emmet_core::fsx::canonical(&self.root) {
                walk(&root, &root, &mut map);
            }
            map
        })
    }
}

/// Indexes every file under `dir`. A link is kept only when it resolves inside the library.
fn walk(root: &Path, dir: &Path, map: &mut HashMap<String, PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        let Ok(kind) = e.file_type() else { continue };
        let target = if kind.is_symlink() {
            match crate::paths::inside(root, &path) {
                Some(t) if t.is_file() => t,
                _ => continue,
            }
        } else if kind.is_dir() {
            walk(root, &path, map);
            continue;
        } else {
            path.clone()
        };
        if let Ok(rel) = path.strip_prefix(root) {
            map.insert(rel.to_string_lossy().replace('\\', "/").to_lowercase(), target);
        }
    }
}

/// The files a part file refers to (line type 1), named the way LDrawLoader names them: lowercase, `/`, with
/// `s/` under parts/ and `48/` under p/.
fn references(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut tokens = line.split_whitespace();
        if tokens.next() != Some("1") {
            continue;
        }
        let rest: Vec<&str> = tokens.skip(13).collect();
        if rest.is_empty() {
            continue;
        }
        let name = rest.join(" ").replace('\\', "/").to_lowercase();
        let name = if name.starts_with("s/") {
            format!("parts/{name}")
        } else if name.starts_with("48/") {
            format!("p/{name}")
        } else {
            name
        };
        out.push(name);
    }
    out
}

/// Percent-decoding for URL paths and query values (`+` stays `+`: part ids have no spaces). None for a bad escape
/// or non-UTF-8.
fn decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}
