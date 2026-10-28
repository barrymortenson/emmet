// SPDX-License-Identifier: AGPL-3.0-or-later
// The Checks and Parts drawers on the real app (session 1D): a project with a collision lists the finding; a click
// highlights exactly its parts as an issue and shows the model.py lines that placed them (design law 2); "Ask agent to
// fix" with no chat says to start one; fixing the model clears the finding. The flow logs no console errors.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { $, $$, expect } from "@wdio/globals";
import { nextScene, open, screenshots, shown, stats } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const project = projectDir("checks");
const modelPy = join(project, "model.py");
const highlighted = () => browser.execute(() => window.__emmet.viewer().highlighted().sort());
const errors = () => browser.execute(() => window.__emmetErrors.slice());

describe("checks", () => {
  it("lists the collision the validator found", async () => {
    await open(project);
    await expect($("#tab-checks")).toHaveText("Checks · 1 error");
    await $("#tab-checks").click();
    const finding = await $('[data-finding="F-0001"]');
    await expect(finding).toHaveText(expect.stringContaining("wall/a (Brick 2 x 4) and wall/b (Brick 2 x 4) overlap by 1 stud."));
    await expect(finding).toHaveText(expect.stringContaining("model.py:10"));
  });

  it("clicking the finding highlights its parts and shows the lines that placed them", async () => {
    await $('[data-finding="F-0001"]').click();
    await browser.waitUntil(async () => (await stats()).highlighted === 2, { timeoutMsg: "the parts weren't highlighted" });
    expect(await highlighted()).toEqual(["wall/a", "wall/b"]);
    const detail = await $("#finding-detail");
    await expect(detail).toHaveText(expect.stringContaining("model.py:11"));
    await expect(detail).toHaveText(expect.stringContaining('11 |     m.add("3001", "Blue", at=(1, 0, 1), id="wall/b")'));
    await expect(detail).toHaveText(expect.stringContaining("Move wall/b by 1 stud"));
    await screenshots("checks-finding");
    // A second click clears it.
    await $('[data-finding="F-0001"]').click();
    await browser.waitUntil(async () => (await stats()).highlighted === 0, { timeoutMsg: "the highlight stayed" });
    await $('[data-finding="F-0001"]').click();
  });

  it("asking the agent to fix it with no chat open says to start one", async () => {
    await $("#ask-fix").click();
    await expect($("#chat-error")).toHaveText(expect.stringContaining("Start a chat with an agent first"));
  });

  it("the Parts drawer lists part, color, quantity, and availability", async () => {
    await $("#tab-parts").click();
    await expect($("#parts-total")).toHaveText(expect.stringContaining("3 pieces"));
    const rows = await $$(".bom tbody tr");
    expect(rows.length).toBe(3);
    // No parts data pack in the E2E data dir: availability is Unknown, never "Not made" (3B), and with no Rebrickable
    // data shown there's no credit line. tests/it/tests/parts_data.rs covers a pack.
    await expect($(".bom thead")).toHaveText(expect.stringContaining("Availability"));
    for (const row of rows) await expect(row.$("td:last-child")).toHaveText("Unknown");
    expect(await (await $("#parts-credit")).isExisting()).toBe(false);
    await screenshots("parts-list");
    await $("#tab-checks").click();
  });

  it("Settings → Data says what the parts data is and credits Rebrickable", async () => {
    await $("#settings-button").click();
    await expect($("#parts-data-status")).toHaveText("No parts data yet: availability shows as Unknown.");
    await expect($("dialog.settings[open]")).toHaveText(
      expect.stringContaining("Part, color, and set data from Rebrickable (rebrickable.com)"),
    );
    await expect($("#color-names")).toBeEnabled();
    await screenshots("settings-data");
    await browser.keys("Escape");
    await browser.waitUntil(async () => !(await (await $("dialog.settings[open]")).isExisting()));
  });

  it("fixing the model clears the finding and its highlight", async () => {
    const gen = await shown();
    writeFileSync(modelPy, readFileSync(modelPy, "utf8").replace("at=(1, 0, 1)", "at=(2, 0, 1)"));
    await nextScene(gen);
    await expect($("#checks-clean")).toBeDisplayed();
    await expect($("#tab-checks")).toHaveText("Checks · clean");
    await browser.waitUntil(async () => (await stats()).highlighted === 0, { timeoutMsg: "the old finding's highlight stayed" });
    // An edit's run builds up part by part (D-34), and lands.
    await browser.waitUntil(async () => (await stats()).animating === 0, { timeout: 10_000, timeoutMsg: "the build animation never ended" });
  });

  it("logged no console errors", async () => {
    const all = await errors();
    for (const e of all) console.log(`console error:\n${e}`);
    // On Windows the embedded WebDriver plugin evaluates its own scripts in the page and logs "JSON error: invalid
    // type: null, expected u32" from them (CI, 2026-10-26): their caller is an <anonymous> script, never the app's
    // bundle. Every other error is the app's and fails the test.
    const fromDriver = (e) => /\n\s*at console\.error \([^)]*\)\n\s*at <anonymous>:\d+:\d+\s*$/.test(e);
    expect(all.filter((e) => !fromDriver(e))).toEqual([]);
  });
});
