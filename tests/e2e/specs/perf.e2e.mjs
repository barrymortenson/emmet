// SPDX-License-Identifier: AGPL-3.0-or-later
// Plan section 13's budgets at 2,000 parts, on the 2,018-part town block (tests/fixtures/projects/scene-stress),
// enforced on the self-hosted Mac (session 4C Done when 1): first load with an empty part cache ≤ 6 s, slider →
// updated viewer ≤ 3 s, orbit frame ≤ 25 ms, frame during a build animation of the whole model ≤ 25 ms, and draw
// calls under ~300. Hosted runners draw WebGL on the CPU, so there the numbers are only reported (the bench's table
// has them all, at every size).
import { expect } from "@wdio/globals";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { nextScene, open, shown, stats } from "../lib/app.mjs";
import { emmet, projectDir } from "../lib/projects.mjs";

const dir = projectDir("perf");
const enforce = process.platform === "darwin" && process.env.RUNNER_ENVIRONMENT !== "github-hosted";
const BUDGET = { firstLoadMs: 6000, updateMs: 3000, orbitFrameMs: 25, animationFrameMs: 25, drawCalls: 300 };
const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];
const within = (name, value) => {
  console.log(`${name}: ${value} (budget ${BUDGET[name]})`);
  if (enforce) expect(value).toBeLessThanOrEqual(BUDGET[name]);
};

describe("the viewer's budgets at 2,000 parts", () => {
  it("loads, updates, orbits, and builds up within section 13's budgets", async () => {
    emmet(["--project", dir, "run"]);
    const placements = JSON.parse(readFileSync(join(dir, "build", "placements.json"), "utf8")).parts;
    expect(placements.length).toBeGreaterThan(2000);
    const first = await browser.execute(async (list) => {
      const v = window.__emmet.newViewer();
      const t0 = performance.now();
      await v.load(list);
      await v.nextFrame();
      const ms = performance.now() - t0;
      v.dispose();
      return ms;
    }, placements);
    within("firstLoadMs", Math.round(first));

    await open(dir);
    const updates = [];
    for (let i = 1; i <= 3; i++) {
      const gen = await shown();
      const t0 = await browser.execute(async (floors) => {
        const t = performance.now();
        await window.__TAURI_INTERNALS__.invoke("set_params", { values: { floors } });
        return t;
      }, 6 - (i % 2));
      await nextScene(gen);
      await browser.waitUntil(async () => (await browser.execute(() => window.__emmet.drawnAt())) > t0);
      updates.push((await browser.execute(() => window.__emmet.drawnAt())) - t0);
    }
    within("updateMs", Math.round(median(updates)));

    const frames = await browser.execute(async () => {
      const v = window.__emmet.viewer();
      v.skipAnimation();
      const frame = () => Promise.race([v.nextFrame().then(() => true), new Promise((r) => setTimeout(() => r(false), 1000))]);
      const run = async (step) => {
        await frame();
        const t0 = performance.now();
        let n = 0;
        while (n < 90 && performance.now() - t0 < 5000) {
          step();
          if (!(await frame())) break;
          n++;
        }
        return n ? (performance.now() - t0) / n : null;
      };
      const orbit = await run(() => v.orbit(0.03));
      v.replayBuild();
      await frame();
      // A replay that didn't start (the pace Off) would time idle frames.
      if (v.stats().animating === 0) throw new Error("the build animation didn't start");
      const animation = await run(() => {});
      const calls = v.stats().drawCalls;
      v.skipAnimation();
      return { orbit, animation, calls };
    });
    within("orbitFrameMs", Number(frames.orbit.toFixed(1)));
    within("animationFrameMs", Number(frames.animation.toFixed(1)));
    within("drawCalls", (await stats()).drawCalls);
  });
});
