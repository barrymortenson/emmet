// SPDX-License-Identifier: AGPL-3.0-or-later
// Unique parts, loaded once each through LDrawLoader and cached by part id (plan section 13, D-23). Each part is one
// geometry per kind (faces, edges, conditional lines) and color slot (its main color, or each fixed color it has),
// merged across the part's sub-objects, so a part draws in as few calls as it can. Each part also has a low-detail
// variant, made on demand, whose studs are 8-sided without their conditional lines and whose underside tubes are gone
// (the stud LOD: what a stud a few pixels across needs).

import { BufferAttribute, BufferGeometry, type Group, type LineSegments, type Material, Matrix4, type Mesh, type Object3D } from "three";
import { LDrawLoader } from "three/addons/loaders/LDrawLoader.js";
import { LDrawConditionalLineMaterial } from "three/addons/materials/LDrawConditionalLineMaterial.js";
import { mergeGeometries } from "three/addons/utils/BufferGeometryUtils.js";
import { type ColorInfo, colorInfo } from "./materials.js";

/** Where the viewer gets LDraw text. The app serves it from the library through the emmet-lib:// protocol. */
export interface PartSource {
  /** For each part id, the part file and every file it uses as one packed .mpd (`0 FILE` blocks). `missing` lists
   * the files a part refers to that the library doesn't have. */
  parts(ids: string[]): Promise<{ parts: Record<string, string>; missing: Record<string, string[]> }>;
  /** The LDraw color table (LDConfig.ldr). */
  colors(): Promise<string>;
}

export type PieceKind = "mesh" | "edges" | "conditional";

/** One drawable of a part: its geometry in the part's own frame, and its color. `fixed` is null for the main color
 * (LDraw 16, or 24 for edges), which each instance fills in; otherwise the loader's material for the fixed color. */
export interface Piece {
  kind: PieceKind;
  geometry: BufferGeometry;
  fixed: Material | null;
}

export interface PartGeometry {
  id: string;
  pieces: Piece[];
  triangles: number;
  /** The low-detail variant once it's made (see PartCache.low), else null. */
  low: PartGeometry | null;
}

const MAIN = "16";
const MAIN_EDGE = "24";

/** LDrawLoader internals the viewer relies on, which @types/three doesn't declare (three r186). */
type Loader = LDrawLoader & {
  edgeMaterialCache: WeakMap<Material, Material>;
  conditionalEdgeMaterialCache: WeakMap<Material, Material>;
  missingColorMaterial: Material;
  parseColorMetaDirective(parser: LineParserShim): Material;
};

/** The color table, as LDrawLoader read it. */
export class Colors {
  readonly loader: Loader;
  private info = new Map<number, ColorInfo>();

  constructor(loader: Loader) {
    this.loader = loader;
  }

  face(code: number): Material {
    return this.loader.getMaterial(String(code)) ?? this.loader.missingColorMaterial;
  }

  /** A color's finish, face and edge colors, and alpha. */
  get(code: number): ColorInfo {
    let c = this.info.get(code);
    if (!c) {
      const face = this.face(code);
      c = colorInfo(code, face, this.loader.edgeMaterialCache.get(face));
      this.info.set(code, c);
    }
    return c;
  }

  /** The color a fixed material is for. */
  of(m: Material): ColorInfo {
    const code = Number(m.userData?.code);
    return Number.isFinite(code) ? this.get(code) : this.get(-1);
  }
}

/** Loads parts through a {@link PartSource}, each once, and keeps them by id. */
export class PartCache {
  readonly colors: Colors;
  private parts = new Map<string, PartGeometry>();
  private texts = new Map<string, string>();
  /** Low-detail variants being made: a part in two finishes is two buckets asking at once, and gets one. */
  private lows = new Map<string, Promise<PartGeometry>>();
  private pending = new Map<string, Promise<void>>();
  /** Parts whose last load failed, with why. They are never drawn with holes (ADR 0005), and the next ensure() that
   * asks for one tries it again (the library may have been installed, or a fetch may have failed once). */
  readonly failed = new Map<string, string>();
  /** The color table, loaded with the first parts; a failed load is tried again next time. */
  private ready: Promise<void> | null = null;

  constructor(private source: PartSource) {
    const loader = new LDrawLoader() as Loader;
    loader.setConditionalLineMaterial(LDrawConditionalLineMaterial);
    this.colors = new Colors(loader);
  }

  private loadColors(): Promise<void> {
    const loader = this.colors.loader;
    return this.source.colors().then(
      (text) => {
        loader.addMaterials(colorMaterials(loader, text));
        loader.addDefaultMaterials();
      },
      (e) => {
        this.ready = null;
        throw e;
      },
    );
  }

  get(id: string): PartGeometry | undefined {
    return this.parts.get(partKey(id));
  }

  get size(): number {
    return this.parts.size;
  }

  /** Loads every id not loaded yet (failed ones included), in one request. Resolves once each is loaded or failed. */
  async ensure(ids: Iterable<string>): Promise<void> {
    await (this.ready ??= this.loadColors());
    const want = [...new Set([...ids].map(partKey))].filter((k) => !this.parts.has(k));
    const fresh = want.filter((k) => !this.pending.has(k));
    if (fresh.length) {
      const batch = this.fetch(fresh);
      for (const k of fresh) this.pending.set(k, batch);
      batch.finally(() => fresh.forEach((k) => this.pending.delete(k)));
    }
    await Promise.all(want.map((k) => this.pending.get(k)));
  }

  /** The part's low-detail variant, made from its text the first time it's asked for. The same part when it has no
   * studs to simplify. */
  low(part: PartGeometry): Promise<PartGeometry> {
    if (part.low) return Promise.resolve(part.low);
    let pending = this.lows.get(part.id);
    if (!pending) {
      pending = this.makeLow(part).finally(() => this.lows.delete(part.id));
      this.lows.set(part.id, pending);
    }
    return pending;
  }

  private async makeLow(part: PartGeometry): Promise<PartGeometry> {
    const text = this.texts.get(part.id);
    const lowText = text && simplifyStuds(text);
    if (!lowText) return (part.low = part);
    try {
      const low = await this.parse(part.id, lowText);
      // A variant that came out no smaller isn't worth a second set of buffers.
      if (low.triangles >= part.triangles) {
        low.pieces.forEach((p) => p.geometry.dispose());
        return (part.low = part);
      }
      return (part.low = low);
    } catch (e) {
      // The full part draws instead: whole, only with more triangles. Said, so a broken variant isn't silent.
      console.warn(`the viewer's low-detail ${part.id} couldn't be made, so it draws in full: ${String(e)}`);
      return (part.low = part);
    }
  }

  private async fetch(ids: string[]): Promise<void> {
    let reply: Awaited<ReturnType<PartSource["parts"]>>;
    try {
      reply = await this.source.parts(ids);
    } catch (e) {
      for (const id of ids) this.failed.set(id, `couldn't fetch the part: ${String(e)}`);
      return;
    }
    await Promise.all(
      ids.map(async (id) => {
        const missing = reply.missing[id];
        const text = reply.parts[id];
        if (missing?.length || text === undefined) {
          this.failed.set(id, `the library is missing ${missing?.length ? missing.join(", ") : `${id}.dat`}`);
          return;
        }
        try {
          this.parts.set(id, await this.parse(id, text));
          this.texts.set(id, text);
          this.failed.delete(id);
        } catch (e) {
          this.failed.set(id, String(e instanceof Error ? e.message : e));
        }
      }),
    );
  }

  private async parse(id: string, text: string): Promise<PartGeometry> {
    const loader = this.colors.loader;
    const group = await new Promise<Group>((resolve, reject) => loader.parse(text, resolve, reject));
    group.updateMatrixWorld(true);
    // Every (kind, color slot)'s triangles or lines, from every sub-object, to merge into one geometry each.
    const slots = new Map<string, { kind: PieceKind; fixed: Material | null; parts: BufferGeometry[] }>();
    group.traverse((o: Object3D) => {
      const kind: PieceKind | null = (o as Mesh).isMesh
        ? "mesh"
        : (o as LineSegments & { isConditionalLine?: boolean }).isConditionalLine
          ? "conditional"
          : (o as LineSegments).isLineSegments
            ? "edges"
            : null;
      if (!kind) return;
      const obj = o as Mesh | LineSegments;
      // A part made of other parts holds them as child groups: bake each one's transform into its own copy.
      let geometry = obj.geometry.clone().applyMatrix4(new Matrix4().copy(o.matrixWorld));
      if (geometry.index) geometry = geometry.toNonIndexed();
      const list = Array.isArray(obj.material) ? obj.material : [obj.material];
      const total = geometry.getAttribute("position").count;
      const groups = geometry.groups.length ? geometry.groups : [{ start: 0, count: total, materialIndex: 0 }];
      for (const g of groups) {
        const m = list[g.materialIndex ?? 0] ?? list[0]!;
        const fixed = isMain(m) ? null : m;
        const key = `${kind}|${fixed ? fixed.uuid : "main"}`;
        let slot = slots.get(key);
        if (!slot) slots.set(key, (slot = { kind, fixed, parts: [] }));
        slot.parts.push(range(geometry, g.start, Math.min(g.count, total - g.start)));
      }
      geometry.dispose();
    });
    const pieces: Piece[] = [];
    let triangles = 0;
    for (const slot of slots.values()) {
      const parts = slot.parts.filter((g) => g.getAttribute("position").count > 0);
      if (!parts.length) continue;
      const geometry = parts.length === 1 ? parts[0]! : mergeGeometries(parts, false);
      // Never a part with a piece missing (ADR 0005): it fails, and the scene says so.
      if (!geometry) throw new Error(`${id}: its ${slot.kind} couldn't be merged`);
      if (parts.length > 1) parts.forEach((g) => g.dispose());
      if (slot.kind === "mesh") triangles += geometry.getAttribute("position").count / 3;
      pieces.push({ kind: slot.kind, geometry, fixed: slot.fixed });
    }
    if (!pieces.length) throw new Error(`${id} has no geometry`);
    // Faces first, then edges, then conditional lines: the bucket's first mesh holds the instance matrices.
    const order: Record<PieceKind, number> = { mesh: 0, edges: 1, conditional: 2 };
    pieces.sort((a, b) => order[a.kind] - order[b.kind] || Number(a.fixed != null) - Number(b.fixed != null));
    return { id, pieces, triangles, low: null };
  }

  dispose(): void {
    for (const p of this.parts.values()) {
      for (const piece of p.pieces) piece.geometry.dispose();
      if (p.low && p.low !== p) for (const piece of p.low.pieces) piece.geometry.dispose();
    }
    this.parts.clear();
    this.texts.clear();
  }
}

/** `count` vertices of a non-indexed geometry from `start`, as a geometry of their own (the attributes copied). */
function range(g: BufferGeometry, start: number, count: number): BufferGeometry {
  const out = new BufferGeometry();
  for (const [name, attr] of Object.entries(g.attributes)) {
    const a = attr as BufferAttribute;
    const array = (a.array as Float32Array).slice(start * a.itemSize, (start + count) * a.itemSize);
    out.setAttribute(name, new BufferAttribute(array, a.itemSize, a.normalized));
  }
  return out;
}

/** A part id as the cache and the part source know it: lowercase, without ".dat". */
export function partKey(id: string): string {
  return id.toLowerCase().replace(/\.dat$/, "");
}

function isMain(m: Material): boolean {
  const code = m.userData?.code;
  return code === MAIN || code === MAIN_EDGE;
}

// The stud LOD's primitives: an 8-sided disc, cylinder, and circle, drawn as LDraw's own 4-4disc, 4-4cyli, and
// 4-4edge are (a unit circle in the XZ plane, the cylinder from y 0 to 1), so a stud's own transforms place them.
const SIDES = 8;
const ring = (i: number) => {
  const a = ((i % SIDES) / SIDES) * Math.PI * 2;
  return [Math.cos(a), Math.sin(a)].map((v) => Number(v.toFixed(4))) as [number, number];
};
const lines = (f: (i: number) => string) => Array.from({ length: SIDES }, (_, i) => f(i)).join("\n");
const LOW_PRIMITIVES = `
0 FILE emmet-lo-disc.dat
0 BFC CERTIFY CCW
${lines((i) => `3 16 0 0 0 ${ring(i)[0]} 0 ${ring(i)[1]} ${ring(i + 1)[0]} 0 ${ring(i + 1)[1]}`)}
0 FILE emmet-lo-cyli.dat
0 BFC CERTIFY CCW
${lines((i) => `4 16 ${ring(i)[0]} 1 ${ring(i)[1]} ${ring(i + 1)[0]} 1 ${ring(i + 1)[1]} ${ring(i + 1)[0]} 0 ${ring(i + 1)[1]} ${ring(i)[0]} 0 ${ring(i)[1]}`)}
0 FILE emmet-lo-edge.dat
${lines((i) => `2 24 ${ring(i)[0]} 0 ${ring(i)[1]} ${ring(i + 1)[0]} 0 ${ring(i + 1)[1]}`)}
0 FILE emmet-lo-stud.dat
0 BFC CERTIFY CCW
1 16 0 0 0 6 0 0 0 -4 0 0 0 6 emmet-lo-cyli.dat
1 16 0 -4 0 6 0 0 0 1 0 0 0 6 emmet-lo-disc.dat
1 16 0 -4 0 6 0 0 0 1 0 0 0 6 emmet-lo-edge.dat
`;
/** Studs drawn low (the top ones, solid or hollow), and studs left out (the tubes under bricks and plates). */
const LOW_STUD = /^(stud|stud2|stud2a|stud10|stud15)\.dat$/i;
const NO_STUD = /^(\d+-\d+)?stud(3|3a|4|4a|4o|4s|4s2|4h)\.dat$/i;

/** The packed part with its studs simplified, or null when it has none to simplify. */
export function simplifyStuds(text: string): string | null {
  let changed = false;
  const out: string[] = [];
  for (const line of text.split(/\r?\n/)) {
    const m = /^(\s*1\s+(?:\S+\s+){13})(\S+)\s*$/.exec(line);
    const file = m?.[2];
    if (m && file && NO_STUD.test(file)) {
      changed = true;
      continue;
    }
    if (m && file && LOW_STUD.test(file)) {
      changed = true;
      out.push(`${m[1]}emmet-lo-stud.dat`);
      continue;
    }
    out.push(line);
  }
  return changed ? `${out.join("\n")}\n${LOW_PRIMITIVES}` : null;
}

/** Every `0 !COLOUR` line of LDConfig.ldr as LDrawLoader materials. */
function colorMaterials(loader: Loader, text: string): Material[] {
  const out: Material[] = [];
  for (const line of text.split(/\r?\n/)) {
    const m = /^\s*0\s+!COLOUR\s+(.*)$/.exec(line);
    if (m) out.push(loader.parseColorMetaDirective(new LineParserShim(m[1]!)));
  }
  return out;
}

/** The token reader `parseColorMetaDirective` expects (LDrawLoader's LineParser isn't exported). */
class LineParserShim {
  private tokens: string[];
  private i = 0;
  lineNumber = 0;
  constructor(text: string) {
    this.tokens = text.trim().split(/\s+/);
  }
  getToken(): string {
    return this.tokens[this.i++] ?? "";
  }
  getRemainingString(): string {
    const rest = this.tokens.slice(this.i).join(" ");
    this.i = this.tokens.length;
    return rest;
  }
  setToEnd(): void {
    this.i = this.tokens.length;
  }
  isAtTheEnd(): boolean {
    return this.i >= this.tokens.length;
  }
  getLineNumberString(): string {
    return "";
  }
}
