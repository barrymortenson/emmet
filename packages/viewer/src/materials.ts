// SPDX-License-Identifier: AGPL-3.0-or-later
// Materials for every LDraw finish, and the studio light rig (plan section 13). A part draws in one InstancedMesh per
// finish, not per color: the finish's material is white and each instance carries its color, so a 2,000-part model
// in 30 colors stays a few hundred draw calls. The color table's finishes come from LDrawLoader's own reading of
// LDConfig.ldr (its material per code); the look of each finish is Emmet's.

import {
  Color,
  DirectionalLight,
  HemisphereLight,
  LineBasicMaterial,
  type Material,
  MeshStandardMaterial,
  PMREMGenerator,
  type Scene,
  type Texture,
  type WebGLRenderer,
} from "three";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";
import { LDrawConditionalLineMaterial } from "three/addons/materials/LDrawConditionalLineMaterial.js";

export type FinishName = "solid" | "transparent" | "chrome" | "pearl" | "metal" | "matte-metal" | "rubber" | "glow";

/** How each finish looks: roughness, metalness, and how much of the studio's reflections it shows. Tuned against
 * photographs of real parts: ABS has a soft gloss, chrome mirrors the room, pearl and metal sit between. */
const LOOK: Record<FinishName, { roughness: number; metalness: number; env: number }> = {
  solid: { roughness: 0.4, metalness: 0, env: 0.5 },
  transparent: { roughness: 0.06, metalness: 0, env: 0.9 },
  chrome: { roughness: 0.12, metalness: 1, env: 1.7 },
  pearl: { roughness: 0.3, metalness: 0.45, env: 1.1 },
  metal: { roughness: 0.3, metalness: 0.85, env: 1.5 },
  "matte-metal": { roughness: 0.55, metalness: 0.5, env: 1 },
  rubber: { roughness: 0.88, metalness: 0, env: 0.25 },
  glow: { roughness: 0.4, metalness: 0, env: 0.4 },
};

/** One LDraw color as the viewer draws it. */
export interface ColorInfo {
  code: number;
  finish: FinishName;
  /** The face color and the edge color, in three's working (linear) space. */
  face: Color;
  edge: Color;
  /** 1 for opaque colors. */
  alpha: number;
  /** The glow, for luminous colors (0 otherwise). */
  luminance: number;
}

/** The finish and look of a color, from LDrawLoader's material for it. */
export function colorInfo(code: number, material: Material, edge: Material | undefined): ColorInfo {
  const m = material as MeshStandardMaterial;
  const alpha = m.transparent ? m.opacity : 1;
  const glowing = m.emissive && (m.emissive.r > 0 || m.emissive.g > 0 || m.emissive.b > 0);
  let finish: FinishName = "solid";
  if (glowing) finish = "glow";
  else if (alpha < 1) finish = "transparent";
  else if (m.metalness >= 0.99) finish = "chrome";
  else if (m.metalness >= 0.8) finish = "metal";
  else if (m.metalness >= 0.35 && m.roughness >= 0.5) finish = "matte-metal";
  else if (m.metalness > 0) finish = "pearl";
  else if (m.roughness >= 0.8) finish = "rubber";
  const face = m.color ? m.color.clone() : new Color(1, 0, 1);
  const edgeColor = (edge as LineBasicMaterial | undefined)?.color?.clone() ?? new Color(0.2, 0.2, 0.2);
  return { code, finish, face, edge: edgeColor, alpha, luminance: glowing ? 1 : 0 };
}

/** The bucket a color draws in, with the part: its finish, and for transparent and glowing colors what the material
 * can't take per instance (the opacity, the glow). */
export function finishKey(c: ColorInfo): string {
  if (c.finish === "transparent") return `transparent@${c.alpha.toFixed(2)}`;
  if (c.finish === "glow") return `glow@${c.code}`;
  return c.finish;
}

/** The materials one finish key draws with: the faces, the edges, and the conditional lines. Instances carry their
 * colors, so each is white (a glow's emissive is its own). */
export interface FinishMaterials {
  face: MeshStandardMaterial;
  edge: LineBasicMaterial;
  conditional: LDrawConditionalLineMaterial;
  transparent: boolean;
}

export class Finishes {
  private byKey = new Map<string, FinishMaterials>();
  /** The room each face material reflects, set on the material itself: three uses a material's own envMapIntensity
   * only with its own envMap (the scene's environment takes the scene's intensity), and each finish has its own. */
  private room: Texture | null = null;

  /** Every face material reflects `room` from now on (the offscreen renderer swaps in its own, then back). */
  setRoom(room: Texture): void {
    this.room = room;
    for (const m of this.faces()) m.envMap = room;
  }

  private faces(): MeshStandardMaterial[] {
    return [...this.fixed.values(), ...[...this.byKey.values()].map((f) => f.face)];
  }
  private fixed = new Map<number, MeshStandardMaterial>();
  private fixedLines = new Map<string, Material>();

  /** The shared materials for `c`'s finish key. */
  for(c: ColorInfo): FinishMaterials {
    const key = finishKey(c);
    let f = this.byKey.get(key);
    if (!f) {
      const transparent = c.alpha < 1;
      const face = faceMaterial(c.finish, transparent ? c.alpha : 1);
      if (c.finish === "glow") face.emissive.copy(c.face).multiplyScalar(0.6);
      face.color.set(1, 1, 1);
      face.envMap = this.room;
      const edge = new LineBasicMaterial({ color: 0xffffff, transparent, opacity: transparent ? c.alpha : 1, depthWrite: !transparent });
      edge.defines = { USE_INSTANCING: "" };
      edge.onBeforeCompile = colorFromInstance;
      edge.customProgramCacheKey = () => "emmet-instanced-edge";
      const conditional = instancedConditional(transparent, transparent ? c.alpha : 1, true);
      f = { face, edge, conditional, transparent };
      this.byKey.set(key, f);
    }
    return f;
  }

  /** A part's fixed color (a print, a part that is always one color): the finish's look in that color. */
  fixedFace(c: ColorInfo): MeshStandardMaterial {
    let m = this.fixed.get(c.code);
    if (!m) {
      m = faceMaterial(c.finish, c.alpha);
      m.color.copy(c.face);
      m.envMap = this.room;
      if (c.finish === "glow") m.emissive.copy(c.face).multiplyScalar(0.6);
      this.fixed.set(c.code, m);
    }
    return m;
  }

  /** A fixed color's lines, instanced without a per-instance color. */
  fixedLine(c: ColorInfo, kind: "edges" | "conditional"): Material {
    const key = `${kind}|${c.code}`;
    let m = this.fixedLines.get(key);
    if (!m) {
      const transparent = c.alpha < 1;
      if (kind === "edges") {
        const e = new LineBasicMaterial({ color: c.edge, transparent, opacity: c.alpha, depthWrite: !transparent });
        e.defines = { USE_INSTANCING: "" };
        m = e;
      } else {
        const cm = instancedConditional(transparent, c.alpha, false);
        cm.uniforms.diffuse!.value.copy(c.edge);
        m = cm;
      }
      this.fixedLines.set(key, m);
    }
    return m;
  }

  all(): Material[] {
    const out: Material[] = [...this.fixed.values(), ...this.fixedLines.values()];
    for (const f of this.byKey.values()) out.push(f.face, f.edge, f.conditional);
    return out;
  }

  dispose(): void {
    for (const m of this.all()) m.dispose();
    this.byKey.clear();
    this.fixed.clear();
    this.fixedLines.clear();
  }
}

function faceMaterial(finish: FinishName, alpha: number): MeshStandardMaterial {
  const look = LOOK[finish];
  const transparent = alpha < 1;
  const m = new MeshStandardMaterial({
    roughness: look.roughness,
    metalness: look.metalness,
    envMapIntensity: look.env,
    transparent,
    opacity: alpha,
    depthWrite: !transparent,
    // Faces sit a hair behind their edges, so the lines draw cleanly on top (as LDrawLoader does).
    polygonOffset: true,
    polygonOffsetFactor: 1,
    polygonOffsetUnits: 1,
  });
  return m;
}

/** LDrawLoader's conditional-line material, reading the instance matrix (and the instance's color, when `colored`). */
function instancedConditional(transparent: boolean, opacity: number, colored: boolean): LDrawConditionalLineMaterial {
  const m = new LDrawConditionalLineMaterial({ transparent, depthWrite: !transparent });
  m.uniforms.diffuse!.value.set(1, 1, 1);
  m.uniforms.opacity!.value = opacity;
  m.vertexShader = m.vertexShader.replaceAll("modelViewMatrix * vec4(", "modelViewMatrix * instanceMatrix * vec4(");
  m.defines = { ...m.defines, USE_INSTANCING: "" };
  if (colored) colorFromInstance(m);
  return m;
}

/** three applies a per-instance color only to an InstancedMesh; instanced lines carry theirs in their own attribute,
 * `emmetColor`, which these shader edits read in place of three's vertex colors. */
export const LINE_COLOR = "emmetColor";

function colorFromInstance(shader: { vertexShader: string; fragmentShader: string }): void {
  const swap = (source: string, from: string, to: string) => {
    if (!source.includes(from)) throw new Error(`three's line shader changed: no ${from}`);
    return source.replace(from, to);
  };
  shader.vertexShader = swap(shader.vertexShader, "#include <color_pars_vertex>", `attribute vec3 ${LINE_COLOR};\nvarying vec3 vEmmetColor;`);
  shader.vertexShader = swap(shader.vertexShader, "#include <color_vertex>", `vEmmetColor = ${LINE_COLOR};`);
  shader.fragmentShader = swap(shader.fragmentShader, "#include <color_pars_fragment>", "varying vec3 vEmmetColor;");
  shader.fragmentShader = swap(shader.fragmentShader, "#include <color_fragment>", "diffuseColor.rgb *= vEmmetColor;");
}

/** The studio: a soft room for reflections (three's RoomEnvironment, prefiltered once per renderer), a key light from
 * the front left and above, a fill from the right, a rim from behind, and a little sky and ground light. The key casts
 * the optional ground shadow. */
export interface Studio {
  environment: Texture;
  key: DirectionalLight;
  dispose(): void;
}

/** The studio's lights in `scene`, lit by `environment` (a scene that shares another's room), or by a room made for
 * `renderer` (an environment map belongs to the renderer that made it). */
export function addStudio(scene: Scene, renderer: WebGLRenderer, environment = studioRoom(renderer)): Studio {
  scene.environment = environment;
  // The room's reflections light most of it; the key gives the faces their shading (and casts the shadow), with a
  // little fill and rim. Tuned with tone mapping on against the before renders and photographs of parts: a part's
  // color reads as itself on its lit faces, not washed toward white.
  scene.add(new HemisphereLight(0xffffff, 0x8a8f99, 0.9));
  const key = new DirectionalLight(0xfff6ec, 1.6);
  key.position.set(-1, 2.2, 1.3);
  scene.add(key, key.target);
  const fill = new DirectionalLight(0xeef3ff, 0.35);
  fill.position.set(1.6, 0.6, -0.8);
  scene.add(fill);
  const rim = new DirectionalLight(0xffffff, 0.3);
  rim.position.set(0.6, 1.2, -2);
  scene.add(rim);
  return {
    environment,
    key,
    dispose: () => environment.dispose(),
  };
}

export function studioRoom(renderer: WebGLRenderer): Texture {
  const pmrem = new PMREMGenerator(renderer);
  const room = new RoomEnvironment();
  const environment = pmrem.fromScene(room, 0.04).texture;
  room.dispose();
  pmrem.dispose();
  return environment;
}
