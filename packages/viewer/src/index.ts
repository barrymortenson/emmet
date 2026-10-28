// SPDX-License-Identifier: AGPL-3.0-or-later
// The Emmet viewer (plan section 13): a three.js scene built from Emmet's placement list, one InstancedMesh per
// (part, color), with instanced edges. Plain TypeScript with no UI framework; the README lists the API.

import {
  Box3,
  Color,
  DirectionalLight,
  Group,
  HemisphereLight,
  InstancedBufferGeometry,
  InstancedMesh,
  LineSegments,
  type Material,
  Matrix4,
  PerspectiveCamera,
  Scene,
  Vector3,
  WebGLRenderer,
} from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { CAP_MS, DROP, PART_MS, buildOrder, drop, stagger } from "./animate.js";
import { type Diff, type Placement, bucketKey, diffPlacements, dirtyBuckets } from "./diff.js";
import { type PartGeometry, PartCache, type PartSource, partKey } from "./parts.js";

export { CAP_MS, DROP, PART_MS, buildOrder, drop, stagger };
export { bucketKey, diffPlacements, dirtyBuckets };
export type { Diff, Placement, PartSource };
export type { PartGeometry } from "./parts.js";

export type View = "iso" | "front" | "back" | "left" | "right" | "top";
export type Theme = "light" | "dark";
export type HighlightMode = "issue" | "select" | "hover";

export interface ViewerOptions {
  partSource: PartSource;
  theme?: Theme;
  /** "auto" follows the device pixel ratio up to 2; "low" draws at 1. */
  quality?: "auto" | "low";
}

export interface UpdateOptions {
  /** "build" drops the added parts in one at a time, in build order (D-34); "none" (the default, and every slider
   * run) applies the update at once. Reduced motion turns "build" off. */
  animate?: "build" | "none";
}

export interface RenderOptions {
  view?: View;
  /** Width and height in pixels (default 768). */
  size?: number;
  highlight?: string[];
}

/** Where a render's camera was, in the viewer's frame (Y up, LDraw units). */
export interface RenderCamera {
  view: View;
  position: [number, number, number];
  target: [number, number, number];
  fov: number;
}

export interface Rendered {
  png: Uint8Array;
  camera: RenderCamera;
}

export interface Stats {
  drawCalls: number;
  triangles: number;
  /** Mean time between the last frames drawn back to back, in ms (0 before any). */
  frameMs: number;
  /** The JS heap in bytes where the webview reports it (WebView2), else null (WKWebView). */
  heap: number | null;
  instances: number;
  buckets: number;
  parts: number;
  geometries: number;
  textures: number;
  programs: number;
  /** How many ids are highlighted. */
  highlighted: number;
  /** Parts the build animation hasn't put in place yet (0 when none is running). */
  animating: number;
  /** Parts the scene uses that couldn't load, with why; they aren't drawn. */
  failed: Record<string, string>;
  /** Placements on screen per diff kind in the last update. */
  lastDiff: { added: number; removed: number; moved: number; recolored: number; unchanged: number };
}

export interface Viewer {
  load(placements: Placement[]): Promise<Diff>;
  update(placements: Placement[], options?: UpdateOptions): Promise<Diff>;
  /** Highlights the ids and dims the rest; "issue" also tints them (a finding's parts). [] clears it. */
  highlight(ids: string[], options?: { mode?: HighlightMode }): void;
  /** The highlighted ids. */
  highlighted(): string[];
  /** Jumps a running build animation to its end. */
  skipAnimation(): void;
  frame(ids?: string[]): void;
  setView(view: View): void;
  setTheme(theme: Theme): void;
  /** A PNG of the whole model from `view`, drawn offscreen with only `highlight` highlighted: the window's camera,
   * canvas, and highlight don't change. Refuses when a part the scene uses failed or the last update failed. */
  render(options?: RenderOptions): Promise<Rendered>;
  stats(): Stats;
  /** The camera's world matrix, column-major (tests check the camera never moves on its own). */
  cameraMatrix(): number[];
  /** Turns the camera around the target by `radians`, as a drag would. For benches and tests. */
  orbit(radians: number): void;
  /** Resolves after the next frame is drawn. */
  nextFrame(): Promise<void>;
  dispose(): void;
}

const BACKGROUND: Record<Theme, number> = { light: 0xf3f4f6, dark: 0x1f2125 };
/** Renders (for agents, exports, and baselines) look the same whatever the window's theme. */
const RENDER_BACKGROUND = new Color(0xf3f4f6);
const DIM = new Color(0.35, 0.35, 0.38);
const WHITE = new Color(1, 1, 1);
/** A finding's parts: their own color, pushed toward red. */
const ISSUE = new Color(1, 0.42, 0.36);
const HIDDEN = new Matrix4().makeScale(0, 0, 0);
const FOV = 35;

/** Where the camera sits for each view, from the target, in the viewer's frame (Y up; LDraw is flipped to it). DSL x
 * (right) is +Z here, y (back) is +X, and z (up) is +Y. */
const VIEWS: Record<View, { dir: Vector3; up: Vector3 }> = {
  iso: { dir: new Vector3(-1, 0.85, 1).normalize(), up: new Vector3(0, 1, 0) },
  front: { dir: new Vector3(-1, 0, 0), up: new Vector3(0, 1, 0) },
  back: { dir: new Vector3(1, 0, 0), up: new Vector3(0, 1, 0) },
  left: { dir: new Vector3(0, 0, -1), up: new Vector3(0, 1, 0) },
  right: { dir: new Vector3(0, 0, 1), up: new Vector3(0, 1, 0) },
  top: { dir: new Vector3(0, 1, 0), up: new Vector3(1, 0, 0) },
};

interface Drawable {
  object: InstancedMesh | LineSegments;
  kind: "mesh" | "edges" | "conditional";
}

/** One part in a running build animation: where it is in its bucket, where it ends up, and when it starts. */
interface Falling {
  bucket: Bucket;
  index: number;
  home: Matrix4;
  at: number;
  done: boolean;
}

/** Every instance of one part in one color. */
interface Bucket {
  key: string;
  part: PartGeometry;
  color: number;
  ids: string[];
  capacity: number;
  drawables: Drawable[];
}

export function createViewer(canvas: HTMLCanvasElement, options: ViewerOptions): Viewer {
  return new EmmetViewer(canvas, options);
}

class EmmetViewer implements Viewer {
  private renderer: WebGLRenderer;
  private scene = new Scene();
  /** LDraw is −Y up; this flips it to three's +Y up. */
  private root = new Group();
  private camera = new PerspectiveCamera(FOV, 1, 1, 100_000);
  private controls: OrbitControls;
  private cache: PartCache;
  private placements = new Map<string, Placement>();
  private buckets = new Map<string, Bucket>();
  private lit = new Set<string>();
  private mode: HighlightMode = "select";
  /** The running build animation: its parts, and when it started (performance.now()). */
  private falling: { parts: Falling[]; start: number } | null = null;
  private dirty = true;
  private raf = 0;
  private frames: number[] = [];
  private waiters: (() => void)[] = [];
  private lastDiff: Stats["lastDiff"] = { added: 0, removed: 0, moved: 0, recolored: 0, unchanged: 0 };
  private resize: ResizeObserver | null = null;
  private theme: Theme;
  private quality: "auto" | "low";
  private offscreen: WebGLRenderer | null = null;
  /** Updates and renders run one at a time, in call order, so a newer scene never lands under an older one and two
   * renders never share a highlight. */
  private queue: Promise<unknown> = Promise.resolve();
  /** Why the last update failed, if it did: a render then refuses rather than draw a stale scene. */
  private broken: string | null = null;

  constructor(
    private canvas: HTMLCanvasElement,
    options: ViewerOptions,
  ) {
    this.theme = options.theme ?? "light";
    this.quality = options.quality ?? "auto";
    this.cache = new PartCache(options.partSource);
    this.renderer = new WebGLRenderer({ canvas, antialias: true });
    this.renderer.setPixelRatio(this.pixelRatio());
    this.scene.background = new Color(BACKGROUND[this.theme]);
    this.root.rotation.x = Math.PI;
    this.scene.add(this.root);
    addLights(this.scene);
    this.camera.position.set(-400, 340, 400);
    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.enableDamping = true;
    this.controls.addEventListener("change", () => this.invalidate());
    // A click in the viewer skips the build animation; orbiting (a drag) doesn't.
    let down: [number, number] | null = null;
    canvas.addEventListener("pointerdown", (e) => (down = [e.clientX, e.clientY]));
    canvas.addEventListener("pointerup", (e) => {
      if (down && Math.hypot(e.clientX - down[0], e.clientY - down[1]) < 4) this.skipAnimation();
      down = null;
    });
    this.fit();
    if (typeof ResizeObserver !== "undefined") {
      // Applied on the next frame, so a resize never lays out again inside the observer's own callback.
      this.resize = new ResizeObserver(() =>
        requestAnimationFrame(() => {
          this.fit();
          this.invalidate();
        }),
      );
      this.resize.observe(canvas);
    }
    this.loop = this.loop.bind(this);
    this.raf = requestAnimationFrame(this.loop);
  }

  load(placements: Placement[]): Promise<Diff> {
    return this.update(placements);
  }

  update(placements: Placement[], options: UpdateOptions = {}): Promise<Diff> {
    return this.enqueue(async () => {
      try {
        const diff = await this.apply(placements, options.animate === "build" && !reducedMotion());
        this.broken = null;
        return diff;
      } catch (e) {
        this.broken = e instanceof Error ? e.message : String(e);
        throw e;
      }
    });
  }

  private enqueue<T>(job: () => Promise<T>): Promise<T> {
    const run = this.queue.then(job);
    this.queue = run.catch(() => {});
    return run;
  }

  /** Failed parts the current scene uses, with why. */
  private failedHere(): [string, string][] {
    const used = new Set([...this.placements.values()].map((p) => partKey(p.part)));
    return [...this.cache.failed].filter(([id]) => used.has(id));
  }

  /** Loads every new part first, then changes the scene in one synchronous pass, so no frame shows half a diff. With
   * `animate`, the added parts then drop in one at a time; an animation still running finishes first. */
  private async apply(placements: Placement[], animate: boolean): Promise<Diff> {
    const failedBefore = new Set(this.cache.failed.keys());
    await this.cache.ensure(placements.map((p) => p.part));
    this.skipAnimation();
    const diff = diffPlacements(this.placements.values(), placements);
    const dirty = dirtyBuckets(diff, this.placements);
    // A part that failed before and loads now has no bucket yet, though its placements didn't change.
    for (const p of placements) {
      const k = partKey(p.part);
      if (failedBefore.has(k) && this.cache.get(p.part)) dirty.add(bucketKey(p));
    }
    this.placements = new Map(placements.map((p) => [p.id, p]));
    const members = new Map<string, Placement[]>();
    for (const p of this.placements.values()) {
      const k = bucketKey(p);
      if (!dirty.has(k)) continue;
      let list = members.get(k);
      if (!list) members.set(k, (list = []));
      list.push(p);
    }
    for (const k of dirty) this.writeBucket(k, members.get(k) ?? []);
    if (animate && diff.added.length) this.startAnimation(diff.added);
    this.lastDiff = {
      added: diff.added.length,
      removed: diff.removed.length,
      moved: diff.moved.length,
      recolored: diff.recolored.length,
      unchanged: diff.unchanged,
    };
    this.invalidate();
    return diff;
  }

  /** Rewrites one bucket's instances, growing its buffers when they're full and dropping it when it's empty. */
  private writeBucket(key: string, list: Placement[]): void {
    let bucket = this.buckets.get(key);
    const first = list[0];
    const part = first && this.cache.get(first.part);
    if (!first || !part) {
      if (bucket) this.dropBucket(bucket);
      return;
    }
    if (!bucket || bucket.capacity < list.length) {
      if (bucket) this.dropBucket(bucket);
      bucket = this.makeBucket(key, part, first.color, Math.max(4, 2 ** Math.ceil(Math.log2(list.length))));
    }
    bucket.ids = list.map((p) => p.id);
    const m = new Matrix4();
    for (const d of bucket.drawables) {
      if (!(d.object instanceof InstancedMesh)) continue;
      const mesh = d.object;
      list.forEach((p, i) => mesh.setMatrixAt(i, m.fromArray(p.matrix)));
      mesh.count = list.length;
      mesh.instanceMatrix.needsUpdate = true;
    }
    for (const d of bucket.drawables) {
      if (d.object instanceof LineSegments) (d.object.geometry as InstancedBufferGeometry).instanceCount = list.length;
    }
    this.paint(bucket);
  }

  private makeBucket(key: string, part: PartGeometry, color: number, capacity: number): Bucket {
    const colors = this.cache.colors;
    const drawables: Drawable[] = [];
    const meshes = part.pieces.filter((p) => p.kind === "mesh");
    const meshObjects = meshes.map((piece) => {
      const materials = piece.materials.map((m) => m ?? colors.face(color));
      const mesh = new InstancedMesh(piece.geometry, materials.length === 1 ? materials[0]! : materials, capacity);
      mesh.frustumCulled = false;
      mesh.count = 0;
      mesh.setColorAt(0, WHITE);
      return mesh;
    });
    meshObjects.forEach((object) => drawables.push({ object, kind: "mesh" }));
    // Lines share the first mesh's instance matrices; a part with lines but no faces gets a mesh-less holder.
    const holder = meshObjects[0] ?? new InstancedMesh(undefined, undefined, capacity);
    for (const piece of part.pieces) {
      if (piece.kind === "mesh") continue;
      const geometry = new InstancedBufferGeometry();
      for (const [name, attr] of Object.entries(piece.geometry.attributes)) geometry.setAttribute(name, attr);
      for (const g of piece.geometry.groups) geometry.addGroup(g.start, g.count, g.materialIndex);
      geometry.setAttribute("instanceMatrix", holder.instanceMatrix);
      geometry.instanceCount = 0;
      const materials: Material[] = piece.materials.map((m) =>
        piece.kind === "edges"
          ? colors.edge(m ?? colors.edgeFor(color))
          : colors.conditional(m ?? colors.conditionalFor(color)),
      );
      const lines = new LineSegments(geometry, materials.length === 1 ? materials[0]! : materials);
      lines.frustumCulled = false;
      drawables.push({ object: lines, kind: piece.kind });
    }
    if (!meshObjects.length) {
      // It only carries the matrices; it has no geometry to draw.
      holder.visible = false;
      drawables.unshift({ object: holder, kind: "mesh" });
    }
    for (const d of drawables) this.root.add(d.object);
    const bucket: Bucket = { key, part, color, ids: [], capacity, drawables };
    this.buckets.set(key, bucket);
    return bucket;
  }

  private dropBucket(bucket: Bucket): void {
    for (const d of bucket.drawables) {
      this.root.remove(d.object);
      if (d.object instanceof InstancedMesh) d.object.dispose();
      else {
        // The line geometry borrows the part's attributes, which other buckets still draw: let go of them first, so
        // disposing frees only this bucket's own binding state.
        const g = d.object.geometry;
        for (const name of Object.keys(g.attributes)) g.deleteAttribute(name);
        g.dispose();
      }
    }
    this.buckets.delete(bucket.key);
  }

  /** Sets each instance's tint: full color, or dimmed while something else is highlighted, or tinted as an issue. */
  private paint(bucket: Bucket): void {
    const any = this.lit.size > 0;
    const on = this.mode === "issue" ? ISSUE : WHITE;
    for (const d of bucket.drawables) {
      if (!(d.object instanceof InstancedMesh) || !d.object.instanceColor) continue;
      const mesh = d.object;
      bucket.ids.forEach((id, i) => mesh.setColorAt(i, !any ? WHITE : this.isLit(id) ? on : DIM));
      mesh.instanceColor!.needsUpdate = true;
    }
  }

  /** True when the id, or a submodel copy holding the part (its id up to a "/"), is lit: a copy's id lights all its
   * parts (towers/2 lights towers/2/wall/3). */
  private isLit(id: string): boolean {
    for (let key = id; ; ) {
      if (this.lit.has(key)) return true;
      const cut = key.lastIndexOf("/");
      if (cut <= 0) return false;
      key = key.slice(0, cut);
    }
  }

  highlight(ids: string[], options: { mode?: HighlightMode } = {}): void {
    this.lit = new Set(ids);
    this.mode = options.mode ?? "select";
    for (const b of this.buckets.values()) this.paint(b);
    this.invalidate();
  }

  highlighted(): string[] {
    return [...this.lit];
  }

  /** Hides the added parts and schedules them in build order. */
  private startAnimation(added: Placement[]): void {
    const order = buildOrder(added);
    const gap = stagger(order.length);
    const index = new Map<string, [Bucket, number]>();
    for (const b of this.buckets.values()) b.ids.forEach((id, i) => index.set(id, [b, i]));
    const parts: Falling[] = [];
    order.forEach((p, n) => {
      const found = index.get(p.id);
      if (!found) return; // a part that failed to load has no instance to animate
      const [bucket, i] = found;
      parts.push({ bucket, index: i, home: new Matrix4().fromArray(p.matrix), at: n * gap, done: false });
      this.setInstance(bucket, i, HIDDEN);
    });
    if (parts.length) this.falling = { parts, start: performance.now() };
  }

  /** Moves the falling parts to where they are at `now`; the animation ends when the last one lands. */
  private stepAnimation(now: number): void {
    const f = this.falling;
    if (!f) return;
    const m = new Matrix4();
    let left = 0;
    for (const part of f.parts) {
      if (part.done) continue;
      const d = drop(now - f.start - part.at);
      if (!d.visible) {
        this.setInstance(part.bucket, part.index, HIDDEN);
      } else {
        m.copy(part.home);
        m.elements[13]! -= d.lift; // up is −Y in LDraw
        this.setInstance(part.bucket, part.index, m);
      }
      part.done = d.done;
      if (!d.done) left++;
    }
    if (!left) this.falling = null;
  }

  skipAnimation(): void {
    const f = this.falling;
    if (!f) return;
    for (const part of f.parts) this.setInstance(part.bucket, part.index, part.home);
    this.falling = null;
    this.invalidate();
  }

  /** Sets one instance's matrix in every mesh of its bucket (the lines share the first mesh's). */
  private setInstance(bucket: Bucket, index: number, m: Matrix4): void {
    for (const d of bucket.drawables) {
      if (!(d.object instanceof InstancedMesh)) continue;
      d.object.setMatrixAt(index, m);
      d.object.instanceMatrix.needsUpdate = true;
    }
  }

  frame(ids?: string[]): void {
    const box = this.bounds(ids);
    if (box.isEmpty()) return;
    const dir = this.camera.position.clone().sub(this.controls.target).normalize();
    this.place(this.camera, box, dir.lengthSq() > 0 ? dir : VIEWS.iso.dir, this.camera.up);
    this.controls.target.copy(box.getCenter(new Vector3()));
    this.controls.update();
    this.invalidate();
  }

  setView(view: View): void {
    const box = this.bounds();
    const v = VIEWS[view];
    this.camera.up.copy(v.up);
    if (box.isEmpty()) {
      this.camera.position.copy(v.dir).multiplyScalar(600);
      this.controls.target.set(0, 0, 0);
    } else {
      this.place(this.camera, box, v.dir, v.up);
      this.controls.target.copy(box.getCenter(new Vector3()));
    }
    this.controls.update();
    this.invalidate();
  }

  setTheme(theme: Theme): void {
    this.theme = theme;
    this.scene.background = new Color(BACKGROUND[theme]);
    this.invalidate();
  }

  render(options: RenderOptions = {}): Promise<Rendered> {
    return this.enqueue(() => this.renderNow(options));
  }

  private async renderNow(options: RenderOptions): Promise<Rendered> {
    if (this.broken) throw new Error(`can't render: the last update failed (${this.broken})`);
    const failed = this.failedHere();
    if (failed.length) throw new Error(`can't render: ${failed.map(([id, why]) => `${id} (${why})`).join("; ")}`);
    const size = Math.round(Math.min(Math.max(options.size ?? 768, 16), 4096));
    const name = options.view ?? "iso";
    const view = VIEWS[name];
    if (!view) throw new Error(`there's no view '${String(name)}'`);
    // A second renderer on a detached canvas: the window's canvas, camera, and size stay as they are.
    const canvas = document.createElement("canvas");
    this.offscreen ??= new WebGLRenderer({ canvas, antialias: true, preserveDrawingBuffer: true });
    const r = this.offscreen;
    r.setPixelRatio(1);
    r.setSize(size, size, false);
    const camera = new PerspectiveCamera(FOV, 1, 1, 100_000);
    camera.up.copy(view.up);
    const box = this.bounds();
    if (box.isEmpty()) camera.position.copy(view.dir).multiplyScalar(600);
    else this.place(camera, box, view.dir, view.up);
    // The render's own highlight (none unless asked) and background, swapped in and back around one synchronous
    // draw, so the window never draws a frame with them and its own highlight never leaks into a render.
    const before = this.lit;
    const mode = this.mode;
    const background = this.scene.background;
    this.lit = new Set(options.highlight ?? []);
    this.mode = "select";
    for (const b of this.buckets.values()) this.paint(b);
    this.scene.background = RENDER_BACKGROUND;
    // A render shows the finished model, never a frame of the build animation, which carries on in the window.
    const falling = this.falling;
    if (falling) for (const part of falling.parts) this.setInstance(part.bucket, part.index, part.home);
    try {
      r.render(this.scene, camera);
    } finally {
      this.scene.background = background;
      this.lit = before;
      this.mode = mode;
      for (const b of this.buckets.values()) this.paint(b);
      if (falling) {
        for (const part of falling.parts) part.done = false;
        this.stepAnimation(performance.now());
        this.invalidate();
      }
    }
    // preserveDrawingBuffer keeps the pixels until the next draw on this canvas.
    const blob = await new Promise<Blob | null>((resolve) => r.domElement.toBlob(resolve, "image/png"));
    if (!blob) throw new Error("the webview couldn't encode the PNG");
    const target = box.isEmpty() ? new Vector3() : box.getCenter(new Vector3());
    return {
      png: new Uint8Array(await blob.arrayBuffer()),
      camera: { view: name, position: camera.position.toArray(), target: target.toArray(), fov: camera.fov },
    };
  }

  stats(): Stats {
    const info = this.renderer.info;
    const frames = this.frames;
    const deltas = frames.slice(1).map((t, i) => t - frames[i]!);
    const memory = (performance as Performance & { memory?: { usedJSHeapSize: number } }).memory;
    let instances = 0;
    for (const b of this.buckets.values()) instances += b.ids.length;
    return {
      drawCalls: info.render.calls,
      triangles: info.render.triangles,
      frameMs: deltas.length ? deltas.reduce((a, b) => a + b, 0) / deltas.length : 0,
      heap: memory ? memory.usedJSHeapSize : null,
      instances,
      buckets: this.buckets.size,
      parts: this.cache.size,
      geometries: info.memory.geometries,
      textures: info.memory.textures,
      programs: info.programs?.length ?? 0,
      highlighted: this.lit.size,
      animating: this.falling ? this.falling.parts.filter((p) => !p.done).length : 0,
      failed: Object.fromEntries(this.failedHere()),
      lastDiff: { ...this.lastDiff },
    };
  }

  cameraMatrix(): number[] {
    this.camera.updateMatrixWorld();
    return this.camera.matrixWorld.toArray();
  }

  orbit(radians: number): void {
    const offset = this.camera.position.clone().sub(this.controls.target);
    offset.applyAxisAngle(this.camera.up, radians);
    this.camera.position.copy(this.controls.target).add(offset);
    this.camera.lookAt(this.controls.target);
    this.invalidate();
  }

  nextFrame(): Promise<void> {
    this.invalidate();
    return new Promise((resolve) => this.waiters.push(resolve));
  }

  dispose(): void {
    cancelAnimationFrame(this.raf);
    this.resize?.disconnect();
    this.controls.dispose();
    for (const b of [...this.buckets.values()]) this.dropBucket(b);
    this.cache.dispose();
    this.renderer.dispose();
    this.offscreen?.dispose();
  }

  private invalidate(): void {
    this.dirty = true;
  }

  /** Draws only when something changed: a diff, the camera, the size, or orbit damping still settling. */
  private loop(now: number): void {
    this.raf = requestAnimationFrame(this.loop);
    const moving = this.controls.update();
    if (this.falling) {
      this.stepAnimation(performance.now());
      this.dirty = true;
    }
    if (!this.dirty && !moving) {
      this.frames = [];
      return;
    }
    this.dirty = false;
    this.renderer.render(this.scene, this.camera);
    this.frames.push(now);
    if (this.frames.length > 61) this.frames.shift();
    const waiters = this.waiters.splice(0);
    for (const w of waiters) w();
  }

  private pixelRatio(): number {
    return this.quality === "low" ? 1 : Math.min(globalThis.devicePixelRatio || 1, 2);
  }

  private fit(): void {
    const w = this.canvas.clientWidth || this.canvas.width || 1;
    const h = this.canvas.clientHeight || this.canvas.height || 1;
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
  }

  /** The box around the given ids (or everything), in the scene's frame. */
  private bounds(ids?: string[]): Box3 {
    this.root.updateMatrixWorld(true);
    const box = new Box3();
    const want = ids && new Set(ids);
    const m = new Matrix4();
    const corner = new Vector3();
    for (const b of this.buckets.values()) {
      const local = partBox(b.part);
      b.ids.forEach((id) => {
        if (want && !want.has(id)) return;
        m.fromArray(this.placements.get(id)!.matrix).premultiply(this.root.matrixWorld);
        for (let i = 0; i < 8; i++) {
          corner.set(
            i & 1 ? local.max.x : local.min.x,
            i & 2 ? local.max.y : local.min.y,
            i & 4 ? local.max.z : local.min.z,
          );
          box.expandByPoint(corner.applyMatrix4(m));
        }
      });
    }
    return box;
  }

  /** Puts `camera` on `dir` from the box's center, just far enough back that every corner of the box is in view. */
  private place(camera: PerspectiveCamera, box: Box3, dir: Vector3, up: Vector3): void {
    const center = box.getCenter(new Vector3());
    camera.up.copy(up);
    camera.position.copy(center).add(dir);
    camera.lookAt(center);
    camera.updateMatrixWorld();
    const right = new Vector3().setFromMatrixColumn(camera.matrixWorld, 0);
    const upward = new Vector3().setFromMatrixColumn(camera.matrixWorld, 1);
    const tanV = Math.tan((camera.fov * Math.PI) / 360);
    const tanH = tanV * camera.aspect;
    let distance = 0;
    const v = new Vector3();
    for (let i = 0; i < 8; i++) {
      v.set(i & 1 ? box.max.x : box.min.x, i & 2 ? box.max.y : box.min.y, i & 4 ? box.max.z : box.min.z).sub(center);
      // A corner sits `toward` nearer the camera than the center, so the camera backs off that much more.
      const toward = v.dot(dir);
      distance = Math.max(distance, Math.abs(v.dot(right)) / tanH + toward, Math.abs(v.dot(upward)) / tanV + toward);
    }
    distance = Math.max(distance * 1.08, 1);
    camera.position.copy(center).addScaledVector(dir, distance);
    camera.near = Math.max(distance / 100, 0.1);
    camera.far = distance * 100;
    camera.lookAt(center);
    camera.updateProjectionMatrix();
    camera.updateMatrixWorld();
  }
}

const boxes = new WeakMap<PartGeometry, Box3>();

function partBox(part: PartGeometry): Box3 {
  let box = boxes.get(part);
  if (!box) {
    box = new Box3();
    for (const p of part.pieces) {
      p.geometry.computeBoundingBox();
      if (p.geometry.boundingBox) box.union(p.geometry.boundingBox);
    }
    boxes.set(part, box);
  }
  return box;
}

function addLights(scene: Scene): void {
  scene.add(new HemisphereLight(0xffffff, 0x8a8f99, 2.0));
  const key = new DirectionalLight(0xffffff, 1.8);
  key.position.set(-1, 2, 1.4);
  scene.add(key);
  const fill = new DirectionalLight(0xffffff, 0.6);
  fill.position.set(1.5, 0.5, -1);
  scene.add(fill);
}

/** The system's reduced-motion setting (plan section 13): it turns the build animation off. */
function reducedMotion(): boolean {
  return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
}
