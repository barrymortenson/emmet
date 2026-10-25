# ADR 0005: Headless render in a hidden window

Status: accepted, 2026-10-24 (session 0B). Confirms D-24 and settles Q-04.

## Context

D-24: renders for the agent go through the viewer's own code, in the running app or, when it isn't open, in a hidden
window of the app. Q-04 asked whether a hidden-window WebGL render works on WKWebView and WebView2, or whether a
throttled hidden context needs a fallback (an off-screen visible window).

## The spike

`tests/spikes/headless-render/` is a tiny Tauri app (`emmet-spike-render`). It opens one window, hidden by default
(`--mode offscreen` places a visible window at −10,000, −10,000 instead), renders once with three.js 0.186.1, and
sends the PNG to Rust as a raw IPC body (`save_png`), which writes it and exits. With `--model` and `--library` it
renders an LDraw model through `LDrawLoader`, served from an `ldraw:` custom protocol that serves only files inside the library folder and
the one model file, and matches names case-insensitively. It prints `mark <step> <ms since start>`, and
exits with an error after 60 s. Its test, `tests/spikes/headless-render/tests/render.rs`, renders a cube and a small
ldraw-mini model at 256 × 256 and checks the PNG's signature, size, and that it isn't blank.

Findings that carry over to the viewer (1B):

- `LDrawLoader` reports a part it can't load with `console.warn` and carries on, so a broken model still renders. The
  spike traps that warning and fails the render, and its test checks the house draws all 5 parts (`mark parts 5`);
  the viewer must do the same rather than show a model with holes.

- The protocol's responses need `Access-Control-Allow-Origin`: the page's origin is the app's, so the fetches are
  cross-origin (the first attempt failed with "Load failed").
- `LDrawLoader` in r186 needs `setConditionalLineMaterial(LDrawConditionalLineMaterial)` before it loads anything.
- One render with `preserveDrawingBuffer: true` needs no animation frame, so a hidden window's paused
  `requestAnimationFrame` doesn't matter for agent renders.

## Results

macOS (this Mac, WKWebView, Apple silicon), debug build, from process start:

| Render | Window | Script loaded | Model loaded | Rendered | PNG written |
|---|---|---|---|---|---|
| Cube, hidden, 512² | 164 ms | 301 | 310 | 429 | **468 ms** |
| Cube, hidden, 3 more runs | | | | | 426, 298, 323 ms |
| Cube, off-screen visible, 512² | 166 | 338 | 344 | 354 | **360 ms** |
| 40182-1 (176 type-1 lines in the OMR file), hidden, 768² | 160 | 297 | 1,658 | 1,906 | **1,934 ms** |
| 21022-1 (270 type-1 lines), hidden, 768² | 147 | 281 | 1,057 | 1,086 | **1,096 ms** |
| 6743-1 (166 type-1 lines), hidden, 768² |
| ldraw-mini house (5 parts), hidden, 256² (`cargo test`) | 210 | 353 | 1,068 | | **1,139 ms** | 142 | 275 | 2,099 | 2,165 | **2,191 ms** |

The images are correct (the cube, and the rung 3 shortlist renders in `docs/spec/ladder/`), so a hidden WKWebView
renders WebGL without the fallback. Model time is almost all part loading on a cold cache; the viewer caches parts.

Windows (CI, `windows-latest`, WebView2, no GPU, so WebGL runs on a software renderer; PR #3 run 40152761697, the
windows job's "Spike reports" step, the two tests running in parallel at 256²):

| Render | Window | Script loaded | Model loaded | Rendered | PNG written |
|---|---|---|---|---|---|
| Cube, hidden | 625 ms | 747 | 777 | 859 | **871 ms** |
| ldraw-mini house (5 parts), hidden | 594 | 720 | 9,718 | 9,840 | **9,849 ms** |

Both PNGs pass the test's checks, so a hidden WebView2 renders WebGL too. After the review added the parts check, run
40154331195 shows the house drawing all of them on Windows (`mark parts 5 8803`, PNG written at 8,888 ms). **Loading parts is the cost:** about 9 s for
a 5-part model, against 0.7 s on the Mac (the house row above, from script to loaded). LDrawLoader fetches each part, subpart, and primitive as its own request,
so the likely cause is the cost per custom-protocol request on WebView2 (or the software renderer's slower CPU); 0B
didn't measure which. This doesn't change the decision, but it is a risk for section 13's
first-load budget (≤ 1.5 s for rung 1): 1B should serve parts in bulk (a packed part bundle per model, or one request
for all of a model's files) and measure on Windows first.

## Decision

D-24 stands: render mode opens a **hidden** window and renders through `viewer.render`. The off-screen visible window
stays as the fallback, behind a setting, for a webview that ever stops rendering while hidden; 0B needed it on
neither OS. The `ldraw:`-style protocol is how the viewer loads parts (section 13's `emmet-lib://`), with its path
check.

## Rerun

```sh
cargo build -p emmet-spike-render
target/debug/emmet-spike-render --out /tmp/cube.png
target/debug/emmet-spike-render --out /tmp/cube.png --mode offscreen
target/debug/emmet-spike-render --out /tmp/m.png --size 768 --model <a .mpd> --library <an unzipped LDraw library>
cargo test -p emmet-spike-render -- --nocapture
```
