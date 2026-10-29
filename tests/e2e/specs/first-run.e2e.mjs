// SPDX-License-Identifier: AGPL-3.0-or-later
// First run (plan section 14's key flow, session 4A Done when 2), on a fresh data dir (wdio.conf.mjs's first-run mode)
// against lib/library-server.mjs: welcome → the library download, which drops partway and picks up where it stopped
// on Try again → Python → the agents (the fake CLIs, ready) → Start → the Cottage, built from Documents/Emmet with no
// dialog. Then ⌘N makes a new model from a prompt: an empty baseplate. Each screen in both themes goes to
// tests/e2e/out/screens/.
import { $, expect } from "@wdio/globals";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { screenshots } from "../lib/app.mjs";
import { agentHome } from "../lib/agents.mjs";
import { dropNext, served } from "../lib/library-server.mjs";

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
    // A third of the zip, then the connection drops.
    await dropNext(250_000);
    await (await $("#get-started")).click();
    await (await $("#setup-failed")).waitForDisplayed({ timeout: 60_000 });
    expect(await text("#setup-failed")).toContain("The download stopped");
    expect(await text("#setup-failed")).toContain("Emmet picks up where it stopped.");
    expect(await text("#library-progress")).toContain("Stopped at");
    await screenshots("fr-library-failed", { baseline: !real });

    await (await $("#setup-retry")).click();
    await browser.waitUntil(async () => (await text("#python-progress")).includes("Done"), { timeout: 120_000, timeoutMsg: "Python never finished" });
    expect(await text("#library-progress")).toContain("Done");
    // The second request asked for the rest, from where the first stopped.
    const starts = (await served()).map((s) => s.start);
    expect(starts[0]).toBe(0);
    expect(starts[1]).toBeGreaterThan(0);
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
    await (await $("#start-cottage")).click();
    await browser.waitUntil(() => browser.execute(() => window.__emmet.project() != null && window.__emmet.shown() >= 1), { timeout: 60_000, timeoutMsg: "the Cottage never showed" });
    const root = await browser.execute(() => window.__emmet.project());
    expect(root).toBe(join(agentHome, "Documents", "Emmet", "Cottage"));
    expect(readFileSync(join(root, "emmet.toml"), "utf8")).toContain('name = "Cottage"');
    await expect($("#first-run")).not.toBeExisting();
    await browser.waitUntil(async () => (await text("#model-state .panel-head-text")) === "No problems", { timeout: 60_000 });
    if (real) console.log(`first run, real: the Cottage built ${Math.round((Date.now() - started) / 1000)} s after Get started`);
  });

  it("makes a new model from a prompt with ⌘N: an empty baseplate, every build setting on Any", async () => {
    await browser.keys([mod, "n"]);
    await (await $("#start-prompt")).waitForClickable();
    await (await $("#start-prompt")).click();
    await browser.waitUntil(() => browser.execute(() => window.__emmet.project()?.endsWith("New model")), { timeout: 60_000 });
    expect(existsSync(join(agentHome, "Documents", "Emmet", "New model", "model.py"))).toBe(true);
    await browser.waitUntil(async () => (await text("#model-state .panel-head-text")) === "Empty", { timeout: 60_000 });
    await expect($("#settings-list")).toBeDisplayed();
    await expect($("#export-button")).not.toBeExisting();
    await screenshots("new-empty", { baseline: !real });
  });
});
