// SPDX-License-Identifier: AGPL-3.0-or-later
// Part pictures for the Parts list (docs/design/frozen's data-table): each part in its color, rendered by a second,
// hidden viewer from the same library, one at a time, and kept for the session. The window's own viewer, its camera,
// and its highlight never change.
import { createViewer, type Theme, type Viewer } from "@emmet/viewer";
import { library } from "./library";

/** Rendered at twice the 40 px the table draws, for sharp pictures on a Retina screen. */
const SIZE = 96;
const IDENTITY = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

let viewer: { v: Viewer; theme: Theme } | null = null;
const cache = new Map<string, Promise<string | null>>();
let queue: Promise<unknown> = Promise.resolve();

function hidden(theme: Theme): Viewer {
  if (viewer?.theme === theme) return viewer.v;
  viewer?.v.dispose();
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = SIZE;
  return (viewer = { v: createViewer(canvas, { partSource: library, theme }), theme }).v;
}

/** A data: URL of `part` in `color`, or null when it can't be drawn (the table shows no picture). */
export function partPicture(part: string, color: number, theme: Theme): Promise<string | null> {
  const key = `${theme}|${part}|${color}`;
  let p = cache.get(key);
  if (!p) {
    p = (queue = queue.then(async () => {
      try {
        const v = hidden(theme);
        await v.update([{ id: "p", part, color, matrix: IDENTITY, step: 1, submodel: [] }]);
        const { png } = await v.render({ view: "iso", size: SIZE });
        let s = "";
        for (let i = 0; i < png.length; i += 0x8000) s += String.fromCharCode(...png.subarray(i, i + 0x8000));
        return `data:image/png;base64,${btoa(s)}`;
      } catch {
        return null;
      }
    })) as Promise<string | null>;
    cache.set(key, p);
  }
  return p;
}
