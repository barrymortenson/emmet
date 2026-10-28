// SPDX-License-Identifier: AGPL-3.0-or-later
//! The connector model (plan section 11): where a part's studs, anti-studs, holes, pins, bars, clips, hinge fingers,
//! and odd plugs are, in its own LDraw frame. Three sources, in order:
//!
//! 1. The primitive table ([`PRIMITIVES`]): top studs and underside tubes that the LDCad data has no entry for.
//! 2. The LDCad Shadow Library (`data/connections/ldcad-shadow.ldr`, CC BY-SA 4.0): `!LDCAD SNAP_*` lines for about
//!    4,000 parts, subparts, and primitives.
//! 3. Emmet's overrides (`data/connections/*.toml`), for parts the others get wrong or miss (track, the 9V switch).
//!
//! A file's connectors are LDCad's: those of every file it references, moved into its frame, then its own snap lines
//! in order (`SNAP_CLEAR` drops what came before, `SNAP_INCL` adds another shadow entry's). A file with no shadow
//! entry falls back to the primitive table. An override with `replace` takes the place of everything else.
//!
//! LDCad's convention holds throughout: a shape runs from its start along its own −Y (a stud from its base up, an
//! anti-stud from the part's underside up into the part), in sections of a radius and a length.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::OnceLock;

use glam::{DMat3, DVec3};

use crate::ldraw::geometry::Measurer;
use crate::ldraw::{Line, normalize_ref};

const SHADOW: &str = include_str!("../../../../data/connections/ldcad-shadow.ldr");

/// The override files, by name, so a test can check each one has its test model.
pub const OVERRIDES: &[(&str, &str)] = &[
    ("track.toml", include_str!("../../../../data/connections/track.toml")),
    ("hinges.toml", include_str!("../../../../data/connections/hinges.toml")),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Male,
    Female,
}

/// What kind of shape a connector is (LDCad's four snap metas).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A cylinder: studs, anti-studs, tubes, pins and pin holes, axles and axle holes, bars.
    Cyl,
    /// A clip, which holds a male cylinder of its radius (a bar, a handle).
    Clip,
    /// Hinge fingers, which mesh with fingers of the same radius.
    Finger,
    /// An odd shape that matches only its own group (an electric plug, a window's glass).
    Gen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    Primitive,
    Shadow,
    Override,
}

/// A section's profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Round,
    Axle,
    Square,
    /// LDCad's `_L` and `L_`: a slightly wider tip that compresses (a pin's ridge); never a reason to reject a match.
    Flex,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Section {
    pub shape: Shape,
    pub radius: f64,
    pub len: f64,
}

/// How a connector survives a scaled reference to its file (LDCad's `scale`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scale {
    None,
    YOnly,
    ROnly,
    YandR,
}

/// One connector in a part's LDraw frame.
#[derive(Debug, Clone)]
pub struct Connector {
    pub kind: Kind,
    pub gender: Gender,
    /// Where the shape starts.
    pub pos: DVec3,
    /// The unit direction the shape runs from its start (LDCad's −Y of the snap's orientation).
    pub axis: DVec3,
    /// A unit direction across the axis (LDCad's X), for grids and generic shapes.
    pub across: DVec3,
    /// Along the axis: a cylinder's sections, a clip's one, a finger set's fingers (their widths, radius each).
    pub secs: Rc<[Section]>,
    /// Whether the shape's start and end are open (a stud's base is closed, its tip open).
    pub open: (bool, bool),
    pub slide: bool,
    /// A generic shape that mates in any orientation (LDCad's `placement=free`, or a sphere's bounding: a ball joint).
    pub free: bool,
    pub group: Option<Rc<str>>,
    pub source: Source,
    /// LDCad's id, which a `SNAP_CLEAR` can name.
    id: Option<Rc<str>>,
    scale: Scale,
}

impl Connector {
    pub fn len(&self) -> f64 {
        self.secs.iter().map(|s| s.len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.secs.is_empty()
    }

    /// The connector under a placement's matrix: its rotation (scale taken out, mirroring kept) and translation.
    pub fn transformed_world(&self, m: DMat3, t: DVec3) -> Option<Connector> {
        self.transformed(rotation(m), t)
    }

    /// The connector under `m · p + t`, or None when the reference scales it in a way it doesn't allow.
    fn transformed(&self, m: DMat3, t: DVec3) -> Option<Connector> {
        let axis = m * self.axis;
        let across = m * self.across;
        let side = m * self.axis.cross(self.across);
        let (ya, ra, rb) = (axis.length(), across.length(), side.length());
        let unit = |v: f64| (v - 1.0).abs() < 1e-3;
        let y_only = unit(ra) && unit(rb);
        let r_only = unit(ya) && (ra - rb).abs() < 1e-3;
        let allowed = (unit(ya) && unit(ra) && unit(rb))
            || match self.scale {
                Scale::None => false,
                Scale::YOnly => y_only,
                Scale::ROnly => r_only,
                Scale::YandR => y_only || r_only,
            };
        if !allowed || ya < 1e-9 || ra < 1e-9 {
            return None;
        }
        let secs = if unit(ya) && unit(ra) {
            self.secs.clone()
        } else {
            self.secs
                .iter()
                .map(|s| Section {
                    shape: s.shape,
                    radius: s.radius * ra,
                    len: s.len * ya,
                })
                .collect()
        };
        Some(Connector {
            pos: m * self.pos + t,
            axis: axis / ya,
            across: across / ra,
            secs,
            group: self.group.clone(),
            id: self.id.clone(),
            ..*self
        })
    }
}

// ---------------------------------------------------------------------------------------------------------------
// LDCad's meta lines

/// One `!LDCAD SNAP_*` line.
#[derive(Debug, Clone)]
enum Snap {
    /// A connector, with its grid of copies (offsets across its frame) already expanded.
    Add(Vec<Connector>),
    /// `SNAP_INCL`: another shadow entry's snaps under a transform, once per grid offset.
    Include {
        reference: String,
        m: DMat3,
        offsets: Vec<DVec3>,
    },
    /// `SNAP_CLEAR`: drop what came before, all of it or one id.
    Clear(Option<String>),
}

/// The `[key=value]` pairs of a meta line, keys lowercased.
fn props(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut rest = text;
    while let Some(start) = rest.find('[') {
        let Some(end) = rest[start..].find(']') else { break };
        let inner = &rest[start + 1..start + end];
        if let Some((k, v)) = inner.split_once('=') {
            out.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
        rest = &rest[start + end + 1..];
    }
    out
}

fn numbers(s: &str) -> Vec<f64> {
    s.split_whitespace().filter_map(|t| t.parse().ok()).collect()
}

fn vec3(s: Option<&String>) -> DVec3 {
    let n = s.map(|s| numbers(s)).unwrap_or_default();
    if n.len() == 3 {
        DVec3::new(n[0], n[1], n[2])
    } else {
        DVec3::ZERO
    }
}

/// An LDraw matrix `a b c d e f g h i` (rows).
fn mat3(s: Option<&String>) -> DMat3 {
    let n = s.map(|s| numbers(s)).unwrap_or_default();
    if n.len() == 9 {
        DMat3::from_cols(
            DVec3::new(n[0], n[3], n[6]),
            DVec3::new(n[1], n[4], n[7]),
            DVec3::new(n[2], n[5], n[8]),
        )
    } else {
        DMat3::IDENTITY
    }
}

/// A grid (`C 4 C 2 20 20`: counts along X and Z, each centered with `C`, then the steps), as offsets in the snap's
/// own X and Z.
fn grid(s: Option<&String>) -> Vec<(f64, f64)> {
    let Some(s) = s else { return vec![(0.0, 0.0)] };
    let mut tokens = s.split_whitespace().peekable();
    let count = |tokens: &mut std::iter::Peekable<std::str::SplitWhitespace>| {
        let centered = tokens.peek().is_some_and(|t| t.eq_ignore_ascii_case("c"));
        if centered {
            tokens.next();
        }
        (
            tokens
                .next()
                .and_then(|t| t.parse::<f64>().ok())
                .unwrap_or(1.0)
                .max(1.0) as usize,
            centered,
        )
    };
    let (nx, cx) = count(&mut tokens);
    let (nz, cz) = count(&mut tokens);
    let sx: f64 = tokens.next().and_then(|t| t.parse().ok()).unwrap_or(0.0);
    let sz: f64 = tokens.next().and_then(|t| t.parse().ok()).unwrap_or(0.0);
    let at = |i: usize, n: usize, c: bool, step: f64| {
        if c {
            (i as f64 - (n as f64 - 1.0) / 2.0) * step
        } else {
            i as f64 * step
        }
    };
    let mut out = Vec::with_capacity(nx * nz);
    for i in 0..nx.min(64) {
        for k in 0..nz.min(64) {
            out.push((at(i, nx, cx, sx), at(k, nz, cz, sz)));
        }
    }
    out
}

fn sections(s: &str) -> Vec<Section> {
    let t: Vec<&str> = s.split_whitespace().collect();
    t.chunks(3)
        .filter(|c| c.len() == 3)
        .filter_map(|c| {
            let shape = match c[0].to_ascii_uppercase().as_str() {
                "R" => Shape::Round,
                "A" => Shape::Axle,
                "S" => Shape::Square,
                "_L" | "L_" => Shape::Flex,
                _ => return None,
            };
            Some(Section {
                shape,
                radius: c[1].parse().ok()?,
                len: c[2].parse().ok()?,
            })
        })
        .collect()
}

fn scale_mode(p: &HashMap<String, String>) -> Scale {
    match p.get("scale").map(|s| s.to_lowercase()).as_deref() {
        Some("yonly") => Scale::YOnly,
        Some("ronly") => Scale::ROnly,
        Some("yandr") => Scale::YandR,
        _ => Scale::None,
    }
}

fn truthy(p: &HashMap<String, String>, key: &str) -> bool {
    p.get(key).is_some_and(|v| v.eq_ignore_ascii_case("true"))
}

/// Parses one `!LDCAD SNAP_*` line (the text after `0 `). None for anything else, or a line it can't read.
fn parse_snap(line: &str, source: Source) -> Option<Snap> {
    let rest = line.trim().strip_prefix("!LDCAD")?.trim_start();
    let (meta, body) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let p = props(body);
    let ori = mat3(p.get("ori"));
    let pos = vec3(p.get("pos"));
    let id: Option<Rc<str>> = p.get("id").map(|s| Rc::from(s.to_lowercase().as_str()));
    let group: Option<Rc<str>> = p.get("group").map(|s| Rc::from(s.to_lowercase().as_str()));
    let offsets: Vec<DVec3> = grid(p.get("grid"))
        .into_iter()
        .map(|(x, z)| ori * DVec3::new(x, 0.0, z))
        .collect();
    let gender = |key: &str, default: Gender| match p.get(key).map(|g| g.to_uppercase()).as_deref() {
        Some("F") => Gender::Female,
        Some("M") => Gender::Male,
        _ => default,
    };
    let (kind, gender, secs, open) = match meta.to_ascii_uppercase().as_str() {
        "SNAP_CLEAR" => return Some(Snap::Clear(p.get("id").map(|s| s.to_lowercase()))),
        "SNAP_INCL" => {
            return Some(Snap::Include {
                reference: normalize_ref(p.get("ref")?),
                m: ori,
                offsets: offsets.into_iter().map(|o| o + pos).collect(),
            });
        }
        "SNAP_CYL" => {
            let g = gender("gender", Gender::Male);
            let secs = sections(p.get("secs")?);
            let open = match p.get("caps").map(|c| c.to_lowercase()).as_deref() {
                Some("none") => (true, true),
                Some("two") => (false, false),
                Some("a") => (false, true),
                Some("b") => (true, false),
                // "one", the default: a male's base, a female's far end.
                _ if g == Gender::Male => (false, true),
                _ => (true, false),
            };
            (Kind::Cyl, g, secs, open)
        }
        "SNAP_CLP" => {
            let radius = p.get("radius").and_then(|r| r.parse().ok()).unwrap_or(4.0);
            let len = p.get("length").and_then(|r| r.parse().ok()).unwrap_or(8.0);
            let s = Section {
                shape: Shape::Round,
                radius,
                len,
            };
            (Kind::Clip, Gender::Female, vec![s], (true, true))
        }
        "SNAP_FGR" => {
            // LDCad gives no default radius; fingers without one match only each other.
            let radius = p.get("radius").and_then(|r| r.parse().ok()).unwrap_or(0.0);
            let secs = numbers(p.get("seq")?)
                .into_iter()
                .map(|len| Section {
                    shape: Shape::Round,
                    radius,
                    len,
                })
                .collect();
            (Kind::Finger, gender("genderofs", Gender::Male), secs, (true, true))
        }
        "SNAP_GEN" => {
            // Its size, for the collision rule's seat: LDCad's bounding (`cyl r h`, `sph r`, `box x y z`).
            let b: Vec<String> = p
                .get("bounding")
                .map(|v| v.split_whitespace().map(str::to_lowercase).collect())
                .unwrap_or_default();
            let n: Vec<f64> = b.iter().skip(1).filter_map(|t| t.parse().ok()).collect();
            let (radius, len) = match (b.first().map(String::as_str), n.as_slice()) {
                (Some("cyl"), [r, h, ..]) => (*r, *h),
                (Some("sph"), [r, ..]) => (*r, 2.0 * r),
                (Some("box"), [x, y, z, ..]) => (x.max(*z), *y),
                _ => (0.0, 0.0),
            };
            (
                Kind::Gen,
                gender("gender", Gender::Male),
                vec![Section {
                    shape: Shape::Round,
                    radius,
                    len,
                }],
                (true, true),
            )
        }
        _ => return None,
    };
    if secs.is_empty() {
        return None;
    }
    let axis = (ori * DVec3::NEG_Y).normalize_or_zero();
    let across = (ori * DVec3::X).normalize_or_zero();
    if axis == DVec3::ZERO || across == DVec3::ZERO {
        return None;
    }
    let total: f64 = secs.iter().map(|s| s.len).sum();
    let start = if truthy(&p, "center") {
        pos - axis * total / 2.0
    } else {
        pos
    };
    let secs: Rc<[Section]> = secs.into();
    let one = Connector {
        kind,
        gender,
        pos: start,
        axis,
        across,
        secs,
        open,
        slide: truthy(&p, "slide"),
        free: kind == Kind::Gen
            && (p.get("placement").is_some_and(|v| v.eq_ignore_ascii_case("free"))
                || p.get("bounding").is_some_and(|v| v.to_lowercase().starts_with("sph"))),
        group,
        source,
        id,
        scale: scale_mode(&p),
    };
    Some(Snap::Add(
        offsets
            .iter()
            .map(|o| Connector {
                pos: one.pos + *o,
                ..one.clone()
            })
            .collect(),
    ))
}

/// The first connector of one snap line (tests).
#[cfg(test)]
pub(crate) fn parse_one(line: &str) -> Connector {
    match parse_snap(line, Source::Shadow) {
        Some(Snap::Add(v)) => v.into_iter().next().expect("one connector"),
        other => panic!("{line}: {other:?}"),
    }
}

/// The shadow entries by library reference (`3001.dat`, `s/3001s01.dat`, `stud.dat`), parts/ before p/ as in
/// LDraw's search order.
fn shadow() -> &'static HashMap<String, Vec<String>> {
    static SHADOW_INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    SHADOW_INDEX.get_or_init(|| index_ldr(SHADOW))
}

/// `0 FILE <dir>/<name>` blocks of `!LDCAD` lines, keyed by the name inside parts/ or p/.
fn index_ldr(text: &str) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    let mut current: Option<(String, bool)> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("0 FILE ") {
            let name = normalize_ref(name);
            let (key, parts) = if let Some(k) = name.strip_prefix("parts/") {
                (k.to_string(), true)
            } else {
                (name.strip_prefix("p/").unwrap_or(&name).to_string(), false)
            };
            // parts/ wins: a p/ entry never replaces one already read from parts/.
            if !parts && out.contains_key(&key) {
                current = None;
            } else {
                out.insert(key.clone(), Vec::new());
                current = Some((key, parts));
            }
        } else if line == "0 NOFILE" {
            current = None;
        } else if let (Some((key, _)), Some(meta)) = (&current, line.strip_prefix("0 ")) {
            out.get_mut(key).expect("current entry").push(meta.to_string());
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------------------------
// The primitive table

/// Emmet's own connectors for stud primitives the LDCad data has no entry for, by base name, as LDCad snap lines in
/// the primitive's frame. Top studs point up their own −Y from their base; underside tubes (`stud4` and its family,
/// drawn from y = −4 at the part's underside up to 0, and scaled to the part's height) take the four studs around
/// them, at the corners of a stud pitch, and a stud or a bar inside (a tube's 4.8 mm bore is a stud's width, a 2 x 2
/// plate centered on one stud; a 1 x 1 round plate's underside, drawn with the same tube, holds a bar).
pub const PRIMITIVES: &[(&[&str], &[&str])] = &[
    (
        // Top studs.
        &[
            "studa",
            "studh",
            "studhl",
            "studhr",
            "studx",
            "studxa",
            "stud-logo",
            "stud-logo2",
            "stud-logo3",
            "stud-logo4",
            "stud-logo5",
        ],
        &["!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 6 4]"],
    ),
    (
        // Open top studs: a stud outside, a bar hole inside.
        &[
            "stud9",
            "stud17",
            "stud2-logo",
            "stud2-logo2",
            "stud2-logo3",
            "stud2-logo4",
            "stud2-logo5",
        ],
        &[
            "!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 6 4]",
            "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 4 4] [pos=0 -4 0] [ori=1 0 0 0 -1 0 0 0 -1]",
        ],
    ),
    (
        // Underside tubes.
        &[
            "stud4", "stud4a", "stud4h", "stud4s", "stud4s2", "stud4oda", "stud4f1n", "stud4f1s", "stud4f1w",
            "stud4f2n", "stud4f2s", "stud4f2w", "stud4f3n", "stud4f3s", "stud4f4n", "stud4f4s", "stud4f5n", "stud16a",
            "stud16od", "stud21a", "stud22a", "stud23", "stud23d",
        ],
        &[
            "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 4] [scale=YOnly] [pos=0 -4 0] [ori=1 0 0 0 -1 0 0 0 -1] \
             [grid=C 2 C 2 20 20]",
            "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 4] [scale=YOnly] [pos=0 -4 0] [ori=1 0 0 0 -1 0 0 0 -1]",
            "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 4 4] [scale=YOnly] [pos=0 -4 0] [ori=1 0 0 0 -1 0 0 0 -1]",
        ],
    ),
];

fn primitive(base: &str) -> Option<&'static [&'static str]> {
    PRIMITIVES
        .iter()
        .find(|(names, _)| names.contains(&base))
        .map(|(_, lines)| *lines)
}

// ---------------------------------------------------------------------------------------------------------------
// Overrides

/// One part's override.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Override {
    /// The part (or subpart, `s/…`), without `.dat`.
    pub part: String,
    /// Why the other sources are wrong or short.
    pub reason: String,
    /// A model in `tests/fixtures/validator/connections/` that needs this override to validate.
    pub test: String,
    /// LDCad snap lines in the part's frame.
    pub snaps: Vec<String>,
    /// Use only these snaps, nothing the part inherits or the shadow data has.
    #[serde(default)]
    pub replace: bool,
}

#[derive(serde::Deserialize)]
struct OverrideFile {
    #[serde(rename = "override", default)]
    overrides: Vec<Override>,
}

/// Every override, from every file in [`OVERRIDES`]; a file that doesn't parse is a bug its test catches.
pub fn overrides() -> &'static HashMap<String, Override> {
    static ALL: OnceLock<HashMap<String, Override>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut out = HashMap::new();
        for (name, text) in OVERRIDES {
            let file: OverrideFile = toml::from_str(text).unwrap_or_else(|e| panic!("data/connections/{name}: {e}"));
            for o in file.overrides {
                out.insert(format!("{}.dat", normalize_ref(&o.part)), o);
            }
        }
        out
    })
}

// ---------------------------------------------------------------------------------------------------------------
// The scanner

const MAX_DEPTH: usize = 32;

/// Finds the connectors of parts, caching each file's in its own frame.
pub struct Scanner<'a> {
    measurer: Measurer<'a>,
    files: HashMap<String, Rc<Vec<Connector>>>,
    /// Shadow entries' own snaps, expanded (for `SNAP_INCL`).
    includes: HashMap<String, Rc<Vec<Connector>>>,
}

impl<'a> Scanner<'a> {
    pub fn new(measurer: Measurer<'a>) -> Self {
        Self {
            measurer,
            files: HashMap::new(),
            includes: HashMap::new(),
        }
    }

    pub fn measurer(&mut self) -> &mut Measurer<'a> {
        &mut self.measurer
    }

    /// Every connector of a part file (`3001.dat`), in the part's frame. Empty for a part with none or a file that's
    /// missing.
    pub fn connectors(&mut self, reference: &str) -> Rc<Vec<Connector>> {
        self.file(&normalize_ref(reference), 0)
    }

    fn file(&mut self, key: &str, depth: usize) -> Rc<Vec<Connector>> {
        if let Some(found) = self.files.get(key) {
            return found.clone();
        }
        let mut out: Vec<Connector> = Vec::new();
        let over = overrides().get(key);
        if depth <= MAX_DEPTH && !over.is_some_and(|o| o.replace) {
            // Inherited: every referenced file's connectors, in this file's frame.
            if let Some(lines) = self.measurer.lines(key) {
                for line in lines.iter() {
                    if let Line::Sub { pos, matrix, file, .. } = line {
                        let child = self.file(&normalize_ref(file), depth + 1);
                        out.extend(child.iter().filter_map(|c| c.transformed(*matrix, *pos)));
                    }
                }
            }
            let base = key.rsplit('/').next().unwrap_or(key);
            let base = base.strip_suffix(".dat").unwrap_or(base);
            if let Some(snaps) = shadow().get(key) {
                let snaps = snaps.clone();
                self.apply(&mut out, &snaps, Source::Shadow, depth);
            } else if let Some(lines) = primitive(base) {
                let lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
                self.apply(&mut out, &lines, Source::Primitive, depth);
            }
        }
        if let Some(o) = over {
            let snaps = o.snaps.clone();
            self.apply(&mut out, &snaps, Source::Override, depth);
        }
        dedupe(&mut out);
        let out = Rc::new(out);
        self.files.insert(key.to_string(), out.clone());
        out
    }

    fn apply(&mut self, out: &mut Vec<Connector>, lines: &[String], source: Source, depth: usize) {
        for line in lines {
            match parse_snap(line, source) {
                Some(Snap::Add(cs)) => out.extend(cs),
                Some(Snap::Clear(None)) => out.clear(),
                Some(Snap::Clear(Some(id))) => out.retain(|c| c.id.as_deref() != Some(id.as_str())),
                Some(Snap::Include { reference, m, offsets }) => {
                    let inc = self.include(&reference, source, depth + 1);
                    for o in offsets {
                        out.extend(inc.iter().filter_map(|c| c.transformed(m, o)));
                    }
                }
                None => {}
            }
        }
    }

    /// A shadow entry's own snaps (and its includes'), without what its file inherits: LDCad's `SNAP_INCL`.
    fn include(&mut self, key: &str, source: Source, depth: usize) -> Rc<Vec<Connector>> {
        if let Some(found) = self.includes.get(key) {
            return found.clone();
        }
        let mut out = Vec::new();
        if depth <= MAX_DEPTH
            && let Some(snaps) = shadow().get(key)
        {
            let snaps = snaps.clone();
            self.apply(&mut out, &snaps, source, depth);
        }
        let out = Rc::new(out);
        self.includes.insert(key.to_string(), out.clone());
        out
    }
}

/// Drops connectors that repeat another exactly: the same kind, gender, place, axis, group, and sections (a tube's
/// anti-studs where the part's shadow entry lists them too). Two at one place that differ (a tube's short anti-stud and
/// the shadow's deeper one, or a turntable's pin hole) both stay.
fn dedupe(v: &mut Vec<Connector>) {
    let key = |c: &Connector| {
        let r = |x: f64| (x * 10.0).round() as i64;
        let (p, a) = (c.pos, c.axis);
        (
            c.kind,
            c.gender,
            [r(p.x), r(p.y), r(p.z)],
            [r(a.x), r(a.y), r(a.z)],
            c.group.clone(),
            c.secs.iter().map(|s| (r(s.radius), r(s.len))).collect::<Vec<_>>(),
        )
    };
    let mut seen = std::collections::HashSet::new();
    v.retain(|c| seen.insert(key(c)));
}

/// The rotation part of a placement's matrix with its scale taken out, for directions.
pub fn rotation(m: DMat3) -> DMat3 {
    let n = |v: DVec3| v.try_normalize().unwrap_or(v);
    DMat3::from_cols(n(m.x_axis), n(m.y_axis), n(m.z_axis))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(line: &str) -> Vec<Connector> {
        match parse_snap(line, Source::Shadow) {
            Some(Snap::Add(v)) => v,
            other => panic!("{line}: {other:?}"),
        }
    }

    #[test]
    fn a_centered_grid_of_anti_studs() {
        // 3001s01's anti-studs: a 4 x 2 grid of 20-deep holes at the underside, centered on the part.
        let v = snap("!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 20] [pos=0 24 0] [grid=C 4 C 2 20 20]");
        assert_eq!(v.len(), 8);
        let mut xs: Vec<i64> = v.iter().map(|c| c.pos.x as i64).collect();
        xs.sort();
        xs.dedup();
        assert_eq!(xs, vec![-30, -10, 10, 30]);
        assert!(
            v.iter()
                .all(|c| c.pos.y == 24.0 && c.axis == DVec3::NEG_Y && c.open == (true, false))
        );
        assert_eq!(v[0].len(), 20.0);
    }

    #[test]
    fn a_centered_shape_starts_half_its_length_back() {
        // A technic pin hole through a beam (connhole): 20 long, centered on its position, along +Z.
        let v = snap(
            "!LDCAD SNAP_CYL [gender=F] [caps=none] [secs=R 8 2 R 6 16 R 8 2] [center=true] [slide=true] \
             [pos=0 0 0] [ori=1 0 0 0 0 1 0 -1 0]",
        );
        let c = &v[0];
        assert!((c.axis - DVec3::Z).length() < 1e-9, "{:?}", c.axis);
        assert!((c.pos - DVec3::new(0.0, 0.0, -10.0)).length() < 1e-9, "{:?}", c.pos);
        assert!(c.slide && c.open == (true, true) && c.secs.len() == 3);
    }

    #[test]
    fn an_uncentered_grid_adds_to_the_position() {
        let v = snap("!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=S 6 4] [pos=-10 8 10] [grid=1 2 0 20]");
        let zs: Vec<f64> = v.iter().map(|c| c.pos.z).collect();
        assert_eq!(zs, vec![10.0, 30.0]);
    }

    #[test]
    fn a_reference_scaled_along_its_axis_keeps_a_y_only_connector() {
        // A tube scaled −5 in Y (a brick's height) keeps its anti-studs, 20 deep, pointing up from the underside.
        let tube = snap(
            "!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 4] [scale=YOnly] [pos=0 -4 0] [ori=1 0 0 0 -1 0 0 0 -1]",
        );
        let m = DMat3::from_diagonal(DVec3::new(1.0, -5.0, 1.0));
        let c = tube[0].transformed(m, DVec3::new(0.0, 4.0, 0.0)).expect("kept");
        assert_eq!((c.pos.y, c.len()), (24.0, 20.0));
        assert!((c.axis - DVec3::NEG_Y).length() < 1e-9);
        // A stud (scale none) scaled that way is dropped.
        let stud = snap("!LDCAD SNAP_CYL [gender=M] [caps=one] [secs=R 6 4]");
        assert!(stud[0].transformed(m, DVec3::ZERO).is_none());
    }

    #[test]
    fn dedupe_keeps_connectors_that_differ_at_one_place() {
        // 3680, a turntable base: its tube's short anti-stud, the shadow's deeper one, and its grouped pin hole, all
        // starting at its middle.
        let mut v = vec![
            parse_one("!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 2] [pos=0 8 0]"),
            parse_one("!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 4] [pos=0 8 0]"),
            parse_one(
                "!LDCAD SNAP_CYL [group=turnTablePin] [gender=F] [caps=none] [secs=R 6 2 R 4 2] [center=true] [pos=0 6 0]",
            ),
            parse_one("!LDCAD SNAP_CYL [gender=F] [caps=one] [secs=R 6 4] [pos=0 8 0]"),
        ];
        dedupe(&mut v);
        assert_eq!(v.len(), 3);
        assert!(v.iter().any(|c| c.group.as_deref() == Some("turntablepin")));
    }

    #[test]
    fn the_shadow_data_and_overrides_load() {
        let s = shadow();
        assert!(s.len() > 4000, "{} entries", s.len());
        assert!(s["stud.dat"].iter().any(|l| l.contains("SNAP_CYL")));
        assert!(s.contains_key("s/3001s01.dat"));
        // Every snap line of every entry parses.
        for (key, lines) in s {
            for l in lines {
                assert!(parse_snap(l, Source::Shadow).is_some(), "{key}: {l}");
            }
        }
        for o in overrides().values() {
            for l in &o.snaps {
                assert!(parse_snap(l, Source::Override).is_some(), "{}: {l}", o.part);
            }
        }
    }
}
