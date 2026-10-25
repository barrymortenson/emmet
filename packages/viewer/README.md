# @emmet/viewer

The Emmet viewer (plan section 13): a three.js scene built from Emmet's placement list (`build/placements.json`).
Plain TypeScript with no UI framework inside it (eslint enforces no React). The app, render mode, and the bench all
use this same code, so an agent's render is what the window shows.

## How it draws

- **Unique parts once.** Each part id loads once through three's `LDrawLoader`, from a packed `.mpd` (the part file
  plus every subpart and primitive it uses, as `0 FILE` blocks) that the part source supplies. Main-color slots (LDraw
  16 for faces, 24 for edges) stay empty, to be filled per color; a part's own fixed colors stay.
- **Instancing.** One `InstancedMesh` per (part, color). Edges and conditional lines are instanced too: they share the
  mesh's `instanceMatrix`, so a diff only rewrites instance buffers. A 2,000-brick grid is 18 draw calls.
- **Diff updates.** `update()` diffs the new list against the scene by Emmet id (added, removed, moved, recolored),
  loads every new part first, then rewrites only the touched buckets in one synchronous pass. No frame shows half a
  diff, and the camera and highlight stay.
- **The camera never moves on its own.** Only `frame()` and `setView()` move it. The app frames a project's first
  scene by calling `setView("iso")`.
- **Missing parts are reported, not drawn with holes** (ADR 0005): the scene's are listed in `stats().failed`, and
  `render()` refuses to render while any are. The next update tries a failed part again (a dropped fetch, or a library
  installed since).
- **Renders are their own.** `render()` draws offscreen with only its `highlight` (none by default), queued behind
  updates, so the window's camera and highlight never leak into a render and two renders never mix.
- **On demand.** The scene draws when something changed (a diff, the camera, the size, orbit damping), not every
  frame.

## API

```ts
import { createViewer } from "@emmet/viewer";

const viewer = createViewer(canvas, { partSource, theme: "light", quality: "auto" });
await viewer.load(placements);                         // first scene (same as update)
const diff = await viewer.update(placements, { animate: "none" }); // { added, removed, moved, recolored, unchanged }
viewer.highlight(["roof/1", "door"]);                  // others dim; [] clears
viewer.frame(ids?);                                    // fit the ids (or everything), keeping the view direction
viewer.setView("iso" | "front" | "back" | "left" | "right" | "top");
viewer.setTheme("light" | "dark");
const { png, camera } = await viewer.render({ view: "iso", size: 768, highlight }); // offscreen; the window's camera stays
viewer.stats();        // drawCalls, triangles, frameMs, heap (WebView2 only), instances, buckets, parts, geometries,
                       // textures, programs, highlighted, failed, lastDiff
viewer.cameraMatrix(); // the camera's world matrix (tests check it doesn't move)
viewer.orbit(radians); // turn around the target, as a drag would (bench)
await viewer.nextFrame();
viewer.dispose();
```

A `PartSource` supplies LDraw text:

```ts
interface PartSource {
  parts(ids: string[]): Promise<{ parts: Record<string, string>; missing: Record<string, string[]> }>; // packed .mpd per id
  colors(): Promise<string>; // LDConfig.ldr
}
```

The app's part source (`apps/desktop/src/library.ts`) asks the host's `emmet-lib://localhost/pack?parts=…` for all
of a model's new parts in one request.

Coming later (section 13): `replayBuild`, `skipAnimation`, and `animate: "build"` with the build animation (1D),
`showSteps` (4E), and `on("pick")`.

## Coordinates

Placement matrices are column-major, in LDraw units with −Y up. The viewer flips LDraw to three's +Y up, so DSL x
(right) is +Z, y (back) is +X, and z (up) is +Y. `render()`'s `camera` is in that frame.

## Tests and bench

- `test/diff.test.mjs`: the diff, in Node, against `dist/` (`just test`).
- The viewer in the real app: `tests/e2e/specs/viewer.e2e.mjs`, `leak.e2e.mjs`, `visual.e2e.mjs` (`just e2e`).
- `just bench`: first load, slider → updated viewer, and orbit frame time for 93, 500, and 2,000 parts, in the app
  (`tests/e2e/specs/bench.e2e.mjs`; ADR 0008 has the numbers).
