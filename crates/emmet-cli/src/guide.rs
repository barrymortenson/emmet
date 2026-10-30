// SPDX-License-Identifier: AGPL-3.0-or-later
//! The build guides (session 3D, `doctrine/guides/`): what a scale or a style asks of a model, in rules an agent can
//! follow, and which techniques it reaches for. `emmet guide` and the MCP `guide` tool give the ones for the project's
//! `[build]` settings, or one by name. Each guide is Markdown: a `# Title` line, an `Also:` line of other names it
//! answers to, then its text.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use emmet_core::envelope::{Error, ErrorCode, Result};
use emmet_core::project::BuildConfig;

/// Each guide: its name, whether it's a scale or a style, and its text.
const GUIDES: &[(&str, Kind, &str)] = &[
    ("micro", Kind::Scale, include_str!("../doctrine/guides/micro.md")),
    (
        "micropolis",
        Kind::Scale,
        include_str!("../doctrine/guides/micropolis.md"),
    ),
    ("midi", Kind::Scale, include_str!("../doctrine/guides/midi.md")),
    (
        "figure-scale",
        Kind::Scale,
        include_str!("../doctrine/guides/figure-scale.md"),
    ),
    ("large", Kind::Scale, include_str!("../doctrine/guides/large.md")),
    ("car-1-8", Kind::Scale, include_str!("../doctrine/guides/car-1-8.md")),
    ("town", Kind::Style, include_str!("../doctrine/guides/town.md")),
    ("medieval", Kind::Style, include_str!("../doctrine/guides/medieval.md")),
    ("blocky", Kind::Style, include_str!("../doctrine/guides/blocky.md")),
    (
        "video-game",
        Kind::Style,
        include_str!("../doctrine/guides/video-game.md"),
    ),
    (
        "dinosaur-park",
        Kind::Style,
        include_str!("../doctrine/guides/dinosaur-park.md"),
    ),
    ("space", Kind::Style, include_str!("../doctrine/guides/space.md")),
    ("pirate", Kind::Style, include_str!("../doctrine/guides/pirate.md")),
    ("modern", Kind::Style, include_str!("../doctrine/guides/modern.md")),
    ("fantasy", Kind::Style, include_str!("../doctrine/guides/fantasy.md")),
    (
        "sculpture",
        Kind::Style,
        include_str!("../doctrine/guides/sculpture.md"),
    ),
    (
        "vehicles",
        Kind::Subject,
        include_str!("../doctrine/guides/vehicles.md"),
    ),
    (
        "interiors",
        Kind::Subject,
        include_str!("../doctrine/guides/interiors.md"),
    ),
    (
        "landscaping",
        Kind::Subject,
        include_str!("../doctrine/guides/landscaping.md"),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// What the details are sized for (`[build] scale`).
    Scale,
    /// The look (`[build] style`).
    Style,
    /// A kind of thing in any style: vehicles, interiors, landscaping.
    Subject,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Summary {
    /// What `guide` takes.
    pub name: String,
    pub kind: Kind,
    /// Its title.
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Guide {
    pub name: String,
    pub kind: Kind,
    pub title: String,
    /// The whole guide, as Markdown.
    pub text: String,
}

/// The guides asked for (by name, or the project's scale and style), and the list of every guide when nothing named
/// one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Guides {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guides: Vec<Guide>,
    /// Every guide, when `guides` doesn't cover what the project asks for.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub list: Vec<Summary>,
    /// What to do about the settings no guide covers: none set, or a style in the user's own words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// One guide by name (any of the names its `Also:` line gives), or the guides for a project's build settings.
pub fn guides(name: Option<&str>, build: Option<&BuildConfig>) -> Result<Guides> {
    if let Some(name) = name {
        let Some(g) = find(name) else {
            let mut close: Vec<(usize, &str)> = GUIDES
                .iter()
                .map(|(n, _, _)| (emmet_core::text::edit_distance(&key(name), &key(n)), *n))
                .collect();
            close.sort();
            let names: Vec<String> = close.iter().take(3).map(|(_, n)| format!("'{n}'")).collect();
            return Err(Error::new(
                ErrorCode::ParamInvalid,
                format!("There's no guide '{name}'."),
                format!(
                    "Did you mean {}? emmet guide with no name lists them all.",
                    names.join(", ")
                ),
            ));
        };
        return Ok(Guides {
            guides: vec![g],
            list: vec![],
            note: None,
        });
    }
    let build = build.cloned().unwrap_or_default();
    let mut out = Guides {
        guides: vec![],
        list: vec![],
        note: None,
    };
    let mut notes = vec![];
    for (setting, value) in [("scale", &build.scale), ("style", &build.style)] {
        let Some(value) = value.as_deref().map(str::trim).filter(|v| !v.is_empty()) else {
            continue;
        };
        match find(value) {
            Some(g) => out.guides.push(g),
            None => notes.push(format!(
                "The {setting} \"{value}\" is the user's own words: read the closest guides from the list (guide with \
                 a name) and follow them where they fit."
            )),
        }
    }
    if build.scale.is_none() {
        // Any scale is figure scale unless the prompt names another (the doctrine's default), so its rules come along:
        // the live check's blocky house read only the style's guide and left its baseplate bare.
        out.guides
            .insert(0, find("figure-scale").expect("the figure-scale guide"));
    }
    if build.scale.is_none() && build.style.is_none() {
        notes.push(
            "No scale or style is set in emmet.toml's [build], so this is figure scale's guide. When the prompt names a \
             scale or style (\"a micro city\", \"blocky\", \"like a video game\"), call guide with it too."
                .into(),
        );
    }
    if !notes.is_empty() {
        out.note = Some(notes.join(" "));
        out.list = list();
    }
    Ok(out)
}

/// Every guide, by name, kind, and title.
pub fn list() -> Vec<Summary> {
    GUIDES
        .iter()
        .map(|(n, kind, text)| Summary {
            name: n.to_string(),
            kind: *kind,
            title: title(text),
        })
        .collect()
}

fn find(name: &str) -> Option<Guide> {
    let want = key(name);
    GUIDES
        .iter()
        .find(|(n, _, text)| key(n) == want || also(text).iter().any(|a| key(a) == want))
        .map(|(n, kind, text)| Guide {
            name: n.to_string(),
            kind: *kind,
            title: title(text),
            text: body(text),
        })
}

/// A name as it's compared: lowercase letters and digits only, so "Figure scale", "figure-scale", and "FIGURE_SCALE"
/// match.
fn key(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn title(text: &str) -> String {
    text.lines()
        .next()
        .and_then(|l| l.strip_prefix("# "))
        .unwrap_or("")
        .trim()
        .to_string()
}

/// The `Also:` line's names, comma-separated.
fn also(text: &str) -> Vec<&str> {
    text.lines()
        .find_map(|l| l.strip_prefix("Also: "))
        .map(|l| l.split(',').map(str::trim).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default()
}

/// The guide without its `Also:` line.
fn body(text: &str) -> String {
    text.lines()
        .filter(|l| !l.starts_with("Also: "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// The guides as text: each in full, or the list, then the note.
pub fn text(g: &Guides) -> String {
    let mut parts: Vec<String> = g.guides.iter().map(|g| g.text.clone()).collect();
    if !g.list.is_empty() {
        let lines: Vec<String> = g
            .list
            .iter()
            .map(|s| {
                let kind = match s.kind {
                    Kind::Scale => "scale",
                    Kind::Style => "style",
                    Kind::Subject => "subject",
                };
                format!("{} ({kind}): {}", s.name, s.title)
            })
            .collect();
        parts.push(format!("Guides:\n{}", lines.join("\n")));
    }
    if let Some(note) = &g.note {
        parts.push(note.clone());
    }
    parts.join("\n\n---\n\n")
}
