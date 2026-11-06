// SPDX-License-Identifier: AGPL-3.0-or-later
//! The pages, by section 08's step rules, as a display list in points from the page's top left (y down): a cover, the
//! steps (the step number over its parts box, the build with its new parts outlined, a sub-assembly in its own box
//! with numbered sub-steps and a line to where it goes, arrows, zoom boxes, the turn icon), the parts list, and page
//! numbers. [`super::pdf`] writes it; the E2E page baselines draw the same list.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::font;
use super::{Drawn, DrawnStep, Picture, Sources, names, part_index};
use crate::envelope::{Error, Result};
use crate::steps::{BoxLine, Step};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Paper {
    A4,
    Letter,
}

impl Paper {
    /// Width and height in points.
    pub fn size(self) -> (f64, f64) {
        match self {
            Paper::A4 => (595.28, 841.89),
            Paper::Letter => (612.0, 792.0),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Paper::A4 => "a4",
            Paper::Letter => "letter",
        }
    }

    /// Letter where it's the usual paper (the US, Canada, Mexico, and a few more), A4 everywhere else, from a locale
    /// like "en-US" or "en_US.UTF-8".
    pub fn for_locale(locale: &str) -> Paper {
        // The region is the two-letter subtag after the language (and script), before any extension ("-u-…"):
        // "en-US", "zh-Hant-TW", "en-US-u-ca-gregory". None ("en", "C") is A4.
        let region = locale
            .split(['.', '@'])
            .next()
            .unwrap_or("")
            .split(['-', '_'])
            .skip(1)
            .take_while(|t| t.len() > 1)
            .find(|t| t.len() == 2 && t.chars().all(|c| c.is_ascii_alphabetic()))
            .unwrap_or("")
            .to_ascii_uppercase();
        if [
            "US", "CA", "MX", "PH", "CL", "CO", "VE", "GT", "CR", "PR", "DO", "SV", "PA",
        ]
        .contains(&region.as_str())
        {
            Paper::Letter
        } else {
            Paper::A4
        }
    }
}

pub type Rgb = [u8; 3];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Start,
    Middle,
    End,
}

/// One thing drawn on a page, in points from its top left.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Item {
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        radius: f64,
        fill: Option<Rgb>,
        stroke: Option<Rgb>,
        width: f64,
    },
    /// A line through the points, or a closed shape.
    Path {
        points: Vec<[f64; 2]>,
        closed: bool,
        fill: Option<Rgb>,
        stroke: Option<Rgb>,
        width: f64,
    },
    /// Helvetica (or bold), `y` at the baseline.
    Text {
        x: f64,
        y: f64,
        size: f64,
        bold: bool,
        color: Rgb,
        align: Align,
        text: String,
    },
    Image {
        image: usize,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Page {
    /// The number printed at its foot (none on the cover).
    pub number: Option<u32>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Book {
    pub paper: Paper,
    pub width: f64,
    pub height: f64,
    pub pages: Vec<Page>,
    /// Every picture once (a part's is shared by every parts box it's in).
    pub images: Vec<Picture>,
}

// The look, in points. The colors print well in gray too: the boxes are light, the lines and numbers dark.
const MARGIN: f64 = 32.0;
const FOOT: f64 = 24.0;
const GAP: f64 = 16.0;
const INK: Rgb = [29, 31, 34];
const MUTED: Rgb = [96, 102, 110];
const WHITE: Rgb = [255, 255, 255];
/// The parts box, and every part's picture behind it.
pub const BOX_FILL: Rgb = [223, 234, 244];
const BOX_LINE: Rgb = [150, 174, 196];
/// A sub-assembly's box, and its sub-steps' pictures behind it.
pub const CALLOUT_FILL: Rgb = [241, 245, 249];
const CALLOUT_LINE: Rgb = [92, 118, 142];
const ARROW: Rgb = [232, 128, 20];
const PIC: f64 = 30.0;
const COUNT: f64 = 9.0;
const BOX_PAD: f64 = 5.0;
const NUMBER: f64 = 32.0;
const SUB_NUMBER: f64 = 18.0;
const NUMBER_COLUMN: f64 = 54.0;
const SUB_NUMBER_COLUMN: f64 = 26.0;
const CALLOUT_PAD: f64 = 10.0;
/** A step's number and a one-row parts box over its picture: two such steps fit a page. */
const STEP_HEAD: f64 = NUMBER + 4.0 + 2.0 * BOX_PAD + PIC + COUNT + 3.0 + 6.0;

/// Images, each once.
struct Images {
    list: Vec<Picture>,
    parts: HashMap<usize, usize>,
}

impl Images {
    fn add(&mut self, p: &Picture) -> usize {
        self.list.push(p.clone());
        self.list.len() - 1
    }

    fn part(&mut self, drawn: &Drawn, i: usize) -> Option<usize> {
        if let Some(&at) = self.parts.get(&i) {
            return Some(at);
        }
        let at = self.add(drawn.parts.get(i)?);
        self.parts.insert(i, at);
        Some(at)
    }
}

/// Something placed on a page, measured before it's placed.
struct Block {
    w: f64,
    h: f64,
    /// Drawn at (x, y).
    items: Vec<Item>,
    /// Where each sub-assembly copy the block's step puts on is, from the block's top left.
    copies: Vec<(String, f64, f64)>,
}

impl Block {
    fn at(&self, x: f64, y: f64) -> Vec<Item> {
        self.items.iter().map(|i| shift(i, x, y)).collect()
    }
}

fn shift(item: &Item, dx: f64, dy: f64) -> Item {
    let mut i = item.clone();
    match &mut i {
        Item::Rect { x, y, .. } | Item::Text { x, y, .. } | Item::Image { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Item::Path { points, .. } => {
            for p in points {
                p[0] += dx;
                p[1] += dy;
            }
        }
    }
    i
}

fn text(x: f64, y: f64, size: f64, bold: bool, color: Rgb, align: Align, s: &str) -> Item {
    Item::Text {
        x,
        y,
        size,
        bold,
        color,
        align,
        text: s.to_string(),
    }
}

/// What lays a step out: the pictures, the parts' names, and the scale.
struct Ctx<'a> {
    drawn: &'a Drawn,
    images: Images,
    parts: std::collections::BTreeMap<(String, u32), usize>,
}

impl Ctx<'_> {
    /// A parts box at (0, 0) no wider than `max`: each part's picture over its count, larger parts first (steps.json's
    /// order), wrapping to more rows when it must.
    fn parts_box(&mut self, lines: &[BoxLine], max: f64) -> (Vec<Item>, f64, f64) {
        if lines.is_empty() {
            return (Vec::new(), 0.0, 0.0);
        }
        let cell = PIC + 6.0;
        let per_row = (((max - 2.0 * BOX_PAD) / cell).floor() as usize).max(1);
        let rows = lines.len().div_ceil(per_row);
        let cols = lines.len().min(per_row);
        let w = 2.0 * BOX_PAD + cols as f64 * cell - 6.0;
        let row_h = PIC + COUNT + 3.0;
        let h = 2.0 * BOX_PAD + rows as f64 * row_h;
        let mut items = vec![Item::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
            radius: 4.0,
            fill: Some(BOX_FILL),
            stroke: Some(BOX_LINE),
            width: 0.75,
        }];
        for (i, l) in lines.iter().enumerate() {
            let (c, r) = (i % per_row, i / per_row);
            let x = BOX_PAD + c as f64 * cell;
            let y = BOX_PAD + r as f64 * row_h;
            if let Some(img) = self
                .parts
                .get(&(l.part.clone(), l.color))
                .copied()
                .and_then(|p| self.images.part(self.drawn, p))
            {
                items.push(Item::Image {
                    image: img,
                    x,
                    y,
                    w: PIC,
                    h: PIC,
                });
            }
            items.push(text(
                x + PIC / 2.0,
                y + PIC + COUNT,
                COUNT,
                true,
                INK,
                Align::Middle,
                &format!("{}x", l.count),
            ));
        }
        (items, w, h)
    }

    /// One step: its number (the turn icon beside it), its parts box under it, then the build at `scale` points per LDU,
    /// with a zoom box beside or over it.
    fn step(&mut self, step: &Step, d: &DrawnStep, scale: f64, turned: bool, sub: bool, max_w: f64) -> Block {
        let (number, column) = if sub {
            (SUB_NUMBER, SUB_NUMBER_COLUMN)
        } else {
            (NUMBER, NUMBER_COLUMN)
        };
        // The step number at the top, its parts box under it (Barry 2026-11-05), then the build. The number's text
        // goes into the page after the box's counts, so a page reads each box's counts, then its step.
        let top = number + 4.0;
        let (parts, bw, bh) = self.parts_box(&step.parts, max_w);
        let mut items: Vec<Item> = parts.iter().map(|i| shift(i, 0.0, top)).collect();
        let head = top + if bh > 0.0 { bh + 6.0 } else { 0.0 };
        let (pw, ph) = picture_size(d, scale, max_w - column);
        items.push(text(
            0.0,
            number * 0.86,
            number,
            false,
            INK,
            Align::Start,
            &step.n.to_string(),
        ));
        if turned {
            let x = font::width(&step.n.to_string(), number, false) + 10.0;
            items.extend(turn_icon(x, (number - 24.0) / 2.0, 24.0));
        }
        let (px, py) = (0.0, head);
        let image = self.images.add(&d.picture);
        items.push(Item::Image {
            image,
            x: px,
            y: py,
            w: pw,
            h: ph,
        });
        let sx = pw / f64::from(d.picture.width.max(1));
        let sy = ph / f64::from(d.picture.height.max(1));
        for a in &d.arrows {
            items.extend(arrow(
                [px + a.from[0] * sx, py + a.from[1] * sy],
                [px + a.to[0] * sx, py + a.to[1] * sy],
                3.0,
                ARROW,
            ));
        }
        let mut reach = pw;
        let mut bottom = head + ph;
        if let Some(z) = &d.zoom {
            // Beside the picture, and no taller than it, when there's room; else over the picture's corner farthest
            // from what it shows. Either way the block holds it: a box past the block ran out of a sub-assembly's box.
            let aspect = f64::from(z.picture.height) / f64::from(z.picture.width.max(1));
            let free = (max_w - pw - 10.0).min(170.0).min(ph / aspect);
            let beside = free >= 90.0;
            let zw = if beside {
                free
            } else {
                (pw * 0.34).clamp(70.0, 170.0).min((ph - 8.0).max(40.0) / aspect)
            };
            let zh = zw * aspect;
            let at = [px + z.x * sx, py + z.y * sy];
            let (zx, zy) = if beside {
                reach += 10.0 + zw;
                (px + pw + 10.0, py)
            } else {
                let corners = [
                    (px + 4.0, py + 4.0),
                    (px + pw - zw - 4.0, py + 4.0),
                    (px + 4.0, py + ph - zh - 4.0),
                    (px + pw - zw - 4.0, py + ph - zh - 4.0),
                ];
                let far = |c: &(f64, f64)| (c.0 + zw / 2.0 - at[0]).hypot(c.1 + zh / 2.0 - at[1]);
                *corners
                    .iter()
                    .max_by(|a, b| far(a).total_cmp(&far(b)))
                    .expect("corners")
            };
            // The line runs from the box's center; the box, drawn over it, leaves the part outside it.
            items.push(Item::Path {
                points: vec![[zx + zw / 2.0, zy + zh / 2.0], at],
                closed: false,
                fill: None,
                stroke: Some(CALLOUT_LINE),
                width: 1.0,
            });
            items.push(Item::Rect {
                x: zx - 3.0,
                y: zy - 3.0,
                w: zw + 6.0,
                h: zh + 6.0,
                radius: 4.0,
                fill: Some(WHITE),
                stroke: Some(CALLOUT_LINE),
                width: 1.5,
            });
            let image = self.images.add(&z.picture);
            items.push(Item::Image {
                image,
                x: zx,
                y: zy,
                w: zw,
                h: zh,
            });
            bottom = bottom.max(zy + zh + 3.0);
        }
        let copies = d
            .copies
            .iter()
            .map(|s| (s.id.clone(), px + s.x * sx, py + s.y * sy))
            .collect();
        Block {
            w: reach.max(bw),
            h: bottom,
            items,
            copies,
        }
    }
}

/// The picture's size at `scale` points per LDU, no wider than `max_w`.
fn picture_size(d: &DrawnStep, scale: f64, max_w: f64) -> (f64, f64) {
    let ldu = d.ldu.max(1e-6);
    let mut w = f64::from(d.picture.width) / ldu * scale;
    let mut h = f64::from(d.picture.height) / ldu * scale;
    if w > max_w {
        h *= max_w / w;
        w = max_w;
    }
    (w, h)
}

/// An arrow from `from` to `to`: a shaft and a filled head.
fn arrow(from: [f64; 2], to: [f64; 2], width: f64, color: Rgb) -> Vec<Item> {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt().max(1e-6);
    let (ux, uy) = (dx / len, dy / len);
    let head = width * 3.2;
    let base = [to[0] - ux * head, to[1] - uy * head];
    vec![
        Item::Path {
            points: vec![from, base],
            closed: false,
            fill: None,
            stroke: Some(color),
            width,
        },
        Item::Path {
            points: vec![
                to,
                [base[0] - uy * head * 0.6, base[1] + ux * head * 0.6],
                [base[0] + uy * head * 0.6, base[1] - ux * head * 0.6],
            ],
            closed: true,
            fill: Some(color),
            stroke: None,
            width: 0.0,
        },
    ]
}

/// The turn icon: a circular arrow, `size` across, its top left at (x, y).
fn turn_icon(x: f64, y: f64, size: f64) -> Vec<Item> {
    let (cx, cy, r) = (x + size / 2.0, y + size / 2.0, size * 0.38);
    let points: Vec<[f64; 2]> = (0..=18)
        .map(|i| {
            let a = std::f64::consts::PI * (0.15 + 1.45 * f64::from(i) / 18.0);
            [cx + r * a.cos(), cy - r * a.sin() * 0.6]
        })
        .collect();
    let end = *points.last().expect("points");
    let mut items = vec![Item::Path {
        points,
        closed: false,
        fill: None,
        stroke: Some(INK),
        width: 2.2,
    }];
    items.push(Item::Path {
        points: vec![
            [end[0] + 5.0, end[1] + 1.0],
            [end[0] - 3.0, end[1] - 4.5],
            [end[0] - 2.0, end[1] + 4.5],
        ],
        closed: true,
        fill: Some(INK),
        stroke: None,
        width: 0.0,
    });
    items
}

/// A row on a page: blocks side by side, or a slice of a sub-assembly's box.
struct Row {
    h: f64,
    items: Vec<Item>,
    copies: Vec<(String, f64, f64)>,
    /// A slice of this callout (its index in the book's callouts), drawn inside its box.
    callout: Option<usize>,
    /// The callout this step puts on: the line runs from its box to the copy, when both are on the page.
    puts_on: Vec<(usize, String)>,
    /// The callout whose last slice this is: kept on the page of the step after it.
    keep_with_next: bool,
}

/// A sub-assembly's box: the "2x" for its copies.
struct Callout {
    count: usize,
}

/// Lays the book out.
pub fn book(src: &Sources, paper: Paper) -> Result<Book> {
    let (pw, ph) = paper.size();
    let content_w = pw - 2.0 * MARGIN;
    let content_h = ph - 2.0 * MARGIN - FOOT;
    let drawn = src.drawn;
    if drawn.runs.len() != src.jobs.runs.len() || drawn.parts.len() != src.jobs.parts.len() {
        return Err(Error::internal(
            "The webview drew a different set of pictures than the book asked for.",
        ));
    }
    let mut ctx = Ctx {
        drawn,
        images: Images {
            list: Vec::new(),
            parts: HashMap::new(),
        },
        parts: part_index(src.jobs),
    };
    let main = &drawn.runs[0];
    if main.len() != src.steps.steps.len() {
        return Err(Error::internal(format!(
            "The webview drew {} steps, and the model has {}.",
            main.len(),
            src.steps.steps.len()
        )));
    }
    // Each step's picture as big as fits the page's width and a share of its height (two steps to a page, as a
    // rule); the steps of one camera run share a crop, so they share a size.
    let fit = |d: &DrawnStep| {
        let ldu = d.ldu.max(1e-6);
        let (w, h) = (f64::from(d.picture.width) / ldu, f64::from(d.picture.height) / ldu);
        ((content_w - NUMBER_COLUMN) / w.max(1e-6)).min(((content_h - GAP) / 2.0 - STEP_HEAD) / h.max(1e-6))
    };

    let mut pages = vec![cover(src, &mut ctx, pw, ph)];
    let mut rows: Vec<Row> = Vec::new();
    let mut callouts: Vec<Callout> = Vec::new();
    let mut pending: Option<Block> = None;
    let mut last_turn = main.first().map_or(0, |d| d.turn);
    let half = (content_w - GAP) / 2.0;
    let flush = |rows: &mut Vec<Row>, pending: &mut Option<Block>| {
        if let Some(b) = pending.take() {
            rows.push(Row {
                h: b.h,
                items: b.at(0.0, 0.0),
                copies: b.copies,
                callout: None,
                puts_on: Vec::new(),
                keep_with_next: false,
            });
        }
    };
    for (step, d) in src.steps.steps.iter().zip(main) {
        if step.n != d.n {
            return Err(Error::internal(format!(
                "The webview drew step {} where step {} goes.",
                d.n, step.n
            )));
        }
        let turned = d.turn != last_turn;
        last_turn = d.turn;
        // The sub-assemblies this step puts on, each in its box, just before it.
        let mut puts_on = Vec::new();
        for placed in &step.submodels {
            flush(&mut rows, &mut pending);
            let Some((at, sub_rows)) = sub_assembly(
                &mut ctx,
                src,
                &placed.file,
                placed.count,
                content_w,
                fit(d),
                &mut callouts,
                0,
            )?
            else {
                continue;
            };
            rows.extend(sub_rows);
            if let Some(last) = rows.last_mut() {
                last.keep_with_next = true;
            }
            for id in &placed.ids {
                puts_on.push((at, id.clone()));
            }
        }
        let block = ctx.step(step, d, fit(d), turned, false, content_w);
        if !puts_on.is_empty() {
            flush(&mut rows, &mut pending);
            rows.push(Row {
                h: block.h,
                items: block.at(0.0, 0.0),
                copies: block.copies,
                callout: None,
                puts_on,
                keep_with_next: false,
            });
            continue;
        }
        // Two small steps share a row.
        match pending.take() {
            Some(prev) if block.w <= half && prev.w <= half => {
                let mut items = prev.at(0.0, 0.0);
                items.extend(block.at(half + GAP, 0.0));
                let mut copies = prev.copies;
                copies.extend(block.copies.into_iter().map(|(id, x, y)| (id, x + half + GAP, y)));
                rows.push(Row {
                    h: prev.h.max(block.h),
                    items,
                    copies,
                    callout: None,
                    puts_on: Vec::new(),
                    keep_with_next: false,
                });
            }
            Some(prev) => {
                pending = Some(prev);
                flush(&mut rows, &mut pending);
                pending = Some(block);
            }
            None => pending = Some(block),
        }
        if pending.as_ref().is_some_and(|b| b.w > half) {
            flush(&mut rows, &mut pending);
        }
    }
    flush(&mut rows, &mut pending);
    pages.extend(paginate(rows, &callouts, content_w, content_h));
    pages.extend(parts_pages(src, &mut ctx, content_w, content_h));
    for (i, page) in pages.iter_mut().enumerate().skip(1) {
        let n = i as u32 + 1;
        page.number = Some(n);
        let x = if n.is_multiple_of(2) { MARGIN } else { pw - MARGIN };
        let align = if n.is_multiple_of(2) { Align::Start } else { Align::End };
        page.items
            .push(text(x, ph - MARGIN + 4.0, 11.0, false, MUTED, align, &n.to_string()));
    }
    Ok(Book {
        paper,
        width: pw,
        height: ph,
        pages,
        images: ctx.images.list,
    })
}

/// A sub-assembly's box, `count` times: the boxes of the sub-assemblies its sub-steps put on first (a chair before the
/// table set it goes into), then its own. Its index in `callouts` and the rows, or None when the book has no pictures
/// of it.
#[allow(clippy::too_many_arguments)]
fn sub_assembly(
    ctx: &mut Ctx,
    src: &Sources,
    file: &str,
    count: usize,
    content_w: f64,
    scale: f64,
    callouts: &mut Vec<Callout>,
    depth: usize,
) -> Result<Option<(usize, Vec<Row>)>> {
    // A file that puts itself on would never end; LDraw forbids it, and so does this.
    if depth > 16 {
        return Ok(None);
    }
    let Some(run) = src.jobs.runs.iter().position(|r| r.file.eq_ignore_ascii_case(file)) else {
        return Ok(None);
    };
    let Some(model) = src.steps.submodels.iter().find(|m| m.file.eq_ignore_ascii_case(file)) else {
        return Ok(None);
    };
    let mut rows = Vec::new();
    for sub in &model.steps {
        for placed in &sub.submodels {
            if let Some((_, inner)) = sub_assembly(
                ctx,
                src,
                &placed.file,
                placed.count,
                content_w,
                scale,
                callouts,
                depth + 1,
            )? {
                rows.extend(inner);
            }
        }
    }
    let at = callouts.len();
    callouts.push(Callout { count });
    rows.extend(callout_rows(
        ctx,
        &model.steps,
        &ctx.drawn.runs[run],
        content_w,
        scale,
        at,
    )?);
    Ok(Some((at, rows)))
}

/// A sub-assembly's sub-steps as rows inside its box: each a small step block, several to a row.
fn callout_rows(
    ctx: &mut Ctx,
    steps: &[Step],
    drawn: &[DrawnStep],
    content_w: f64,
    main_scale: f64,
    at: usize,
) -> Result<Vec<Row>> {
    if drawn.len() != steps.len() {
        return Err(Error::internal(
            "The webview drew a sub-assembly with a different number of steps.",
        ));
    }
    let inner = content_w - 2.0 * CALLOUT_PAD;
    // Its largest sub-step a third of the box wide and at most 150 points tall, but no smaller than the step that puts
    // it on, so a small unit reads; that floor can make a sub-step wider (up to the box) or taller.
    let fit = drawn
        .iter()
        .map(|d| {
            let ldu = d.ldu.max(1e-6);
            let (w, h) = (f64::from(d.picture.width) / ldu, f64::from(d.picture.height) / ldu);
            ((inner / 3.0 - SUB_NUMBER_COLUMN - GAP) / w.max(1.0)).min(150.0 / h.max(1.0))
        })
        .fold(f64::INFINITY, f64::min);
    let scale = if fit.is_finite() {
        fit.max(main_scale)
    } else {
        main_scale
    };
    let blocks: Vec<Block> = steps
        .iter()
        .zip(drawn)
        .map(|(s, d)| ctx.step(s, d, scale, false, true, inner))
        .collect();
    let mut rows = Vec::new();
    let mut row: Vec<Block> = Vec::new();
    let mut used = 0.0;
    let push_row = |row: &mut Vec<Block>, rows: &mut Vec<Row>| {
        if row.is_empty() {
            return;
        }
        let mut items = Vec::new();
        let mut x = CALLOUT_PAD;
        let mut h: f64 = 0.0;
        for b in row.drain(..) {
            items.extend(b.at(x, CALLOUT_PAD));
            x += b.w + GAP;
            h = h.max(b.h);
        }
        rows.push(Row {
            h: h + 2.0 * CALLOUT_PAD,
            items,
            copies: Vec::new(),
            callout: Some(at),
            puts_on: Vec::new(),
            keep_with_next: false,
        });
    };
    for b in blocks {
        if !row.is_empty() && used + b.w > inner {
            push_row(&mut row, &mut rows);
            used = 0.0;
        }
        used += b.w + GAP;
        row.push(b);
    }
    push_row(&mut row, &mut rows);
    Ok(rows)
}

/// Rows onto pages: a callout's rows framed in one box per page, its last row on the page of the step it goes on,
/// and the line from the box to each copy when they share a page.
fn paginate(rows: Vec<Row>, callouts: &[Callout], content_w: f64, content_h: f64) -> Vec<Page> {
    let mut pages: Vec<Page> = Vec::new();
    let mut items: Vec<Item> = Vec::new();
    let mut y = 0.0;
    // Each callout's box on this page: (its callout, top, bottom).
    let mut boxes: Vec<(usize, f64, f64)> = Vec::new();
    let finish = |items: &mut Vec<Item>, boxes: &mut Vec<(usize, f64, f64)>, pages: &mut Vec<Page>| {
        let mut page = Vec::new();
        for &(c, top, bottom) in boxes.iter() {
            page.push(Item::Rect {
                x: MARGIN,
                y: MARGIN + top,
                w: content_w,
                h: bottom - top,
                radius: 6.0,
                fill: Some(CALLOUT_FILL),
                stroke: Some(CALLOUT_LINE),
                width: 1.5,
            });
            if callouts[c].count > 1 {
                page.push(text(
                    MARGIN + content_w - CALLOUT_PAD,
                    MARGIN + bottom - CALLOUT_PAD,
                    18.0,
                    true,
                    INK,
                    Align::End,
                    &format!("{}x", callouts[c].count),
                ));
            }
        }
        page.append(items);
        boxes.clear();
        pages.push(Page {
            number: None,
            items: page,
        });
    };
    let mut i = 0;
    while i < rows.len() {
        let row = &rows[i];
        // A callout's last row and the step after it go on one page when they can.
        let need = if row.keep_with_next {
            row.h + GAP + rows.get(i + 1).map_or(0.0, |r| r.h)
        } else {
            row.h
        };
        let fits_empty = need <= content_h;
        if y > 0.0 && y + need > content_h && (fits_empty || y + row.h > content_h) {
            finish(&mut items, &mut boxes, &mut pages);
            y = 0.0;
        }
        if let Some(c) = row.callout {
            match boxes.last_mut() {
                Some((at, _, bottom)) if *at == c => *bottom = y + row.h,
                _ => boxes.push((c, y, y + row.h)),
            }
        }
        items.extend(row.items.iter().map(|it| shift(it, MARGIN, MARGIN + y)));
        for (c, id) in &row.puts_on {
            let Some(&(_, _, bottom)) = boxes.iter().find(|(at, _, _)| at == c) else {
                continue;
            };
            let Some(&(_, cx, cy)) = row.copies.iter().find(|(cid, _, _)| cid == id) else {
                continue;
            };
            let (tx, ty) = (MARGIN + cx, MARGIN + y + cy);
            let x0 = tx.clamp(MARGIN + 24.0, MARGIN + content_w - 24.0);
            let y0 = MARGIN + bottom;
            if ty <= y0 + 8.0 {
                continue;
            }
            items.push(Item::Path {
                points: vec![[x0, y0], [x0, ty - 10.0], [tx, ty - 10.0]],
                closed: false,
                fill: None,
                stroke: Some(CALLOUT_LINE),
                width: 1.5,
            });
            items.extend(arrow([tx, ty - 10.0], [tx, ty], 1.5, CALLOUT_LINE));
        }
        y += row.h + GAP;
        i += 1;
    }
    if !items.is_empty() || !boxes.is_empty() {
        finish(&mut items, &mut boxes, &mut pages);
    }
    pages
}

/// The cover: the model's name, the finished model, and how many pieces (with the age, when it's set).
fn cover(src: &Sources, ctx: &mut Ctx, pw: f64, ph: f64) -> Page {
    let mut items = Vec::new();
    let title = font::fit(src.title, 34.0, true, pw - 2.0 * MARGIN);
    items.push(text(pw / 2.0, MARGIN + 40.0, 34.0, true, INK, Align::Middle, &title));
    let pic = &src.drawn.cover;
    let (aw, ah) = (pw - 2.0 * MARGIN, ph - 2.0 * MARGIN - 150.0);
    let k = (aw / f64::from(pic.width.max(1))).min(ah / f64::from(pic.height.max(1)));
    let (w, h) = (f64::from(pic.width) * k, f64::from(pic.height) * k);
    let image = ctx.images.add(pic);
    items.push(Item::Image {
        image,
        x: (pw - w) / 2.0,
        y: MARGIN + 70.0 + (ah - h) / 2.0,
        w,
        h,
    });
    let pieces = src.bom.totals.pieces;
    items.push(text(
        pw - MARGIN,
        ph - MARGIN - 4.0,
        16.0,
        false,
        INK,
        Align::End,
        &format!("{pieces} pieces"),
    ));
    if let Some(age) = src.age.filter(|a| !a.is_empty()) {
        let w = font::width(age, 20.0, true) + 20.0;
        items.push(Item::Rect {
            x: MARGIN,
            y: ph - MARGIN - 30.0,
            w,
            h: 32.0,
            radius: 6.0,
            fill: None,
            stroke: Some(INK),
            width: 2.0,
        });
        items.push(text(
            MARGIN + 10.0,
            ph - MARGIN - 7.0,
            20.0,
            true,
            INK,
            Align::Start,
            age,
        ));
    }
    Page { number: None, items }
}

/// The parts list: every part and color with its count, part id, and color name, by color.
fn parts_pages(src: &Sources, ctx: &mut Ctx, content_w: f64, content_h: f64) -> Vec<Page> {
    let names = names(src.bom);
    let mut lines: Vec<_> = src.bom.lines.iter().collect();
    lines.sort_by(|a, b| a.color_name.cmp(&b.color_name).then(a.part.cmp(&b.part)));
    let (cw, chh) = (76.0, 84.0);
    let cols = ((content_w + 6.0) / (cw + 6.0)).floor().max(1.0) as usize;
    let head = 30.0;
    let rows_per_page = (((content_h - head) / (chh + 6.0)).floor() as usize).max(1);
    let per_page = cols * rows_per_page;
    let mut pages = Vec::new();
    for (p, chunk) in lines.chunks(per_page).enumerate() {
        let mut items = Vec::new();
        if p == 0 {
            items.push(text(MARGIN, MARGIN + 16.0, 16.0, true, INK, Align::Start, "Parts"));
            items.push(text(
                MARGIN + content_w,
                MARGIN + 16.0,
                11.0,
                false,
                MUTED,
                Align::End,
                &format!("{} pieces", src.bom.totals.pieces),
            ));
        }
        for (i, l) in chunk.iter().enumerate() {
            let (c, r) = (i % cols, i / cols);
            let x = MARGIN + c as f64 * (cw + 6.0);
            let y = MARGIN + head + r as f64 * (chh + 6.0);
            items.push(Item::Rect {
                x,
                y,
                w: cw,
                h: chh,
                radius: 4.0,
                fill: Some(BOX_FILL),
                stroke: None,
                width: 0.0,
            });
            if let Some(img) = ctx
                .parts
                .get(&(l.part.clone(), l.color))
                .copied()
                .and_then(|i| ctx.images.part(ctx.drawn, i))
            {
                items.push(Item::Image {
                    image: img,
                    x: x + (cw - 42.0) / 2.0,
                    y: y + 4.0,
                    w: 42.0,
                    h: 42.0,
                });
            }
            items.push(text(
                x + 5.0,
                y + 58.0,
                11.0,
                true,
                INK,
                Align::Start,
                &format!("{}x", l.qty),
            ));
            items.push(text(
                x + 5.0,
                y + 69.0,
                7.5,
                false,
                INK,
                Align::Start,
                &font::fit(&l.part, 7.5, false, cw - 10.0),
            ));
            let color = names
                .get(&(l.part.clone(), l.color))
                .map_or(l.color_name.as_str(), |(_, c)| c.as_str());
            items.push(text(
                x + 5.0,
                y + 79.0,
                7.0,
                false,
                MUTED,
                Align::Start,
                &font::fit(color, 7.0, false, cw - 10.0),
            ));
        }
        pages.push(Page { number: None, items });
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{Spot, Zoom};

    fn picture(width: u32, height: u32) -> Picture {
        Picture {
            jpeg: String::new(),
            width,
            height,
        }
    }

    /// The farthest right and down any box, picture, or line in `items` reaches.
    fn reach(items: &[Item]) -> (f64, f64) {
        items.iter().fold((0.0_f64, 0.0_f64), |(x, y), it| match it {
            Item::Rect { x: ix, y: iy, w, h, .. } | Item::Image { x: ix, y: iy, w, h, .. } => {
                (x.max(ix + w), y.max(iy + h))
            }
            Item::Path { points, .. } => points.iter().fold((x, y), |(x, y), p| (x.max(p[0]), y.max(p[1]))),
            Item::Text { .. } => (x, y),
        })
    }

    #[test]
    fn the_locale_picks_the_paper() {
        for (locale, paper) in [
            ("en-US", Paper::Letter),
            ("en_US.UTF-8", Paper::Letter),
            ("fr_CA", Paper::Letter),
            ("es-MX", Paper::Letter),
            ("en-US-u-ca-gregory", Paper::Letter),
            ("en-GB", Paper::A4),
            ("de_DE.UTF-8", Paper::A4),
            ("zh-Hant-TW", Paper::A4),
            ("sr_RS@latin", Paper::A4),
            ("es-419", Paper::A4),
            ("en", Paper::A4),
            ("C", Paper::A4),
            ("", Paper::A4),
        ] {
            assert_eq!(Paper::for_locale(locale), paper, "{locale}");
        }
    }

    // Rung 2's bench, page 32: sub-step 1's zoom box, beside a short, wide picture, ran out of the sub-assembly's box
    // and across the lines to where the copies go.
    #[test]
    fn a_zoom_box_stays_inside_its_step() {
        let drawn = Drawn {
            cover: picture(1, 1),
            runs: Vec::new(),
            parts: Vec::new(),
        };
        let step = Step {
            n: 1,
            title: String::new(),
            group: String::new(),
            part_ids: Vec::new(),
            parts: Vec::new(),
            submodels: Vec::new(),
        };
        for (pw, ph, max_w) in [(300, 100, 500.0), (300, 60, 500.0), (300, 40, 320.0), (300, 200, 320.0)] {
            let d = DrawnStep {
                n: 1,
                picture: picture(pw, ph),
                ldu: 1.0,
                turn: 0,
                copies: Vec::<Spot>::new(),
                arrows: Vec::new(),
                zoom: Some(Zoom {
                    picture: picture(480, 480),
                    x: 10.0,
                    y: 10.0,
                }),
            };
            let mut ctx = Ctx {
                drawn: &drawn,
                images: Images {
                    list: Vec::new(),
                    parts: HashMap::new(),
                },
                parts: Default::default(),
            };
            let block = ctx.step(&step, &d, 1.0, false, true, max_w);
            let (x, y) = reach(&block.items);
            assert!(
                x <= block.w + 3.0 && y <= block.h,
                "{pw} x {ph} in {max_w}: reaches ({x}, {y}), block {} x {}",
                block.w,
                block.h
            );
        }
    }
}
