// SPDX-License-Identifier: AGPL-3.0-or-later
// The frozen screens (docs/design/frozen, session 4A Done when 1): the app in each frozen state that's 4A's, not the
// chat's internals (4B) or the viewer's polish (4C), in both themes, each compared with its baseline in
// tests/e2e/baselines/<os>/screens/ (lib/app.mjs). First run, new-empty, untrusted, import, and the versions screens
// are taken in their own specs. The PR's side-by-sides put these beside the frozen screens.
import { $, expect } from "@wdio/globals";
import { copyFileSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { invoke, nextScene, open, screenshots, shown } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("screens");
const mod = process.platform === "darwin" ? "Meta" : "Control";
const shot = (id) => screenshots(id, { baseline: true });
const choose = async (group, choice) => (await (await $(`[aria-label='${group}']`)).$(`button=${choice}`)).click();
/** Waits for the build a change causes. */
async function settle(change) {
  const before = await shown();
  await change();
  await nextScene(before);
  await browser.pause(300);
}

describe("the frozen screens", () => {
  before(async () => {
    await open(dir);
  });

  it("built and clean, with the knobs", async () => {
    await shot("built");
  });

  it("a knob moved, and another arrangement", async () => {
    await settle(() => invoke("set_params", { values: { weathering: 20 } }));
    await expect($("#knobs-bar")).toBeDisplayed();
    await shot("sliders");
    await settle(() => invoke("set_params", { values: { weathering: 20, seed: 2 } }));
    await shot("shuffled");
    await settle(() => (async () => (await $("#knobs-reset")).click())());
  });

  it("build settings changed, waiting to rebuild; and a setting's tooltip", async () => {
    await (await $("#settings-change")).click();
    await choose("Style", "Medieval");
    await choose("Detail", "Medium");
    await shot("settings");
    await (await $("#settings-undo")).click();
    // Radix shows the meaning on keyboard focus; focus again until it does.
    await browser.waitUntil(
      () =>
        browser.execute(() => {
          const name = document.querySelector("#settings-list .setting-name");
          if (!document.querySelector("[role='tooltip']")) {
            name?.blur();
            name?.focus();
          }
          return !!document.querySelector("[role='tooltip']");
        }),
      { timeout: 10_000, interval: 700, timeoutMsg: "the setting's tooltip never showed" },
    );
    await shot("tooltip");
  });

  it("a sub-assembly selected in the Steps tree", async () => {
    await (await $("#tab-steps")).click();
    await (await $("#steps-tree [data-sub='window.ldr']")).click();
    await shot("structure");
  });

  it("the project menu, Export, a toast, About, and Settings", async () => {
    await (await $("#tab-options")).click();
    await (await $("#project-chip")).click();
    await browser.keys("Enter");
    await (await $("#project-menu")).waitForDisplayed();
    await shot("project-menu");
    await browser.keys("Escape");
    await browser.keys([mod, "e"]);
    await (await $("[data-menu='csv']")).waitForDisplayed();
    await shot("export");
    await (await $("[data-menu='csv']")).click();
    await (await $("#toast")).waitForDisplayed();
    await shot("export-done");
    await (await $("#toast [aria-label='Dismiss']")).click();
    await (await $("#toast")).waitForExist({ reverse: true });
    await (await $("#project-chip")).click();
    await browser.keys("Enter");
    await (await $("[data-menu='about']")).click();
    await (await $("#about")).waitForDisplayed();
    await expect($("#about-versions")).toHaveText(expect.stringContaining("Version 0.1.0"));
    await shot("about");
    await (await $("#about-licenses")).click();
    for (const credit of ["LDraw part authors", "Rebrickable", "LDCad Shadow Library", "at your own risk"]) await expect($("#licenses")).toHaveText(expect.stringContaining(credit));
    await browser.keys("Escape");
    await browser.keys([mod, ","]);
    await (await $("#settings")).waitForDisplayed();
    await expect($("#projects-folder")).toHaveText(expect.stringContaining("Documents/Emmet"));
    await shot("settings-dialog");
    await browser.keys("Escape");
  });

  it("offline: what still works", async () => {
    await browser.execute(() => window.__emmet.offline(true));
    try {
      await expect($("#banner-offline .banner-text")).toHaveText("You're offline.");
      await shot("offline");
    } finally {
      await browser.execute(() => window.__emmet.offline(null));
    }
  });

  it("an old library: Later, or Update the library", async () => {
    // The shared data dir's library, labeled an older release for this test, then put back as it was.
    const data = process.env.EMMET_DATA_DIR;
    const json = join(data, "library.json");
    copyFileSync(json, `${json}.bak`);
    try {
      const lib = JSON.parse(readFileSync(json, "utf8"));
      writeFileSync(json, JSON.stringify({ ...lib, release: "2026-06" }));
      await open(dir);
      await expect($("#banner-old-library")).toHaveText(expect.stringContaining("Your parts library is 2026-06. This Emmet expects 2026-10."));
      await shot("old-library");
      await (await $("#update-library")).click();
      await (await $("#banner-old-library")).waitForExist({ reverse: true, timeout: 60_000, timeoutMsg: "Update the library didn't finish" });
    } finally {
      copyFileSync(`${json}.bak`, json);
    }
  });

  it("a finding: its plain sentence, Ask agent to fix, It's fine", async () => {
    await open(projectDir("screens-floating"));
    await (await $("#tab-checks")).click();
    await (await $(".finding[data-finding]")).click();
    await shot("findings");
  });

  it("the build failed: the last good model still shown", async () => {
    await open(dir);
    const model = join(dir, "model.py");
    const good = readFileSync(model, "utf8");
    try {
      writeFileSync(model, good.replace("    m = Model(", "    m = Model(flip=True, "));
      await (await $("#build-failed")).waitForDisplayed({ timeout: 30_000 });
      await expect($("#model-state .panel-head-text")).toHaveText("Build failed");
      // No code, file name, or line number on screen (law 2).
      expect(await (await $("#build-failed")).getText()).not.toMatch(/model\.py|Traceback|flip/);
      await shot("failed");
    } finally {
      writeFileSync(model, good);
    }
  });
});
