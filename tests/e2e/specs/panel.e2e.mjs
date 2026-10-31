// SPDX-License-Identifier: AGPL-3.0-or-later
// The options panel's tabs (plan section 14, session 4A Done when 3): Options' build settings (a change waits with
// Undo and Rebuild, and Age sets Detail and Pieces) and knobs (a draft until Keep writes model.py, or Reset drops it);
// Checks (a finding in plain words, It's fine allows it in emmet.toml, Check it again brings it back); Parts (a row per
// part and color, with its picture); Steps (a step shows the build up to it; a sub-assembly highlights every copy);
// and Export (a toast says what was written where).
import { $, $$, expect } from "@wdio/globals";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { nextScene, open, shown, stats } from "../lib/app.mjs";
import { agentHome, scenario } from "../lib/agents.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("panel");
const read = (f) => readFileSync(join(dir, f), "utf8");
const text = async (sel) => (await $(sel)).getText();
const mod = process.platform === "darwin" ? "Meta" : "Control";
const click = async (sel) => (await $(sel)).click();
/** A choice in a segmented control or chips, by its group's name and its text. */
const choose = async (group, choice) => (await (await $(`[aria-label='${group}']`)).$(`button=${choice}`)).click();

describe("the options panel", () => {
  before(async () => {
    await open(dir);
  });

  it("holds a build setting change until Rebuild, which writes emmet.toml", async () => {
    // Rebuild asks the agent: a short replay answers it.
    scenario("claude", "plain-turn");
    await click("#settings-change");
    await choose("Style", "Medieval");
    await expect($("#settings-bar")).toHaveText(expect.stringContaining("1 changed"));
    await click("#settings-undo");
    await expect($("#settings-bar")).not.toBeExisting();
    // Age leads: 9+ sets Detail and Pieces to what 9+ sets usually have.
    await choose("Age", "9+");
    await expect($("#settings-bar")).toHaveText(expect.stringContaining("3 changed"));
    await expect($("[aria-label='Detail'] [data-state='on']")).toHaveText("Medium");
    await click("#settings-rebuild");
    await browser.waitUntil(() => read("emmet.toml").includes('age = "9+"'), { timeout: 10_000, timeoutMsg: "emmet.toml never got the settings" });
    expect(read("emmet.toml")).toContain('detail = "Medium"');
    expect(read("emmet.toml")).toContain('pieces = "Medium"');
    // Rebuild asks the agent, which starts on it: the chat says what changed in words, sent from Build settings.
    await browser.waitUntil(() => browser.execute(() => !window.__emmet.chat().running && window.__emmet.chat().items > 0), { timeout: 30_000, timeoutMsg: "the agent never took the rebuild" });
    await click("#history-toggle");
    await expect($("#chat-log")).toHaveText(expect.stringContaining("Sent from Build settings"), { wait: 10_000 });
    await expect($("#chat-log")).toHaveText(expect.stringContaining("Rework the model for the new build settings: age 9+ (was Any)"));
    await click("#history-toggle");
  });

  it("keeps a moved knob as a draft until Keep writes it into model.py", async () => {
    const before = await shown();
    const thumb = await $("#param-weathering-thumb");
    await thumb.click();
    await browser.keys(["ArrowRight", "ArrowRight"]);
    await expect($("#param-weathering")).toHaveText(expect.stringContaining("was 0%"));
    await expect($("#knobs-bar")).toHaveText(expect.stringContaining("1 change"));
    await nextScene(before);
    // The scene may be the build the Rebuild before started, so the draft's write is waited for itself.
    await browser.waitUntil(() => JSON.parse(read("params.json")).weathering === 2, { timeout: 20_000, timeoutMsg: "params.json never got the draft" });
    const kept = await shown();
    await click("#knobs-keep");
    await browser.waitUntil(() => /weathering[^\n]*= 2\b/.test(read("model.py")), { timeout: 20_000, timeoutMsg: "Keep didn't write model.py" });
    // Keep writes model.py, then drops the draft from params.json.
    await browser.waitUntil(() => JSON.parse(read("params.json")).weathering === undefined, { timeout: 10_000, timeoutMsg: "Keep left the draft in params.json" });
    await expect($("#knobs-bar")).not.toBeExisting();
    await nextScene(kept);

    // Shuffle is a draft too, and Reset drops it.
    await click("#shuffle-seed");
    await expect($("#knobs-bar")).toBeDisplayed();
    await browser.waitUntil(() => JSON.parse(read("params.json")).seed != null, { timeout: 10_000 });
    await click("#knobs-reset");
    await browser.waitUntil(() => JSON.parse(read("params.json")).seed == null, { timeout: 10_000, timeoutMsg: "Reset left the draft" });
    await expect($("#knobs-bar")).not.toBeExisting();
  });

  it("lists the parts with their pictures, most first", async () => {
    await click("#tab-parts");
    const rows = await $$("#parts-list tbody tr");
    const scene = await browser.execute(() => window.__TAURI_INTERNALS__.invoke("ui_state"));
    expect(rows.length).toBe(scene.scene.bom.lines.length);
    expect(await text("#parts-total")).toMatch(/^\d+ parts · \d+ kinds$/);
    await browser.waitUntil(() => browser.execute(() => document.querySelector("#parts-list tbody img")?.src.startsWith("data:image/png")), { timeout: 30_000, timeoutMsg: "no part picture drew" });
    // A picture is the part alone: around it the table's own surface shows, in either theme.
    const corner = await browser.execute(async () => {
      const img = document.querySelector("#parts-list tbody img");
      await img.decode();
      const c = document.createElement("canvas");
      c.width = img.naturalWidth;
      c.height = img.naturalHeight;
      const g = c.getContext("2d");
      g.drawImage(img, 0, 0);
      return g.getImageData(0, 0, 1, 1).data[3];
    });
    expect(corner).toBe(0);
    await rows[0].click();
    expect((await browser.execute(() => window.__emmet.viewer().highlighted())).length).toBe(scene.scene.bom.lines[0].qty);
  });

  it("shows the build up to a step, and every copy of a sub-assembly", async () => {
    await click("#tab-steps");
    const scene = (await browser.execute(() => window.__TAURI_INTERNALS__.invoke("ui_state"))).scene;
    const total = scene.tree.steps.length;
    await expect($("#steps-head")).toHaveText(`Step ${total} of ${total}`);
    await click("#steps-tree [data-n='3']");
    await expect($("#steps-head")).toHaveText(`Step 3 of ${total}`);
    const upTo3 = scene.placements.filter((p) => p.step <= 3).length;
    await browser.waitUntil(async () => (await stats()).instances === upTo3, { timeout: 10_000, timeoutMsg: `the viewer didn't show the ${upTo3} parts up to step 3` });
    // The windows: five copies, every one of their parts highlighted.
    await click("#steps-tree [data-sub='window.ldr']");
    const copies = scene.placements.filter((p) => p.submodel.includes("window.ldr")).map((p) => p.id).sort();
    expect((await browser.execute(() => window.__emmet.viewer().highlighted())).sort()).toEqual(copies);
    await browser.waitUntil(async () => (await stats()).instances === scene.placements.length, { timeout: 10_000 });
  });

  it("exports the parts list into Downloads, and says so", async () => {
    await browser.keys([mod, "e"]);
    await (await $("[data-menu='csv']")).waitForDisplayed();
    await click("[data-menu='csv']");
    await expect($("#toast")).toHaveText(expect.stringContaining("Saved “Panel parts.csv” to Downloads"));
    expect(existsSync(join(agentHome, "Downloads", "Panel parts.csv"))).toBe(true);
    await expect($("#toast-action")).toHaveText("Show in folder");
  });

  it("says a finding in plain words, and It's fine allows it until Check it again", async () => {
    await open(projectDir("panel-floating"));
    await click("#tab-checks");
    const finding = await $(".finding[data-finding]");
    await expect(finding).toHaveText(expect.stringMatching(/held by anything/));
    await expect(finding).toHaveText(expect.stringContaining("Floating"));
    // No ids, file names, or line numbers (law 2): the validator's "cap (Plate 2 x 2)" reads "Plate 2 x 2".
    expect(await finding.getText()).not.toMatch(/model\.py|:\d+\b|\bcap\b/);
    await finding.click();
    expect((await browser.execute(() => window.__emmet.viewer().highlighted())).length).toBeGreaterThan(0);
    await expect($("#model-state .panel-head-text")).toHaveText(/problem/);

    const before = await shown();
    await click(".finding [id^='allow-']");
    await nextScene(before);
    expect(readFileSync(join(projectDir("panel-floating"), "emmet.toml"), "utf8")).toContain("[[check.allow]]");
    await expect($("#checks-clean")).toBeDisplayed();
    await expect($("[data-allowed='floating']")).toHaveText(expect.stringContaining("Allowed"));

    const allowed = await shown();
    await (await $("[data-allowed='floating'] button")).click();
    await nextScene(allowed);
    await expect($(".finding[data-finding]")).toBeDisplayed();
    expect(readFileSync(join(projectDir("panel-floating"), "emmet.toml"), "utf8")).not.toContain("[[check.allow]]");
  });
});
