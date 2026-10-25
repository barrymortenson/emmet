// SPDX-License-Identifier: AGPL-3.0-or-later
//! ADR 0005: three.js renders in a hidden window on this OS (WKWebView on macOS, WebView2 on Windows), both a cube and
//! an LDraw model through LDrawLoader, and writes a PNG. The timings go to the test log for the ADR.

use std::path::Path;
use std::process::Command;

/// Width and height from a PNG's IHDR chunk, after checking the signature.
fn png_size(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "not a PNG");
    let be = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    (be(16), be(20))
}

fn render(name: &str, extra: &[&str]) -> Vec<u8> {
    let out = std::env::temp_dir().join(format!("emmet-spike-render-{name}-{}.png", std::process::id()));
    let r = Command::new(env!("CARGO_BIN_EXE_emmet-spike-render"))
        .args(["--out", out.to_str().unwrap(), "--size", "256"])
        .args(extra)
        .output()
        .expect("spawn the render spike");
    eprintln!(
        "{name}: {}{}",
        String::from_utf8_lossy(&r.stdout),
        String::from_utf8_lossy(&r.stderr)
    );
    assert!(r.status.success(), "{name} exited with {}", r.status);
    let bytes = std::fs::read(&out).unwrap();
    std::fs::remove_file(&out).ok();
    assert_eq!(png_size(&bytes), (256, 256), "{name}");
    bytes
}

#[test]
fn renders_a_cube_in_a_hidden_window() {
    // A blank 256×256 PNG is a few hundred bytes; the cube is several KB.
    assert!(render("cube", &[]).len() > 3000);
}

#[test]
fn renders_an_ldraw_model_from_ldraw_mini() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = here.join("tests/house.ldr");
    let library = here.join("../../fixtures/ldraw-mini");
    let png = render(
        "house",
        &[
            "--model",
            model.to_str().unwrap(),
            "--library",
            library.to_str().unwrap(),
        ],
    );
    assert!(png.len() > 3000);
}
