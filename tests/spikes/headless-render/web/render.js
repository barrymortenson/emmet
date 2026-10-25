// SPDX-License-Identifier: AGPL-3.0-or-later
// ADR 0005 spike: render once (a cube, or an LDraw model through LDrawLoader), then send the PNG to Rust.
import * as THREE from "three";
import { LDrawConditionalLineMaterial } from "./vendor/LDrawConditionalLineMaterial.js";
import { LDrawLoader } from "./vendor/LDrawLoader.js";

const { invoke } = window.__TAURI__.core;
const { size, model, base } = window.__SPIKE__;
const mark = (name) => invoke("mark", { name });
window.addEventListener("error", (e) => invoke("fail", { message: String(e.message) }));
window.addEventListener("unhandledrejection", (e) => invoke("fail", { message: String(e.reason?.stack ?? e.reason) }));
// LDrawLoader reports a part it couldn't load with console.warn and carries on; a partial model must fail the render.
const missing = [];
const warn = console.warn;
console.warn = (...args) => {
  if (args.some((a) => String(a?.message ?? a).includes("could not be loaded"))) missing.push(String(args[0]?.message ?? args[0]));
  warn(...args);
};

await mark("script");
const renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
renderer.setSize(size, size);
renderer.setClearColor(0xffffff);
document.body.appendChild(renderer.domElement);
const scene = new THREE.Scene();
scene.add(new THREE.HemisphereLight(0xffffff, 0x8899aa, 2.2));
const sun = new THREE.DirectionalLight(0xffffff, 2.0);
sun.position.set(1, 2, 1.5);
scene.add(sun);

let object;
if (model) {
  const loader = new LDrawLoader();
  loader.setConditionalLineMaterial(LDrawConditionalLineMaterial);
  loader.setPartsLibraryPath(base);
  await loader.preloadMaterials(`${base}LDConfig.ldr`);
  object = await loader.loadAsync(`${base}model/${encodeURIComponent(model)}`);
  object.rotation.x = Math.PI; // LDraw is −Y up
  if (missing.length) await invoke("fail", { message: missing.join("; ") });
  await mark(`parts ${object.children.length}`);
} else {
  object = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), new THREE.MeshStandardMaterial({ color: 0xc91a09 }));
  object.rotation.set(0.5, 0.7, 0);
}
scene.add(object);
await mark("loaded");

// An isometric-ish view that fits the bounding sphere.
const sphere = new THREE.Box3().setFromObject(object).getBoundingSphere(new THREE.Sphere());
const camera = new THREE.PerspectiveCamera(35, 1, sphere.radius / 100, sphere.radius * 100);
const dir = new THREE.Vector3(1, 0.8, 1.2).normalize();
camera.position.copy(sphere.center).addScaledVector(dir, sphere.radius / Math.sin(THREE.MathUtils.degToRad(35 / 2)));
camera.lookAt(sphere.center);
renderer.render(scene, camera);
await mark("rendered");

const blob = await new Promise((resolve) => renderer.domElement.toBlob(resolve, "image/png"));
await invoke("save_png", new Uint8Array(await blob.arrayBuffer()));
