// SPDX-License-Identifier: AGPL-3.0-or-later
// The viewer bench (plan section 13's budgets), run by `just bench`, not by `just e2e`. For the town block
// (tests/fixtures/projects/scene-stress) at rung 1's size (one building, 184 parts), rung 2's (478), and 2,000+ parts
// (2,018), the Cottage (905), and a 2,000-brick grid (one part, many instances): first load into a viewer with an
// empty part cache (the host's library cache is cold for each scene's parts the first time), slider → updated viewer
// (set_params, the watcher, the run, the diff), orbit frame time, and the frame time while the whole model builds up
// in the build animation. Writes tests/e2e/out/bench-<os>.{json,md} and the CI job summary.
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { nextScene, open, shown, stats } from "../lib/app.mjs";
import { os, out } from "../lib/paths.mjs";
import { emmet, projectDir } from "../lib/projects.mjs";

const SCENES = [
  { name: "rung 1 (town, 1 building)", dir: projectDir("bench-rung1"), change: (i) => ({ buildings: 1, floors: 1 + (i % 2) }) },
  { name: "rung 2 (town, 1 floor)", dir: projectDir("bench-rung2"), change: (i) => ({ buildings: 4, floors: 1 + (i % 2) }) },
  { name: "cottage", dir: projectDir("bench"), change: (i) => ({ width: 16 + 2 * (i % 2) }) },
  { name: "2,000 (town block)", dir: projectDir("bench-stress"), change: (i) => ({ floors: 6 - (i % 2) }) },
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
      // The whole model added at once, as an agent's first build is, until the last part lands (at most 6 s).
      const animation = await browser.execute(async () => {
        const v = window.__emmet.viewer();
        const { scene } = await window.__TAURI_INTERNALS__.invoke("ui_state");
        await v.update([]);
        await v.update(scene.placements, { animate: "build" });
        const t0 = performance.now();
        let n = 0;
        while (v.stats().animating > 0 && performance.now() - t0 < 10_000) {
          await Promise.race([v.nextFrame(), new Promise((r) => setTimeout(r, 1000))]);
          n++;
        }
        return { ms: n ? (performance.now() - t0) / n : null, frames: n };
      });
      // What a frame costs to draw, which the display's refresh interval hides from the orbit frame time on a fast GPU.
      const drawMs = await browser.execute(() => window.__emmet.viewer().drawCost(60));
      const s = await stats();
      rows.push({
        scene: scene.name,
        parts: s.instances,
        firstLoadMs: Math.round(load.ms),
        updateMs: Math.round(median(updates)),
        orbitFrameMs: orbit.ms == null ? null : Number(orbit.ms.toFixed(1)),
        orbitFrames: orbit.frames,
        drawMs: Number(drawMs.toFixed(1)),
        animationFrameMs: animation.ms == null ? null : Number(animation.ms.toFixed(1)),
        animationFrames: animation.frames,
        drawCalls: s.drawCalls,
        triangles: s.triangles,
      });
    }
    const table = [
      `Viewer bench (${os}, ${process.env.RUNNER_ENVIRONMENT ?? "local"})`,
      "",
      "| Scene | Parts | First load | Slider → viewer (median of 6) | Orbit frame | Draw cost | Build animation frame | Draw calls | Triangles |",
      "|---|---|---|---|---|---|---|---|---|",
      ...rows.map((r) => `| ${r.scene} | ${r.parts} | ${r.firstLoadMs} ms | ${r.updateMs} ms | ${r.orbitFrameMs} ms${r.orbitFrames < ORBIT_FRAMES ? ` (${r.orbitFrames} frames)` : ""} | ${r.drawMs} ms | ${r.animationFrameMs} ms (${r.animationFrames} frames) | ${r.drawCalls} | ${r.triangles.toLocaleString("en-US")} |`),
    ].join("\n");
    console.log(`\n${table}\n`);
    mkdirSync(out, { recursive: true });
    writeFileSync(join(out, `bench-${os}.json`), JSON.stringify(rows, null, 2));
    writeFileSync(join(out, `bench-${os}.md`), `${table}\n`);
    if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, `\n${table}\n`);
  });
});
