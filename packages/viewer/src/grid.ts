// SPDX-License-Identifier: AGPL-3.0-or-later
// The ground grid (Barry 2026-11-10): a faint grid on the table under the model in the window, a square a stud with a
// stronger line every 8 studs, fading out with distance. Pictures leave it out. One plane with the lines drawn in its
// shader, so it's one draw call at any size and its lines stay a pixel wide however far the camera is.

import { Color, Mesh, PlaneGeometry, ShaderMaterial, Vector2 } from "three";
import type { Theme } from "./index.js";

/** A stud, in LDU: the grid's square. */
export const STUD = 20;
/** Every this many studs, a stronger line. */
export const MAJOR = 8;
/** How far the grid reaches past the model's footprint, in LDU (16 studs), and how far it reaches round an empty
 * model's origin. */
const MARGIN = 16 * STUD;
/** How far under the model's bottom it lies, in LDU: under the ground shadow (half an LDU down), so neither the shadow
 * nor a baseplate's underside fights it. */
const BELOW = 0.75;

/** The lines' color and opacity in each theme: dark lines on the light background, light ones on the dark. */
const LOOK: Record<Theme, { color: number; minor: number; major: number }> = {
  light: { color: 0x5b6472, minor: 0.16, major: 0.32 },
  dark: { color: 0xa7adb8, minor: 0.1, major: 0.22 },
};

const VERTEX = /* glsl */ `
varying vec2 vGround;
void main() {
  vec4 world = modelMatrix * vec4(position, 1.0);
  vGround = world.xz;
  gl_Position = projectionMatrix * viewMatrix * world;
}
`;

// A line's coverage from its distance to the nearest grid line in screen pixels (fwidth), so it's antialiased and a
// pixel wide; a spacing too fine to tell apart on screen fades out rather than shimmer.
const FRAGMENT = /* glsl */ `
uniform vec3 color;
uniform float minor;
uniform float major;
uniform vec2 center;
uniform float radius;
varying vec2 vGround;
float lines(vec2 p, float spacing) {
  vec2 cell = p / spacing;
  vec2 width = fwidth(cell);
  vec2 off = abs(fract(cell - 0.5) - 0.5) / max(width, vec2(1e-6));
  float on = 1.0 - min(min(off.x, off.y), 1.0);
  return on * (1.0 - smoothstep(0.12, 0.3, max(width.x, width.y)));
}
void main() {
  float fade = 1.0 - smoothstep(radius * 0.55, radius, distance(vGround, center));
  float alpha = max(lines(vGround, ${STUD.toFixed(1)}) * minor, lines(vGround, ${(STUD * MAJOR).toFixed(1)}) * major) * fade;
  if (alpha < 0.004) discard;
  gl_FragColor = vec4(color, alpha);
  #include <colorspace_fragment>
}
`;

export class Grid {
  readonly mesh: Mesh<PlaneGeometry, ShaderMaterial>;

  constructor(theme: Theme) {
    const material = new ShaderMaterial({
      uniforms: {
        color: { value: new Color() },
        minor: { value: 0 },
        major: { value: 0 },
        center: { value: new Vector2() },
        radius: { value: MARGIN },
      },
      vertexShader: VERTEX,
      fragmentShader: FRAGMENT,
      transparent: true,
      depthWrite: false,
      toneMapped: false,
    });
    this.mesh = new Mesh(new PlaneGeometry(1, 1), material);
    this.mesh.rotation.x = -Math.PI / 2;
    // Before the model and the shadow, which draw over it.
    this.mesh.renderOrder = -1;
    this.mesh.name = "ground-grid";
    this.setTheme(theme);
    this.place(null);
  }

  setTheme(theme: Theme): void {
    const look = LOOK[theme];
    const u = this.mesh.material.uniforms;
    (u.color!.value as Color).setHex(look.color);
    u.minor!.value = look.minor;
    u.major!.value = look.major;
  }

  /** Under the model's footprint (in the scene's frame, Y up: its min and max x and z, and its bottom y), reaching
   * `MARGIN` past it; with no model, round the origin on the table. The lines stay on whole studs, wherever it is. */
  place(footprint: { minX: number; maxX: number; minZ: number; maxZ: number; bottom: number } | null): void {
    const f = footprint ?? { minX: 0, maxX: 0, minZ: 0, maxZ: 0, bottom: 0 };
    const cx = (f.minX + f.maxX) / 2;
    const cz = (f.minZ + f.maxZ) / 2;
    const radius = Math.hypot(f.maxX - f.minX, f.maxZ - f.minZ) / 2 + MARGIN;
    this.mesh.position.set(cx, f.bottom - BELOW, cz);
    this.mesh.scale.set(radius * 2, radius * 2, 1);
    const u = this.mesh.material.uniforms;
    (u.center!.value as Vector2).set(cx, cz);
    u.radius!.value = radius;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    this.mesh.material.dispose();
  }
}
