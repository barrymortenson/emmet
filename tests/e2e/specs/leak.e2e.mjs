// SPDX-License-Identifier: AGPL-3.0-or-later
// The leak test (plan section 13, 1B Done when 4): 100 slider changes grow memory by at most 20 %, and the viewer's own
// resources stop growing once every width has been seen. The changes go through set_params, the command the slider
// calls. WebView2's baseline is the JS heap at first load. WKWebView's is the WebContent footprint after one warm-up
// cycle through every width, since its first-load footprint varies 125–400 MB between runs and the first cycle's
// one-time allocations (part geometry, shaders, JIT) aren't leaks (Barry, 2026-10-25).
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect } from "@wdio/globals";
import { invoke, nextScene, open, shown } from "../lib/app.mjs";
import { measure } from "../lib/memory.mjs";
import { os, out } from "../lib/paths.mjs";
import { projectDir } from "../lib/projects.mjs";

// three.js counts a geometry once it's been drawn, so the counts are read after the next frame.
const resources = async () => {
  const s = await browser.execute(async () => {
    const v = window.__emmet.viewer();
    await v.nextFrame();
    return v.stats();
  });
  return { geometries: s.geometries, textures: s.textures, programs: s.programs, buckets: s.buckets };
};

describe("memory over 100 slider changes", () => {
  it("grows at most 20 % over 100 changes", async () => {
    await open(projectDir("leak"));
    const change = async (i) => {
      // Every width from 6 to 16 in turn, with the chimney on and off: each change differs from the one before, so
      // each runs, and the chimney's bucket is made and dropped every time (the churn a leak would show in).
      const gen = await shown();
      await invoke("set_params", { values: { width: 6 + (i % 11), chimney: i % 2 === 0 } });
      await nextScene(gen);
    };
    const warmup = process.platform === "darwin" ? 11 : 0;
    for (let i = 0; i < warmup; i++) await change(i);
    await browser.pause(1000);
    const first = await measure();
    let settled, now;
    for (let i = warmup; i < warmup + 100; i++) {
      await change(i);
      // i = 21 and i = 87 are the same scene (width 16, no chimney), three cycles of 22 apart.
      if (i === 21) settled = await resources();
      if (i === 87) now = await resources();
    }
    await browser.pause(1000);
    const last = await measure();
    const growth = last.bytes / first.bytes - 1;
    const start = warmup ? `after ${warmup} warm-up changes` : "at first load";
    const line = `${last.what}: ${(first.bytes / 2 ** 20).toFixed(1)} MB ${start}, ${(last.bytes / 2 ** 20).toFixed(1)} MB after 100 more changes (${(growth * 100).toFixed(1)} %)`;
    console.log(line);
    mkdirSync(out, { recursive: true });
    writeFileSync(join(out, `leak-${os}.txt`), `${line}\n`);
    expect(growth).toBeLessThanOrEqual(0.2);
    // Two full cycles in, the viewer has every bucket it needs; the same scene later must not hold more.
    expect(now).toEqual(settled);
  });
});
