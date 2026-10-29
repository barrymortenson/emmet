// SPDX-License-Identifier: AGPL-3.0-or-later
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { open } from "../lib/app.mjs";
import { out } from "../lib/paths.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = join(out, "look");
const shot = async (name) => {
  await browser.pause(400);
  await browser.saveScreenshot(join(dir, `${name}.png`));
};

describe("look", () => {
  it("shoots", async () => {
    mkdirSync(dir, { recursive: true });
    await shot("0-launch");
    await open(projectDir("look"));
    await browser.pause(1500);
    await shot("1-cottage");
    for (const t of ["checks", "parts", "steps"]) {
      await (await $(`#tab-${t}`)).click();
      await browser.pause(1500);
      await shot(`2-${t}`);
    }
    await browser.execute(() => window.__emmet.theme("dark"));
    await shot("3-steps-dark");
    await (await $("#tab-options")).click();
    await shot("4-options-dark");
    await (await $("#project-chip")).click();
    await shot("5-menu");
    await browser.keys("Escape");
    console.log("errors", JSON.stringify(await browser.execute(() => window.__emmetErrors)));
  });
});
