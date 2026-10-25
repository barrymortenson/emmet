// SPDX-License-Identifier: AGPL-3.0-or-later
//! LDraw files (.ldr, .dat, .mpd): line types 0–5, `0 FILE` / `0 NOFILE`, `0 STEP`, and Emmet's `0 !EMMET` meta.
//! Numbers are f64 and colors are LDraw codes (direct colors `0x2RRGGBB` included).

pub mod geometry;
pub mod placements;

use std::fmt::Write as _;

use glam::{DMat3, DVec3};

/// A subfile reference as a lookup key: lowercased, `\` turned into `/`, surrounding space trimmed (plan section 12).
pub fn normalize_ref(name: &str) -> String {
    name.trim().replace('\\', "/").to_lowercase()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Meta {
    /// `0 FILE <name>`: starts a file in an .mpd.
    File(String),
    /// `0 NOFILE`: ends one.
    NoFile,
    /// `0 STEP`
    Step,
    /// `0 !EMMET ID <id> SRC <file:line>`: the Emmet id and source line of the next part.
    EmmetPart { id: String, src: String },
    /// `0 !EMMET <anything else>`
    Emmet(String),
    /// Any other `0` line, as the text after the `0` (comments, headers, other meta).
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Blank,
    Meta(Meta),
    /// Line type 1: `1 <color> x y z a b c d e f g h i <file>`. `matrix` holds a–i row by row.
    Sub {
        color: u32,
        pos: DVec3,
        matrix: DMat3,
        file: String,
    },
    /// Line type 2.
    Edge {
        color: u32,
        points: [DVec3; 2],
    },
    /// Line type 3.
    Tri {
        color: u32,
        points: [DVec3; 3],
    },
    /// Line type 4.
    Quad {
        color: u32,
        points: [DVec3; 4],
    },
    /// Line type 5: two points, then two control points.
    Optional {
        color: u32,
        points: [DVec3; 4],
    },
}

/// One file of a document. A plain .ldr or .dat is one file with no name.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct File {
    pub name: Option<String>,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub files: Vec<File>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("line {line}: {message}")]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl Document {
    /// The file a subfile reference names, case-insensitively and with either separator.
    pub fn file(&self, name: &str) -> Option<&File> {
        let key = normalize_ref(name);
        self.files
            .iter()
            .find(|f| f.name.as_deref().map(normalize_ref).as_deref() == Some(key.as_str()))
    }
}

/// Parses LDraw text. Lines before the first `0 FILE` belong to an unnamed first file; `0 NOFILE` closes a file.
pub fn parse(text: &str) -> Result<Document, ParseError> {
    let mut doc = Document::default();
    let mut current = File::default();
    let mut open = false; // the current file has content or a name
    for (i, raw) in text.lines().enumerate() {
        let line = parse_line(raw).map_err(|message| ParseError { line: i + 1, message })?;
        match line {
            Line::Meta(Meta::File(name)) => {
                if open {
                    doc.files.push(std::mem::take(&mut current));
                }
                current.name = Some(name);
                open = true;
            }
            Line::Meta(Meta::NoFile) => {
                doc.files.push(std::mem::take(&mut current));
                open = false;
            }
            Line::Blank if !open => {}
            other => {
                current.lines.push(other);
                open = true;
            }
        }
    }
    if open {
        doc.files.push(current);
    }
    Ok(doc)
}

/// Parses one line of LDraw text.
pub fn parse_line(raw: &str) -> Result<Line, String> {
    let text = raw.trim();
    let mut tokens = text.split_whitespace();
    let Some(kind) = tokens.next() else {
        return Ok(Line::Blank);
    };
    match kind {
        "0" => Ok(Line::Meta(parse_meta(text[1..].trim_start()))),
        "1" => {
            let color = parse_color(tokens.next())?;
            let n = numbers::<12>(&mut tokens)?;
            // The file name is the rest of the line after 14 tokens, spaces included.
            let file = rest_after_tokens(text, 14).ok_or("line type 1 has no file name")?;
            Ok(Line::Sub {
                color,
                pos: DVec3::new(n[0], n[1], n[2]),
                matrix: DMat3::from_cols_array(&[n[3], n[6], n[9], n[4], n[7], n[10], n[5], n[8], n[11]]),
                file: file.to_string(),
            })
        }
        "2" => {
            let color = parse_color(tokens.next())?;
            let n = numbers::<6>(&mut tokens)?;
            Ok(Line::Edge {
                color,
                points: points(&n),
            })
        }
        "3" => {
            let color = parse_color(tokens.next())?;
            let n = numbers::<9>(&mut tokens)?;
            Ok(Line::Tri {
                color,
                points: points(&n),
            })
        }
        "4" | "5" => {
            let color = parse_color(tokens.next())?;
            let n = numbers::<12>(&mut tokens)?;
            if kind == "4" {
                Ok(Line::Quad {
                    color,
                    points: points(&n),
                })
            } else {
                Ok(Line::Optional {
                    color,
                    points: points(&n),
                })
            }
        }
        other => Err(format!("unknown line type '{other}'")),
    }
}

fn parse_meta(body: &str) -> Meta {
    let mut words = body.split_whitespace();
    match words.next() {
        Some("FILE") => Meta::File(body[4..].trim().to_string()),
        Some("NOFILE") => Meta::NoFile,
        Some("STEP") if words.next().is_none() => Meta::Step,
        Some("!EMMET") => {
            let rest = body[6..].trim();
            let w: Vec<&str> = rest.split_whitespace().collect();
            match w.as_slice() {
                ["ID", id, "SRC", src] => Meta::EmmetPart {
                    id: id.to_string(),
                    src: src.to_string(),
                },
                _ => Meta::Emmet(rest.to_string()),
            }
        }
        _ => Meta::Comment(body.to_string()),
    }
}

fn parse_color(token: Option<&str>) -> Result<u32, String> {
    let t = token.ok_or("missing color")?;
    let parsed = match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => t.parse(),
    };
    parsed.map_err(|_| format!("bad color '{t}'"))
}

fn numbers<const N: usize>(tokens: &mut std::str::SplitWhitespace) -> Result<[f64; N], String> {
    let mut out = [0.0; N];
    for (i, slot) in out.iter_mut().enumerate() {
        let t = tokens
            .next()
            .ok_or_else(|| format!("expected {N} numbers, found {i}"))?;
        *slot = t.parse().map_err(|_| format!("bad number '{t}'"))?;
    }
    Ok(out)
}

fn points<const P: usize>(n: &[f64]) -> [DVec3; P] {
    std::array::from_fn(|i| DVec3::new(n[3 * i], n[3 * i + 1], n[3 * i + 2]))
}

fn rest_after_tokens(text: &str, count: usize) -> Option<&str> {
    let mut rest = text;
    for _ in 0..count {
        rest = rest.trim_start();
        let end = rest.find(char::is_whitespace)?;
        rest = &rest[end..];
    }
    let rest = rest.trim();
    (!rest.is_empty()).then_some(rest)
}

/// Writes a document as LDraw text with `\n` line ends. Named files get `0 FILE` and `0 NOFILE` lines.
pub fn write(doc: &Document) -> String {
    let mut out = String::new();
    for file in &doc.files {
        if let Some(name) = &file.name {
            let _ = writeln!(out, "0 FILE {name}");
        }
        for line in &file.lines {
            out.push_str(&write_line(line));
            out.push('\n');
        }
        if file.name.is_some() {
            out.push_str("0 NOFILE\n");
        }
    }
    out
}

pub fn write_line(line: &Line) -> String {
    match line {
        Line::Blank => String::new(),
        Line::Meta(meta) => match meta {
            Meta::File(name) => format!("0 FILE {name}"),
            Meta::NoFile => "0 NOFILE".into(),
            Meta::Step => "0 STEP".into(),
            Meta::EmmetPart { id, src } => format!("0 !EMMET ID {id} SRC {src}"),
            Meta::Emmet(rest) => format!("0 !EMMET {rest}"),
            Meta::Comment(text) if text.is_empty() => "0".into(),
            Meta::Comment(text) => format!("0 {text}"),
        },
        Line::Sub {
            color,
            pos,
            matrix,
            file,
        } => {
            let r = matrix.transpose().to_cols_array(); // rows a b c / d e f / g h i
            let mut s = format!("1 {}", color_str(*color));
            for v in pos.to_array().iter().chain(r.iter()) {
                s.push(' ');
                s.push_str(&num(*v));
            }
            format!("{s} {file}")
        }
        Line::Edge { color, points } => shape(2, *color, points),
        Line::Tri { color, points } => shape(3, *color, points),
        Line::Quad { color, points } => shape(4, *color, points),
        Line::Optional { color, points } => shape(5, *color, points),
    }
}

fn shape(kind: u8, color: u32, points: &[DVec3]) -> String {
    let mut s = format!("{kind} {}", color_str(color));
    for v in points.iter().flat_map(|p| p.to_array()) {
        s.push(' ');
        s.push_str(&num(v));
    }
    s
}

fn color_str(color: u32) -> String {
    if color >= 0x0200_0000 {
        format!("0x{color:07X}")
    } else {
        color.to_string()
    }
}

/// A number the way LDraw files write them: the shortest text that reads back as the same value, integers bare,
/// never `-0`. Float noise from arithmetic (more than 10 decimals) is rounded to 1e-9 first.
pub fn num(v: f64) -> String {
    let s = format!("{v}");
    let decimals = s.split_once('.').map_or(0, |(_, d)| d.len());
    let s = if decimals > 10 {
        format!("{}", (v * 1e9).round() / 1e9)
    } else {
        s
    };
    if s == "-0" { "0".into() } else { s }
}
