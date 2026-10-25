// SPDX-License-Identifier: AGPL-3.0-or-later
//! The dimension parser for parts search: "2x4", "1 x 2 x 3", "2×2", "1 x 1 x 2/3", "4x4 round".

/// Dimensions found in a query, in the order written, and the query with them taken out.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub dims: Vec<f64>,
    pub rest: String,
}

/// Finds the first run of two or three numbers joined by `x` (or `×`, `*`, `by`), with or without spaces. Numbers
/// may be decimals (`0.667`) or fractions (`2/3`).
pub fn parse(query: &str) -> Parsed {
    let lower = query.to_lowercase().replace('×', " x ").replace('*', " x ");
    // Split "2x4" into "2 x 4" without touching words like "box".
    let mut spaced = String::new();
    let chars: Vec<char> = lower.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        let digit_before = i > 0 && chars[i - 1].is_ascii_digit();
        let digit_after = chars.get(i + 1).is_some_and(|n| n.is_ascii_digit());
        if *c == 'x' && (digit_before || digit_after) {
            spaced.push_str(" x ");
        } else {
            spaced.push(*c);
        }
    }
    let words: Vec<&str> = spaced.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        if let Some(first) = number(words[i]) {
            let mut dims = vec![first];
            let mut j = i + 1;
            while j + 1 < words.len() && (words[j] == "x" || words[j] == "by") && dims.len() < 3 {
                match number(words[j + 1]) {
                    Some(n) => {
                        dims.push(n);
                        j += 2;
                    }
                    None => break,
                }
            }
            if dims.len() >= 2 {
                let rest = words[..i]
                    .iter()
                    .chain(&words[j..])
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ");
                return Parsed { dims, rest };
            }
        }
        i += 1;
    }
    Parsed {
        dims: Vec::new(),
        rest: words.join(" "),
    }
}

fn number(w: &str) -> Option<f64> {
    if let Some((a, b)) = w.split_once('/') {
        let (a, b): (f64, f64) = (a.parse().ok()?, b.parse().ok()?);
        return (b != 0.0).then(|| a / b);
    }
    w.parse().ok().filter(|n: &f64| n.is_finite() && *n > 0.0)
}
