// SPDX-License-Identifier: AGPL-3.0-or-later
// Part pictures for the Parts list and a step's parts box (docs/design/frozen's data-table and parts-box): each part
// in its color, and a sub-assembly's sub-steps, rendered by a second, hidden viewer from the same library, one at a
// time, and kept for the session. A render looks the same in either theme (its background is transparent), so one
// hidden viewer serves both. The window's own viewer, its camera, and its highlight never change.
import { createViewer, type Placement, type Viewer } from "@emmet/viewer";
import { library } from "./library";

/** Rendered at 96 px for the 40 px the table draws, sharp on a Retina screen. */
const SIZE = 96;
const IDENTITY = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

let viewer: Viewer | null = null;
const cache = new Map<string, Promise<string | null>>();
let queue: Promise<unknown> = Promise.resolve();

function hidden(): Viewer {
  if (viewer) return viewer;
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = SIZE;
  return (viewer = createViewer(canvas, { partSource: library }));
}

/** Renders `placements` in the hidden viewer (one at a time, behind every other picture) as a data: URL. A picture
 * that fails (a part the library doesn't have) fails alone: the queue goes on to the next. */
function picture(placements: Placement[]): Promise<string> {
  const run = queue.then(async () => {
    const v = hidden();
    await v.update(placements);
    const { png } = await v.render({ view: "iso", size: SIZE, transparent: true });
    let s = "";
    for (let i = 0; i < png.length; i += 0x8000) s += String.fromCharCode(...png.subarray(i, i + 0x8000));
    return `data:image/png;base64,${btoa(s)}`;
  });
  queue = run.catch(() => {});
  return run;
}

/** Some placed parts together (a sub-assembly's copy built up to one of its sub-steps), kept by `key`. */
export function assemblyPicture(key: string, placements: { id: string; part: string; color: number; matrix: (number | null)[]; step: number }[]): Promise<string | null> {
  let p = cache.get(`assembly|${key}`);
  if (!p) {
    const list = placements.map((x) => ({ id: x.id, part: x.part, color: x.color, matrix: x.matrix as number[], step: x.step, submodel: [] }));
    p = picture(list).catch(() => {
      cache.delete(`assembly|${key}`);
      return null;
    });
    cache.set(`assembly|${key}`, p);
  }
  return p;
}

/** A data: URL of `part` in `color`, or null when it can't be drawn (the table shows no picture, and the next time the
 * table asks, it's tried again). */
export function partPicture(part: string, color: number): Promise<string | null> {
  const key = `${part}|${color}`;
  let p = cache.get(key);
  if (!p) {
    p = picture([{ id: "p", part, color, matrix: IDENTITY, step: 1, submodel: [] }]).catch(() => {
      cache.delete(key);
      return null;
    });
    cache.set(key, p);
  }
  return p;
}
