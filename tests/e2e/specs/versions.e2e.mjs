// SPDX-License-Identifier: AGPL-3.0-or-later
// Versions (plan section 07, session 4A Done when 4): Save version (⌘S) names one, with its thumbnail; a change; the
// versions list opens it read-only to compare; Restore saves the current state first, then gives back the sources
// byte for byte; Open as new project copies it out and opens it.
import { $, $$, expect } from "@wdio/globals";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { invoke, nextScene, open, screenshots, shown, stats } from "../lib/app.mjs";
import { agentHome } from "../lib/agents.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("versions");
const sources = () => ["model.py", "params.json", "emmet.toml"].map((f) => readFileSync(join(dir, f)));
const mod = process.platform === "darwin" ? "Meta" : "Control";

/** The project chip's menu, by keyboard (Radix opens a menu on pointerdown, which the embedded driver doesn't send). */
async function menu(item) {
  await (await $("#project-chip")).click();
  await browser.keys("Enter");
  await (await $(`[data-menu='${item}']`)).waitForDisplayed();
  await (await $(`[data-menu='${item}']`)).click();
}

describe("versions", () => {
  let saved;

  it("saves a named version with its thumbnail", async () => {
    await open(dir);
    saved = sources();
    await browser.keys([mod, "s"]);
    await (await $("#version-name")).waitForDisplayed();
    await screenshots("version-save", { baseline: true });
    await (await $("#version-name")).setValue("Roof and chimney");
    await (await $("#version-save")).click();
    const row = await $("#versions-list [data-n='1']");
    await row.waitForDisplayed({ timeout: 10_000 });
    await expect(row).toHaveText(expect.stringContaining("Roof and chimney"));
    expect(await (await row.$("img")).getAttribute("src")).toMatch(/^data:image\/png;base64,/);
    await browser.keys("Escape");
  });

  it("opens the version to compare, and Restore gives back the same bytes after saving the state before", async () => {
    // A change to the model: the chimney comes off.
    const before = await shown();
    writeFileSync(join(dir, "params.json"), '{\n  "chimney": false\n}\n');
    await nextScene(before);
    const changed = (await stats()).instances;

    await menu("versions");
    await (await $("#versions-list [data-n='1']")).click();
    await expect($("#version-bar")).toHaveText(expect.stringContaining("Version 1: Roof and chimney"));
    await browser.waitUntil(async () => (await stats()).instances > changed, { timeout: 10_000, timeoutMsg: "the version's model never showed" });
    await screenshots("version-view", { baseline: true });

    // A build setting changed since: the restore puts the version's back, and the open project says so (the panel
    // would otherwise write the changed one over it).
    const any = { base: null, scale: null, age: null, detail: null, pieces: null, style: null };
    await invoke("set_build_settings", { settings: { ...any, style: "Medieval" } });
    expect((await invoke("ui_state")).project.build.style).toBe("Medieval");

    const restoring = await shown();
    await (await $("#version-restore")).click();
    await expect($("#toast")).toHaveText(expect.stringContaining("Restored version 1. The model before is version 2."));
    await nextScene(restoring);
    expect(sources()).toEqual(saved);
    expect((await invoke("ui_state")).project.build.style).toBe(null);
    await menu("versions");
    const auto = await $("#versions-list [data-n='2']");
    await expect(auto).toHaveText(expect.stringContaining("Before restoring version 1"));
    expect(await auto.getAttribute("data-auto")).toBe("true");
    await screenshots("versions", { baseline: true });
    await browser.keys("Escape");
  });

  it("opens a version as a new project", async () => {
    await menu("versions");
    await (await $("#versions-list [data-n='1']")).click();
    await (await $("#version-open")).click();
    const root = join(agentHome, "Documents", "Emmet", "Versions version 1");
    await browser.waitUntil(() => browser.execute((r) => window.__emmet.project() === r && window.__emmet.shown() >= 1, root), { timeout: 30_000, timeoutMsg: "the new project never showed" });
    await expect($("#project-chip")).toHaveText("Versions version 1");
    expect(readFileSync(join(root, "model.py"))).toEqual(saved[0]);
    expect((await $$("#banner-untrusted")).length).toBe(0);
  });
});
