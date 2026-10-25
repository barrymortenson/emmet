# ADR 0008: The viewer's own scene, instanced, with parts packed per request

Status: accepted, 2026-10-25 (session 1B). Confirms D-23.

## Context

D-23 proposed that the viewer build its own scene from Emmet's placement list, using three's `LDrawLoader` only to
load each unique part once, then instancing, so that diff updates, highlight, steps, and performance are possible.
1B was to confirm it with bench numbers, and record an ADR first if `LDrawLoader` per unique part was too slow on
WKWebView. ADR 0005 also found that loading parts one file per request was the cost on WebView2 (about 9 s for a
5-part model in CI) and asked 1B to serve parts in bulk and measure Windows first.

## Decision

D-23 stands, as built in `packages/viewer` (its README has the API):

- **One request per model, one parse per part.** The host's `emmet-lib://localhost/pack?parts=a,b,…` returns every
  file the parts use, each once, and the app turns each part into a packed `.mpd` (`0 FILE` blocks) that
  `LDrawLoader.parse` reads without fetching anything. A model's parts load in one request, not hundreds.
- **Instancing.** One `InstancedMesh` per (part, color). Edges and conditional lines are instanced too, sharing the
  mesh's `instanceMatrix` through an `InstancedBufferGeometry`: the edge material gets `USE_INSTANCING`, and the
  conditional-line shader multiplies `instanceMatrix` into its four `modelViewMatrix` products. Main-color slots (16,
  24) take the bucket's color; a part's fixed colors stay.
- **Diff by Emmet id**, applied in one synchronous pass after every new part has loaded, rewriting only the buckets
  it touches.
- **Renders** use a second `WebGLRenderer` on a detached canvas, with a fixed light background and the camera fit to
  the model's box, so the window's camera, size, and theme never change and a render looks the same in either
  theme.

The viewer relies on three r186 `LDrawLoader` internals that `@types/three` doesn't declare: `parseColorMetaDirective`
(fed by a small token reader, since `LineParser` isn't exported), `edgeMaterialCache`, `conditionalEdgeMaterialCache`,
and `missingColorMaterial`. A three.js update that changes them fails the E2E specs and the visual baselines.

## Bench

`just bench` (`tests/e2e/specs/bench.e2e.mjs`) in the real app. First load is a new viewer with an empty part cache;
slider → viewer is `set_params` through the watcher, the run, and the diff until the viewer has drawn it (median of
6); orbit is the mean frame time over 120 frames turning the camera.

macOS, this Mac (M-series, WKWebView), debug build, 2026-10-25:

| Scene | Parts | First load | Slider → viewer | Orbit frame | Draw calls | Triangles |
|---|---|---|---|---|---|---|
| cottage | 93 | 689 ms | 500 ms | 13.9 ms | 30 | 76,306 |
| grid 500 | 500 | 17 ms | 427 ms | 13.9 ms | 18 | 350,000 |
| grid 2000 | 2000 | 21 ms | 478 ms | 13.9 ms | 18 | 1,400,000 |

Against section 13: first load ≤ 1.5 s (rung 1), slider → viewer ≤ 0.8 s warm, orbit ≤ 16 ms; all hold. The orbit
frame time is the display's refresh interval (72 Hz here), so drawing isn't the limit. The grid scenes are one part
(3001), already parsed by then, so their first load is the instancing alone.

Windows (CI, `windows-latest`, WebView2 with WebGL on a software renderer, no GPU; PR #11 run 40219092964):

| Scene | Parts | First load | Slider → viewer | Orbit frame | Draw calls | Triangles |
|---|---|---|---|---|---|---|
| cottage | 93 | 7,247 ms | 948 ms | 35 ms | 30 | 76,306 |
| grid 500 | 500 | 131 ms | 785 ms | 130.8 ms (77 frames in 10 s) | 18 | 350,000 |
| grid 2000 | 2000 | 68 ms | 923 ms | 583.7 ms (18 frames in 10 s) | 18 | 1,400,000 |

**Open risk: first load on Windows.** 7.2 s for the cottage's parts misses section 13's 1.5 s, and orbit misses 16 ms,
on a runner that draws WebGL on the CPU. Packing parts helped (ADR 0005 measured about 9 s for 5 parts, one request
per file); what's left is parsing and drawing on that CPU, which says little about a real GPU. The check on real
Windows hardware (the H-04 VM, from 1C) decides whether WebView2 needs more, such as caching parsed part geometry
across launches. D-23 stands either way: the per-part cost is the loader's work, not the scene design.

## Consequences

- No ADR for an alternative to `LDrawLoader` per part: it is fast enough on WKWebView, which 1B was to check. On
  WebView2 the answer waits on real hardware (the risk above).
- The bench lives with the E2E specs, since it needs the real app; `packages/viewer` keeps the scene code and its
  unit test. Plan section 13 says so.
- WKWebView has no JS heap API, so the leak test reads the WebContent process's footprint on macOS, from after one
  warm-up cycle (docs/decisions.md, 2026-10-25).
