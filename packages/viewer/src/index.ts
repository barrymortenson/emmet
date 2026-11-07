// SPDX-License-Identifier: AGPL-3.0-or-later
// The Emmet viewer (plan section 13): a three.js scene built from Emmet's placement list, one InstancedMesh per
// (part, finish) with each instance's color its own, and instanced edges. Plain TypeScript with no UI framework; the
// README lists the API.

import {
  Box3,
  Color,
  Group,
  InstancedBufferAttribute,
  InstancedBufferGeometry,
  InstancedMesh,
  LineSegments,
  type Material,
  Matrix4,
  Mesh,
  MeshBasicMaterial,
  NeutralToneMapping,
  type Object3D,
  OrthographicCamera,
  PCFShadowMap,
  PerspectiveCamera,
  PlaneGeometry,
  Quaternion,
  Ray,
  Raycaster,
  Scene,
  ShadowMaterial,
  type Texture,
  Vector2,
  Vector3,
  WebGLRenderer,
} from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import {
  CAP_MS,
  DROP,
  FAST_MS,
  GLIDE_MS,
  LEAVE_MS,
  PART_MS,
  type Pace,
  buildOrder,
  drop,
  glide,
  leave,
  stagger,
} from "./animate.js";
import { type Diff, type Placement, bucketKey, diffPlacements, dirtyBuckets, topStep } from "./diff.js";
import { type ColorInfo, type FinishMaterials, Finishes, LINE_COLOR, type Studio, addStudio, finishKey, studioRoom } from "./materials.js";
import { Outline } from "./outline.js";
import { type PartGeometry, PartCache, type PartSource, partKey, simplifyStuds } from "./parts.js";

export { CAP_MS, DROP, FAST_MS, GLIDE_MS, LEAVE_MS, PART_MS, buildOrder, drop, glide, leave, stagger };
export { bucketKey, diffPlacements, dirtyBuckets, simplifyStuds, topStep };
export type { Diff, Pace, Placement, PartSource };
export type { PartGeometry } from "./parts.js";

export type View = "iso" | "front" | "back" | "left" | "right" | "top";
export type Theme = "light" | "dark";
/** A highlight's kind (plan section 13): a finding's parts (tinted red), the user's pick, or an agent's pointing.
 * The part under the pointer is drawn lighter alongside any of them. */
export type HighlightMode = "issue" | "select" | "agent";
/** Settings' quality: what it costs to draw a frame. "auto" picks one by the GPU. */
export type Quality = "auto" | "fast" | "balanced" | "best";
export type Preset = Exclude<Quality, "auto">;

export interface ViewerOptions {
  partSource: PartSource;
  theme?: Theme;
  quality?: Quality;
  pace?: Pace;
}

export interface UpdateOptions {
  /** "build" puts the change in as the build animation does (D-34): added parts drop in one at a time in build
   * order, removed ones lift away, moved ones glide; "none" (the default, and every knob's run) applies it at once.
   * The pace setting and reduced motion can turn "build" off. */
  animate?: "build" | "none";
}

export interface RenderOptions {
  view?: View;
  /** Width and height in pixels (default 768). */
  size?: number;
  highlight?: string[];
  /** The build up to this step, as the steps show it (its new parts outlined); the whole model by default. */
  step?: number;
  /** No background: the picture's pixels around the model are transparent (the Parts list's and the parts box's
   * pictures). */
  transparent?: boolean;
  /** What the camera frames: the build up to this step (a run of steps shares one camera, so the model stays put from
   * page to page), or these ids' box grown by two studs each way (a zoom box). By default the build up to `step`, or
   * everything. */
  frame?: number | string[] | Extent;
  /** Quarter turns of the model around its vertical axis, as a builder turns it to work on another side: 2 shows the
   * far corner. */
  turn?: number;
  /** The ground shadow (true by default; instructions draw none, as the books do). */
  shadow?: boolean;
  /** Ids (a part's, or a copy's for all its parts) to report on: where each is in the picture and how much of it is
   * in sight. */
  report?: string[];
  /** The width of a step's outline in pixels (1.5 by default; a printed page needs it wider). */
  outline?: number;
}

/** A box in the scene's frame (LDraw units, Y up), as `boxOf` gives it. */
export interface Extent {
  min: [number, number, number];
  max: [number, number, number];
}

/** Where an id is in a render (pixels from the top left) and the share of it in sight, from 0 (hidden behind other
 * parts, or out of the picture) to 1. */
export interface Sighting {
  box: [number, number, number, number];
  center: [number, number];
  seen: number;
  /** For one part that doesn't go on from above (turned on its side, or upside down under something): the way it
   * moves on, its own down, as a unit vector in the picture. */
  moves?: [number, number];
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
  /** For each id in `report` that's in the scene. */
  report?: Record<string, Sighting>;
  /** Where the box the camera frames is in the picture (pixels: left, top, right, bottom). */
  framed: [number, number, number, number];
}

/** A point on the canvas, in CSS pixels from its top left; `visible` is false when it's behind the camera or off the
 * canvas. */
export interface ScreenPoint {
  x: number;
  y: number;
  visible: boolean;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ViewerEvents {
  /** The part under the pointer, or null, with where the pointer is on the canvas (CSS pixels). */
  hover: (hit: { id: string; x: number; y: number } | null) => void;
  /** A click: the part clicked, or null for nothing. Shift, ⌘, or Ctrl held means add to the selection. */
  pick: (id: string | null, modifiers: { add: boolean }) => void;
  /** Shift-drag's box while it's drawn (canvas CSS pixels), and null when it's let go. */
  boxing: (box: Rect | null) => void;
  /** The parts in a box let go: those whose centers are inside it. */
  box: (ids: string[]) => void;
  /** The camera or the canvas changed: what's drawn over the viewer (pins, the parts box's leader) follows it. */
  camera: () => void;
  /** A build animation started (true) or ended (false). */
  animation: (running: boolean) => void;
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
  /** Buckets drawing their parts' low-detail studs. */
  lowDetail: number;
  /** Buckets drawn with transparency, back to front. */
  transparent: number;
  parts: number;
  geometries: number;
  textures: number;
  programs: number;
  /** How many ids are highlighted, and how. */
  highlighted: number;
  mode: HighlightMode;
  /** The part under the pointer. */
  hovered: string | null;
  /** Parts the steps or the cut-away hide. */
  hidden: number;
  /** The step shown (null for the whole model), how many parts its outline goes around, and the cut-away's top layer
   * (null when it's off). */
  step: number | null;
  outlined: number;
  cut: number | null;
  /** Parts the build animation hasn't put in place yet (0 when none is running). */
  animating: number;
  /** Parts the scene uses that couldn't load, with why; they aren't drawn. */
  failed: Record<string, string>;
  /** How many placements the last update added, removed, moved, recolored, or left unchanged. */
  lastDiff: { added: number; removed: number; moved: number; recolored: number; unchanged: number };
  quality: Preset;
  pace: Pace;
  shadow: boolean;
}

export interface Viewer {
  load(placements: Placement[]): Promise<Diff>;
  update(placements: Placement[], options?: UpdateOptions): Promise<Diff>;
  /** Highlights the ids: the rest fade toward the background, and the ids draw over them in their own colors; "issue"
   * tints them red (a finding's parts). [] clears it. A copy's id lights all its parts. */
  highlight(ids: string[], options?: { mode?: HighlightMode }): void;
  /** The highlighted ids. */
  highlighted(): string[];
  /** The build up to step `upTo` (earlier steps whole, its new parts outlined, later steps hidden), or with null the
   * whole model. */
  showSteps(upTo?: number | null): void;
  /** Hides every part above the cut-away's top layer (a layer is a plate, counted up from the model's bottom), or
   * with null shows them all again. */
  cutAway(layer: number | null): void;
  /** How many layers (plates) tall the model is. */
  layers(): number;
  /** Plays the whole model from empty in build order (nothing, when the pace is Off or reduced motion is on). */
  replayBuild(): void;
  /** Jumps a running build animation to its end. */
  skipAnimation(): void;
  setPace(pace: Pace): void;
  frame(ids?: string[]): void;
  setView(view: View): void;
  setTheme(theme: Theme): void;
  setQuality(quality: Quality): void;
  /** Turns the camera around the target, as a drag would: `radians` around, and `tilt` up or down. */
  orbit(radians: number, tilt?: number): void;
  /** Moves the camera toward the target (a factor under 1) or away from it (over 1). */
  zoom(factor: number): void;
  /** A PNG of the whole model (or the build up to `step`) from `view`, drawn offscreen with only `highlight`
   * highlighted: the window's camera,
   * canvas, and highlight don't change. Refuses when a part the scene uses failed or the last update failed. */
  render(options?: RenderOptions): Promise<Rendered>;
  /** The box around the ids (a copy's id takes in its parts), or around the build up to `step`, or everything; null
   * when there's nothing. For framing instruction steps. */
  boxOf(ids?: string[], step?: number): Extent | null;
  /** A PNG of what the window shows, from its own camera, at `scale` times the canvas's size (a screenshot). */
  snapshot(scale?: number): Promise<Uint8Array>;
  /** Where the top of the ids' box is on the canvas (a finding's pin, a sub-assembly's leader), or null for none. */
  screenPoint(ids: string[]): ScreenPoint | null;
  /** The ids shown whose centers are inside a box on the canvas (CSS pixels). */
  idsInBox(box: Rect): string[];
  /** The part at a point on the canvas (CSS pixels), or null. */
  pickAt(x: number, y: number): string | null;
  on<K extends keyof ViewerEvents>(event: K, fn: ViewerEvents[K]): () => void;
  stats(): Stats;
  /** The camera's world matrix, column-major (tests check the camera never moves on its own). */
  cameraMatrix(): number[];
  /** Draws `frames` frames back to back, each turned a little and waited for (a one-pixel read, since WebKit's
   * gl.finish() doesn't wait), and returns the mean time a frame takes to draw: what a frame costs, where the refresh
   * rate hides it from frameMs. For the bench. */
  drawCost(frames: number): number;
  /** Resolves after the next frame is drawn. */
  nextFrame(): Promise<void>;
  dispose(): void;
}

const BACKGROUND: Record<Theme, number> = { light: 0xf3f4f6, dark: 0x1f2125 };
/** Renders (for agents, exports, and baselines) look the same whatever the window's theme. */
const RENDER_BACKGROUND = new Color(0xf3f4f6);
/** How far a highlight fades the rest toward the background: far enough that the highlighted parts are what you see,
 * not so far that you lose where they are. */
const VEIL = 0.8;
const WHITE = new Color(1, 1, 1);
/** A finding's parts: their own color, pushed toward red. */
const ISSUE = new Color(1, 0.42, 0.36);
/** A step's new parts' outline: the design's red (`--fg-bad`) in each theme. */
const OUTLINE: Record<Theme, Color> = { light: new Color(0xc0261b), dark: new Color(0xff6b5e) };
/** How much lighter the part under the pointer draws. */
const HOVER = 0.28;
/** How far each theme softens the lines toward the faces' color: LDraw's near-black edges turn to noise at a distance,
 * the more so on a dark background. */
const SOFT_EDGES: Record<Theme, number> = { light: 0.18, dark: 0.32 };
const HIDDEN = new Matrix4().makeScale(0, 0, 0);
const FOV = 35;
/** A zoom box's frame: the ids' box grown by two studs each way, so the parts around them show where they are. */
const ZOOM_MARGIN = 40;
const PLATE = 8;
/** A stud is 12 LDU across: under this many drawing-buffer (device) pixels on screen, per quality, its bucket draws
 * low-detail studs. */
const LOD_PX: Record<Preset, number> = { fast: Infinity, balanced: 5, best: 2.5 };
/** A press that moves less than this is a click, not a drag. */
const DRAG_PX = 4;

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
  /** Draws in the bucket's own colors; false for a part's fixed color, whose per-instance color is only a tint. */
  main: boolean;
  /** A main line's per-instance colors (a mesh's are its instanceColor). */
  colors: InstancedBufferAttribute | null;
}

/** One part in a running build animation. */
interface Motion {
  bucket: Bucket;
  index: number;
  kind: "drop" | "glide" | "leave";
  home: Matrix4;
  from: Matrix4 | null;
  at: number;
  done: boolean;
}

/** Every instance of one part in one finish. */
interface Bucket {
  key: string;
  part: PartGeometry;
  /** What it draws now: the part, or its low-detail variant. */
  drawn: PartGeometry;
  finish: FinishMaterials;
  ids: string[];
  /** Removed parts still lifting away in a build animation, drawn after the ids. */
  ghosts: Placement[];
  capacity: number;
  drawables: Drawable[];
  /** Around its instances, in the scene's frame (for the stud LOD and the transparent order). */
  center: Vector3;
  radius: number;
}

export function createViewer(canvas: HTMLCanvasElement, options: ViewerOptions): Viewer {
  return new EmmetViewer(canvas, options);
}

/** The preset "auto" picks for a GPU: Fast on a software renderer, Balanced on integrated graphics, Best otherwise. */
export function presetFor(gpu: string): Preset {
  if (/swiftshader|llvmpipe|software|basic render/i.test(gpu)) return "fast";
  if (/intel|mali|adreno|powervr/i.test(gpu)) return "balanced";
  return "best";
}

class EmmetViewer implements Viewer {
  private renderer: WebGLRenderer;
  private scene = new Scene();
  /** LDraw is −Y up; this flips it to three's +Y up. */
  private root = new Group();
  private camera = new PerspectiveCamera(FOV, 1, 1, 100_000);
  private controls: OrbitControls;
  private cache: PartCache;
  private finishes = new Finishes();
  private studio: Studio;
  private ground: Mesh<PlaneGeometry, ShadowMaterial>;
  private placements = new Map<string, Placement>();
  private buckets = new Map<string, Bucket>();
  /** Where each id draws: its bucket and instance. */
  private where = new Map<string, [Bucket, number]>();
  private lit = new Set<string>();
  private mode: HighlightMode = "select";
  private hovered: string | null = null;
  private step: number | null = null;
  private cut: number | null = null;
  /** The model's bottom (LDraw's largest y) and height, in LDU, for the cut-away's layers. */
  private extent = { bottom: 0, height: 0 };
  /** The highlighted parts again, drawn over the veil: copies of their buckets' instances, rebuilt when the highlight
   * or the scene changes. The outlined parts likewise, drawn flat into the outline's mask. */
  private overlay = new Scene();
  private overlayRoot = new Group();
  private overlayBuilt = false;
  private outlineScene = new Scene();
  private outlineRoot = new Group();
  private outlineBuilt = false;
  private outline = new Outline();
  private veil = new Mesh(
    new PlaneGeometry(2, 2),
    new MeshBasicMaterial({ transparent: true, opacity: VEIL, depthTest: false, depthWrite: false, toneMapped: false }),
  );
  private veilScene = new Scene();
  private veilCamera = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
  /** The running build animation: its parts, and when it started (performance.now()). */
  private motion: { parts: Motion[]; start: number } | null = null;
  private pace: Pace;
  private dirty = true;
  private lodDirty = true;
  private shadowDirty = true;
  private raf = 0;
  private frames: number[] = [];
  private waiters: (() => void)[] = [];
  private listeners = new Map<keyof ViewerEvents, Set<(...args: never[]) => void>>();
  private lastDiff: Stats["lastDiff"] = { added: 0, removed: 0, moved: 0, recolored: 0, unchanged: 0 };
  private resize: ResizeObserver | null = null;
  private theme: Theme;
  private quality: Quality;
  private preset: Preset;
  private offscreen: { renderer: WebGLRenderer; environment: Texture } | null = null;
  /** Updates and renders run one at a time, in call order, so a newer scene never lands under an older one and two
   * renders never share a highlight. */
  private queue: Promise<unknown> = Promise.resolve();
  /** Why the last update failed, if it did: a render then refuses rather than draw a stale scene. */
  private broken: string | null = null;
  private raycaster = new Raycaster();
  private pointer: { x: number; y: number } | null = null;
  private hoverQueued = false;
  private cleanup: (() => void)[] = [];

  constructor(
    private canvas: HTMLCanvasElement,
    options: ViewerOptions,
  ) {
    this.theme = options.theme ?? "light";
    this.quality = options.quality ?? "auto";
    this.pace = options.pace ?? "animated";
    this.cache = new PartCache(options.partSource);
    this.renderer = makeRenderer(canvas, false);
    this.preset = this.resolve(this.quality);
    this.renderer.setPixelRatio(this.pixelRatio());
    this.scene.background = new Color(BACKGROUND[this.theme]);
    this.root.rotation.x = Math.PI;
    this.scene.add(this.root);
    this.studio = addStudio(this.scene, this.renderer);
    this.finishes.setRoom(this.studio.environment);
    this.ground = new Mesh(new PlaneGeometry(1, 1), new ShadowMaterial({ opacity: 0.16, depthWrite: false }));
    this.ground.rotation.x = -Math.PI / 2;
    this.ground.receiveShadow = true;
    this.ground.visible = false;
    this.scene.add(this.ground);
    this.overlayRoot.rotation.x = Math.PI;
    this.overlay.add(this.overlayRoot);
    addStudio(this.overlay, this.renderer, this.studio.environment);
    this.outlineRoot.rotation.x = Math.PI;
    this.outlineScene.add(this.outlineRoot);
    this.veil.position.z = -0.5;
    this.veilScene.add(this.veil);
    this.camera.position.set(-400, 340, 400);
    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.enableDamping = true;
    this.controls.addEventListener("change", () => {
      this.invalidate();
      this.lodDirty = true;
    });
    this.applyPreset();
    this.listen();
    this.fit();
    if (typeof ResizeObserver !== "undefined") {
      // Applied on the next frame, so a resize never lays out again inside the observer's own callback.
      this.resize = new ResizeObserver(() =>
        requestAnimationFrame(() => {
          this.fit();
          this.moved();
        }),
      );
      this.resize.observe(canvas);
    }
    this.loop = this.loop.bind(this);
    this.raf = requestAnimationFrame(this.loop);
  }

  /** Pointer gestures: hovering names a part; a click picks one (and skips the build animation); shift-drag draws a
   * box that selects; any other drag orbits (OrbitControls). */
  private listen(): void {
    const c = this.canvas;
    let down: { x: number; y: number; box: boolean } | null = null;
    const local = (e: PointerEvent) => {
      const r = c.getBoundingClientRect();
      return { x: e.clientX - r.left, y: e.clientY - r.top };
    };
    const rect = (a: { x: number; y: number }, b: { x: number; y: number }): Rect => ({
      x: Math.min(a.x, b.x),
      y: Math.min(a.y, b.y),
      width: Math.abs(a.x - b.x),
      height: Math.abs(a.y - b.y),
    });
    const onDown = (e: PointerEvent) => {
      down = { ...local(e), box: e.shiftKey && e.button === 0 };
      if (down.box) {
        // OrbitControls listens on the same canvas: it mustn't turn the camera while a box is drawn.
        this.controls.enabled = false;
        // The box keeps its moves when the pointer leaves the canvas; a pointer the webview can't capture (a test's
        // synthetic one) still draws the box.
        try {
          c.setPointerCapture(e.pointerId);
        } catch {
          /* drawn without capture */
        }
      }
    };
    const onMove = (e: PointerEvent) => {
      const at = local(e);
      if (down?.box) {
        this.emit("boxing", rect(down, at));
        return;
      }
      if (down) return;
      this.pointer = at;
      this.queueHover();
    };
    const onUp = (e: PointerEvent) => {
      const at = local(e);
      const start = down;
      down = null;
      if (!start) return;
      const moved = Math.hypot(at.x - start.x, at.y - start.y);
      if (start.box) {
        this.controls.enabled = true;
        this.emit("boxing", null);
        if (moved >= DRAG_PX) {
          this.emit("box", this.idsInBox(rect(start, at)));
          return;
        }
      }
      if (moved < DRAG_PX) {
        // A click in the viewer skips the build animation; orbiting (a drag) doesn't.
        this.skipAnimation();
        this.emit("pick", this.pickAt(at.x, at.y), { add: e.shiftKey || e.metaKey || e.ctrlKey });
      }
    };
    // A cancelled pointer (or a box let go where the canvas couldn't capture it) ends the gesture.
    const onCancel = () => {
      if (down?.box) {
        this.controls.enabled = true;
        this.emit("boxing", null);
      }
      down = null;
    };
    const onLeave = () => {
      this.pointer = null;
      if (this.hovered) {
        this.setHover(null);
        this.emit("hover", null);
      }
    };
    c.addEventListener("pointerdown", onDown);
    c.addEventListener("pointermove", onMove);
    c.addEventListener("pointerup", onUp);
    c.addEventListener("pointerleave", onLeave);
    c.addEventListener("pointercancel", onCancel);
    c.addEventListener("lostpointercapture", onCancel);
    this.cleanup.push(() => {
      c.removeEventListener("pointercancel", onCancel);
      c.removeEventListener("lostpointercapture", onCancel);
      c.removeEventListener("pointerdown", onDown);
      c.removeEventListener("pointermove", onMove);
      c.removeEventListener("pointerup", onUp);
      c.removeEventListener("pointerleave", onLeave);
    });
  }

  /** Hover picks at most once a frame, at the pointer's latest place. */
  private queueHover(): void {
    if (this.hoverQueued) return;
    this.hoverQueued = true;
    requestAnimationFrame(() => {
      this.hoverQueued = false;
      const p = this.pointer;
      const id = p ? this.pickAt(p.x, p.y) : null;
      this.setHover(id);
      this.emit("hover", id && p ? { id, x: p.x, y: p.y } : null);
    });
  }

  private setHover(id: string | null): void {
    if (id === this.hovered) return;
    const before = this.hovered;
    this.hovered = id;
    for (const k of [before, id]) {
      const at = k ? this.where.get(k) : undefined;
      if (at) this.paintOne(at[0], at[1]);
    }
    this.overlayBuilt = false;
    this.invalidate();
  }

  on<K extends keyof ViewerEvents>(event: K, fn: ViewerEvents[K]): () => void {
    let set = this.listeners.get(event);
    if (!set) this.listeners.set(event, (set = new Set()));
    const f = fn as (...args: never[]) => void;
    set.add(f);
    return () => void set.delete(f);
  }

  private emit<K extends keyof ViewerEvents>(event: K, ...args: Parameters<ViewerEvents[K]>): void {
    for (const fn of this.listeners.get(event) ?? []) (fn as (...a: Parameters<ViewerEvents[K]>) => void)(...args);
  }

  load(placements: Placement[]): Promise<Diff> {
    return this.update(placements);
  }

  update(placements: Placement[], options: UpdateOptions = {}): Promise<Diff> {
    return this.enqueue(async () => {
      try {
        const diff = await this.apply(placements, options.animate === "build" && this.animates());
        this.broken = null;
        return diff;
      } catch (e) {
        this.broken = e instanceof Error ? e.message : String(e);
        throw e;
      }
    });
  }

  private animates(): boolean {
    return this.pace !== "off" && !reducedMotion();
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

  /** The bucket a placement draws in: its part and its color's finish. */
  private keyOf = (p: Pick<Placement, "part" | "color">): string => `${partKey(p.part)}|${finishKey(this.color(p.color))}`;

  private color(code: number): ColorInfo {
    return this.cache.colors.get(code);
  }

  /** Loads every new part first, then changes the scene in one synchronous pass, so no frame shows half a diff. With
   * `animate`, the change then plays as the build animation; one still running finishes first. */
  private async apply(placements: Placement[], animate: boolean): Promise<Diff> {
    const failedBefore = new Set(this.cache.failed.keys());
    await this.cache.ensure(placements.map((p) => p.part));
    const before = this.placements;
    const diff = diffPlacements(before.values(), placements);
    const dirty = dirtyBuckets(diff, before, this.keyOf);
    // A part that failed before and loads now has no bucket yet, though its placements didn't change.
    for (const p of placements) {
      if (failedBefore.has(partKey(p.part)) && this.cache.get(p.part)) dirty.add(this.keyOf(p));
    }
    // An update that changes nothing (the app sends the scene again when the steps or a compare close) leaves a
    // running build animation, such as Replay build, to play on.
    if (!dirty.size) {
      // Still the new list: a rebuild can renumber the steps without moving a part, and the steps read them.
      this.placements = new Map(placements.map((p) => [p.id, p]));
      if (this.step != null || this.cut != null) this.refill();
      this.lastDiff = { added: 0, removed: 0, moved: 0, recolored: 0, unchanged: diff.unchanged };
      return diff;
    }
    this.skipAnimation();
    this.placements = new Map(placements.map((p) => [p.id, p]));
    const members = new Map<string, Placement[]>();
    for (const p of this.placements.values()) {
      const k = this.keyOf(p);
      if (!dirty.has(k)) continue;
      let list = members.get(k);
      if (!list) members.set(k, (list = []));
      list.push(p);
    }
    // Removed parts, and a recolored part's old self, lift away from where they were, so their buckets keep them as
    // ghosts until the animation ends.
    const ghosts = new Map<string, Placement[]>();
    if (animate) {
      for (const p of [...diff.removed, ...diff.recolored.map((r) => before.get(r.id)!)]) {
        if (!this.cache.get(p.part)) continue;
        const k = this.keyOf(p);
        dirty.add(k);
        let list = ghosts.get(k);
        if (!list) ghosts.set(k, (list = []));
        list.push(p);
      }
    }
    // The cut-away counts layers from the model's bottom: measured first, and with a new bottom every bucket is cut
    // again, not only the ones the diff touched.
    const bottom = this.extent.bottom;
    this.measure();
    if (this.cut != null && this.extent.bottom !== bottom) for (const b of this.buckets.values()) dirty.add(b.key);
    for (const k of dirty) this.writeBucket(k, members.get(k) ?? [], ghosts.get(k) ?? []);
    if (animate) this.startAnimation(diff, before, ghosts);
    this.lastDiff = {
      added: diff.added.length,
      removed: diff.removed.length,
      moved: diff.moved.length,
      recolored: diff.recolored.length,
      unchanged: diff.unchanged,
    };
    this.overlayBuilt = false;
    this.outlineBuilt = false;
    this.shadowDirty = true;
    this.lodDirty = true;
    this.invalidate();
    return diff;
  }

  /** Rewrites one bucket's instances, growing its buffers when they're full and dropping it when it's empty. */
  private writeBucket(key: string, list: Placement[], ghosts: Placement[] = []): void {
    let bucket = this.buckets.get(key);
    const first = list[0] ?? ghosts[0];
    const part = first && this.cache.get(first.part);
    if (bucket) for (const id of bucket.ids) if (this.where.get(id)?.[0] === bucket) this.where.delete(id);
    if (!first || !part) {
      if (bucket) this.dropBucket(bucket);
      return;
    }
    const need = list.length + ghosts.length;
    if (!bucket || bucket.capacity < need) {
      const drawn = bucket?.drawn ?? part;
      if (bucket) this.dropBucket(bucket);
      bucket = this.makeBucket(key, part, drawn, this.color(first.color), Math.max(4, 2 ** Math.ceil(Math.log2(need))));
    }
    const b = bucket;
    b.ids = list.map((p) => p.id);
    b.ghosts = ghosts;
    b.ids.forEach((id, i) => this.where.set(id, [b, i]));
    this.fill(b);
  }

  /** Every instance's matrix (hidden where the steps or the cut-away hide it) and color, and the bucket's bounds. */
  private fill(bucket: Bucket): void {
    const n = bucket.ids.length + bucket.ghosts.length;
    const m = new Matrix4();
    const box = new Box3();
    const local = partBox(bucket.part);
    const corner = new Box3();
    for (let i = 0; i < n; i++) {
      const p = this.at(bucket, i);
      m.fromArray(p.matrix);
      this.setInstance(bucket, i, this.shows(p) ? m : HIDDEN);
      if (i < bucket.ids.length) box.union(corner.copy(local).applyMatrix4(m));
    }
    for (const d of bucket.drawables) {
      if (d.object instanceof InstancedMesh) d.object.count = n;
      else (d.object.geometry as InstancedBufferGeometry).instanceCount = n;
    }
    // In the scene's frame: the root flips LDraw's y and z.
    const center = box.isEmpty() ? new Vector3() : box.getCenter(new Vector3());
    bucket.center.set(center.x, -center.y, -center.z);
    bucket.radius = box.isEmpty() ? 0 : box.getSize(new Vector3()).length() / 2;
    this.paint(bucket);
  }

  /** The placement drawn at a bucket's instance: an id's, or past the ids a ghost's. */
  private at(bucket: Bucket, i: number): Placement {
    return i < bucket.ids.length ? this.placements.get(bucket.ids[i]!)! : bucket.ghosts[i - bucket.ids.length]!;
  }

  /** Whether the steps and the cut-away leave a part showing. */
  private shows(p: Placement): boolean {
    if (this.step != null && topStep(p) > this.step) return false;
    if (this.cut != null) {
      // Its bottom's height over the model's bottom (LDraw's y grows down).
      const above = this.extent.bottom - partBottom(p, this.cache.get(p.part));
      if (above >= this.cut * PLATE - 0.01) return false;
    }
    return true;
  }

  private makeBucket(key: string, part: PartGeometry, drawn: PartGeometry, color: ColorInfo, capacity: number): Bucket {
    const bucket: Bucket = {
      key,
      part,
      drawn,
      finish: this.finishes.for(color),
      ids: [],
      ghosts: [],
      capacity,
      drawables: [],
      center: new Vector3(),
      radius: 0,
    };
    this.build(bucket);
    this.buckets.set(key, bucket);
    return bucket;
  }

  /** The bucket's meshes and lines, for what it draws now. */
  private build(bucket: Bucket): void {
    const { capacity, finish } = bucket;
    const drawables: Drawable[] = [];
    for (const piece of bucket.drawn.pieces) {
      if (piece.kind !== "mesh") continue;
      const material = piece.fixed ? this.finishes.fixedFace(this.cache.colors.of(piece.fixed)) : finish.face;
      const mesh = new InstancedMesh(piece.geometry, material, capacity);
      mesh.frustumCulled = false;
      mesh.count = 0;
      mesh.setColorAt(0, WHITE);
      mesh.castShadow = !finish.transparent;
      drawables.push({ object: mesh, kind: "mesh", main: !piece.fixed, colors: null });
    }
    // Lines share the first mesh's instance matrices; a part with lines but no faces gets a mesh-less holder.
    let holder = drawables[0]?.object as InstancedMesh | undefined;
    if (!holder) {
      // It only carries the matrices; it has no geometry to draw.
      holder = new InstancedMesh(undefined, undefined, capacity);
      holder.visible = false;
      holder.frustumCulled = false;
      drawables.push({ object: holder, kind: "mesh", main: false, colors: null });
    }
    for (const piece of bucket.drawn.pieces) {
      if (piece.kind === "mesh") continue;
      const geometry = new InstancedBufferGeometry();
      for (const [name, attr] of Object.entries(piece.geometry.attributes)) geometry.setAttribute(name, attr);
      geometry.setAttribute("instanceMatrix", holder.instanceMatrix);
      geometry.instanceCount = 0;
      let colors: InstancedBufferAttribute | null = null;
      let material: Material;
      if (piece.fixed) material = this.finishes.fixedLine(this.cache.colors.of(piece.fixed), piece.kind);
      else {
        colors = new InstancedBufferAttribute(new Float32Array(capacity * 3).fill(1), 3);
        geometry.setAttribute(LINE_COLOR, colors);
        material = piece.kind === "edges" ? finish.edge : finish.conditional;
      }
      const lines = new LineSegments(geometry, material);
      lines.frustumCulled = false;
      // Fast leaves the conditional lines out of the window (renders keep them).
      if (piece.kind === "conditional") lines.visible = this.preset !== "fast";
      drawables.push({ object: lines, kind: piece.kind, main: !piece.fixed, colors });
    }
    for (const d of drawables) this.root.add(d.object);
    bucket.drawables = drawables;
  }

  /** Takes a bucket's meshes and lines out of the scene and frees what's only theirs. */
  private unbuild(bucket: Bucket): void {
    this.overlayBuilt = false;
    this.outlineBuilt = false;
    for (const d of bucket.drawables) {
      this.root.remove(d.object);
      if (d.object instanceof InstancedMesh) d.object.dispose();
      else disposeLines(d.object);
    }
    bucket.drawables = [];
  }

  private dropBucket(bucket: Bucket): void {
    this.unbuild(bucket);
    this.buckets.delete(bucket.key);
  }

  private paint(bucket: Bucket): void {
    this.overlayBuilt = false;
    const n = bucket.ids.length + bucket.ghosts.length;
    for (let i = 0; i < n; i++) this.paintOne(bucket, i);
  }

  /** One instance's colors: its own (a fixed color only tinted), red-tinted as an issue, lighter under the pointer,
   * and `fade` of the way in from the background (the build animation's fade). The highlight's own fade is the veil's
   * (draw()). */
  private paintOne(bucket: Bucket, i: number, fade = 1): void {
    const p = this.at(bucket, i);
    const c = this.color(p.color);
    const id = i < bucket.ids.length ? bucket.ids[i]! : null;
    const issue = id != null && this.mode === "issue" && this.isLit(id);
    const hover = id != null && id === this.hovered;
    const background = BACKGROUND_COLORS[this.theme];
    const color = new Color();
    for (const d of bucket.drawables) {
      if (d.object instanceof InstancedMesh) {
        if (!d.object.instanceColor || !d.object.visible) continue;
        color.copy(d.main ? c.face : WHITE);
        if (issue) color.multiply(ISSUE);
        if (hover) color.lerp(WHITE, HOVER);
        if (fade < 1) color.lerp(background, 1 - fade);
        d.object.setColorAt(i, color);
        d.object.instanceColor.needsUpdate = true;
      } else if (d.colors) {
        color.copy(c.edge).lerp(c.face, SOFT_EDGES[this.theme]);
        if (fade < 1) color.lerp(background, 1 - fade);
        d.colors.setXYZ(i, color.r, color.g, color.b);
        d.colors.needsUpdate = true;
      }
    }
  }

  /** Draws a frame: the model, then while something is highlighted a veil of the background over it and the
   * highlighted parts again on top, in their own colors; then a step's new parts' outline. The highlighted parts keep
   * the depth test, so a part behind another stays behind it. */
  private draw(r: WebGLRenderer, camera: PerspectiveCamera, theme: Theme, outline?: number): void {
    // A frame's passes count as one in stats().
    r.info.autoReset = false;
    r.info.reset();
    this.sortTransparent(camera);
    r.render(this.scene, camera);
    const autoClear = r.autoClear;
    r.autoClear = false;
    try {
      if (this.lit.size) {
        if (!this.overlayBuilt) this.buildOverlay();
        this.veil.material.color.setHex(BACKGROUND[theme]);
        r.render(this.veilScene, this.veilCamera);
        r.render(this.overlay, camera);
      }
      if (this.step != null) {
        if (!this.outlineBuilt) this.buildOutline();
        if (this.outlineRoot.children.length) this.outline.draw(r, this.outlineScene, camera, OUTLINE[theme], outline ? outline / 1.5 : r.getPixelRatio());
      }
    } finally {
      r.autoClear = autoClear;
    }
  }

  /** Transparent buckets draw after the opaque ones (three's own order) and back to front among themselves, by their
   * distance from the camera; each one's lines go just after its faces. */
  private sortTransparent(camera: PerspectiveCamera): void {
    for (const b of this.buckets.values()) {
      if (!b.finish.transparent) continue;
      const d = b.center.distanceTo(camera.position);
      b.drawables.forEach((x, i) => (x.object.renderOrder = -d + i * 1e-3));
    }
  }

  /** Copies the shown instances `pick` chooses into `root`: their meshes (and their lines, unless `flat`) with the
   * same geometry, material, matrix, and color. Flat copies carry no color, for the outline's mask. */
  private copyInstances(root: Group, pick: (id: string) => boolean, flat: boolean): void {
    const m = new Matrix4();
    const c = new Color();
    for (const b of this.buckets.values()) {
      const at = b.ids.flatMap((id, i) => (pick(id) && this.shows(this.placements.get(id)!) ? [i] : []));
      if (!at.length) continue;
      let holder: InstancedMesh | null = null;
      for (const d of b.drawables) {
        if (!(d.object instanceof InstancedMesh)) continue;
        const src = d.object;
        const copy = new InstancedMesh(src.geometry, src.material, at.length);
        copy.frustumCulled = false;
        copy.visible = src.visible;
        at.forEach((i, n) => {
          src.getMatrixAt(i, m);
          copy.setMatrixAt(n, m);
          if (!flat && src.instanceColor) {
            src.getColorAt(i, c);
            copy.setColorAt(n, c);
          }
        });
        holder ??= copy;
        root.add(copy);
      }
      if (flat || !holder) continue;
      for (const d of b.drawables) {
        if (d.object instanceof InstancedMesh) continue;
        const src = d.object.geometry as InstancedBufferGeometry;
        const g = new InstancedBufferGeometry();
        for (const [name, attr] of Object.entries(src.attributes)) {
          if (name !== "instanceMatrix" && name !== LINE_COLOR) g.setAttribute(name, attr);
        }
        g.setAttribute("instanceMatrix", holder.instanceMatrix);
        if (d.colors) {
          const colors = new InstancedBufferAttribute(new Float32Array(at.length * 3), 3);
          at.forEach((i, n) => colors.setXYZ(n, d.colors!.getX(i), d.colors!.getY(i), d.colors!.getZ(i)));
          g.setAttribute(LINE_COLOR, colors);
        }
        g.instanceCount = at.length;
        const lines = new LineSegments(g, d.object.material);
        lines.frustumCulled = false;
        lines.visible = d.object.visible;
        root.add(lines);
      }
    }
  }

  private buildOverlay(): void {
    clearCopies(this.overlayRoot);
    this.overlayBuilt = true;
    this.copyInstances(this.overlayRoot, (id) => this.isLit(id), false);
  }

  private buildOutline(): void {
    clearCopies(this.outlineRoot);
    this.outlineBuilt = true;
    const step = this.step;
    if (step != null) this.copyInstances(this.outlineRoot, (id) => topStep(this.placements.get(id)!) === step, true);
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
    if (!ids.length) clearCopies(this.overlayRoot);
    this.overlayBuilt = false;
    this.invalidate();
  }

  highlighted(): string[] {
    return [...this.lit];
  }

  showSteps(upTo: number | null = null): void {
    if (upTo === this.step) return;
    this.skipAnimation();
    this.step = upTo;
    this.refill();
  }

  cutAway(layer: number | null): void {
    const next = layer == null ? null : Math.max(1, Math.min(this.layers(), Math.round(layer)));
    if (next === this.cut) return;
    this.skipAnimation();
    this.cut = next;
    this.refill();
  }

  layers(): number {
    return Math.max(1, Math.ceil(this.extent.height / PLATE - 0.01));
  }

  /** The steps or the cut-away changed: every instance's matrix again (nothing loads or is rebuilt). */
  private refill(): void {
    for (const b of this.buckets.values()) this.fill(b);
    this.overlayBuilt = false;
    this.outlineBuilt = false;
    this.shadowDirty = true;
    this.invalidate();
  }

  /** The model's bottom and height (LDraw's y grows down, so its bottom is the largest y). */
  private measure(): void {
    let low = -Infinity;
    let high = Infinity;
    for (const p of this.placements.values()) {
      const part = this.cache.get(p.part);
      if (!part) continue;
      low = Math.max(low, partBottom(p, part));
      high = Math.min(high, partTop(p, part));
    }
    this.extent = Number.isFinite(low) ? { bottom: low, height: low - high } : { bottom: 0, height: 0 };
  }

  /** Plays a change as section 13 lays it out: the added parts hidden, then each drops in, in build order; moved
   * parts glide from where they were; removed parts (the ghosts) lift away. */
  private startAnimation(diff: Diff, before: ReadonlyMap<string, Placement>, ghosts: Map<string, Placement[]>): void {
    const parts: Motion[] = [];
    const order = buildOrder(diff.added.filter((p) => this.shows(p)));
    const gap = stagger(order.length, this.pace);
    order.forEach((p, n) => {
      const found = this.where.get(p.id);
      if (!found) return; // a part that failed to load has no instance to animate
      parts.push({ bucket: found[0], index: found[1], kind: "drop", home: new Matrix4().fromArray(p.matrix), from: null, at: n * gap, done: false });
    });
    for (const p of diff.moved) {
      const found = this.where.get(p.id);
      const was = before.get(p.id);
      if (!found || !was || !this.shows(p)) continue;
      parts.push({ bucket: found[0], index: found[1], kind: "glide", home: new Matrix4().fromArray(p.matrix), from: new Matrix4().fromArray(was.matrix), at: 0, done: false });
    }
    for (const [key, list] of ghosts) {
      const bucket = this.buckets.get(key);
      if (!bucket) continue;
      list.forEach((g, n) => {
        if (this.shows(g)) parts.push({ bucket, index: bucket.ids.length + n, kind: "leave", home: new Matrix4().fromArray(g.matrix), from: null, at: 0, done: false });
      });
    }
    if (!parts.length) {
      this.dropGhosts();
      return;
    }
    this.motion = { parts, start: performance.now() };
    this.stepAnimation(this.motion.start);
    this.emit("animation", true);
  }

  replayBuild(): void {
    void this.enqueue(async () => {
      this.skipAnimation();
      if (!this.animates()) return;
      const all = [...this.placements.values()];
      this.startAnimation({ added: all, removed: [], moved: [], recolored: [], unchanged: 0 }, new Map(), new Map());
      this.invalidate();
    });
  }

  /** Moves every animating part to where it is at `now`; the animation ends when the last one is done. */
  private stepAnimation(now: number): void {
    const f = this.motion;
    if (!f) return;
    const m = new Matrix4();
    const q0 = new Quaternion();
    const q1 = new Quaternion();
    const p0 = new Vector3();
    const p1 = new Vector3();
    const s = new Vector3();
    let left = 0;
    for (const part of f.parts) {
      if (part.done) continue;
      const elapsed = now - f.start - part.at;
      if (part.kind === "glide") {
        const g = glide(elapsed);
        part.from!.decompose(p0, q0, s);
        part.home.decompose(p1, q1, s);
        m.compose(p0.lerp(p1, g.t), q0.slerp(q1, g.t), s);
        this.setInstance(part.bucket, part.index, m);
        part.done = g.done;
      } else {
        const d = part.kind === "drop" ? drop(elapsed) : leave(elapsed);
        if (!d.visible) this.setInstance(part.bucket, part.index, HIDDEN);
        else {
          m.copy(part.home);
          m.elements[13]! -= d.lift; // up is −Y in LDraw
          this.setInstance(part.bucket, part.index, m);
        }
        this.paintOne(part.bucket, part.index, d.fade);
        part.done = d.done;
      }
      if (!part.done) left++;
    }
    if (!left) this.endAnimation();
  }

  private endAnimation(): void {
    this.motion = null;
    this.dropGhosts();
    this.shadowDirty = true;
    this.lodDirty = true;
    this.emit("animation", false);
  }

  /** The ghosts have lifted away: their buckets go on without them. */
  private dropGhosts(): void {
    for (const b of [...this.buckets.values()]) {
      if (!b.ghosts.length) continue;
      b.ghosts = [];
      if (!b.ids.length) this.dropBucket(b);
      else this.fill(b);
    }
  }

  skipAnimation(): void {
    const f = this.motion;
    if (!f) return;
    for (const part of f.parts) {
      if (part.kind === "leave") continue;
      this.setInstance(part.bucket, part.index, part.home);
      this.paintOne(part.bucket, part.index);
    }
    this.endAnimation();
    this.invalidate();
  }

  setPace(pace: Pace): void {
    this.pace = pace;
    if (pace === "off") this.skipAnimation();
  }

  /** Sets one instance's matrix in every mesh of its bucket (the lines share the first mesh's). Picking's bounds go
   * stale with it: three keeps an InstancedMesh's bounding sphere once computed, and only raycasts inside it. */
  private setInstance(bucket: Bucket, index: number, m: Matrix4): void {
    this.overlayBuilt = false;
    this.outlineBuilt = false;
    for (const d of bucket.drawables) {
      if (!(d.object instanceof InstancedMesh)) continue;
      d.object.setMatrixAt(index, m);
      d.object.instanceMatrix.needsUpdate = true;
      d.object.boundingSphere = null;
      d.object.boundingBox = null;
    }
  }

  frame(ids?: string[]): void {
    const box = this.bounds(ids, undefined, true);
    if (box.isEmpty()) return;
    const dir = this.camera.position.clone().sub(this.controls.target).normalize();
    this.place(this.camera, box, dir.lengthSq() > 0 ? dir : VIEWS.iso.dir, this.camera.up);
    this.controls.target.copy(box.getCenter(new Vector3()));
    this.controls.update();
    this.moved();
  }

  setView(view: View): void {
    const box = this.bounds(undefined, undefined, true);
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
    this.moved();
  }

  private moved(): void {
    this.invalidate();
    this.lodDirty = true;
    this.emit("camera");
  }

  setQuality(quality: Quality): void {
    if (quality === this.quality) return;
    this.quality = quality;
    const preset = this.resolve(quality);
    if (preset === this.preset) return;
    this.preset = preset;
    this.renderer.setPixelRatio(this.pixelRatio());
    this.fit();
    this.applyPreset();
    for (const b of this.buckets.values()) {
      for (const d of b.drawables) if (d.kind === "conditional") d.object.visible = preset !== "fast";
    }
    this.lodDirty = true;
    this.invalidate();
  }

  private resolve(q: Quality): Preset {
    if (q !== "auto") return q;
    const gl = this.renderer.getContext();
    const info = gl.getExtension("WEBGL_debug_renderer_info");
    const gpu = String(info ? gl.getParameter(info.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER));
    return presetFor(gpu);
  }

  /** Best draws the ground shadow; the others don't. */
  private applyPreset(): void {
    const shadow = this.preset === "best";
    this.renderer.shadowMap.enabled = shadow;
    this.renderer.shadowMap.autoUpdate = false;
    this.studio.key.castShadow = shadow;
    this.ground.visible = shadow;
    this.shadowDirty = true;
    // The materials compile again with or without the shadow map.
    for (const m of this.finishes.all()) m.needsUpdate = true;
  }

  /** The key light's shadow, and the ground it falls on, fitted under the model. */
  private fitShadow(r: WebGLRenderer): void {
    const box = this.bounds(undefined, undefined, true);
    if (box.isEmpty()) {
      this.ground.visible = false;
      return;
    }
    const center = box.getCenter(new Vector3());
    const size = box.getSize(new Vector3());
    const span = Math.max(size.x, size.z) * 1.6 + 80;
    this.ground.visible = true;
    this.ground.scale.set(span, span, 1);
    this.ground.position.set(center.x, box.min.y - 0.5, center.z);
    const light = this.studio.key;
    const dir = light.position.clone().sub(light.target.position).normalize();
    const reach = size.length() + 200;
    light.target.position.copy(center);
    light.position.copy(center).addScaledVector(dir, reach);
    light.target.updateMatrixWorld();
    light.updateMatrixWorld();
    const cam = light.shadow.camera;
    const half = size.length() / 2 + 40;
    cam.left = -half;
    cam.right = half;
    cam.top = half;
    cam.bottom = -half;
    cam.near = 1;
    cam.far = reach * 2;
    cam.updateProjectionMatrix();
    light.shadow.mapSize.set(2048, 2048);
    light.shadow.radius = 4;
    light.shadow.bias = -0.0005;
    light.shadow.needsUpdate = true;
    r.shadowMap.needsUpdate = true;
  }

  setTheme(theme: Theme): void {
    this.theme = theme;
    this.scene.background = new Color(BACKGROUND[theme]);
    for (const b of this.buckets.values()) this.paint(b);
    this.invalidate();
  }

  orbit(radians: number, tilt = 0): void {
    const offset = this.camera.position.clone().sub(this.controls.target);
    offset.applyAxisAngle(this.camera.up, radians);
    if (tilt) {
      const right = new Vector3().crossVectors(this.camera.up, offset).normalize();
      const turned = offset.clone().applyAxisAngle(right, -tilt);
      // Never past straight up or straight down.
      if (Math.abs(turned.clone().normalize().dot(this.camera.up)) < 0.995) offset.copy(turned);
    }
    this.camera.position.copy(this.controls.target).add(offset);
    this.camera.lookAt(this.controls.target);
    this.moved();
  }

  zoom(factor: number): void {
    const offset = this.camera.position.clone().sub(this.controls.target).multiplyScalar(Math.max(0.05, factor));
    this.camera.position.copy(this.controls.target).add(offset);
    this.camera.near = Math.max(offset.length() / 100, 0.1);
    this.camera.updateProjectionMatrix();
    this.moved();
  }

  render(options: RenderOptions = {}): Promise<Rendered> {
    return this.enqueue(() => this.renderNow(options));
  }

  /** The offscreen renderer and its own room (an environment map belongs to the renderer that made it). It always
   * draws the ground shadow, so renders look the same on every machine. */
  private offscreenRenderer(): { renderer: WebGLRenderer; environment: Texture } {
    if (!this.offscreen) {
      const renderer = makeRenderer(document.createElement("canvas"), true);
      renderer.shadowMap.enabled = true;
      renderer.shadowMap.autoUpdate = true;
      this.offscreen = { renderer, environment: studioRoom(renderer) };
    }
    return this.offscreen;
  }

  /** Draws offscreen: full-detail studs and conditional lines, the shadow, the given highlight, step, cut-away, and
   * theme, and
   * the finished model (never a frame of the build animation, which carries on in the window). The window's own
   * state comes back around one synchronous draw, so it never draws a frame with the render's. */
  private drawOffscreen(
    camera: PerspectiveCamera,
    width: number,
    height: number,
    state: { lit: Set<string>; mode: HighlightMode; step: number | null; cut: number | null; theme: Theme; background: Color | null; shadow?: boolean; outline?: number },
    measure?: () => void,
  ): WebGLRenderer {
    const { renderer: r, environment } = this.offscreenRenderer();
    r.setPixelRatio(1);
    r.setSize(width, height, false);
    const before = { lit: this.lit, mode: this.mode, step: this.step, cut: this.cut, theme: this.theme, hovered: this.hovered };
    const background = this.scene.background;
    const lows = [...this.buckets.values()].filter((b) => b.drawn !== b.part);
    let conditionals: Drawable[] = [];
    const motion = this.motion;
    const shadow = { cast: this.studio.key.castShadow, ground: this.ground.visible };
    const ghosts = new Map<Bucket, Placement[]>();
    try {
      if (motion) {
        for (const part of motion.parts) if (part.kind !== "leave") this.setInstance(part.bucket, part.index, part.home);
        this.motion = null;
      }
      for (const b of lows) this.setDetail(b, b.part);
      // After the rebuild to full detail, whose conditional lines Fast builds hidden.
      conditionals = [...this.buckets.values()].flatMap((b) => b.drawables.filter((d) => d.kind === "conditional" && !d.object.visible));
      for (const d of conditionals) d.object.visible = true;
      Object.assign(this, { lit: state.lit, mode: state.mode, step: state.step, cut: state.cut, theme: state.theme, hovered: null });
      this.scene.background = state.background;
      this.scene.environment = environment;
      this.overlay.environment = environment;
      this.finishes.setRoom(environment);
      for (const b of this.buckets.values()) {
        // The ghosts are gone from the finished model.
        if (b.ghosts.length) ghosts.set(b, b.ghosts);
        b.ghosts = [];
        this.fill(b);
      }
      this.overlayBuilt = false;
      this.outlineBuilt = false;
      const shadowed = state.shadow ?? true;
      this.studio.key.castShadow = shadowed;
      if (shadowed) this.fitShadow(r);
      else this.ground.visible = false;
      this.draw(r, camera, state.theme, state.outline);
      // With the render's state still in place: what's shown, and where.
      measure?.();
    } finally {
      Object.assign(this, before);
      this.scene.background = background;
      this.scene.environment = this.studio.environment;
      this.overlay.environment = this.studio.environment;
      this.finishes.setRoom(this.studio.environment);
      this.studio.key.castShadow = shadow.cast;
      this.ground.visible = shadow.ground;
      for (const d of conditionals) d.object.visible = false;
      for (const [b, g] of ghosts) b.ghosts = g;
      for (const b of this.buckets.values()) this.fill(b);
      for (const b of lows) if (b.part.low) this.setDetail(b, b.part.low);
      this.overlayBuilt = false;
      this.outlineBuilt = false;
      if (motion) {
        this.motion = motion;
        for (const part of motion.parts) part.done = false;
        this.stepAnimation(performance.now());
      }
      this.shadowDirty = true;
      this.invalidate();
    }
    return r;
  }

  private async renderNow(options: RenderOptions): Promise<Rendered> {
    if (this.broken) throw new Error(`can't render: the last update failed (${this.broken})`);
    const failed = this.failedHere();
    if (failed.length) throw new Error(`can't render: ${failed.map(([id, why]) => `${id} (${why})`).join("; ")}`);
    const size = Math.round(Math.min(Math.max(options.size ?? 768, 16), 4096));
    const name = options.view ?? "iso";
    const view = VIEWS[name];
    if (!view) throw new Error(`there's no view '${String(name)}'`);
    const turn = new Quaternion().setFromAxisAngle(new Vector3(0, 1, 0), (Math.round(options.turn ?? 0) * Math.PI) / 2);
    const dir = view.dir.clone().applyQuaternion(turn);
    const up = view.up.clone().applyQuaternion(turn);
    const camera = new PerspectiveCamera(FOV, 1, 1, 100_000);
    camera.up.copy(up);
    const frame = options.frame;
    const box = Array.isArray(frame)
      ? this.bounds(frame).expandByScalar(ZOOM_MARGIN)
      : typeof frame === "object"
        ? new Box3(new Vector3(...frame.min), new Vector3(...frame.max))
        : this.bounds(undefined, frame ?? options.step);
    if (box.isEmpty()) camera.position.copy(dir).multiplyScalar(600);
    else this.place(camera, box, dir, up);
    let report: Record<string, Sighting> | undefined;
    const r = this.drawOffscreen(
      camera,
      size,
      size,
      {
        lit: new Set(options.highlight ?? []),
        mode: "select",
        step: options.step ?? null,
        cut: null,
        theme: "light",
        background: options.transparent ? null : RENDER_BACKGROUND,
        shadow: options.shadow,
        outline: options.outline,
      },
      options.report ? () => (report = this.sight(options.report!, camera, size)) : undefined,
    );
    const png = await toPng(r);
    const target = box.isEmpty() ? new Vector3() : box.getCenter(new Vector3());
    let framed: [number, number, number, number] = [0, 0, size, size];
    if (!box.isEmpty()) {
      let [x0, y0, x1, y1] = [Infinity, Infinity, -Infinity, -Infinity];
      for (let i = 0; i < 8; i++) {
        const p = new Vector3(i & 1 ? box.max.x : box.min.x, i & 2 ? box.max.y : box.min.y, i & 4 ? box.max.z : box.min.z).project(camera);
        const [x, y] = [((p.x + 1) / 2) * size, ((1 - p.y) / 2) * size];
        [x0, y0, x1, y1] = [Math.min(x0, x), Math.min(y0, y), Math.max(x1, x), Math.max(y1, y)];
      }
      framed = [Math.max(0, x0), Math.max(0, y0), Math.min(size, x1), Math.min(size, y1)];
    }
    return {
      png,
      camera: { view: name, position: camera.position.toArray(), target: target.toArray(), fov: camera.fov },
      framed,
      ...(report ? { report } : {}),
    };
  }

  /** Where each id is in a `size`-pixel render from `camera`, and how much of it is in sight: of the rays from the
   * camera to points inside each of its parts (the center of its box, and its corners pulled 40 % in), the share that
   * reach the part's box before any other shown part's. Boxes, not triangles: a ray through a 32 × 32 baseplate's
   * studs costs 100,000 triangle tests, and the books only need to know whether a part is out of sight. */
  private sight(ids: string[], camera: PerspectiveCamera, size: number): Record<string, Sighting> {
    this.root.updateMatrixWorld(true);
    camera.updateMatrixWorld();
    const boxes = new Map<string, Box3>();
    for (const [id] of this.where) {
      const p = this.placements.get(id);
      if (p && this.shows(p)) boxes.set(id, this.bounds([id]));
    }
    const pixel = (v: Vector3): [number, number] => {
      const p = v.clone().project(camera);
      return [((p.x + 1) / 2) * size, ((1 - p.y) / 2) * size];
    };
    const ray = new Ray();
    const hit = new Vector3();
    const out: Record<string, Sighting> = {};
    for (const id of ids) {
      const parts = this.where.has(id) ? [id] : [...this.where.keys()].filter((k) => k.startsWith(`${id}/`));
      const shown = parts.filter((k) => boxes.has(k));
      if (!shown.length) continue;
      const box = this.bounds(shown);
      let [x0, y0, x1, y1] = [Infinity, Infinity, -Infinity, -Infinity];
      const v = new Vector3();
      for (let i = 0; i < 8; i++) {
        v.set(i & 1 ? box.max.x : box.min.x, i & 2 ? box.max.y : box.min.y, i & 4 ? box.max.z : box.min.z);
        const [x, y] = pixel(v);
        [x0, y0, x1, y1] = [Math.min(x0, x), Math.min(y0, y), Math.max(x1, x), Math.max(y1, y)];
      }
      let seen = 0;
      let tried = 0;
      for (const part of shown) {
        const own = boxes.get(part)!;
        const center = own.getCenter(new Vector3());
        const points = [center];
        for (let i = 0; i < 8; i++) {
          v.set(i & 1 ? own.max.x : own.min.x, i & 2 ? own.max.y : own.min.y, i & 4 ? own.max.z : own.min.z);
          points.push(v.clone().lerp(center, 0.4));
        }
        for (const point of points) {
          tried++;
          const ndc = point.clone().project(camera);
          if (Math.abs(ndc.x) > 1 || Math.abs(ndc.y) > 1) continue;
          ray.origin.copy(camera.position);
          ray.direction.copy(point).sub(camera.position).normalize();
          const enter = ray.intersectBox(own, hit) ? hit.distanceTo(camera.position) : camera.position.distanceTo(point);
          let blocked = false;
          for (const [other, b] of boxes) {
            if (other === part || !ray.intersectBox(b, hit)) continue;
            // A box the ray enters before the part's own (a neighbor sharing a face doesn't count).
            if (hit.distanceTo(camera.position) < enter - 1 && !b.containsPoint(point)) {
              blocked = true;
              break;
            }
          }
          if (!blocked) seen++;
        }
      }
      const mid = box.getCenter(new Vector3());
      const center = pixel(mid);
      out[id] = { box: [x0, y0, x1, y1], center, seen: tried ? seen / tried : 0 };
      const p = parts.length === 1 && parts[0] === id ? this.placements.get(id) : undefined;
      if (p) {
        // Its own down (LDraw's +Y) in the model; a part whose down is the model's goes on from above.
        const down = new Vector3(p.matrix[4] ?? 0, p.matrix[5] ?? 0, p.matrix[6] ?? 0).normalize();
        if (down.y < 0.9) {
          const along = down.transformDirection(this.root.matrixWorld).multiplyScalar(20).add(mid);
          const [ax, ay] = pixel(along);
          const len = Math.hypot(ax - center[0], ay - center[1]);
          if (len > 1e-3) out[id]!.moves = [(ax - center[0]) / len, (ay - center[1]) / len];
        }
      }
    }
    return out;
  }

  boxOf(ids?: string[], step?: number): Extent | null {
    const box = this.bounds(ids, step);
    return box.isEmpty() ? null : { min: box.min.toArray(), max: box.max.toArray() };
  }

  snapshot(scale = 2): Promise<Uint8Array> {
    return this.enqueue(async () => {
      const w = Math.min(4096, Math.max(1, Math.round((this.canvas.clientWidth || this.canvas.width) * scale)));
      const h = Math.min(4096, Math.max(1, Math.round((this.canvas.clientHeight || this.canvas.height) * scale)));
      const camera = this.camera.clone();
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      const r = this.drawOffscreen(camera, w, h, {
        lit: this.lit,
        mode: this.mode,
        step: this.step,
        cut: this.cut,
        theme: this.theme,
        background: new Color(BACKGROUND[this.theme]),
      });
      return toPng(r);
    });
  }

  screenPoint(ids: string[]): ScreenPoint | null {
    const box = this.bounds(ids, undefined, true);
    if (box.isEmpty()) return null;
    return this.project(new Vector3((box.min.x + box.max.x) / 2, box.max.y, (box.min.z + box.max.z) / 2));
  }

  private project(v: Vector3): ScreenPoint {
    this.camera.updateMatrixWorld();
    const p = v.clone().project(this.camera);
    const w = this.canvas.clientWidth || this.canvas.width;
    const h = this.canvas.clientHeight || this.canvas.height;
    const x = ((p.x + 1) / 2) * w;
    const y = ((1 - p.y) / 2) * h;
    return { x, y, visible: p.z > -1 && p.z < 1 && x >= 0 && y >= 0 && x <= w && y <= h };
  }

  idsInBox(box: Rect): string[] {
    this.root.updateMatrixWorld(true);
    const out: string[] = [];
    const m = new Matrix4();
    const c = new Vector3();
    for (const b of this.buckets.values()) {
      const center = partBox(b.part).getCenter(new Vector3());
      for (const id of b.ids) {
        const p = this.placements.get(id)!;
        if (!this.shows(p)) continue;
        m.fromArray(p.matrix).premultiply(this.root.matrixWorld);
        const s = this.project(c.copy(center).applyMatrix4(m));
        if (s.visible && s.x >= box.x && s.x <= box.x + box.width && s.y >= box.y && s.y <= box.y + box.height) out.push(id);
      }
    }
    return out;
  }

  pickAt(x: number, y: number): string | null {
    const w = this.canvas.clientWidth || this.canvas.width;
    const h = this.canvas.clientHeight || this.canvas.height;
    if (!w || !h) return null;
    this.camera.updateMatrixWorld();
    this.root.updateMatrixWorld(true);
    this.raycaster.setFromCamera(new Vector2((x / w) * 2 - 1, 1 - (y / h) * 2), this.camera);
    const targets: Object3D[] = [];
    const owner = new Map<Object3D, Bucket>();
    for (const b of this.buckets.values()) {
      for (const d of b.drawables) {
        if (!(d.object instanceof InstancedMesh) || !d.object.visible) continue;
        targets.push(d.object);
        owner.set(d.object, b);
      }
    }
    for (const hit of this.raycaster.intersectObjects(targets, false)) {
      const b = owner.get(hit.object);
      const i = hit.instanceId;
      if (b && i != null && i < b.ids.length) return b.ids[i]!;
    }
    return null;
  }

  stats(): Stats {
    const info = this.renderer.info;
    const frames = this.frames;
    const deltas = frames.slice(1).map((t, i) => t - frames[i]!);
    const memory = (performance as Performance & { memory?: { usedJSHeapSize: number } }).memory;
    let instances = 0;
    let lowDetail = 0;
    let transparent = 0;
    for (const b of this.buckets.values()) {
      instances += b.ids.length;
      if (b.drawn !== b.part) lowDetail++;
      if (b.finish.transparent) transparent++;
    }
    let hidden = 0;
    let outlined = 0;
    for (const p of this.placements.values()) {
      if (!this.shows(p)) hidden++;
      else if (this.step != null && topStep(p) === this.step) outlined++;
    }
    return {
      drawCalls: info.render.calls,
      triangles: info.render.triangles,
      frameMs: deltas.length ? deltas.reduce((a, b) => a + b, 0) / deltas.length : 0,
      heap: memory ? memory.usedJSHeapSize : null,
      instances,
      buckets: this.buckets.size,
      lowDetail,
      transparent,
      parts: this.cache.size,
      geometries: info.memory.geometries,
      textures: info.memory.textures,
      programs: info.programs?.length ?? 0,
      highlighted: this.lit.size,
      mode: this.mode,
      hovered: this.hovered,
      hidden,
      step: this.step,
      outlined,
      cut: this.cut,
      animating: this.motion ? this.motion.parts.filter((p) => !p.done).length : 0,
      failed: Object.fromEntries(this.failedHere()),
      lastDiff: { ...this.lastDiff },
      quality: this.preset,
      pace: this.pace,
      shadow: this.renderer.shadowMap.enabled,
    };
  }

  cameraMatrix(): number[] {
    this.camera.updateMatrixWorld();
    return this.camera.matrixWorld.toArray();
  }

  drawCost(frames: number): number {
    const gl = this.renderer.getContext();
    const pixel = new Uint8Array(4);
    const wait = () => gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
    this.draw(this.renderer, this.camera, this.theme);
    wait();
    const t0 = performance.now();
    for (let i = 0; i < frames; i++) {
      this.orbit(0.01);
      this.draw(this.renderer, this.camera, this.theme);
      wait();
    }
    return (performance.now() - t0) / Math.max(frames, 1);
  }

  nextFrame(): Promise<void> {
    this.invalidate();
    return new Promise((resolve) => this.waiters.push(resolve));
  }

  dispose(): void {
    cancelAnimationFrame(this.raf);
    this.resize?.disconnect();
    for (const f of this.cleanup) f();
    this.controls.dispose();
    for (const b of [...this.buckets.values()]) this.dropBucket(b);
    clearCopies(this.overlayRoot);
    clearCopies(this.outlineRoot);
    this.outline.dispose();
    this.veil.geometry.dispose();
    this.veil.material.dispose();
    this.ground.geometry.dispose();
    this.ground.material.dispose();
    this.finishes.dispose();
    this.studio.dispose();
    this.cache.dispose();
    // Its WebGL contexts go now, not when they're collected: a webview keeps only so many.
    this.renderer.dispose();
    this.renderer.forceContextLoss();
    if (this.offscreen) {
      this.offscreen.environment.dispose();
      this.offscreen.renderer.dispose();
      this.offscreen.renderer.forceContextLoss();
    }
  }

  private invalidate(): void {
    this.dirty = true;
  }

  /** Draws only when something changed: a diff, the camera, the size, orbit damping still settling, or the build
   * animation. Once the camera settles, each bucket's stud detail is chosen for how big its studs are on screen. */
  private loop(now: number): void {
    this.raf = requestAnimationFrame(this.loop);
    const moving = this.controls.update();
    if (this.motion) {
      this.stepAnimation(performance.now());
      this.dirty = true;
      this.shadowDirty = true;
    }
    if (!this.motion && !moving && this.lodDirty) {
      this.lodDirty = false;
      this.chooseDetail();
    }
    if (!this.dirty && !moving) {
      this.frames = [];
      return;
    }
    this.dirty = false;
    if (this.renderer.shadowMap.enabled && this.shadowDirty) {
      this.shadowDirty = false;
      this.fitShadow(this.renderer);
    }
    this.draw(this.renderer, this.camera, this.theme);
    if (moving) this.emit("camera");
    this.frames.push(now);
    if (this.frames.length > 61) this.frames.shift();
    const waiters = this.waiters.splice(0);
    for (const w of waiters) w();
  }

  /** The stud LOD: a bucket whose nearest instance's studs are smaller on screen than the preset's limit draws its
   * part's low-detail variant (made the first time it's wanted); otherwise the full part. */
  private chooseDetail(): void {
    const limit = LOD_PX[this.preset];
    const h = this.renderer.getDrawingBufferSize(new Vector2()).y;
    const tan = Math.tan((this.camera.fov * Math.PI) / 360);
    for (const b of this.buckets.values()) {
      const d = Math.max(b.center.distanceTo(this.camera.position) - b.radius, this.camera.near);
      const px = (12 / (2 * d * tan)) * h;
      if (px >= limit) {
        this.setDetail(b, b.part);
        continue;
      }
      if (b.part.low) {
        this.setDetail(b, b.part.low);
        continue;
      }
      void this.cache.low(b.part).then(() => {
        this.lodDirty = true;
        this.invalidate();
      });
    }
  }

  private setDetail(b: Bucket, drawn: PartGeometry): void {
    if (b.drawn === drawn || this.buckets.get(b.key) !== b) return;
    this.unbuild(b);
    b.drawn = drawn;
    this.build(b);
    this.fill(b);
    this.shadowDirty = true;
    this.invalidate();
  }

  private pixelRatio(): number {
    const dpr = globalThis.devicePixelRatio || 1;
    return this.preset === "fast" ? 1 : Math.min(dpr, 2);
  }

  private fit(): void {
    const w = this.canvas.clientWidth || this.canvas.width || 1;
    const h = this.canvas.clientHeight || this.canvas.height || 1;
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
  }

  /** The box around the given ids (a copy's id takes in its parts), else everything; only what's shown, or the build
   * up to `step`, when asked. In the scene's frame. A part's id is looked up, not searched for: pins and the leader
   * ask for a few parts' box every frame the camera moves. */
  private bounds(ids?: string[], step?: number, shownOnly = false): Box3 {
    this.root.updateMatrixWorld(true);
    const box = new Box3();
    const m = new Matrix4();
    const corner = new Vector3();
    const add = (p: Placement, part: PartGeometry) => {
      if (step != null && topStep(p) > step) return;
      if (shownOnly && !this.shows(p)) return;
      const local = partBox(part);
      m.fromArray(p.matrix).premultiply(this.root.matrixWorld);
      for (let i = 0; i < 8; i++) {
        corner.set(i & 1 ? local.max.x : local.min.x, i & 2 ? local.max.y : local.min.y, i & 4 ? local.max.z : local.min.z);
        box.expandByPoint(corner.applyMatrix4(m));
      }
    };
    if (!ids) {
      for (const b of this.buckets.values()) for (const id of b.ids) add(this.placements.get(id)!, b.part);
      return box;
    }
    const copies: string[] = [];
    for (const id of ids) {
      const at = this.where.get(id);
      if (at) add(this.placements.get(id)!, at[0].part);
      else copies.push(`${id}/`);
    }
    if (copies.length) {
      for (const [id, [b]] of this.where) if (copies.some((c) => id.startsWith(c))) add(this.placements.get(id)!, b.part);
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

const BACKGROUND_COLORS: Record<Theme, Color> = { light: new Color(BACKGROUND.light), dark: new Color(BACKGROUND.dark) };

function makeRenderer(canvas: HTMLCanvasElement, offscreen: boolean): WebGLRenderer {
  const r = new WebGLRenderer(offscreen ? { canvas, antialias: true, alpha: true, preserveDrawingBuffer: true } : { canvas, antialias: true });
  // Neutral tone mapping keeps the color table's colors close to true while the studio's highlights roll off.
  r.toneMapping = NeutralToneMapping;
  r.toneMappingExposure = 1;
  r.shadowMap.type = PCFShadowMap;
  return r;
}

/** Frees a line copy's own state: its geometry borrows the part's attributes, which other buckets still draw, and the
 * holder's instance matrices, so those are let go first and only its own colors and binding state go. */
function disposeLines(lines: LineSegments): void {
  const g = lines.geometry;
  for (const name of Object.keys(g.attributes)) if (name !== LINE_COLOR) g.deleteAttribute(name);
  g.dispose();
}

function clearCopies(root: Group): void {
  for (const o of [...root.children]) {
    root.remove(o);
    if (o instanceof InstancedMesh) o.dispose();
    else if (o instanceof LineSegments) disposeLines(o);
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

/** A placed part's lowest point in LDraw's y (which grows down: the largest y), and its highest. */
function partBottom(p: Placement, part: PartGeometry | undefined): number {
  return extremeY(p, part, true);
}

function partTop(p: Placement, part: PartGeometry | undefined): number {
  return extremeY(p, part, false);
}

/** The y of a placed part's box corners at their largest (or smallest): y = m1·x + m5·y + m9·z + m13, axis by axis. */
function extremeY(p: Placement, part: PartGeometry | undefined, largest: boolean): number {
  const m = p.matrix;
  let out = m[13] ?? 0;
  if (!part) return out;
  const b = partBox(part);
  for (const [k, lo, hi] of [
    [1, b.min.x, b.max.x],
    [5, b.min.y, b.max.y],
    [9, b.min.z, b.max.z],
  ] as const) {
    const a = (m[k] ?? 0) * lo;
    const c = (m[k] ?? 0) * hi;
    out += largest ? Math.max(a, c) : Math.min(a, c);
  }
  return out;
}

async function toPng(r: WebGLRenderer): Promise<Uint8Array> {
  // preserveDrawingBuffer keeps the pixels until the next draw on this canvas.
  const blob = await new Promise<Blob | null>((resolve) => r.domElement.toBlob(resolve, "image/png"));
  if (!blob) throw new Error("the webview couldn't encode the PNG");
  return new Uint8Array(await blob.arrayBuffer());
}

/** The system's reduced-motion setting (plan section 13): it turns the build animation off. */
function reducedMotion(): boolean {
  return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
}
