// SPDX-License-Identifier: AGPL-3.0-or-later
// The build animation (plan section 13, D-34; session 4C): a knob's run never animates; another run (an edit, the
// agent's) does, and Skip ends it; Replay build plays the whole model from empty; Settings' Fast fits it into 0.6 s and
// Off turns it off; reduced motion turns it off too; a render made mid-animation shows the finished model. The camera
// never moves on its own through any of it (law 1).
import { $, expect } from "@wdio/globals";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { invoke, nextScene, open, restingCamera, shown, stats } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("animate");
const modelPy = join(dir, "model.py");
const original = readFileSync(modelPy, "utf8");
const mod = process.platform === "darwin" ? "Meta" : "Control";

/** The page's clock, for drawnAt(). */
const now = () => browser.execute(() => performance.now());
/** The most parts animating at once, from `change()` until its update is in the viewer and 0.4 s after (an animation
 * starts as the update lands, and a part takes 160 ms, so one that ran would be seen). */
const mostAnimating = async (change) => {
  const t0 = await now();
  await change();
  let most = 0;
  await browser.waitUntil(
    async () => {
      most = Math.max(most, (await stats()).animating);
      return (await browser.execute(() => window.__emmet.drawnAt())) > t0;
    },
    { timeout: 20_000, interval: 16, timeoutMsg: "the change never reached the viewer" },
  );
  const until = Date.now() + 400;
  while (Date.now() < until) most = Math.max(most, (await stats()).animating);
  return most;
};

async function pace(choice) {
  await browser.keys([mod, ","]);
  await (await $("[aria-label='Build animation']")).waitForExist({ timeout: 10_000 });
  await (await (await $("[aria-label='Build animation']")).$(`button=${choice}`)).click();
  await browser.waitUntil(async () => (await stats()).pace === choice.toLowerCase(), { timeoutMsg: `the viewer's pace isn't ${choice}` });
  await (await $("#settings-done")).click();
}

describe("the build animation", () => {
  let still;
  before(async () => {
    await open(dir);
    still = await restingCamera();
  });
  afterEach(async () => {
    expect(await restingCamera()).toEqual(still);
  });
  // A failed test mustn't leave the next specs at Fast or Off.
  after(async () => {
    const { prefs } = await invoke("get_prefs");
    await invoke("set_prefs", { prefs: { ...prefs, build_animation: "animated" } });
    await browser.execute(() => window.__emmet.viewer().setPace("animated"));
  });

  it("a knob's run shows at once: nothing animates", async () => {
    const gen = await shown();
    const before = (await stats()).instances;
    await $("#tab-options").click();
    const most = await mostAnimating(() => $("#param-roof-switch").click());
    expect((await stats()).instances).not.toBe(before);
    expect(most).toBe(0);
    expect((await stats()).lastDiff.removed).toBeGreaterThan(0);
    await $("#knobs-reset").click();
    await nextScene(gen + 1);
    await browser.waitUntil(async () => (await stats()).instances === before);
  });

  it("an edit's run builds up part by part, and Skip ends it at once", async () => {
    const gen = await shown();
    writeFileSync(modelPy, original.replace("chimney: bool = True", "chimney: bool = False"));
    await nextScene(gen);
    writeFileSync(modelPy, original);
    await nextScene(gen + 1);
    await browser.waitUntil(async () => (await stats()).animating > 0, { timeout: 10_000, timeoutMsg: "the chimney didn't build up" });
    await $("#skip-animation").click();
    await browser.waitUntil(async () => (await stats()).animating === 0, { timeoutMsg: "Skip didn't end the animation" });
    await expect($("#skip-animation")).not.toBeExisting();
  });

  it("Replay build plays the whole model from empty, and a render mid-way shows the finished model", async () => {
    const finished = await browser.execute(async () => {
      const { png } = await window.__emmet.viewer().render({ size: 256 });
      return Array.from(png);
    });
    await $("#replay").click();
    await browser.waitUntil(async () => (await stats()).animating > 100, { timeoutMsg: "Replay build didn't start from empty" });
    const midway = await browser.execute(async () => {
      const { png } = await window.__emmet.viewer().render({ size: 256 });
      return Array.from(png);
    });
    expect((await stats()).animating).toBeGreaterThan(0);
    expect(midway).toEqual(finished);
    await $("#skip-animation").click();
  });

  it("Replay build from a step plays the whole model", async () => {
    await $("#tab-steps").click();
    await (await $("#steps-tree [role='treeitem'][data-n='3']")).click();
    await browser.waitUntil(async () => (await stats()).step === 3);
    await $("#replay").click();
    await browser.waitUntil(async () => (await stats()).animating > 100, { timeoutMsg: "Replay build didn't start" });
    // Still playing a moment later: closing the step didn't end it.
    await browser.pause(300);
    const s = await stats();
    expect(s.animating).toBeGreaterThan(0);
    expect(s.step).toBe(null);
    await $("#skip-animation").click();
  });

  it("Fast fits the whole model into 0.6 s, and Off doesn't animate at all", async () => {
    await pace("Fast");
    // Timed in the page, from the replay to the animation's end: WebDriver's round trips stay out of it.
    const ms = await browser.execute(
      () =>
        new Promise((resolve) => {
          const v = window.__emmet.viewer();
          let t0 = 0;
          const off = v.on("animation", (running) => {
            if (running) t0 = performance.now();
            else {
              off();
              resolve(performance.now() - t0);
            }
          });
          document.getElementById("replay").click();
        }),
    );
    // 0.6 s for the batch, and a frame or two.
    expect(ms).toBeGreaterThan(100);
    expect(ms).toBeLessThan(800);
    await pace("Off");
    await expect($("#replay")).toBeDisabled();
    expect(await mostAnimating(() => writeFileSync(modelPy, original.replace("chimney: bool = True", "chimney: bool = False")))).toBe(0);
    expect(await mostAnimating(() => writeFileSync(modelPy, original))).toBe(0);
    await pace("Animated");
  });

  it("reduced motion turns the build animation off", async () => {
    await browser.execute(() => {
      const real = window.matchMedia.bind(window);
      window.__realMatchMedia = real;
      window.matchMedia = (q) => (q.includes("prefers-reduced-motion") ? { matches: true, media: q, addEventListener() {}, removeEventListener() {} } : real(q));
    });
    try {
      await $("#replay").click();
      await browser.pause(300);
      expect((await stats()).animating).toBe(0);
      expect(await mostAnimating(() => writeFileSync(modelPy, original.replace("chimney: bool = True", "chimney: bool = False")))).toBe(0);
      expect(await mostAnimating(() => writeFileSync(modelPy, original))).toBe(0);
    } finally {
      await browser.execute(() => (window.matchMedia = window.__realMatchMedia));
    }
  });
});
