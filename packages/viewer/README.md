# @emmet/viewer

The Emmet viewer (plan section 13): a three.js scene built from Emmet's placement list (`build/placements.json`).
Plain TypeScript with no UI framework inside it (eslint enforces no React). The app, render mode, and the bench all
use this same code, so an agent's render is what the window shows.

## How it draws

- **Unique parts once.** Each part id loads once through three's `LDrawLoader`, from a packed `.mpd` (the part file
  plus every subpart and primitive it uses, as `0 FILE` blocks) that the part source supplies. Each part becomes one
  geometry per kind (faces, edges, conditional lines) and color slot (its main color, LDraw 16 or 24, or each fixed
  color it has), merged across its sub-objects.
- **Instancing by finish.** One `InstancedMesh` per (part, finish): the finish's material is white and every instance
  carries its own color (faces, and the edges' softened edge color), so one part in twenty solid colors is one bucket.
  Transparent colors bucket by their alpha and glowing ones by color. Edges and conditional lines are instanced too:
  they share the mesh's `instanceMatrix`. A 2,018-part model is about 230 draw calls.
- **Materials and light** (`src/materials.ts`): every LDraw finish (solid, transparent, chrome, pearl, metal, matte
  metal, rubber, glow) with its own roughness, metalness, and reflections of a studio room (three's `RoomEnvironment`),
  a key, fill, and rim light, neutral tone mapping, and an optional ground shadow from the key.
- **Quality presets.** Fast (pixel ratio 1, no conditional lines, low-detail studs at any size), Balanced, and Best
  (the ground shadow). "auto" picks one by the GPU: Fast on a software renderer, Balanced on integrated graphics, Best
  otherwise.
- **Stud LOD.** Once the camera settles, a bucket whose nearest studs are under 5 pixels on screen (2.5 at Best) draws
  its part's low-detail variant: 8-sided studs without conditional lines, and no tubes underneath. The variant is made
  the first time it's wanted. Renders always use the full detail.
- **Transparent parts** draw after the opaque ones, back to front by bucket, with depth writes off.
- **Diff updates.** `update()` diffs the new list against the scene by Emmet id (added, removed, moved, recolored),
  loads every new part first, then rewrites only the touched buckets in one synchronous pass. No frame shows half a
  diff, and the camera, highlight, steps, and cut-away stay.
- **The camera never moves on its own.** Only `frame()`, `setView()`, `orbit()`, and `zoom()` move it, and the user's
  drag. The app frames a project's first scene by calling `setView("iso")`.
- **Missing parts are reported, not drawn with holes** (ADR 0005): the scene's are listed in `stats().failed`, and
  `render()` refuses to render while any are. The next update tries a failed part again.
- **Renders are their own.** `render()` draws offscreen with only its `highlight` (none by default) and `step`, the
  full detail, and the ground shadow, queued behind updates, so the window's camera and highlight never leak into a
  render and two renders never mix. A render has a light gray background, or none with `transparent`.
- **On demand.** The scene draws when something changed (a diff, the camera, the size, orbit damping, the
  animation), not every frame.

## API

```ts
import { createViewer } from "@emmet/viewer";

const viewer = createViewer(canvas, { partSource, theme: "light", quality: "auto", pace: "animated" });
await viewer.load(placements);                         // first scene (same as update)
const diff = await viewer.update(placements, { animate: "build" }); // { added, removed, moved, recolored, unchanged }
viewer.replayBuild();                                  // the whole model from empty, in build order
viewer.skipAnimation();                                // jump a running build animation to its end
viewer.setPace("animated" | "fast" | "off");
viewer.highlight(["roof/1", "door"], { mode: "issue" | "select" | "agent" }); // others fade; "issue" tints; [] clears
viewer.highlighted();                                  // the highlighted ids
viewer.showSteps(12);                                  // the build up to step 12, its new parts outlined; null: all
viewer.cutAway(8); viewer.layers();                    // hide above layer (plate) 8; null shows all
viewer.frame(ids?);                                    // fit the ids (or everything), keeping the view direction
viewer.setView("iso" | "front" | "back" | "left" | "right" | "top");
viewer.orbit(radians, tilt?); viewer.zoom(factor);     // as a drag or the scroll wheel would
viewer.setTheme("light" | "dark");
viewer.setQuality("auto" | "fast" | "balanced" | "best");
const { png, camera } = await viewer.render({ view: "iso", size: 768, highlight, step }); // offscreen
const shot = await viewer.snapshot(2);                 // what the window shows, at twice its size (Screenshot)
viewer.screenPoint(ids);                               // where the ids' top is on the canvas (pins, leaders)
viewer.pickAt(x, y); viewer.idsInBox({ x, y, width, height });
const off = viewer.on("hover" | "pick" | "boxing" | "box" | "camera" | "animation", handler);
viewer.stats();        // drawCalls, triangles, frameMs, heap (WebView2 only), instances, buckets, lowDetail,
                       // transparent, highlighted, mode, hovered, hidden, step, outlined, cut, animating, failed,
                       // lastDiff, quality, pace, shadow, …
viewer.cameraMatrix(); // the camera's world matrix (tests check it doesn't move)
viewer.drawCost(60);   // the mean time to draw a frame, waited for (the bench)
await viewer.nextFrame();
viewer.dispose();
```

Pointer gestures on the canvas: hover names a part (`hover`), a click picks one (`pick`, with `add` for shift, ⌘, or
Ctrl) and skips the build animation, shift-drag draws a box (`boxing`) and picks the parts whose centers are in it
(`box`), and any other drag orbits.

A `PartSource` supplies LDraw text:

```ts
interface PartSource {
  parts(ids: string[]): Promise<{ parts: Record<string, string>; missing: Record<string, string[]> }>; // packed .mpd per id
  colors(): Promise<string>; // LDConfig.ldr
}
```

The app's part source (`apps/desktop/src/library.ts`) asks the host's `emmet-lib://localhost/pack?parts=…` for all
of a model's new parts in one request.

**Build animation** (D-34, section 13): `animate: "build"` drops the added parts in one at a time, in build order
(step, a sub-assembly's parts at the step that places the copy, then bottom-up a plate at a time, then nearest the
part before), each falling three plates and fading in over 160 ms with ease-out; removed parts lift and fade out
(150 ms) and moved parts glide (200 ms). Animated staggers parts about 30 ms apart with the whole batch within 6 s,
Fast fits it into 0.6 s, and Off applies it at once; reduced motion turns it off. Every instance is in the scene from
the start (hidden until its turn), so `stats().instances` counts them at once and `stats().animating` counts what
hasn't landed. A new update, a click in the viewer (not a drag), or `skipAnimation()` finishes it at once; a render
always shows the final state. The order and timing live in `src/animate.ts`, tested in Node.

**Steps** (section 08's step rules): `showSteps(n)` hides the later steps and outlines step n's new parts (a
full-screen pass around their silhouette, `src/outline.ts`); the app draws the parts box over the viewer.

## Coordinates

Placement matrices are column-major, in LDraw units with −Y up. The viewer flips LDraw to three's +Y up, so DSL x
(right) is +Z, y (back) is +X, and z (up) is +Y. `render()`'s `camera` is in that frame.

## Tests and bench

- `test/diff.test.mjs`, `test/animate.test.mjs`, `test/size.test.mjs`: the diff, the animation's order and timing, the
  stud LOD's simplified part, and the bundle's size, in Node against `dist/` (`just test`).
- The viewer in the real app: `tests/e2e/specs/{viewer,interact,animate,perf,leak,visual}.e2e.mjs` (`just e2e`).
- `just bench`: first load, slider → updated viewer, orbit frame time, draw cost, and the frame during a build
  animation, for the town block at 184, 478, and 2,018 parts, the Cottage, and a 2,000-brick grid
  (`tests/e2e/specs/bench.e2e.mjs`; ADR 0008 has the numbers).
