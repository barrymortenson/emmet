// SPDX-License-Identifier: AGPL-3.0-or-later
// The pictures for building instructions (session 4E, ADR 0014): every job emmet_core::instructions asks for, drawn
// through viewer.render by one hidden viewer, as JPEGs on the color they'll sit on, with what the layout needs to
// know about each: the crop (the same for a group of steps, so the model stays put), the scale, the turn, where each
// sub-assembly copy goes, arrows for parts that don't go on from above, and a zoom box for new parts out of sight.
// The app's Export → Instructions and render mode's book job (emmet export --format pdf) both draw through here.
import { type Extent, type Placement, type RenderOptions, type Rendered, type Sighting, type Viewer, createViewer } from "@emmet/viewer";
import type { Drawn, DrawnStep, Jobs, Picture, Run, StepJob } from "./bindings";
import { base64, library } from "./library";

const QUALITY = 0.7;
/** New parts this little in sight (0 to 1) get a zoom box. */
const HIDDEN = 0.34;
/** A group's new parts this little in sight turn the model, if a turn shows them better by this much more. */
const TURN_SEEN = 0.5;
const TURN_GAIN = 0.15;
/** A quick render to choose the turn or the zoom's angle. */
const PROBE = 192;
const ZOOM = 480;
/** A step's picture prints at most about 4.2 inches tall and 6.6 wide (layout.rs): this many pixels is ~150 dpi. */
const MAX_HEIGHT = 640;
const MAX_WIDTH = 1100;
/** How far a cut edge fades, as a share of the picture's shorter side. */
const FEATHER = 0.07;
/** Room around the crop, in pixels, for the outline. */
const PAD = 10;
/** The step outline's width in pixels: about 0.6 mm on a printed page. */
const OUTLINE = 4.5;
const IDENTITY = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

type Rgb = [number, number, number];
type Box = { x: number; y: number; w: number; h: number };

/** Draws every job. `progress` hears how many pictures are done of how many. */
export async function drawBook(all: Placement[], jobs: Jobs, progress?: (done: number, total: number) => void): Promise<Drawn> {
  const total = 1 + jobs.runs.reduce((n, r) => n + r.groups.reduce((m, g) => m + g.length, 0), 0) + jobs.parts.length;
  let done = 0;
  const tick = () => progress?.(++done, total);
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 64;
  const viewer = createViewer(canvas, { partSource: library, theme: "light" });
  try {
    await viewer.load(all);
    const shot = await viewer.render({ view: "iso", size: jobs.cover_size, transparent: true });
    const cover = await jpeg(shot.png, await alphaBox(shot.png, PAD * 2), rgb(jobs.page_background));
    tick();
    const runs: DrawnStep[][] = [];
    for (const run of jobs.runs) {
      if (run.copy) await viewer.update(subScene(all, run));
      else if (runs.length) await viewer.update(all);
      runs.push(await drawRun(viewer, run, jobs, run.copy ? rgb(jobs.callout_background) : rgb(jobs.page_background), tick));
    }
    const parts: Picture[] = [];
    for (const p of jobs.parts) {
      await viewer.update([{ id: "p", part: p.part, color: p.color, matrix: IDENTITY, step: 1, submodel: [] }]);
      const r = await viewer.render({ view: "iso", size: jobs.part_size, transparent: true, shadow: false });
      parts.push(await jpeg(r.png, square(await alphaBox(r.png, 4)), rgb(jobs.part_background)));
      tick();
    }
    return { cover, runs, parts };
  } finally {
    viewer.dispose();
  }
}

function rgb(c: number[]): Rgb {
  return [c[0] ?? 255, c[1] ?? 255, c[2] ?? 255];
}

/** One copy of a sub-assembly on its own, its parts at their steps inside it. */
function subScene(all: Placement[], run: Run): Placement[] {
  return all
    .filter((p) => p.copies?.[run.depth]?.id === run.copy)
    .map((p) => {
      const copies = p.copies!.slice(run.depth + 1);
      return { ...p, copies: copies.length ? copies : undefined };
    });
}

async function drawRun(viewer: Viewer, run: Run, jobs: Jobs, background: Rgb, tick: () => void): Promise<DrawnStep[]> {
  const out: DrawnStep[] = [];
  let turn = 0;
  // A sub-assembly is small: one camera for all of it. The main model's camera follows the work.
  const steps = run.groups.flat();
  const queue: Pending[] = run.copy ? [{ steps, frame: viewer.boxOf() }] : cameraRuns(viewer, steps);
  for (let g = 0; g < queue.length; g++) {
    const { steps: group, frame, hidden } = queue[g]!;
    const last = group[group.length - 1]!.n;
    const options = (t: number): RenderOptions => ({ view: "iso", size: jobs.step_size, transparent: true, shadow: false, frame: frame ?? last, turn: t, outline: OUTLINE });
    // Whether to turn: the run's new parts (or, for a run split off at a step whose parts were out of sight, that
    // step's), seen from the turn before, at that step. Only when they're out of sight are the other turns tried.
    const check = hidden ?? { n: last, ids: group.flatMap((s) => s.new) };
    let end = await viewer.render({ ...options(turn), step: last, report: [...reported(group[group.length - 1]!), ...check.ids] });
    if (g > 0 && check.ids.length) {
      const probe = check.n === last ? end : await viewer.render({ ...options(turn), size: PROBE, step: check.n, report: check.ids });
      const now = mean(check.ids.map((id) => probe.report?.[id]?.seen ?? 0));
      const best = now < TURN_SEEN ? await bestTurn(viewer, check.ids, options, check.n, turn, now) : turn;
      if (best !== turn) {
        turn = best;
        end = await viewer.render({ ...options(turn), step: last, report: reported(group[group.length - 1]!) });
      }
    }
    const base = options(turn);
    // The drawn pixels, within the box the camera frames (a frame inside a big layout has model to every edge).
    const drawn = await alphaBox(end.png, PAD);
    const crop = within(drawn, end.framed, PAD);
    const feather = crop.w < drawn.w - 1 || crop.h < drawn.h - 1;
    const ldu = pixelsPerLdu(end, jobs.step_size);
    for (const [i, s] of group.entries()) {
      const r = s.n === last ? end : await viewer.render({ ...base, step: s.n, report: reported(s) });
      // A step whose new parts are out of sight from this camera starts a run of its own, which may turn the model.
      if (i > 0 && s.new.length && mean(s.new.map((id) => r.report?.[id]?.seen ?? 0)) < TURN_SEEN) {
        queue.splice(g + 1, 0, { steps: group.slice(i), frame, hidden: { n: s.n, ids: s.new } });
        break;
      }
      out.push(await step(viewer, s, r, crop, ldu, turn, background, feather));
      tick();
    }
  }
  return out;
}

/** A camera run to draw: its steps and frame, and for a run split off at a step whose new parts were out of sight,
 * that step and its parts (what decides the turn). */
type Pending = { steps: StepJob[]; frame: Extent | null; hidden?: { n: number; ids: string[] } };

/** How much room a frame leaves around a step's new parts: half their size, and at least this many LDU (3 studs). */
const CONTEXT = 60;
/** A frame at least this wide (12 studs), so a single brick still shows where it is. */
const MIN_FRAME = 240;
/** A frame within this factor of the whole build's size takes in the whole build. */
const WHOLE = 1.6;
/** A camera run goes on while its frame stays within this factor of the size it started at, and while each step's
 * own frame is no more than this many times smaller than the run's. */
const GROWTH = 1.6;
const ZOOM_IN = 2.5;

/** Consecutive steps that share one camera (section 08: the camera stays put through a run of steps). Each step's
 * frame is its new parts with room around them, or the whole build when that's not much more; a run takes the next
 * step while the frame doesn't grow past GROWTH times its first size. Then every step in a run frames the same box. */
function cameraRuns(viewer: Viewer, steps: StepJob[]): Pending[] {
  const runs: { steps: StepJob[]; frame: Extent | null; start: number }[] = [];
  for (const s of steps) {
    const f = stepFrame(viewer, s);
    const cur = runs[runs.length - 1];
    if (cur && (!f || !cur.frame)) {
      cur.steps.push(s);
      cur.frame ??= f;
      if (f && cur.start === 0) cur.start = size(f);
      continue;
    }
    if (cur && cur.frame && f) {
      // The run goes on unless the frame would grow too much, or this step's work wants a much closer look.
      const u = union(cur.frame, f);
      if (size(u) <= GROWTH * cur.start && size(f) * ZOOM_IN >= size(cur.frame)) {
        cur.steps.push(s);
        cur.frame = u;
        continue;
      }
    }
    runs.push({ steps: [s], frame: f, start: f ? size(f) : 0 });
  }
  return runs;
}

function stepFrame(viewer: Viewer, s: StepJob): Extent | null {
  const work = viewer.boxOf([...s.new, ...s.copies]);
  const build = viewer.boxOf(undefined, s.n);
  if (!work || !build) return build;
  const grow = Math.max(CONTEXT, size(work) / 2);
  const f: Extent = { min: [0, 0, 0], max: [0, 0, 0] };
  for (let i = 0; i < 3; i++) {
    const mid = (work.min[i]! + work.max[i]!) / 2;
    const half = Math.max((work.max[i]! - work.min[i]!) / 2 + grow, i === 1 ? 0 : MIN_FRAME / 2);
    // Within the build: no empty room past the model's edges.
    f.min[i] = Math.max(build.min[i]!, mid - half);
    f.max[i] = Math.min(build.max[i]!, mid + half);
  }
  return size(build) <= WHOLE * size(f) ? build : f;
}

function size(e: Extent): number {
  return Math.hypot(e.max[0] - e.min[0], e.max[1] - e.min[1], e.max[2] - e.min[2]);
}

function union(a: Extent, b: Extent): Extent {
  return {
    min: [Math.min(a.min[0], b.min[0]), Math.min(a.min[1], b.min[1]), Math.min(a.min[2], b.min[2])],
    max: [Math.max(a.max[0], b.max[0]), Math.max(a.max[1], b.max[1]), Math.max(a.max[2], b.max[2])],
  };
}

/** The crop inside the framed box (grown by `pad`). */
function within(crop: Box, framed: [number, number, number, number], pad: number): Box {
  const x0 = Math.max(crop.x, framed[0] - pad);
  const y0 = Math.max(crop.y, framed[1] - pad);
  const x1 = Math.min(crop.x + crop.w, framed[2] + pad);
  const y1 = Math.min(crop.y + crop.h, framed[3] + pad);
  return x1 > x0 && y1 > y0 ? { x: x0, y: y0, w: x1 - x0, h: y1 - y0 } : crop;
}

function reported(s: StepJob): string[] {
  return [...s.new, ...s.copies];
}

/** The turn that shows a group's new parts best, when it's better than the one before (`now`) by enough to turn. */
async function bestTurn(viewer: Viewer, ids: string[], options: (t: number) => RenderOptions, at: number, before: number, now: number): Promise<number> {
  let best = { turn: before, seen: now };
  for (const t of [0, 1, 2, 3]) {
    if (t === before) continue;
    const r = await viewer.render({ ...options(t), size: PROBE, step: at, report: ids });
    const seen = mean(ids.map((id) => r.report?.[id]?.seen ?? 0));
    if (seen > best.seen) best = { turn: t, seen };
  }
  return best.seen >= now + TURN_GAIN ? best.turn : before;
}

function mean(xs: number[]): number {
  return xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : 0;
}

/** How many pixels an LDU is at the center of what the camera frames. */
function pixelsPerLdu(r: Rendered, size: number): number {
  const [px, py, pz] = r.camera.position;
  const [tx, ty, tz] = r.camera.target;
  const d = Math.hypot(px - tx, py - ty, pz - tz);
  return size / (2 * d * Math.tan((r.camera.fov * Math.PI) / 360));
}

async function step(viewer: Viewer, s: StepJob, r: Rendered, crop: Box, ldu: number, turn: number, background: Rgb, feather: boolean): Promise<DrawnStep> {
  // No more pixels than a page prints at about 150 dpi.
  const k = Math.min(1, MAX_HEIGHT / crop.h, MAX_WIDTH / crop.w);
  const at = (p: [number, number]): [number, number] => [(p[0] - crop.x) * k, (p[1] - crop.y) * k];
  const report = r.report ?? {};
  const copies = s.copies.flatMap((id) => {
    const seen = report[id];
    if (!seen) return [];
    const [x, y] = at(seen.center);
    return [{ id, x, y }];
  });
  const arrows = s.new.flatMap((id) => {
    const seen = report[id];
    return seen?.moves ? [arrowFor(seen, at)] : [];
  });
  const hidden = s.new.filter((id) => (report[id]?.seen ?? 1) < HIDDEN);
  let zoom: DrawnStep["zoom"] = null;
  if (hidden.length) {
    const angle = await bestAngle(viewer, hidden, s.n, turn);
    const z = await viewer.render({ view: "iso", size: ZOOM, transparent: true, shadow: false, step: s.n, frame: hidden, turn: angle });
    const centers = hidden.map((id) => report[id]?.center).filter((c): c is [number, number] => !!c);
    const [x, y] = at([mean(centers.map((c) => c[0])), mean(centers.map((c) => c[1]))]);
    zoom = { picture: await jpeg(z.png, { x: 0, y: 0, w: ZOOM, h: ZOOM }, [255, 255, 255]), x, y };
  }
  return { n: s.n, picture: await jpeg(r.png, crop, background, k, feather), ldu: ldu * k, turn, copies, arrows, zoom };
}

/** An arrow beside a part, pointing the way it moves on, ending at its edge. */
function arrowFor(seen: Sighting, at: (p: [number, number]) => [number, number]) {
  const [ux, uy] = seen.moves!;
  const [x0, y0, x1, y1] = seen.box;
  const r = Math.max(8, Math.min(x1 - x0, y1 - y0) * 0.35);
  const [cx, cy] = seen.center;
  const to = at([cx - ux * r, cy - uy * r]);
  const from = at([cx - ux * (r + 90), cy - uy * (r + 90)]);
  return { from, to };
}

/** The turn from which the most of `ids` is in sight, framed close. */
async function bestAngle(viewer: Viewer, ids: string[], stepN: number, from: number): Promise<number> {
  let best = { turn: from, seen: -1 };
  for (const t of [from, (from + 1) % 4, (from + 3) % 4, (from + 2) % 4]) {
    const r = await viewer.render({ view: "iso", size: PROBE, transparent: true, shadow: false, step: stepN, frame: ids, turn: t, report: ids });
    const seen = mean(ids.map((id) => r.report?.[id]?.seen ?? 0));
    if (seen > best.seen + 0.05) best = { turn: t, seen };
  }
  return best.turn;
}

async function bitmap(png: Uint8Array): Promise<ImageBitmap> {
  return createImageBitmap(new Blob([png as BlobPart], { type: "image/png" }));
}

/** The box around a picture's drawn (not transparent) pixels, grown by `pad`; the whole picture when it's empty. */
async function alphaBox(png: Uint8Array, pad: number): Promise<Box> {
  const img = await bitmap(png);
  const { width: w, height: h } = img;
  const c = new Canvas2d(w, h);
  c.ctx.drawImage(img, 0, 0);
  const data = c.ctx.getImageData(0, 0, w, h).data;
  let [x0, y0, x1, y1] = [w, h, -1, -1];
  for (let y = 0; y < h; y++) {
    const row = y * w * 4;
    for (let x = 0; x < w; x++) {
      if (data[row + x * 4 + 3]! > 8) {
        if (x < x0) x0 = x;
        if (x > x1) x1 = x;
        if (y < y0) y0 = y;
        y1 = y;
      }
    }
  }
  img.close();
  if (x1 < 0) return { x: 0, y: 0, w, h };
  x0 = Math.max(0, x0 - pad);
  y0 = Math.max(0, y0 - pad);
  x1 = Math.min(w - 1, x1 + pad);
  y1 = Math.min(h - 1, y1 + pad);
  return { x: x0, y: y0, w: x1 - x0 + 1, h: y1 - y0 + 1 };
}

/** The box made square around its center (a part's picture fills a square in its box). */
function square(b: Box): Box {
  const side = Math.max(b.w, b.h);
  return { x: b.x - (side - b.w) / 2, y: b.y - (side - b.h) / 2, w: side, h: side };
}

/** The crop of a transparent PNG on `background`, as a JPEG. */
/** The crop of a transparent PNG on `background`, scaled by `k`, as a JPEG. With `feather`, its edges fade into the
 * background: a close frame cuts through the model, and a soft edge reads as "and so on" where a hard one reads as a
 * wall. */
async function jpeg(png: Uint8Array, crop: Box, background: Rgb, k = 1, feather = false): Promise<Picture> {
  const img = await bitmap(png);
  const w = Math.max(1, Math.round(crop.w * k));
  const h = Math.max(1, Math.round(crop.h * k));
  const art = new Canvas2d(w, h);
  art.ctx.imageSmoothingQuality = "high";
  // The whole image, moved and scaled (WebKit draws nothing for a source rectangle that runs past the image, as a
  // part's square crop can).
  art.ctx.drawImage(img, -crop.x * k, -crop.y * k, img.width * k, img.height * k);
  img.close();
  if (feather) {
    const edge = Math.round(Math.min(w, h) * FEATHER);
    const mask = new Canvas2d(w, h);
    mask.ctx.fillRect(0, 0, w, h);
    mask.ctx.globalCompositeOperation = "destination-out";
    for (const [x0, y0, x1, y1, rx, ry, rw, rh] of [
      [0, 0, edge, 0, 0, 0, edge, h],
      [w, 0, w - edge, 0, w - edge, 0, edge, h],
      [0, 0, 0, edge, 0, 0, w, edge],
      [0, h, 0, h - edge, 0, h - edge, w, edge],
    ] as const) {
      const g = mask.ctx.createLinearGradient(x0, y0, x1, y1);
      g.addColorStop(0, "rgba(0, 0, 0, 1)");
      g.addColorStop(1, "rgba(0, 0, 0, 0)");
      mask.ctx.fillStyle = g;
      mask.ctx.fillRect(rx, ry, rw, rh);
    }
    art.ctx.globalCompositeOperation = "destination-in";
    art.ctx.drawImage(mask.canvas, 0, 0);
  }
  const c = new Canvas2d(w, h);
  c.ctx.fillStyle = `rgb(${background[0]}, ${background[1]}, ${background[2]})`;
  c.ctx.fillRect(0, 0, w, h);
  c.ctx.drawImage(art.canvas, 0, 0);
  const blob = await c.blob("image/jpeg", QUALITY);
  return { jpeg: base64(new Uint8Array(await blob.arrayBuffer())), width: w, height: h };
}

/** A 2D canvas (an HTMLCanvasElement: OffscreenCanvas's toBlob isn't in every webview Emmet runs in). */
class Canvas2d {
  readonly canvas: HTMLCanvasElement;
  readonly ctx: CanvasRenderingContext2D;
  constructor(w: number, h: number) {
    this.canvas = document.createElement("canvas");
    this.canvas.width = w;
    this.canvas.height = h;
    this.ctx = this.canvas.getContext("2d", { willReadFrequently: true })!;
  }
  blob(type: string, quality: number): Promise<Blob> {
    return new Promise((resolve, reject) => this.canvas.toBlob((b) => (b ? resolve(b) : reject(new Error("the webview couldn't encode the JPEG"))), type, quality));
  }
}
