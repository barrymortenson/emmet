// SPDX-License-Identifier: AGPL-3.0-or-later
// The viewer bench (plan section 13's budgets), run by `just bench`, not by `just e2e`. For the cottage (903 parts)
// and the grid at 500 and 2,000 parts: first load into a viewer with an empty part cache (the host's library cache
// is cold for each scene's parts the first time), slider → updated viewer (set_params, the watcher, the run, the
// diff), and orbit frame time. Writes tests/e2e/out/bench-<os>.{json,md} and the CI job summary.
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { nextScene, open, shown, stats } from "../lib/app.mjs";
import { os, out } from "../lib/paths.mjs";
import { emmet, projectDir } from "../lib/projects.mjs";

const SCENES = [
  { name: "cottage", dir: projectDir("bench"), change: (i) => ({ width: 16 + 2 * (i % 2) }) },
  { name: "grid 500", dir: projectDir("bench-grid500"), change: (i) => ({ count: 500 + (i % 2) }) },
  { name: "grid 2000", dir: projectDir("bench-grid2000"), change: (i) => ({ count: 2000 + (i % 2) }) },
];
const UPDATES = 6;
const ORBIT_FRAMES = 120;

const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];

describe("viewer bench", () => {
  const rows = [];

  it("measures load, update, and orbit", async () => {
    for (const scene of SCENES) {
      emmet(["--project", scene.dir, "run"]);
      const placements = JSON.parse(readFileSync(join(scene.dir, "build", "placements.json"), "utf8")).parts;
      const load = await browser.execute(async (list) => {
        const v = window.__emmet.newViewer();
        const t0 = performance.now();
        await v.load(list);
        await v.nextFrame();
        const ms = performance.now() - t0;
        const s = v.stats();
        v.dispose();
        return { ms, failed: Object.keys(s.failed).length };
      }, placements);
      if (load.failed) throw new Error(`${scene.name}: ${load.failed} parts failed to load`);

      await open(scene.dir);
      const updates = [];
      for (let i = 1; i <= UPDATES; i++) {
        const gen = await shown();
        const t0 = await browser.execute(async (values) => {
          const t = performance.now();
          await window.__TAURI_INTERNALS__.invoke("set_params", { values });
          return t;
        }, scene.change(i));
        await nextScene(gen);
        await browser.waitUntil(async () => (await browser.execute(() => window.__emmet.drawnAt())) > t0);
        updates.push((await browser.execute(() => window.__emmet.drawnAt())) - t0);
      }

      // At most ORBIT_FRAMES frames or 10 s: WebView2 on CI's software renderer draws a 2,000-part frame slowly, and a
      // WebDriver script has 30 s. A frame that doesn't come within 1 s (a window behind another, which pauses
      // requestAnimationFrame) ends the measurement instead of the script.
      const orbit = await browser.execute(async (frames) => {
        const v = window.__emmet.viewer();
        const frame = () => Promise.race([v.nextFrame().then(() => true), new Promise((r) => setTimeout(() => r(false), 1000))]);
        await frame();
        const t0 = performance.now();
        let n = 0;
        while (n < frames && performance.now() - t0 < 10_000) {
          v.orbit(0.03);
          if (!(await frame())) break;
          n++;
        }
        return { ms: n ? (performance.now() - t0) / n : null, frames: n };
      }, ORBIT_FRAMES);
      const s = await stats();
      rows.push({
        scene: scene.name,
        parts: s.instances,
        firstLoadMs: Math.round(load.ms),
        updateMs: Math.round(median(updates)),
        orbitFrameMs: orbit.ms == null ? null : Number(orbit.ms.toFixed(1)),
        orbitFrames: orbit.frames,
        drawCalls: s.drawCalls,
        triangles: s.triangles,
      });
    }
    const table = [
      `Viewer bench (${os}, ${process.env.RUNNER_ENVIRONMENT ?? "local"})`,
      "",
      "| Scene | Parts | First load | Slider → viewer (median of 6) | Orbit frame | Draw calls | Triangles |",
      "|---|---|---|---|---|---|---|",
      ...rows.map((r) => `| ${r.scene} | ${r.parts} | ${r.firstLoadMs} ms | ${r.updateMs} ms | ${r.orbitFrameMs} ms${r.orbitFrames < ORBIT_FRAMES ? ` (${r.orbitFrames} frames)` : ""} | ${r.drawCalls} | ${r.triangles} |`),
    ].join("\n");
    console.log(`\n${table}\n`);
    mkdirSync(out, { recursive: true });
    writeFileSync(join(out, `bench-${os}.json`), JSON.stringify(rows, null, 2));
    writeFileSync(join(out, `bench-${os}.md`), `${table}\n`);
    if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, `\n${table}\n`);
  });
});
