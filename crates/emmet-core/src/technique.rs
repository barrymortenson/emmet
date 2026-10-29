// SPDX-License-Identifier: AGPL-3.0-or-later
//! The technique library (plan section 08, data/techniques/): `emmet technique` and the MCP `technique` tool. Each
//! technique is a Python module that `m.use(name, …)` applies. Its docstring is its doc, its `PARTS` list names the
//! parts it uses, its `PROVENANCE` note says where the method came from (plan section 21, 3C; D-48), and its
//! `example(m)` function, which builds it on its own for its render and its test, is the snippet an agent copies.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::envelope::{Error, ErrorCode, Result};
use crate::python::TECHNIQUES;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Summary {
    /// What `m.use` takes.
    pub name: String,
    /// The doc's first line.
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Technique {
    pub name: String,
    pub title: String,
    /// The whole doc: what it builds, its arguments, and how to use it.
    pub doc: String,
    /// The body of its example: model code that uses it.
    pub snippet: String,
    /// The LDraw ids of the parts it uses (part helpers in it may pick other sizes for other arguments).
    pub parts: Vec<String>,
    /// Where its method came from: Emmet's own, and the instruction books it was read in, by set number. Never derived
    /// from an OMR model's file (D-48).
    pub provenance: String,
}

/// The list (no name), or one technique.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Techniques {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub techniques: Vec<Summary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technique: Option<Technique>,
}

pub fn techniques(name: Option<&str>) -> Result<Techniques> {
    let Some(name) = name else {
        let techniques = TECHNIQUES
            .iter()
            .map(|(n, text)| Summary {
                name: n.to_string(),
                title: doc(text).lines().next().unwrap_or("").to_string(),
            })
            .collect();
        return Ok(Techniques {
            techniques,
            technique: None,
        });
    };
    let Some((n, text)) = TECHNIQUES.iter().find(|(n, _)| *n == name.trim()) else {
        let mut close: Vec<(usize, &str)> = TECHNIQUES
            .iter()
            .map(|(n, _)| (crate::text::edit_distance(name, n), *n))
            .collect();
        close.sort();
        let names: Vec<String> = close.iter().take(3).map(|(_, n)| format!("'{n}'")).collect();
        return Err(Error::new(
            ErrorCode::ParamInvalid,
            format!("There's no technique '{name}'."),
            format!("Did you mean {}? emmet technique lists them all.", names.join(", ")),
        ));
    };
    let doc = doc(text);
    Ok(Techniques {
        techniques: vec![],
        technique: Some(Technique {
            name: n.to_string(),
            title: doc.lines().next().unwrap_or("").to_string(),
            snippet: snippet(text),
            parts: parts(text),
            provenance: provenance(text),
            doc,
        }),
    })
}

/// The module docstring.
fn doc(text: &str) -> String {
    let Some(start) = text.find("\"\"\"") else {
        return String::new();
    };
    let rest = &text[start + 3..];
    rest[..rest.find("\"\"\"").unwrap_or(rest.len())].trim().to_string()
}

/// `PROVENANCE = """..."""`, its lines joined.
fn provenance(text: &str) -> String {
    const OPEN: &str = "\nPROVENANCE = \"\"\"";
    let Some(start) = text.find(OPEN) else {
        return String::new();
    };
    let rest = &text[start + OPEN.len()..];
    let note = &rest[..rest.find("\"\"\"").unwrap_or(rest.len())];
    note.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The quoted ids in `PARTS = [...]`.
fn parts(text: &str) -> Vec<String> {
    let Some(start) = text.find("\nPARTS = [") else {
        return vec![];
    };
    let rest = &text[start + 10..];
    let list = &rest[..rest.find(']').unwrap_or(0)];
    list.split('"').skip(1).step_by(2).map(String::from).collect()
}

/// `example(m)`'s body, without its docstring, dedented.
fn snippet(text: &str) -> String {
    let mut lines = text.lines().skip_while(|l| !l.starts_with("def example("));
    lines.next();
    let mut body: Vec<&str> = lines.take_while(|l| l.is_empty() || l.starts_with(' ')).collect();
    while body.last().is_some_and(|l| l.trim().is_empty()) {
        body.pop();
    }
    // The example's own docstring, one line or several.
    if body.first().is_some_and(|l| l.trim_start().starts_with("\"\"\"")) {
        let first = body[0].trim();
        let ends = if first.len() > 3 && first.ends_with("\"\"\"") {
            0
        } else {
            body.iter()
                .skip(1)
                .position(|l| l.trim_end().ends_with("\"\"\""))
                .map_or(0, |i| i + 1)
        };
        body.drain(..=ends);
    }
    body.iter()
        .map(|l| l.strip_prefix("    ").unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
}
