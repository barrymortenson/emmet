// SPDX-License-Identifier: AGPL-3.0-or-later
// The leak test (plan section 13, 1B Done when 4): 100 slider changes grow memory by at most 20 % over the first
// load, and the viewer's own resources stop growing once every width has been seen. The changes go through
// set_params, the command the slider calls.
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
  it("grows at most 20 % over the first load", async () => {
    await open(projectDir("leak"));
    await browser.pause(1000);
    const first = await measure();
    let settled, now;
    for (let i = 0; i < 100; i++) {
      // Every width from 6 to 16 in turn; each differs from the one before, so each change runs.
      const gen = await shown();
      await invoke("set_params", { values: { width: 6 + (i % 11) } });
      await nextScene(gen);
      // i = 21 and i = 98 are both width 16: the same scene, seven cycles apart.
      if (i === 21) settled = await resources();
      if (i === 98) now = await resources();
    }
    await browser.pause(1000);
    const last = await measure();
    const growth = last.bytes / first.bytes - 1;
    const line = `${last.what}: ${(first.bytes / 2 ** 20).toFixed(1)} MB at first load, ${(last.bytes / 2 ** 20).toFixed(1)} MB after 100 changes (${(growth * 100).toFixed(1)} %)`;
    console.log(line);
    mkdirSync(out, { recursive: true });
    writeFileSync(join(out, `leak-${os}.txt`), `${line}\n`);
    expect(growth).toBeLessThanOrEqual(0.2);
    // Two full cycles in, the viewer has every bucket it needs; the same scene later must not hold more.
    expect(now).toEqual(settled);
  });
});
