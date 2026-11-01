// SPDX-License-Identifier: AGPL-3.0-or-later
// The build animation (plan section 13, D-34; session 4C): a knob's run never animates; another run (an edit, the
// agent's) does, and Skip ends it; Replay build plays the whole model from empty; Settings' Fast fits it into 0.6 s and
// Off turns it off; reduced motion turns it off too; a render made mid-animation shows the finished model. The camera
// never moves on its own through any of it (law 1).
import { $, expect } from "@wdio/globals";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { camera, nextScene, open, shown, stats } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("animate");
const modelPy = join(dir, "model.py");
const original = readFileSync(modelPy, "utf8");
const mod = process.platform === "darwin" ? "Meta" : "Control";

/** The most parts animating at once, sampled every frame until `done` says so. */
const sampleUntil = async (done, timeout = 20_000) => {
  let most = 0;
  await browser.waitUntil(
    async () => {
      most = Math.max(most, (await stats()).animating);
      return done();
    },
    { timeout, interval: 16 },
  );
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
    still = await camera();
  });
  afterEach(async () => {
    expect(await camera()).toEqual(still);
  });

  it("a knob's run shows at once: nothing animates", async () => {
    const gen = await shown();
    const before = (await stats()).instances;
    await $("#tab-options").click();
    await $("#param-roof-switch").click();
    const most = await sampleUntil(async () => (await shown()) > gen && (await stats()).instances !== before);
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

  it("Fast fits the whole model into 0.6 s, and Off doesn't animate at all", async () => {
    await pace("Fast");
    await $("#replay").click();
    const t0 = Date.now();
    await browser.waitUntil(async () => (await stats()).animating > 0, { timeoutMsg: "Fast didn't animate" });
    await browser.waitUntil(async () => (await stats()).animating === 0, { timeout: 5_000, interval: 16 });
    // 0.6 s for the batch, and what WebDriver's round trips add.
    expect(Date.now() - t0).toBeLessThan(1_600);
    await pace("Off");
    await expect($("#replay")).toBeDisabled();
    const gen = await shown();
    writeFileSync(modelPy, original.replace("chimney: bool = True", "chimney: bool = False"));
    expect(await sampleUntil(async () => (await shown()) > gen)).toBe(0);
    writeFileSync(modelPy, original);
    expect(await sampleUntil(async () => (await shown()) > gen + 1)).toBe(0);
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
      const gen = await shown();
      writeFileSync(modelPy, original.replace("chimney: bool = True", "chimney: bool = False"));
      expect(await sampleUntil(async () => (await shown()) > gen)).toBe(0);
      writeFileSync(modelPy, original);
      await nextScene(gen + 1);
    } finally {
      await browser.execute(() => (window.matchMedia = window.__realMatchMedia));
    }
  });
});
