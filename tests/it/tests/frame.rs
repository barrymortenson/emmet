// SPDX-License-Identifier: AGPL-3.0-or-later
//! The DSL frame and footprint rules of plan section 08, pinned against a hand-checked .ldr
//! (tests/fixtures/projects/frame/expected.ldr).

use emmet_core::ldraw::{self, Line};
use emmet_it::{emmet_in, project};

fn parts(text: &str) -> Vec<Line> {
    let doc = ldraw::parse(text).unwrap();
    doc.files
        .into_iter()
        .flat_map(|f| f.lines)
        .filter(|l| matches!(l, Line::Sub { .. }))
        .collect()
}

#[test]
fn frame_matches_the_hand_checked_ldr() {
    let dir = project("frame");
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let got = parts(&std::fs::read_to_string(dir.join("build/model.mpd")).unwrap());
    let want = parts(&std::fs::read_to_string(dir.join("expected.ldr")).unwrap());
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(&want).enumerate() {
        assert_eq!(
            ldraw::write_line(g),
            ldraw::write_line(w),
            "part {} of frame/model.py",
            i + 1
        );
    }
    // The covered boxes the run reports: from (0, 0, 0) to the slope's far edge at x 6 and the turned brick's top.
    assert_eq!(
        v["data"]["bbox"],
        serde_json::json!({ "min": [0.0, 0.0, 0.0], "max": [7.5, 6.0, 6.0] })
    );
}
