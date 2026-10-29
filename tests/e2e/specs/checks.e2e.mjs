// SPDX-License-Identifier: AGPL-3.0-or-later
// The Checks and Parts tabs on the real app (session 1D, to the frozen design in 4A): a project with a collision lists
// the finding in plain words, with no ids, file names, or lines on screen (law 2: they go to the agent); a click
// highlights exactly its parts as an issue; "Ask agent to fix" with no chat says to start one; fixing the model clears
// the finding. The flow logs no console errors.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { $, $$, expect } from "@wdio/globals";
import { nextScene, open, shown, stats } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const project = projectDir("checks");
const modelPy = join(project, "model.py");
const highlighted = () => browser.execute(() => window.__emmet.viewer().highlighted().sort());
const errors = () => browser.execute(() => window.__emmetErrors.slice());

describe("checks", () => {
  it("lists the collision the validator found, in plain words", async () => {
    await open(project);
    await expect($("#tab-checks")).toHaveText("Checks1");
    await $("#tab-checks").click();
    const finding = await $('[data-finding="F-0001"]');
    await expect(finding).toHaveText(expect.stringContaining("2 parts overlap"));
    await expect(finding).toHaveText(expect.stringContaining("Collision"));
    await expect(finding).toHaveText(expect.stringContaining("Brick 2 x 4 and Brick 2 x 4 overlap by 1 stud."));
    expect(await finding.getText()).not.toMatch(/model\.py|wall\/a|F-0001/);
  });

  it("clicking the finding highlights its parts, and again clears them", async () => {
    await $('[data-finding="F-0001"]').click();
    await browser.waitUntil(async () => (await stats()).highlighted === 2, { timeoutMsg: "the parts weren't highlighted" });
    expect(await highlighted()).toEqual(["wall/a", "wall/b"]);
    await expect($('[data-finding="F-0001"]')).toHaveAttribute("aria-selected", "true");
    await $('[data-finding="F-0001"]').click();
    await browser.waitUntil(async () => (await stats()).highlighted === 0, { timeoutMsg: "the highlight stayed" });
  });

  it("asking the agent to fix it with no chat open says to start one", async () => {
    await $("#fix-F-0001").click();
    await expect($("#chat-error")).toHaveText(expect.stringContaining("Start a chat with an agent first"));
  });

  it("the Parts tab lists each part and color with its count", async () => {
    await $("#tab-parts").click();
    await expect($("#parts-total")).toHaveText(expect.stringMatching(/^3 parts · \d+ kinds$/));
    const rows = await $$("#parts-list tbody tr");
    expect(rows.length).toBe(3);
    // No parts data pack in the E2E data dir: availability is unknown, never "Not made" (3B), and with no Rebrickable
    // data shown there's no credit line. tests/it/tests/parts_data.rs covers a pack.
    for (const row of rows) expect(await row.getText()).not.toContain("Not made");
    expect(await (await $("#parts-credit")).isExisting()).toBe(false);
    await $("#tab-checks").click();
  });

  it("Settings says what the parts data is, and changes the color names", async () => {
    await browser.keys([process.platform === "darwin" ? "Meta" : "Control", ","]);
    await expect($("#parts-data-status")).toHaveText("None yet: availability shows as unknown");
    await expect($("#color-names")).toBeDisplayed();
    await browser.keys("Escape");
    await (await $("#settings")).waitForExist({ reverse: true });
  });

  it("fixing the model clears the finding and its highlight", async () => {
    const gen = await shown();
    writeFileSync(modelPy, readFileSync(modelPy, "utf8").replace("at=(1, 0, 1)", "at=(2, 0, 1)"));
    await nextScene(gen);
    await expect($("#checks-clean")).toBeDisplayed();
    await expect($("#tab-checks")).toHaveText("Checks");
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
