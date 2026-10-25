// SPDX-License-Identifier: AGPL-3.0-or-later
// Unique parts, loaded once each through LDrawLoader and cached by part id (plan section 13, D-23).

import {
  BufferGeometry,
  Group,
  LineBasicMaterial,
  LineSegments,
  type Material,
  Matrix4,
  Mesh,
  type Object3D,
} from "three";
import { LDrawLoader } from "three/addons/loaders/LDrawLoader.js";
import { LDrawConditionalLineMaterial } from "three/addons/materials/LDrawConditionalLineMaterial.js";

/** Where the viewer gets LDraw text. The app serves it from the library through the emmet-lib:// protocol. */
export interface PartSource {
  /** For each part id, the part file and every file it uses as one packed .mpd (`0 FILE` blocks). `missing` lists
   * the files a part refers to that the library doesn't have. */
  parts(ids: string[]): Promise<{ parts: Record<string, string>; missing: Record<string, string[]> }>;
  /** The LDraw color table (LDConfig.ldr). */
  colors(): Promise<string>;
}

export type PieceKind = "mesh" | "edges" | "conditional";

/** One drawable of a part: its geometry in the part's own frame and a material per geometry group. A material of
 * null is the main color (LDraw 16, or 24 for edges), filled in by the bucket's color. */
export interface Piece {
  kind: PieceKind;
  geometry: BufferGeometry;
  materials: (Material | null)[];
}

export interface PartGeometry {
  id: string;
  pieces: Piece[];
  triangles: number;
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

/** LDrawLoader's material for a color code, and the instanced line materials made from it. */
export class Colors {
  readonly loader: Loader;
  private edges = new WeakMap<Material, LineBasicMaterial>();
  private conditionals = new WeakMap<Material, LDrawConditionalLineMaterial>();

  constructor(loader: Loader) {
    this.loader = loader;
  }

  face(code: number): Material {
    return this.loader.getMaterial(String(code)) ?? this.loader.missingColorMaterial;
  }

  /** An edge material that reads the instance matrix, made once per source material. */
  edge(source: Material): LineBasicMaterial {
    let m = this.edges.get(source);
    if (!m) {
      m = (source as LineBasicMaterial).clone();
      m.defines = { ...m.defines, USE_INSTANCING: "" };
      this.edges.set(source, m);
    }
    return m;
  }

  /** The conditional-line material with the instance matrix applied to the line ends and control points. */
  conditional(source: Material): LDrawConditionalLineMaterial {
    let m = this.conditionals.get(source);
    if (!m) {
      const s = source as LDrawConditionalLineMaterial;
      m = new LDrawConditionalLineMaterial({ transparent: s.transparent, depthWrite: s.depthWrite });
      m.uniforms.diffuse!.value.copy(s.uniforms.diffuse!.value);
      m.uniforms.opacity!.value = s.uniforms.opacity!.value;
      m.vertexShader = m.vertexShader.replaceAll("modelViewMatrix * vec4(", "modelViewMatrix * instanceMatrix * vec4(");
      m.defines = { ...m.defines, USE_INSTANCING: "" };
      this.conditionals.set(source, m);
    }
    return m;
  }

  /** The edge material for a face color, as LDrawLoader pairs them. */
  edgeFor(code: number): Material {
    const face = this.face(code);
    return this.loader.edgeMaterialCache.get(face) ?? face;
  }

  conditionalFor(code: number): Material {
    const edge = this.edgeFor(code);
    return this.loader.conditionalEdgeMaterialCache.get(edge) ?? edge;
  }
}

/** Loads parts through a {@link PartSource}, each once, and keeps them by id. */
export class PartCache {
  readonly colors: Colors;
  private parts = new Map<string, PartGeometry>();
  private pending = new Map<string, Promise<void>>();
  /** Parts that couldn't load, with why. They are never drawn with holes (ADR 0005). */
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
    return this.parts.get(key(id));
  }

  get size(): number {
    return this.parts.size;
  }

  /** Loads every id not loaded yet, in one request. Resolves once each is loaded or failed. */
  async ensure(ids: Iterable<string>): Promise<void> {
    await (this.ready ??= this.loadColors());
    const want = [...new Set([...ids].map(key))].filter((k) => !this.parts.has(k) && !this.failed.has(k));
    const fresh = want.filter((k) => !this.pending.has(k));
    if (fresh.length) {
      const batch = this.fetch(fresh);
      for (const k of fresh) this.pending.set(k, batch);
      batch.finally(() => fresh.forEach((k) => this.pending.delete(k)));
    }
    await Promise.all(want.map((k) => this.pending.get(k)));
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
    const pieces: Piece[] = [];
    let triangles = 0;
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
      const geometry = obj.geometry.clone().applyMatrix4(new Matrix4().copy(o.matrixWorld));
      const list = Array.isArray(obj.material) ? obj.material : [obj.material];
      const materials = list.map((m) => (isMain(m) ? null : m));
      if (geometry.groups.length === 0) geometry.addGroup(0, Infinity, 0);
      if (kind === "mesh") triangles += (geometry.getAttribute("position")?.count ?? 0) / 3;
      pieces.push({ kind, geometry, materials });
    });
    if (!pieces.length) throw new Error(`${id} has no geometry`);
    return { id, pieces, triangles };
  }

  dispose(): void {
    for (const p of this.parts.values()) for (const piece of p.pieces) piece.geometry.dispose();
    this.parts.clear();
  }
}

function key(id: string): string {
  return id.toLowerCase().replace(/\.dat$/, "");
}

function isMain(m: Material): boolean {
  const code = m.userData?.code;
  return code === MAIN || code === MAIN_EDGE;
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
