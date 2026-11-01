// SPDX-License-Identifier: AGPL-3.0-or-later
// A step's new parts outlined (plan section 08's step rules, section 13): the parts drawn flat into a mask, then a
// full-screen pass that draws the outline color wherever a pixel outside the mask has a neighbor inside it. The parts
// are the step's newest, with later steps hidden, so they're on top and their outline is what the eye follows.

import {
  Color,
  type Camera,
  HalfFloatType,
  Mesh,
  MeshBasicMaterial,
  OrthographicCamera,
  PlaneGeometry,
  type Scene,
  ShaderMaterial,
  Vector2,
  WebGLRenderTarget,
  type WebGLRenderer,
} from "three";

export class Outline {
  private target = new WebGLRenderTarget(1, 1, { type: HalfFloatType, depthBuffer: true });
  readonly mask = new MeshBasicMaterial({ color: 0xffffff });
  private quad: Mesh<PlaneGeometry, ShaderMaterial>;
  private camera = new OrthographicCamera(-1, 1, 1, -1, 0, 1);

  constructor() {
    this.quad = new Mesh(
      new PlaneGeometry(2, 2),
      new ShaderMaterial({
        uniforms: {
          mask: { value: this.target.texture },
          texel: { value: new Vector2(1, 1) },
          color: { value: new Color(0xd92d20) },
          width: { value: 2 },
        },
        vertexShader: /* glsl */ `
          varying vec2 vUv;
          void main() { vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }
        `,
        fragmentShader: /* glsl */ `
          uniform sampler2D mask;
          uniform vec2 texel;
          uniform vec3 color;
          uniform float width;
          varying vec2 vUv;
          void main() {
            float inside = texture2D(mask, vUv).r;
            float near = 0.0;
            for (int i = 0; i < 8; i++) {
              float a = float(i) * 0.785398;
              near = max(near, texture2D(mask, vUv + vec2(cos(a), sin(a)) * texel * width).r);
            }
            float edge = clamp(near - inside, 0.0, 1.0);
            if (edge <= 0.0) discard;
            gl_FragColor = vec4(color, edge);
          }
        `,
        transparent: true,
        depthTest: false,
        depthWrite: false,
        toneMapped: false,
      }),
    );
  }

  /** Draws `scene` (the outlined parts, drawn flat) as an outline over what `r` already drew. */
  draw(r: WebGLRenderer, scene: Scene, camera: Camera, color: Color, pixelRatio: number): void {
    const size = r.getDrawingBufferSize(new Vector2());
    if (this.target.width !== size.x || this.target.height !== size.y) this.target.setSize(size.x, size.y);
    const before = r.getRenderTarget();
    const clear = r.getClearColor(new Color());
    const alpha = r.getClearAlpha();
    const background = scene.background;
    const override = scene.overrideMaterial;
    const autoClear = r.autoClear;
    try {
      scene.background = null;
      scene.overrideMaterial = this.mask;
      r.setRenderTarget(this.target);
      r.setClearColor(0x000000, 0);
      r.autoClear = true;
      r.render(scene, camera);
      r.setRenderTarget(before);
      r.autoClear = false;
      const u = this.quad.material.uniforms;
      u.texel!.value.set(1 / size.x, 1 / size.y);
      u.color!.value.copy(color);
      u.width!.value = 1.5 * pixelRatio;
      r.render(this.quad, this.camera);
    } finally {
      scene.background = background;
      scene.overrideMaterial = override;
      r.setRenderTarget(before);
      r.setClearColor(clear, alpha);
      r.autoClear = autoClear;
    }
  }

  dispose(): void {
    this.target.dispose();
    this.mask.dispose();
    this.quad.geometry.dispose();
    this.quad.material.dispose();
  }
}
