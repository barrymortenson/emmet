# ADR 0008: The viewer's own scene, instanced, with parts packed per request

Status: accepted, 2026-10-25 (session 1B). Confirms D-23. Amended 2026-11-01 (session 4C): instancing is per (part,
finish), not per (part, color); see "4C" at the end.

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
  conditional-line shader multiplies `instanceMatrix` into each of its five `modelViewMatrix` products. Main-color slots (16,
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
slider → viewer is `set_params` through the watcher, the run, and the diff until the viewer has applied it (median
of 6); orbit is the mean frame time over 120 frames turning the camera, or as many as fit in 10 s.

macOS, this Mac (M-series, WKWebView), debug build, 2026-10-25:

| Scene | Parts | First load | Slider → viewer | Orbit frame | Draw calls | Triangles |
|---|---|---|---|---|---|---|
| cottage | 93 | 689 ms | 500 ms | 13.9 ms | 30 | 76,306 |
| grid 500 | 500 | 17 ms | 427 ms | 13.9 ms | 18 | 350,000 |
| grid 2000 | 2000 | 21 ms | 478 ms | 13.9 ms | 18 | 1,400,000 |

Against section 13: first load ≤ 1.5 s (rung 1), slider → viewer ≤ 0.8 s warm, orbit ≤ 16 ms; all hold. The orbit
frame time is the display's refresh interval (72 Hz here), so drawing isn't the limit. The grid scenes are one small
part (3001), fetched and parsed again by the bench's fresh viewer but served from the host's warm file cache, so
their first load is mostly the instancing.

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

## 4C: one bucket per part and finish

Profiling 4C's 2,000-part stress model (`tests/fixtures/projects/scene-stress`, a town block of four buildings with
glass, chrome, pearl, and rubber, 216 (part, color) pairs) on this Mac found draw calls the biggest cost: each
(part, color) bucket draws its faces, its edges, and its conditional lines, so the block took 639 draw calls and the
Cottage 523, against section 13's ~300. The orbit frame time sits at the display's refresh interval on an M-series
GPU, so the bench now also reports a frame's draw cost, waited for with a one-pixel read (WebKit's `gl.finish()`
doesn't wait).

The decision: one `InstancedMesh` per (part, finish). The finish's material (Emmet's own, per LDraw finish, in
`packages/viewer/src/materials.ts`) is white, and every instance carries its color: the faces' through
`instanceColor`, the edges' and conditional lines' through an instanced `instanceColor` attribute on their
`InstancedBufferGeometry` (`USE_INSTANCING_COLOR`). Transparent colors bucket by alpha and glowing ones by color,
which the material holds. A part's fixed colors (prints, a part that is always one color) keep their own materials,
tinted only by the highlight. Each part is one geometry per kind and color slot, merged across its sub-objects
(`BufferGeometryUtils.mergeGeometries`, part of three), so a part made of parts draws as one.

The other costs, in order: triangles (studs and the tubes under bricks: a low-detail variant, made on demand from the
part's own text with 8-sided studs and no tubes, drawn when its studs are a few pixels on screen), transparent order
(transparent buckets back to front by distance, after the opaque ones), and, found in the after bench, the finding
pins asking for their parts' box by scanning every part on every camera move (now a lookup by id, and five pins at
most).

| Scene (macOS, this Mac, debug build) | Parts | Draw calls before | after | Draw cost before | after |
|---|---|---|---|---|---|
| (filled from `docs/spec/sessions/4C.md`'s before and after tables) | | | | | |

Rejected: one geometry per color with every part of that color merged into it (fewest draw calls, but a diff, a
highlight, and the build animation would rewrite whole buffers), and `BatchedMesh` (three's, for many geometries in
one draw: it would fold the buckets further, but its per-instance color and visibility API is newer than the WebGL2
paths both webviews were proven on; a candidate if the 5,000-part column, 8I, needs it).
