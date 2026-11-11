// SPDX-License-Identifier: AGPL-3.0-or-later
// The options panel's tabs (plan section 14, session 4A Done when 3): Options' build settings (a change waits with
// Undo and Rebuild, and Age sets Detail) and knobs (a draft until Keep writes model.py, or Reset drops it);
// Checks (a finding in plain words, It's fine allows it in emmet.toml, Check it again brings it back); Parts (a row per
// part and color, with its picture); Steps (a step shows the build up to it; a sub-assembly highlights every copy; a
// sub-step shows its sub-assembly on its own, session 4H); and Export (a toast says what was written where).
import { $, $$, expect } from "@wdio/globals";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { camera, nextScene, open, restingCamera, screenshots, shown, stats } from "../lib/app.mjs";
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
    // Age leads: 9+ sets Detail to what 9+ sets usually have. No setting is a part count (Barry 2026-11-10).
    await expect($("[aria-label='Pieces']")).not.toBeExisting();
    await choose("Age", "9+");
    await expect($("#settings-bar")).toHaveText(expect.stringContaining("2 changed"));
    await expect($("[aria-label='Detail'] [data-state='on']")).toHaveText("Medium");
    await click("#settings-rebuild");
    await browser.waitUntil(() => read("emmet.toml").includes('age = "9+"'), { timeout: 10_000, timeoutMsg: "emmet.toml never got the settings" });
    expect(read("emmet.toml")).toContain('detail = "Medium"');
    expect(read("emmet.toml")).not.toContain("pieces");
    // Rebuild asks the agent, which starts on it: the chat says what changed in words, sent from Build settings.
    await browser.waitUntil(() => browser.execute(() => !window.__emmet.chat().running && window.__emmet.chat().items > 0), { timeout: 30_000, timeoutMsg: "the agent never took the rebuild" });
    await expect($("#chat-log")).toHaveText(expect.stringContaining("Sent from Build settings"), { wait: 10_000 });
    await expect($("#chat-log")).toHaveText(expect.stringContaining("Rework the model for the new build settings: age 9+ (was Any)"));
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
    // A sub-assembly's parts go in at the step that places the copy (their own step is the sub-assembly's); the later
    // steps' parts stay loaded but hidden.
    const upTo3 = scene.placements.filter((p) => (p.copies?.[0]?.step ?? p.step) <= 3).length;
    const shownNow = async () => {
      const s = await stats();
      return s.instances - s.hidden;
    };
    await browser.waitUntil(async () => (await shownNow()) === upTo3, { timeout: 10_000, timeoutMsg: `the viewer didn't show the ${upTo3} parts up to step 3` });
    // The windows: five copies, every one of their parts highlighted.
    await click("#steps-tree [data-sub='window.ldr']");
    const copies = scene.placements.filter((p) => p.submodel.includes("window.ldr")).map((p) => p.id).sort();
    expect((await browser.execute(() => window.__emmet.viewer().highlighted())).sort()).toEqual(copies);
    await browser.waitUntil(async () => (await shownNow()) === scene.placements.length, { timeout: 10_000 });
  });

  // Session 4H's Done when, on the picnic (a table set built apart, with a chair built apart inside it and used
  // twice) and on the Cottage's own sub-assemblies.
  for (const [name, at] of [["the picnic", projectDir("panel-picnic")], ["the Cottage", dir]]) {
    it(`a sub-step shows its sub-assembly on its own, built up to it, and the model comes back: ${name}`, async () => {
      await open(at);
      await click("#tab-steps");
      const scene = (await browser.execute(() => window.__TAURI_INTERNALS__.invoke("ui_state"))).scene;
      const subs = scene.tree.submodels;
      expect(subs.length).toBeGreaterThan(0);
      const total = scene.tree.steps.length;
      const onCanvas = () =>
        browser.execute(() => {
          const c = document.querySelector("#viewer canvas");
          return window.__emmet.viewer().idsInBox({ x: 0, y: 0, width: c.clientWidth, height: c.clientHeight });
        });
      const close = (a, b) => a.every((x, i) => Math.abs(x - b[i]) < 1e-3);
      // What a sub-step adds, from the tree: its own parts, and every part of the sub-assemblies it puts on.
      const adds = (s) => s.items.reduce((n, i) => n + i.count, 0) + s.subs.reduce((n, u) => n + u.count * subs.find((m) => m.file === u.file).parts, 0);
      const firstCopy = (file) => {
        const p = scene.placements.find((q) => q.submodel.includes(file));
        return p.copies[p.submodel.indexOf(file)].id;
      };
      const whole = async (before) => {
        await browser.waitUntil(async () => (await stats()).alone === null && (await stats()).hidden === 0, { timeoutMsg: "the whole model didn't come back" });
        expect(close(await restingCamera(), before)).toBe(true);
        await expect($("#apart-frame")).not.toBeExisting();
      };
      const before = await restingCamera();

      for (const m of subs) {
        const copy = firstCopy(m.file);
        await click(`#steps-tree [data-sub='${m.file}'] .tree-twisty`);
        let framed = null;
        let built = 0;
        for (const s of m.steps) {
          await click(`#steps-tree [data-sub-step='${m.file}/${s.n}']`);
          await browser.waitUntil(async () => (await stats()).alone === copy && (await stats()).step === s.n, { timeoutMsg: `${m.name}'s sub-step ${s.n} didn't show` });
          built += adds(s);
          // That copy alone, built up to the sub-step: nothing else shows, and all of it is on the canvas.
          const st = await stats();
          expect(st.instances - st.hidden).toBe(built);
          expect(st.outlined).toBe(adds(s));
          const ids = await onCanvas();
          expect(ids.length).toBe(built);
          expect(ids.every((id) => id.startsWith(`${copy}/`))).toBe(true);
          await expect($("#steps-head")).toHaveText(`${m.name}: step ${s.n} of ${m.steps.length}`);
          await expect($("#apart-at")).toHaveText(`${s.n} of ${m.steps.length}`);
          await expect($("#apart-frame")).toBeDisplayed();
          await expect($("#step-view .step-number")).toHaveText(String(s.n));
          // The camera frames the whole copy: it looks at the middle of its box, from close by; and it stays put from
          // one sub-step to the next.
          const cam = await restingCamera();
          const box = await browser.execute((c) => window.__emmet.viewer().boxOf([c]), copy);
          const mid = box.min.map((x, i) => (x + box.max[i]) / 2);
          const to = mid.map((x, i) => x - cam[12 + i]);
          const far = Math.hypot(...to);
          expect(-(to[0] * cam[8] + to[1] * cam[9] + to[2] * cam[10]) / far).toBeGreaterThan(0.9999);
          expect(far).toBeLessThan(6 * Math.hypot(...box.min.map((x, i) => box.max[i] - x)) + 100);
          if (framed) expect(close(cam, framed)).toBe(true);
          else expect(close(cam, before)).toBe(false);
          framed = cam;
        }
        // The step keys move through the sub-assembly's steps, and stop at its ends.
        const last = m.steps.length;
        await browser.keys("]");
        await browser.pause(100);
        expect((await stats()).step).toBe(last);
        if (last > 1) {
          await browser.keys("[");
          await browser.waitUntil(async () => (await stats()).step === last - 1, { timeoutMsg: "[ didn't go back a sub-step" });
          expect((await stats()).alone).toBe(copy);
          await click("#apart-next");
          await browser.waitUntil(async () => (await stats()).step === last, { timeoutMsg: "the bar's arrow didn't go on a sub-step" });
        }
        if (m === subs[subs.length - 1]) await screenshots(`viewer-sub-step-${at === dir ? "cottage" : "picnic"}`);
        await click("#apart-off");
        await whole(before);
      }

      // The three ways out, each with the camera back where it was, even after an orbit on the way.
      const m = subs[0];
      const enter = async () => {
        await click(`#steps-tree [data-sub-step='${m.file}/1']`);
        await browser.waitUntil(async () => (await stats()).alone != null);
      };
      await enter();
      await click(`#steps-tree [data-n='${total}']`);
      await whole(before);
      await expect($("#steps-head")).toHaveText(`Step ${total} of ${total}`);

      // Straight from one sub-assembly to another, the camera to go back to is still the model's; the tree keeps one
      // item in the Tab order when the shown sub-step's sub-assembly is closed; the cut-away's keys wait; and a
      // render for the agent draws the whole model and leaves the window as it was.
      await enter();
      if (subs.length > 1) {
        // (Every sub-assembly is still open from the walk through its sub-steps.)
        const other = subs[1];
        await click(`#steps-tree [data-sub-step='${other.file}/1']`);
        await browser.waitUntil(async () => (await stats()).alone === firstCopy(other.file), { timeoutMsg: "the other sub-assembly didn't show" });
        await click(`#steps-tree [data-sub='${other.file}'] .tree-twisty`);
        await expect($$("#steps-tree [role='treeitem'][tabindex='0']")).toBeElementsArrayOfSize(1);
        await expect($(`#steps-tree [data-sub='${other.file}']`)).toHaveAttribute("tabindex", "0");
        await browser.keys(["Alt", "ArrowDown"]);
        await browser.pause(100);
        expect((await stats()).cut).toBe(null);
        const during = await stats();
        // A part outside the copy: the render sees it, so it drew the whole model.
        const outside = scene.placements.find((p) => !p.copies?.length).id;
        const seen = await browser.execute(async (id) => (await window.__emmet.viewer().render({ size: 64, report: [id] })).report?.[id]?.seen ?? null, outside);
        expect(seen).not.toBe(null);
        const after = await stats();
        expect([after.alone, after.step, after.hidden]).toEqual([during.alone, during.step, during.hidden]);
        await browser.keys("Escape");
        await whole(before);
      }

      await enter();
      const inside = await restingCamera();
      await browser.execute(() => window.__emmet.viewer().orbit(0.6, 0.2));
      expect(close(await camera(), inside)).toBe(false);
      await browser.keys("Escape");
      await whole(before);

      await enter();
      await click(`#steps-tree [data-sub='${m.file}']`);
      await whole(before);
      // The row lights every copy in the model, as it does from the model.
      const copies = scene.placements.filter((p) => p.submodel.includes(m.file)).map((p) => p.id).sort();
      expect((await browser.execute(() => window.__emmet.viewer().highlighted())).sort()).toEqual(copies);
      await click(`#steps-tree [data-sub='${m.file}']`);
    });
  }

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
