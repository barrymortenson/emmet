// SPDX-License-Identifier: AGPL-3.0-or-later
// What the specs ask the running app, through the E2E hooks (window.__emmet, only in EMMET_E2E builds) and the
// app's own typed commands. Viewer assertions read viewer.stats() and ids, not pixels (plan section 16).
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { os, out } from "./paths.mjs";

export const stats = () => browser.execute(() => window.__emmet.viewer().stats());
export const camera = () => browser.execute(() => window.__emmet.viewer().cameraMatrix());
/** The generation of the scene on screen (-1 before the first). */
export const shown = () => browser.execute(() => window.__emmet.shown());

/** Calls one of the app's commands, as its UI does. */
export const invoke = (cmd, args = {}) =>
  browser.execute((c, a) => window.__TAURI_INTERNALS__.invoke(c, a), cmd, args);

/** Opens a project from the recents list (lib/projects.mjs put it there) and waits for its first scene. */
export async function open(dir) {
  const { root } = await invoke("open_recent", { path: dir });
  await browser.waitUntil(
    () => browser.execute((r) => window.__emmet.project() === r && window.__emmet.shown() >= 1, root),
    { timeout: 30_000, timeoutMsg: `${dir} never showed a scene` },
  );
  return root;
}

/** Waits for a scene newer than `after`, and returns its generation. */
export async function nextScene(after, timeout = 30_000) {
  let gen = -1;
  await browser.waitUntil(async () => (gen = await shown()) > after, {
    timeout,
    timeoutMsg: `no scene after generation ${after}`,
  });
  return gen;
}

/** Waits until the viewer has drawn `count` instances. */
export async function instances(count, timeout = 30_000) {
  let last;
  await browser
    .waitUntil(async () => (last = await stats()).instances === count, { timeout })
    .catch(() => {
      throw new Error(`the viewer never drew ${count} instances; its stats: ${JSON.stringify(last)}`);
    });
}

/** Saves a window screenshot in the light and dark themes as tests/e2e/out/screens/<os>/<name>-<theme>.png. */
export async function screenshots(name) {
  const dir = join(out, "screens", os);
  mkdirSync(dir, { recursive: true });
  for (const theme of ["light", "dark"]) {
    await browser.execute((t) => window.__emmet.theme(t), theme);
    await browser.execute(() => window.__emmet.viewer().nextFrame());
    await browser.saveScreenshot(join(dir, `${name}-${theme}.png`));
  }
  await browser.execute(() => window.__emmet.theme(null));
}

/** The viewer's render of the scene on screen, as PNG bytes. */
export async function render(options) {
  const b64 = await browser.execute(async (o) => {
    const { png } = await window.__emmet.viewer().render(o);
    let s = "";
    for (let i = 0; i < png.length; i += 0x8000) s += String.fromCharCode(...png.subarray(i, i + 0x8000));
    return btoa(s);
  }, options);
  return Buffer.from(b64, "base64");
}
