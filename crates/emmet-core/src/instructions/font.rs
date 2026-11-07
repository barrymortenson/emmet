// SPDX-License-Identifier: AGPL-3.0-or-later
//! Helvetica and Helvetica-Bold's advance widths (Adobe's AFM files for the standard 14 fonts, in thousandths of the
//! font size), so the layout can center and fit text. The PDF names the fonts and embeds none (ADR 0014): every PDF
//! reader has them, or a metric-compatible face (Arial, Liberation Sans, Nimbus Sans).

/// Widths of characters 32 to 126.
const REGULAR: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // space to /
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, // 0 to 9
    278, 278, 584, 584, 584, 556, 1015, // : to @
    667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944,
    667, 667, 611, // A to Z
    278, 278, 278, 469, 556, 333, // [ to `
    556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722,
    500, 500, 500, // a to z
    334, 260, 334, 584, // { to ~
];

const BOLD: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, // space to /
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, // 0 to 9
    333, 333, 584, 584, 584, 611, 975, // : to @
    722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944,
    667, 667, 611, // A to Z
    333, 278, 333, 584, 556, 333, // [ to `
    556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778,
    556, 556, 500, // a to z
    389, 280, 389, 584, // { to ~
];

/// How wide `text` is at `size` points.
pub fn width(text: &str, size: f64, bold: bool) -> f64 {
    let table = if bold { &BOLD } else { &REGULAR };
    let units: u32 = text
        .chars()
        .map(|c| match c as u32 {
            n @ 32..=126 => u32::from(table[(n - 32) as usize]),
            // Latin-1 letters are about as wide as a lowercase letter.
            _ => 556,
        })
        .sum();
    f64::from(units) * size / 1000.0
}

/// `text` cut to fit `max` points, with an ellipsis ("...") when it had to be cut.
pub fn fit(text: &str, size: f64, bold: bool, max: f64) -> String {
    if width(text, size, bold) <= max {
        return text.to_string();
    }
    let mut out = String::new();
    for c in text.chars() {
        let next = format!("{out}{c}...");
        if width(&next, size, bold) > max {
            break;
        }
        out.push(c);
    }
    format!("{}...", out.trim_end())
}

/// The byte a character is in the PDF's WinAnsiEncoding, or `?` for one it doesn't have.
pub fn win_ansi(c: char) -> u8 {
    match c as u32 {
        n @ (32..=126 | 160..=255) => n as u8,
        _ => match c {
            '\u{2013}' => 150,
            '\u{2014}' => 151,
            '\u{2018}' => 145,
            '\u{2019}' => 146,
            '\u{201C}' => 147,
            '\u{201D}' => 148,
            '\u{2022}' => 149,
            '\u{2026}' => 133,
            _ => b'?',
        },
    }
}
