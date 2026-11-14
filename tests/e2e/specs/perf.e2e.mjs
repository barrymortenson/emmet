// SPDX-License-Identifier: AGPL-3.0-or-later
// Plan section 13's budgets, at each size it names: about 100 parts (rung 1), about 500 (rung 2), and 2,000, on the
// town block (tests/fixtures/projects/scene-stress) with one building, one floor, and all of it. For each: first load
// with an empty part cache, slider → updated viewer, an orbit frame, and a frame during a build animation of the
// whole model; at 2,000 parts, draw calls under ~300 too. Rung 3's set (165 parts) is the size of the first.
//
// The budgets are for a machine with a graphics card (section 13: "M1 / Intel Iris Xe class"), so they're enforced
// on one, on either OS, and only reported where WebGL draws on the CPU: GitHub's runners, and any machine whose
// renderer the viewer takes for software (it draws Fast there). The numbers go to tests/e2e/out/perf-<os>.md.
import { expect } from "@wdio/globals";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { nextScene, open, shown, stats } from "../lib/app.mjs";
import { os, out } from "../lib/paths.mjs";
import { emmet, projectDir } from "../lib/projects.mjs";

// A frame at 60 Hz is 16.7 ms: section 13's "16 ms" is "no dropped frames", so the budget is a frame, rounded up
// past the timer's noise.
const FRAME = 17;
const SCENES = [
  { name: "rung 1 (about 100 parts)", dir: projectDir("perf-rung1"), atLeast: 100, floors: [1, 2], budget: { firstLoadMs: 1500, updateMs: 800, orbitFrameMs: FRAME, animationFrameMs: FRAME } },
  { name: "rung 2 (about 500 parts)", dir: projectDir("perf-rung2"), atLeast: 400, floors: [1, 2], budget: { firstLoadMs: 3000, updateMs: 1500, orbitFrameMs: FRAME, animationFrameMs: FRAME } },
  { name: "2,000 parts", dir: projectDir("perf"), atLeast: 2000, floors: [6, 5], budget: { firstLoadMs: 6000, updateMs: 3000, orbitFrameMs: 25, animationFrameMs: 25, drawCalls: 300 } },
];
const hosted = process.env.RUNNER_ENVIRONMENT === "github-hosted";
const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];
const rows = [];

describe("the viewer's budgets", () => {
  after(() => {
    if (!rows.length) return;
    const { gpu, quality, enforced } = rows[0];
    const cell = (r, k) => (r[k] == null ? "–" : `${r[k]} (≤ ${r.budget[k]})`);
    const table = [
      `Viewer budgets on ${os} (${hosted ? "a hosted runner" : "this machine"}): ${gpu}, ${quality} preset, ${enforced ? "enforced" : "reported only"}`,
      "",
      "| Scene | Parts | First load ms | Slider → viewer ms | Orbit frame ms | Animation frame ms | Draw calls |",
      "|---|---|---|---|---|---|---|",
      ...rows.map((r) => `| ${r.name} | ${r.parts} | ${cell(r, "firstLoadMs")} | ${cell(r, "updateMs")} | ${cell(r, "orbitFrameMs")} | ${cell(r, "animationFrameMs")} | ${r.drawCalls}${r.budget.drawCalls ? ` (≤ ${r.budget.drawCalls})` : ""} |`),
      "",
      ...rows.map((r) => `First load, ${r.name}: ${r.firstLoadParts} ms`),
      "",
    ].join("\n");
    writeFileSync(join(out, `perf-${os}.md`), table);
    console.log(table);
  });

  for (const scene of SCENES) {
    it(`${scene.name}: loads, updates, orbits, and builds up within section 13's budgets`, async () => {
      const dir = scene.dir;
      const BUDGET = scene.budget;
      emmet(["--project", dir, "run"]);
      const placements = JSON.parse(readFileSync(join(dir, "build", "placements.json"), "utf8")).parts;
      expect(placements.length).toBeGreaterThan(scene.atLeast);
      // The window's own viewer says what draws; the project opens after the cold first load is timed.
      const { gpu, quality } = await stats();
      const enforce = !hosted && quality !== "fast";
      // A Mac with a graphics card that lands on Fast has a broken pick, and would report where it should enforce.
      if (process.platform === "darwin" && !hosted) expect(quality).not.toBe("fast");
      const row = { name: scene.name, parts: placements.length, budget: BUDGET, gpu, quality, enforced: enforce };
      rows.push(row);
      const over = [];
      const within = (name, value) => {
        row[name] = value;
        console.log(`${scene.name}: ${name}: ${value} (budget ${BUDGET[name]})`);
        if (enforce && value > BUDGET[name]) over.push(`${name} ${value} > ${BUDGET[name]}`);
      };
      const first = await browser.execute(async (list) => {
        // Where a first load's time goes: the library's answer (one request for every part's files), reading the
        // parts and building the scene, and the first frame (the shaders compile there).
        let fetched = 0;
        const timed = (fn) => async (...args) => {
          const t = performance.now();
          try {
            return await fn(...args);
          } finally {
            fetched += performance.now() - t;
          }
        };
        const source = window.__emmet.library;
        const v = window.__emmet.newViewer({ parts: timed(source.parts), colors: timed(source.colors) });
        const t0 = performance.now();
        await v.load(list);
        const loaded = performance.now();
        await v.nextFrame();
        const ms = performance.now() - t0;
        v.dispose();
        return { ms, fetched, built: loaded - t0 - fetched, frame: t0 + ms - loaded };
      }, placements);
      row.firstLoadParts = `library ${Math.round(first.fetched)}, build ${Math.round(first.built)}, first frame ${Math.round(first.frame)}`;
      within("firstLoadMs", Math.round(first.ms));

      await open(dir);
      const updates = [];
      for (let i = 1; i <= 3; i++) {
        const gen = await shown();
        const t0 = await browser.execute(async (floors) => {
          const t = performance.now();
          await window.__TAURI_INTERNALS__.invoke("set_params", { values: { floors } });
          return t;
        }, scene.floors[i % 2]);
        await nextScene(gen);
        await browser.waitUntil(async () => (await browser.execute(() => window.__emmet.drawnAt())) > t0);
        updates.push((await browser.execute(() => window.__emmet.drawnAt())) - t0);
      }
      within("updateMs", Math.round(median(updates)));

      const frames = await browser.execute(async () => {
        const v = window.__emmet.viewer();
        v.skipAnimation();
        // WebKit's own frame interval with nothing to draw: the floor under the frames below. At 33.3 ms (30 fps), WebKit
        // or the display is throttling the window, and an orbit frame can't come in faster whatever the viewer costs.
        const screen = await new Promise((resolve) => {
          let n = 0;
          const t0 = performance.now();
          const tick = () => (++n < 30 ? requestAnimationFrame(tick) : resolve((performance.now() - t0) / n));
          requestAnimationFrame(tick);
        });
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
        return { screen, orbit, animation, calls };
      });
      console.log(`screenFrameMs: ${frames.screen.toFixed(1)} (WebKit's frame interval with nothing drawn; not a budget)`);
      within("orbitFrameMs", Number(frames.orbit.toFixed(1)));
      within("animationFrameMs", Number(frames.animation.toFixed(1)));
      row.drawCalls = (await stats()).drawCalls;
      if (BUDGET.drawCalls) within("drawCalls", row.drawCalls);
      // Every number is in the table before any of them fails the test.
      expect(over).toEqual([]);
    });
  }
});
