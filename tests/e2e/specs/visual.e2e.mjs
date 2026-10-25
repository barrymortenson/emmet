// SPDX-License-Identifier: AGPL-3.0-or-later
// Visual baselines (plan section 16, 1B Done when 4): three fixture scenes rendered through viewer.render, compared
// with pixelmatch against tests/e2e/baselines/<os>/. A missing baseline is written and the test fails until it's
// committed; EMMET_UPDATE_BASELINES=1 rewrites them all. On a mismatch the render and a diff image go to
// tests/e2e/out/visual/, which CI uploads.
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect } from "@wdio/globals";
import pixelmatch from "pixelmatch";
import { open, render } from "../lib/app.mjs";
import { os, out, repo } from "../lib/paths.mjs";
import { decode, encode } from "../lib/png.mjs";
import { projectDir } from "../lib/projects.mjs";

const SIZE = 512;
/** At most this share of pixels may differ beyond pixelmatch's per-pixel threshold. CI's runs differ by 0 pixels on
 * each OS; 0.1 % (262 pixels) leaves room for driver noise while one missing brick (about 1,000) fails, which the
 * last test proves. */
const TOLERANCE = 0.001;
const baselines = join(repo, "tests", "e2e", "baselines", os);
const update = process.env.EMMET_UPDATE_BASELINES === "1";

const SCENES = [
  { name: "cottage", dir: projectDir("visual"), view: "iso" },
  { name: "colors", dir: projectDir("visual-scene-colors"), view: "iso" },
  { name: "grid", dir: projectDir("visual-scene-grid"), view: "front" },
];

describe("visual baselines", () => {
  for (const scene of SCENES) {
    it(`${scene.name} matches its baseline`, async () => {
      await open(scene.dir);
      const png = await render({ view: scene.view, size: SIZE });
      const baseline = join(baselines, `${scene.name}.png`);
      if (update || !existsSync(baseline)) {
        mkdirSync(baselines, { recursive: true });
        writeFileSync(baseline, png);
        // A copy in the CI artifact, for a baseline made on a runner.
        mkdirSync(join(out, "visual", os, "new"), { recursive: true });
        writeFileSync(join(out, "visual", os, "new", `${scene.name}.png`), png);
        if (!update) throw new Error(`wrote a new baseline, ${baseline}; check it and commit it`);
        return;
      }
      const actual = decode(png);
      const expected = decode(readFileSync(baseline));
      const diff = { width: SIZE, height: SIZE, data: Buffer.alloc(SIZE * SIZE * 4) };
      const bad = pixelmatch(expected.data, actual.data, diff.data, SIZE, SIZE, { threshold: 0.1 });
      const share = bad / (SIZE * SIZE);
      console.log(`${scene.name}: ${bad} pixels differ (${(share * 100).toFixed(2)} %)`);
      if (share > TOLERANCE) {
        const dir = join(out, "visual", os);
        mkdirSync(dir, { recursive: true });
        writeFileSync(join(dir, `${scene.name}-actual.png`), png);
        writeFileSync(join(dir, `${scene.name}-diff.png`), encode(diff));
      }
      expect(share).toBeLessThanOrEqual(TOLERANCE);
    });
  }

  it("the comparison catches one brick missing", async () => {
    const png = await browser.execute(async () => {
      const v = window.__emmet.viewer();
      const scene = await window.__TAURI_INTERNALS__.invoke("ui_state");
      // A brick in the front row of the top layer, which the front view sees whole.
      const list = scene.scene.placements.filter((p) => p.id !== "b405");
      await v.update(list);
      const { png } = await v.render({ view: "front", size: 512 });
      await v.update(scene.scene.placements);
      let s = "";
      for (let i = 0; i < png.length; i += 0x8000) s += String.fromCharCode(...png.subarray(i, i + 0x8000));
      return btoa(s);
    });
    const actual = decode(Buffer.from(png, "base64"));
    const expected = decode(readFileSync(join(baselines, "grid.png")));
    const bad = pixelmatch(expected.data, actual.data, null, SIZE, SIZE, { threshold: 0.1 });
    expect(bad / (SIZE * SIZE)).toBeGreaterThan(TOLERANCE);
  });
});
