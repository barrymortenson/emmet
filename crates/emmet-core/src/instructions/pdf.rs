// SPDX-License-Identifier: AGPL-3.0-or-later
//! Writes a [`Book`] as a PDF 1.4 file (ADR 0014), with no PDF library: the pictures go in as the JPEGs the webview
//! made (DCTDecode, unchanged), the text in Helvetica and Helvetica-Bold, two of the standard 14 fonts, which every
//! reader has, so none is embedded; each page's drawing is a plain content stream. The output is the same bytes for
//! the same book.

use std::fmt::Write as _;

use super::font;
use super::layout::{Align, Book, Item, Rgb};

/// The PDF's bytes.
pub fn write(book: &Book, title: &str) -> Vec<u8> {
    let mut w = Writer::default();
    w.out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    // 1 the catalog, 2 the page tree, 3 and 4 the fonts, 5 the document's info, then the images, then each page and
    // its content.
    let first_image = 6;
    let first_page = first_image + book.images.len();
    let pages: Vec<usize> = (0..book.pages.len()).map(|i| first_page + 2 * i).collect();
    w.object(1, b"<< /Type /Catalog /Pages 2 0 R >>");
    let kids: Vec<String> = pages.iter().map(|p| format!("{p} 0 R")).collect();
    w.object(
        2,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} /MediaBox [0 0 {} {}] >>",
            kids.join(" "),
            pages.len(),
            num(book.width),
            num(book.height)
        )
        .as_bytes(),
    );
    for (n, name) in [(3, "Helvetica"), (4, "Helvetica-Bold")] {
        w.object(
            n,
            format!("<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding >>").as_bytes(),
        );
    }
    w.object(
        5,
        format!("<< /Title {} /Producer (Emmet) >>", string(title)).as_bytes(),
    );
    for (i, img) in book.images.iter().enumerate() {
        let jpeg = crate::base64::decode(&img.jpeg).unwrap_or_default();
        let head = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 \
             /Filter /DCTDecode /Length {} >>",
            img.width,
            img.height,
            jpeg.len()
        );
        w.stream(first_image + i, &head, &jpeg);
    }
    for (i, page) in book.pages.iter().enumerate() {
        let content = content(&page.items, book.height);
        // Only the page's own pictures in its resources.
        let mut used: Vec<usize> = page
            .items
            .iter()
            .filter_map(|it| match it {
                Item::Image { image, .. } => Some(*image),
                _ => None,
            })
            .collect();
        used.sort_unstable();
        used.dedup();
        let images: String = used
            .iter()
            .map(|i| format!("/Im{i} {} 0 R ", first_image + i))
            .collect();
        w.object(
            pages[i],
            format!(
                "<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 3 0 R /F2 4 0 R >> /XObject << {images}>> >> \
                 /Contents {} 0 R >>",
                pages[i] + 1
            )
            .as_bytes(),
        );
        w.stream(
            pages[i] + 1,
            &format!("<< /Length {} >>", content.len()),
            content.as_bytes(),
        );
    }
    w.finish(5)
}

#[derive(Default)]
struct Writer {
    out: Vec<u8>,
    /// Each object's byte offset, by object number.
    offsets: Vec<(usize, usize)>,
}

impl Writer {
    fn object(&mut self, n: usize, body: &[u8]) {
        self.offsets.push((n, self.out.len()));
        self.out.extend_from_slice(format!("{n} 0 obj\n").as_bytes());
        self.out.extend_from_slice(body);
        self.out.extend_from_slice(b"\nendobj\n");
    }

    fn stream(&mut self, n: usize, dict: &str, data: &[u8]) {
        self.offsets.push((n, self.out.len()));
        self.out
            .extend_from_slice(format!("{n} 0 obj\n{dict}\nstream\n").as_bytes());
        self.out.extend_from_slice(data);
        self.out.extend_from_slice(b"\nendstream\nendobj\n");
    }

    fn finish(mut self, info: usize) -> Vec<u8> {
        self.offsets.sort();
        let count = self.offsets.len() + 1;
        let xref = self.out.len();
        let mut table = format!("xref\n0 {count}\n0000000000 65535 f \n");
        for (_, at) in &self.offsets {
            let _ = writeln!(table, "{at:010} 00000 n ");
        }
        let _ = write!(
            table,
            "trailer\n<< /Size {count} /Root 1 0 R /Info {info} 0 R >>\nstartxref\n{xref}\n%%EOF\n"
        );
        self.out.extend_from_slice(table.as_bytes());
        self.out
    }
}

/// A number as PDF writes it: at most two decimals, no trailing zeros.
fn num(v: f64) -> String {
    let s = format!("{:.2}", if v.abs() < 0.005 { 0.0 } else { v });
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.to_string() }
}

fn rgb(c: Rgb) -> String {
    format!(
        "{} {} {}",
        num(f64::from(c[0]) / 255.0),
        num(f64::from(c[1]) / 255.0),
        num(f64::from(c[2]) / 255.0)
    )
}

/// A PDF string in WinAnsiEncoding, with `(`, `)`, and `\` escaped and bytes past ASCII as octal escapes.
fn string(s: &str) -> String {
    let mut out = String::from("(");
    for c in s.chars() {
        match font::win_ansi(c) {
            b'(' => out.push_str("\\("),
            b')' => out.push_str("\\)"),
            b'\\' => out.push_str("\\\\"),
            b @ 32..=126 => out.push(b as char),
            b => {
                let _ = write!(out, "\\{b:03o}");
            }
        }
    }
    out.push(')');
    out
}

/// A page's drawing: PDF's origin is the bottom left, so each y is flipped.
fn content(items: &[Item], height: f64) -> String {
    let mut s = String::from("1 j 1 J\n");
    let y = |v: f64| num(height - v);
    for item in items {
        match item {
            Item::Rect {
                x,
                y: top,
                w,
                h,
                radius,
                fill,
                stroke,
                width,
            } => {
                if fill.is_none() && stroke.is_none() {
                    continue;
                }
                paint_start(&mut s, *fill, *stroke, *width);
                let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
                if r == 0.0 {
                    let _ = writeln!(s, "{} {} {} {} re", num(*x), y(top + h), num(*w), num(*h));
                } else {
                    // Corners as quarter circles from Bézier curves (k ≈ 0.5523).
                    let k = r * (1.0 - 0.5523);
                    let (l, rt, t, b) = (*x, x + w, *top, top + h);
                    let _ = writeln!(s, "{} {} m", num(l + r), y(t));
                    let _ = writeln!(s, "{} {} l", num(rt - r), y(t));
                    let _ = writeln!(
                        s,
                        "{} {} {} {} {} {} c",
                        num(rt - k),
                        y(t),
                        num(rt),
                        y(t + k),
                        num(rt),
                        y(t + r)
                    );
                    let _ = writeln!(s, "{} {} l", num(rt), y(b - r));
                    let _ = writeln!(
                        s,
                        "{} {} {} {} {} {} c",
                        num(rt),
                        y(b - k),
                        num(rt - k),
                        y(b),
                        num(rt - r),
                        y(b)
                    );
                    let _ = writeln!(s, "{} {} l", num(l + r), y(b));
                    let _ = writeln!(
                        s,
                        "{} {} {} {} {} {} c",
                        num(l + k),
                        y(b),
                        num(l),
                        y(b - k),
                        num(l),
                        y(b - r)
                    );
                    let _ = writeln!(s, "{} {} l", num(l), y(t + r));
                    let _ = writeln!(
                        s,
                        "{} {} {} {} {} {} c",
                        num(l),
                        y(t + k),
                        num(l + k),
                        y(t),
                        num(l + r),
                        y(t)
                    );
                    s.push_str("h\n");
                }
                paint_end(&mut s, *fill, *stroke);
            }
            Item::Path {
                points,
                closed,
                fill,
                stroke,
                width,
            } => {
                if points.len() < 2 || (fill.is_none() && stroke.is_none()) {
                    continue;
                }
                paint_start(&mut s, *fill, *stroke, *width);
                for (i, p) in points.iter().enumerate() {
                    let _ = writeln!(s, "{} {} {}", num(p[0]), y(p[1]), if i == 0 { "m" } else { "l" });
                }
                if *closed {
                    s.push_str("h\n");
                }
                paint_end(&mut s, *fill, *stroke);
            }
            Item::Text {
                x,
                y: base,
                size,
                bold,
                color,
                align,
                text,
            } => {
                let w = font::width(text, *size, *bold);
                let x = match align {
                    Align::Start => *x,
                    Align::Middle => x - w / 2.0,
                    Align::End => x - w,
                };
                let _ = writeln!(
                    s,
                    "BT /{} {} Tf {} rg {} {} Td {} Tj ET",
                    if *bold { "F2" } else { "F1" },
                    num(*size),
                    rgb(*color),
                    num(x),
                    y(*base),
                    string(text)
                );
            }
            Item::Image { image, x, y: top, w, h } => {
                let _ = writeln!(
                    s,
                    "q {} 0 0 {} {} {} cm /Im{image} Do Q",
                    num(*w),
                    num(*h),
                    num(*x),
                    y(top + h)
                );
            }
        }
    }
    s
}

fn paint_start(s: &mut String, fill: Option<Rgb>, stroke: Option<Rgb>, width: f64) {
    if let Some(c) = fill {
        let _ = writeln!(s, "{} rg", rgb(c));
    }
    if let Some(c) = stroke {
        let _ = writeln!(s, "{} RG {} w", rgb(c), num(width));
    }
}

fn paint_end(s: &mut String, fill: Option<Rgb>, stroke: Option<Rgb>) {
    s.push_str(match (fill.is_some(), stroke.is_some()) {
        (true, true) => "B\n",
        (true, false) => "f\n",
        _ => "S\n",
    });
}
