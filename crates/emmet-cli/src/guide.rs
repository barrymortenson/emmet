// SPDX-License-Identifier: AGPL-3.0-or-later
//! The build guides (session 3D, `doctrine/guides/`): what a scale or a style asks of a model, in rules an agent can
//! follow, and which techniques it reaches for. `emmet guide` and the MCP `guide` tool give the ones for the project's
//! `[build]` settings, or one by name. Each guide is Markdown: a `# Title` line, an `Also:` line of other names it
//! answers to, then its text.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use emmet_core::envelope::{Error, ErrorCode, Result};
use std::path::Path;

use emmet_core::project::{BuildConfig, Project};

/// Each guide: its name, whether it's a scale, a style, or a subject, and its text.
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

impl Kind {
    /// The kind as the text output and the JSON name it.
    fn word(self) -> &'static str {
        match self {
            Kind::Scale => "scale",
            Kind::Style => "style",
            Kind::Subject => "subject",
        }
    }
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

/// The guides asked for (by name, or for the project's scale and style), and the list of every guide when the settings
/// leave something to choose: none set, or one in the user's own words.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Guides {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guides: Vec<Guide>,
    /// Every guide, when `guides` doesn't cover what the project asks for.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub list: Vec<Summary>,
    /// What to do about the settings no guide covers: none set, or a scale or style in the user's own words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// What `emmet guide` and the MCP `guide` tool answer: the guide named (a blank name is no name), or the guides for
/// the build settings of the project at `project` (or found from the current directory). Only a search from the current
/// directory that finds no project means there are no settings; a project that can't be read is an error, never a
/// quiet "nothing is set".
pub fn answer(name: Option<&str>, project: Option<&Path>) -> Result<Guides> {
    if let Some(name) = name.map(str::trim).filter(|n| !n.is_empty()) {
        return named(name);
    }
    let build = match Project::find(project) {
        Ok(p) => Some(p.config.build),
        Err(e) if e.code == ErrorCode::ProjectNotFound && project.is_none() => None,
        Err(e) => return Err(e),
    };
    Ok(for_build(build.as_ref()))
}

/// The guide with name `name` (or any of the names its `Also:` line gives), of any kind.
pub fn named(name: &str) -> Result<Guides> {
    let Some(g) = find(name, &[Kind::Scale, Kind::Style, Kind::Subject]) else {
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
                "Did you mean {}? Every guide's name is in `guide`'s list (emmet guide outside a project).",
                names.join(", ")
            ),
        ));
    };
    Ok(Guides {
        guides: vec![g],
        list: vec![],
        note: None,
    })
}

/// The guides for a project's build settings (`None`: no project): the scale's (figure scale's when none is set, the
/// doctrine's default) and the style's. A setting no guide of its kind answers to is the user's own words: a note,
/// and the list to choose from.
pub fn for_build(build: Option<&BuildConfig>) -> Guides {
    let build = build.cloned().unwrap_or_default();
    // A setting is set when it has something besides spaces in it; the app trims them, a hand edit may not.
    let set = |v: &Option<String>| v.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(String::from);
    let (scale, style) = (set(&build.scale), set(&build.style));
    let mut out = Guides {
        guides: vec![],
        list: vec![],
        note: None,
    };
    let mut notes = vec![];
    if scale.is_none() {
        out.guides
            .push(find(DEFAULT_SCALE, &[Kind::Scale]).expect("the figure-scale guide"));
    }
    for (setting, value, kinds) in [
        ("scale", &scale, &[Kind::Scale][..]),
        ("style", &style, &[Kind::Style, Kind::Subject][..]),
    ] {
        let Some(value) = value else { continue };
        match find(value, kinds) {
            Some(g) if out.guides.iter().any(|h| h.name == g.name) => {}
            Some(g) => out.guides.push(g),
            None => notes.push(format!(
                "The {setting} \"{value}\" is the user's own words: read the closest guides from the list (guide with \
                 a name) and follow them where they fit."
            )),
        }
    }
    if scale.is_none() && style.is_none() {
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
    out
}

/// The scale the doctrine builds at when the settings and the prompt don't name one.
const DEFAULT_SCALE: &str = "figure-scale";

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

/// The guide of one of `kinds` that answers to `name`.
fn find(name: &str, kinds: &[Kind]) -> Option<Guide> {
    let want = key(name);
    GUIDES
        .iter()
        .filter(|(_, kind, _)| kinds.contains(kind))
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

/// The `Also:` line's names (the second line), comma-separated.
fn also(text: &str) -> Vec<&str> {
    text.lines()
        .nth(1)
        .and_then(|l| l.strip_prefix("Also: "))
        .map(|l| l.split(',').map(str::trim).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default()
}

/// The guide without its `Also:` line (the second line).
fn body(text: &str) -> String {
    text.lines()
        .enumerate()
        .filter(|(i, l)| !(*i == 1 && l.starts_with("Also: ")))
        .map(|(_, l)| l)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// The guides as text: each in full, then the list, then the note.
pub fn text(g: &Guides) -> String {
    let mut parts: Vec<String> = g.guides.iter().map(|g| g.text.clone()).collect();
    if !g.list.is_empty() {
        let lines: Vec<String> = g
            .list
            .iter()
            .map(|s| format!("{} ({}): {}", s.name, s.kind.word(), s.title))
            .collect();
        parts.push(format!("Guides:\n{}", lines.join("\n")));
    }
    if let Some(note) = &g.note {
        parts.push(note.clone());
    }
    parts.join("\n\n---\n\n")
}
