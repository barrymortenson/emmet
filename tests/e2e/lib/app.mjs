// SPDX-License-Identifier: AGPL-3.0-or-later
// What the specs ask the running app, through the E2E hooks (window.__emmet, only in EMMET_E2E builds) and the
// app's own typed commands. Viewer assertions read viewer.stats() and ids, not pixels (plan section 16).
import { $ } from "@wdio/globals";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import pixelmatch from "pixelmatch";
import { os, out, repo } from "./paths.mjs";
import { decode, encode, shrink } from "./png.mjs";

/** Window baselines are kept an eighth the size each way: small to commit, and a moved or restyled control still
 * changes far more than the tolerance. */
const SHRINK = 8;
/** At most this share of a window's pixels may differ: room for the antialiasing of text, not for a layout change. */
const WINDOW_TOLERANCE = 0.01;
const update = process.env.EMMET_UPDATE_BASELINES === "1";

export const stats = () => browser.execute(() => window.__emmet.viewer().stats());
export const camera = () => browser.execute(() => window.__emmet.viewer().cameraMatrix());
/** The generation of the scene on screen (-1 before the first). */
export const shown = () => browser.execute(() => window.__emmet.shown());

/** Calls one of the app's commands, as its UI does. */
export const invoke = (cmd, args = {}) =>
  browser.execute((c, a) => window.__TAURI_INTERNALS__.invoke(c, a), cmd, args);

/** Opens a project from the recents list (lib/projects.mjs put it there) and waits for its first scene: drawn, framed,
 * and the camera still. shown() counts a scene as soon as the viewer takes it, before its parts load and the camera
 * frames it, and the orbit controls round the framed camera off over the next frames (a last-digit change). */
export async function open(dir) {
  const drawn = await browser.execute(() => window.__emmet.drawnAt());
  const { root } = await invoke("open_recent", { path: dir });
  // The viewer's own error, as soon as there is one, rather than a timeout that hides it.
  await browser.waitUntil(
    async () => {
      const [ready, error] = await browser.execute(
        (r, d) => [window.__emmet.project() === r && window.__emmet.shown() >= 1 && window.__emmet.drawnAt() > d, window.__emmet.viewerError()],
        root,
        drawn,
      );
      if (error) throw new Error(`${dir}: ${error}`);
      return ready;
    },
    { timeout: 30_000, timeoutMsg: `${dir} never showed a scene` },
  );
  const still = await browser.execute(async () => {
    const v = window.__emmet.viewer();
    let last = JSON.stringify(v.cameraMatrix());
    let same = 0;
    for (let i = 0; i < 300 && same < 5; i++) {
      await v.nextFrame();
      const now = JSON.stringify(v.cameraMatrix());
      same = now === last ? same + 1 : 0;
      last = now;
    }
    return same >= 5;
  });
  if (!still) throw new Error(`${dir}: the camera never came to rest`);
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

/** Saves a window screenshot in the light and dark themes as tests/e2e/out/screens/<os>/<name>-<theme>.png. With
 * `baseline`, each is compared with tests/e2e/baselines/<os>/screens/<name>-<theme>.png (a frozen screen's id, session
 * 4A): a missing one is written and fails until it's committed; EMMET_UPDATE_BASELINES=1 rewrites them. */
export async function screenshots(name, { baseline = false } = {}) {
  const dir = join(out, "screens", os);
  mkdirSync(dir, { recursive: true });
  const failed = [];
  // A screen is still: no build running, the knobs read, the part pictures drawn.
  await browser.waitUntil(
    () =>
      browser.execute(
        () =>
          !document.querySelector("#model-state .spinner") &&
          !document.body.innerText.includes("Reading the model's knobs") &&
          !document.querySelector(".part-pic-empty"),
      ),
    { timeout: 20_000, timeoutMsg: `${name}: the window never settled` },
  );
  await browser.pause(300);
  if (baseline) await freshAgents();
  // A frozen screen shows no tip (they're taught once each, 4B): Got it, as the user would.
  if (baseline) while (await (await $("#tip-done")).isExisting()) await (await $("#tip-done")).click();
  for (const theme of ["light", "dark"]) {
    await browser.execute((t) => window.__emmet.theme(t), theme);
    await browser.execute(() => window.__emmet.viewer().nextFrame());
    // The controls fade to the theme's colors: a screen taken mid-fade has the other theme's chips.
    await browser.waitUntil(() => browser.execute(() => !document.getAnimations().some((a) => a instanceof CSSTransition)), {
      timeout: 5000,
      timeoutMsg: `${name}-${theme}: the theme's transitions never finished`,
    });
    // A focus ring isn't part of a screen, unless a menu, popover, or dialog is open (moving focus would close it).
    await browser.execute(() => document.querySelector('[role="menu"], [role="dialog"], [role="listbox"], [role="tooltip"]') || document.activeElement?.blur?.());
    const file = join(dir, `${name}-${theme}.png`);
    await browser.saveScreenshot(file);
    if (baseline) {
      const why = compare(`${name}-${theme}`, readFileSync(file));
      if (why) failed.push(why);
    }
  }
  await browser.execute(() => window.__emmet.theme(null));
  if (failed.length) throw new Error(failed.join("\n"));
}

/** The chat's agents as they're detected now. The specs share one app, and the dock keeps the list it last read, so
 * an agent an earlier spec signed out would still show signed out: the dock checks again (a script click, which leaves
 * an open menu or dialog open) and shows each agent as detected. */
async function freshAgents() {
  const agents = await invoke("agents", { refresh: true });
  const again = await browser.execute(() => {
    const button = document.getElementById("agents-again");
    button?.click();
    return !!button;
  });
  if (!again) return;
  const ready = Object.fromEntries(agents.map((a) => [a.kind, a.detection.path != null && a.detection.signed_in]));
  await browser.waitUntil(
    () => browser.execute((r) => Object.entries(r).every(([kind, ok]) => document.getElementById(`agent-${kind}`)?.disabled === !ok), ready),
    { timeout: 10_000, timeoutMsg: `the chat never showed the agents as detected: ${JSON.stringify(ready)}` },
  );
}

/** Compares a window screenshot with its baseline; a reason when it doesn't match. */
function compare(name, png) {
  const small = shrink(decode(png), SHRINK);
  const base = join(repo, "tests", "e2e", "baselines", os, "screens", `${name}.png`);
  if (update || !existsSync(base)) {
    mkdirSync(join(base, ".."), { recursive: true });
    writeFileSync(base, encode(small));
    return update ? null : `wrote a new baseline, ${base}; check it and commit it`;
  }
  const want = decode(readFileSync(base));
  if (want.width !== small.width || want.height !== small.height) return `${name}: the window is ${small.width * SHRINK} × ${small.height * SHRINK}, the baseline ${want.width * SHRINK} × ${want.height * SHRINK}`;
  const diff = { width: small.width, height: small.height, data: Buffer.alloc(small.data.length) };
  const bad = pixelmatch(want.data, small.data, diff.data, small.width, small.height, { threshold: 0.1 });
  const share = bad / (small.width * small.height);
  console.log(`${name}: ${bad} pixels differ (${(share * 100).toFixed(2)} %)`);
  if (share <= WINDOW_TOLERANCE) return null;
  const dir = join(out, "visual", os);
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, `${name}-diff.png`), encode(diff));
  return `${name}: ${(share * 100).toFixed(2)} % of the window differs from its baseline (diff in ${dir})`;
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
