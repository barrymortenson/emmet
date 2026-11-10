// SPDX-License-Identifier: AGPL-3.0-or-later
// First run (plan section 14's key flow, session 4A Done when 2), on a fresh data dir (wdio.conf.mjs's first-run mode)
// against lib/library-server.mjs: welcome → the library download, which drops partway and picks up where it stopped
// on Try again, then drops again after the user has gone on to Start, which says so with its own Try again → Python →
// the agents (the fake CLIs, ready) → Start → the Cottage, built from Documents/Emmet with no dialog. Then ⌘N makes a
// new model from a prompt: the New model dialog, on one page, asks its name and build settings (Barry 2026-11-09), and
// it starts on the base it names (None: no parts, on the viewer's ground grid; Barry 2026-11-10) with those settings
// in Options' line of words. Each screen in both themes goes to tests/e2e/out/screens/.
import { $, expect } from "@wdio/globals";
import { copyFileSync, existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { screenshots } from "../lib/app.mjs";
import { agentHome } from "../lib/agents.mjs";
import { dropNext, releaseDrop, served } from "../lib/library-server.mjs";

const text = async (sel) => (await $(sel)).getText();
/** By hand only: the real library and Python, timed (wdio.conf.mjs). */
const real = process.env.EMMET_E2E_REAL === "1";
let started = 0;
const mod = process.platform === "darwin" ? "Meta" : "Control";

describe("first run", () => {
  it("welcomes, with no project and nothing set up", async () => {
    await (await $("#first-run")).waitForDisplayed({ timeout: 30_000 });
    await expect($("#first-run h1")).toHaveText("Build brick models by describing them");
    expect(await browser.execute(() => window.__emmet.project())).toBe(null);
    await screenshots("fr-welcome", { baseline: !real });
  });

  it("says where a dropped download stopped, and Try again picks up there", async function () {
    if (real) {
      started = Date.now();
      await (await $("#get-started")).click();
      await browser.waitUntil(async () => (await text("#python-progress")).includes("Done"), { timeout: 600_000, interval: 1000, timeoutMsg: "the real setup never finished" });
      console.log(`first run, real: the parts and Python ready ${Math.round((Date.now() - started) / 1000)} s after Get started`);
      return;
    }
    // A quarter of the zip, then the connection drops.
    await dropNext(250_000);
    await (await $("#get-started")).click();
    await (await $("#setup-failed")).waitForDisplayed({ timeout: 60_000 });
    expect(await text("#setup-failed")).toContain("The download stopped");
    expect(await text("#setup-failed")).toContain("Emmet picks up where it stopped.");
    expect(await text("#library-progress")).toContain("Stopped at");
    await screenshots("fr-library-failed", { baseline: !real });

    // Try again; the download stops again once the user has gone on to Start, which says so, with Try again there.
    await dropNext(500_000, { hold: true });
    await (await $("#setup-retry")).click();
    await (await $("#library-continue")).click();
    await (await $("#agents-continue")).click();
    await expect($("#first-run h1")).toHaveText("What do you want to build?");
    await expect($("#start-cottage")).toBeDisabled();
    await releaseDrop();
    await (await $("#setup-failed")).waitForDisplayed({ timeout: 60_000, timeoutMsg: "Start never said the download stopped" });
    expect(await text("#setup-failed")).toContain("The download stopped");
    await expect($("#start-cottage")).toBeDisabled();
    await (await $("#setup-retry")).click();
    await browser.waitUntil(() => $("#start-cottage").isEnabled(), { timeout: 120_000, timeoutMsg: "the parts and Python never finished" });
    // Each request asked for the rest, from where the one before stopped.
    const starts = (await served()).map((s) => s.start);
    expect(starts[0]).toBe(0);
    expect(starts[1]).toBeGreaterThan(0);
    expect(starts[2]).toBeGreaterThan(starts[1]);
    // Back to the library step (the rail has passed it), both done.
    await browser.execute(() => window.__emmet.firstRun("library"));
    expect(await text("#library-progress")).toContain("Done");
    expect(await text("#python-progress")).toContain("Done");
    await screenshots("fr-library", { baseline: !real });
  });

  it("sets the agents up without a terminal, then offers the Cottage or a prompt", async () => {
    await (await $("#library-continue")).click();
    await (await $("#agents")).waitForDisplayed();
    await expect($("#ready-claude")).toHaveText("Ready");
    await expect($("#agents-continue")).toBeDisplayed();
    await screenshots("fr-agents", { baseline: !real });
    await (await $("#agents-continue")).click();
    await expect($("#first-run h1")).toHaveText("What do you want to build?");
    expect(await text("#first-run .lede")).toContain(process.platform === "win32" ? "Documents\\Emmet" : "Documents/Emmet");
    await screenshots("fr-start", { baseline: !real });
  });

  it("builds the Cottage in Documents/Emmet with no dialog", async () => {
    const card = await text("#start-cottage");
    await (await $("#start-cottage")).click();
    await browser.waitUntil(() => browser.execute(() => window.__emmet.project() != null && window.__emmet.shown() >= 1), { timeout: 60_000, timeoutMsg: "the Cottage never showed" });
    const root = await browser.execute(() => window.__emmet.project());
    expect(root).toBe(join(agentHome, "Documents", "Emmet", "Cottage"));
    expect(readFileSync(join(root, "emmet.toml"), "utf8")).toContain('name = "Cottage"');
    await expect($("#first-run")).not.toBeExisting();
    await browser.waitUntil(async () => (await text("#model-state .panel-head-text")) === "No problems", { timeout: 60_000 });
    // The card's count is the Cottage's own (content.md: numbers over impressions).
    const parts = (await browser.execute(() => window.__TAURI_INTERNALS__.invoke("ui_state"))).scene.placements.length;
    expect(card).toContain(`${parts} parts`);
    if (real) console.log(`first run, real: the Cottage built ${Math.round((Date.now() - started) / 1000)} s after Get started`);
  });

  it("makes a new model from a prompt with ⌘N: one dialog asks its name and build settings first", async () => {
    // ⌘N opens the New model dialog itself, over the Cottage: not the start screen again.
    await browser.keys([mod, "n"]);
    await (await $("#new-model")).waitForDisplayed();
    await expect($("#first-run")).not.toBeExisting();
    // No name, no Create: New used to make "New model" with no dialog, and the user couldn't name it.
    await expect($("#new-model-create")).toBeDisabled();
    await expect($("#new-model-name")).toBeFocused();
    await (await $("#new-model-name")).setValue("Fire station");
    const choose = async (group, choice) => (await (await $(`#new-model [aria-label='${group}']`)).$(`button=${choice}`)).click();
    // Size, not Scale and Pieces: one setting for how big it is, and no part count (Barry 2026-11-10).
    await expect($("#new-model [aria-label='Pieces']")).not.toBeExisting();
    await choose("Size", "Midi");
    // Enter in the style field takes the style; it doesn't create the model.
    await (await $("#new-model [aria-label='Your own style']")).setValue("Steampunk");
    await browser.keys("Enter");
    await expect($("#new-model [aria-label='Style'] [data-state='on']")).toHaveText(expect.stringContaining("Steampunk"));
    await expect($("#new-model")).toBeDisplayed();
    // Age leads as in Options: 9+ brings Medium detail. Every question is on the one page.
    await choose("Age", "9+");
    await expect($("#new-model [aria-label='Detail'] [data-state='on']")).toHaveText("Medium");
    // No base: it starts with no parts, where it used to start on a 32 x 32 baseplate whatever Base said.
    await choose("Base", "None");
    await expect($("#new-model-name")).toBeDisplayedInViewport();
    await expect($("#new-model-create")).toBeDisplayedInViewport();
    await screenshots("new-model", { baseline: !real });
    await (await $("#new-model-create")).click();
    await browser.waitUntil(() => browser.execute(() => window.__emmet.project()?.endsWith("Fire station")), { timeout: 60_000 });
    await expect($("#new-model")).not.toBeExisting();
    const dir = join(agentHome, "Documents", "Emmet", "Fire station");
    expect(existsSync(join(dir, "model.py"))).toBe(true);
    const toml = readFileSync(join(dir, "emmet.toml"), "utf8");
    expect(toml).toContain('name = "Fire station"');
    for (const line of ['base = "None"', 'size = "Midi"', 'age = "9+"', 'detail = "Medium"', 'style = "Steampunk"']) expect(toml).toContain(line);
    expect(toml).not.toContain("pieces");
    expect(readFileSync(join(dir, "model.py"), "utf8")).not.toContain("3811");
    await browser.waitUntil(async () => (await text("#model-state .panel-head-text")) === "Empty", { timeout: 60_000 });
    // Options shows them as a line of words with Change, not the whole list.
    await expect($("#settings-summary")).toHaveText(expect.stringContaining("No base · Midi · Ages 9+ · Medium detail · Steampunk"));
    // Nothing in the viewer but the ground grid.
    const stats = await browser.execute(() => window.__emmet.viewer().stats());
    expect([stats.instances, stats.grid]).toEqual([0, true]);
    await expect($("#settings-list")).not.toBeExisting();
    await expect($("#export-button")).not.toBeExisting();
    await screenshots("new-empty", { baseline: !real });
  });

  it("⌘E with nothing to export yet doesn't open Export later, when the model has parts", async () => {
    await browser.keys([mod, "e"]);
    await expect($("#export-button")).not.toBeExisting();
    const dir = join(agentHome, "Documents", "Emmet");
    copyFileSync(join(dir, "Cottage", "model.py"), join(dir, "Fire station", "model.py"));
    await (await $("#export-button")).waitForDisplayed({ timeout: 60_000, timeoutMsg: "the model never built" });
    await browser.pause(500);
    await expect($("[data-menu='csv']")).not.toBeExisting();
  });
});
