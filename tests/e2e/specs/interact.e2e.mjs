// SPDX-License-Identifier: AGPL-3.0-or-later
// The viewer's interactions to the frozen design (session 4C Done when 3; docs/design/components.md's viewer, the
// keyboard map, docs/decisions.md 2026-10-31): hovering names a part and its color; a click picks it, shift-click adds,
// and a click on nothing clears; ⌥-drag's box picks what's in it, and shift-drag pans (Barry 2026-11-10); Frame, the views, and the viewer's orbit and zoom
// keys; a step shows the build up to it with its parts box and a sub-assembly's box; the cut-away and its bar;
// Screenshot to the clipboard and to a file; a finding's pin opens Checks. The camera moves only when asked (law 1).
// Pointer events go to the canvas as the webview would send them (WebDriver's pointer actions aren't on every
// driver).
import { $, $$, expect } from "@wdio/globals";
import { existsSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import { camera, invoke, open, restingCamera, screenshots, stats } from "../lib/app.mjs";
import { agentHome } from "../lib/agents.mjs";
import { projectDir } from "../lib/projects.mjs";

const cottage = projectDir("interact");
const floating = projectDir("interact-floating");
const pins = projectDir("interact-pins");
const town = projectDir("interact-town");

/** Sends a pointer event to the viewer's canvas at (x, y) in its own CSS pixels. */
const pointer = (type, x, y, init = {}) =>
  browser.execute(
    (t, px, py, i) => {
      const c = document.querySelector("#viewer canvas");
      const r = c.getBoundingClientRect();
      c.dispatchEvent(new PointerEvent(t, { bubbles: true, cancelable: true, pointerId: 1, pointerType: "mouse", isPrimary: true, button: 0, buttons: t === "pointerup" ? 0 : 1, clientX: r.left + px, clientY: r.top + py, ...i }));
    },
    type,
    x,
    y,
    init,
  );
const click = async (x, y, init = {}) => {
  await pointer("pointerdown", x, y, init);
  await pointer("pointerup", x, y, init);
};
/** Points on the canvas where a part is, with the part, from a grid over the canvas (the viewer's own pick). */
const parts = () =>
  browser.execute(() => {
    const v = window.__emmet.viewer();
    const c = document.querySelector("#viewer canvas");
    const out = [];
    const seen = new Set();
    for (let y = c.clientHeight * 0.2; y < c.clientHeight * 0.7; y += 23) {
      for (let x = c.clientWidth * 0.2; x < c.clientWidth * 0.8; x += 23) {
        const id = v.pickAt(x, y);
        if (id && !seen.has(id)) {
          seen.add(id);
          out.push({ id, x, y });
        }
      }
    }
    return out;
  });
const highlighted = () => browser.execute(() => window.__emmet.viewer().highlighted());
const nameOf = async (id) => {
  const { scene } = await invoke("ui_state");
  const p = scene.placements.find((q) => q.id === id);
  const l = scene.bom.lines.find((b) => b.part === p.part && b.color === p.color);
  return `${l.name} · ${l.color_name}`;
};

describe("the viewer's interactions", () => {
  before(async () => {
    await open(cottage);
  });

  it("hovering a part names it and its color, and leaving clears it", async () => {
    const [a] = await parts();
    const before = await restingCamera();
    await pointer("pointermove", a.x, a.y, { buttons: 0 });
    await $("#viewer-hover").waitForDisplayed({ timeoutMsg: "hovering a part named nothing" });
    await expect($("#viewer-hover")).toHaveText(await nameOf(a.id));
    expect((await stats()).hovered).toBe(a.id);
    await screenshots("viewer-hover");
    await pointer("pointerleave", a.x, a.y, { buttons: 0 });
    await expect($("#viewer-hover")).not.toBeExisting();
    expect(await camera()).toEqual(before);
  });

  it("a click picks a part, shift-click adds another, and a click on nothing clears", async () => {
    const [a, b] = await parts();
    const before = await restingCamera();
    await click(a.x, a.y);
    await browser.waitUntil(async () => (await highlighted()).join() === a.id, { timeoutMsg: "a click didn't pick the part" });
    expect((await stats()).mode).toBe("select");
    await click(b.x, b.y, { shiftKey: true });
    await browser.waitUntil(async () => (await highlighted()).sort().join() === [a.id, b.id].sort().join(), { timeoutMsg: "shift-click didn't add the part" });
    // Above the model, in the canvas's top corner, there's nothing to pick.
    await click(4, 4);
    await browser.waitUntil(async () => (await highlighted()).length === 0, { timeoutMsg: "a click on nothing didn't clear the pick" });
    expect(await camera()).toEqual(before);
  });

  it("⌥-drag draws a box and picks the parts whose centers are in it, without turning the camera", async () => {
    const before = await restingCamera();
    // The box's corners sit on whole window pixels: WebKit keeps only the whole part of a made-up event's clientX and
    // clientY, and the canvas beside the chat sidebar doesn't start on one, so a corner between pixels would reach the
    // viewer up to a pixel away from the box this test expects. `nudge` keeps a rounding error from landing a pixel low.
    const [x0, y0, x1, y1] = await browser.execute(() => {
      const r = document.querySelector("#viewer canvas").getBoundingClientRect();
      const at = (edge, size, share) => Math.round(edge + size * share) - edge;
      return [at(r.left, r.width, 0.3), at(r.top, r.height, 0.3), at(r.left, r.width, 0.6), at(r.top, r.height, 0.6)];
    });
    const nudge = 1e-6;
    await pointer("pointerdown", x0 + nudge, y0 + nudge, { altKey: true });
    await pointer("pointermove", x1 + nudge, y1 + nudge, { altKey: true });
    await expect($("#viewer-box")).toBeDisplayed();
    await screenshots("viewer-box-select");
    await pointer("pointerup", x1 + nudge, y1 + nudge, { altKey: true });
    await expect($("#viewer-box")).not.toBeExisting();
    const want = await browser.execute((r) => window.__emmet.viewer().idsInBox(r), { x: x0, y: y0, width: x1 - x0, height: y1 - y0 });
    expect(want.length).toBeGreaterThan(5);
    await browser.waitUntil(async () => (await highlighted()).length === want.length, { timeoutMsg: "the box didn't pick its parts" });
    expect((await highlighted()).sort()).toEqual([...want].sort());
    expect(await camera()).toEqual(before);
  });

  it("shift-drag pans: no box, no pick, the camera slides without turning, and then stays put", async () => {
    // The pan is OrbitControls' (a left drag with shift, ⌘, or Ctrl); shift-drag no longer draws the box (Barry
    // 2026-11-10). Its damping's remainder is dropped once the camera comes to rest, so the camera stays put after.
    const picked = (await highlighted()).sort();
    const before = await restingCamera();
    const size = await browser.execute(() => [document.querySelector("#viewer canvas").clientWidth, document.querySelector("#viewer canvas").clientHeight]);
    const [x0, y0, x1, y1] = [size[0] * 0.3, size[1] * 0.3, size[0] * 0.6, size[1] * 0.6];
    await pointer("pointerdown", x0, y0, { shiftKey: true });
    await pointer("pointermove", x1, y1, { shiftKey: true });
    await expect($("#viewer-box")).not.toBeExisting();
    await pointer("pointerup", x1, y1, { shiftKey: true });
    await browser.pause(300);
    expect((await highlighted()).sort()).toEqual(picked);
    const after = await restingCamera();
    for (const i of [0, 1, 2, 4, 5, 6, 8, 9, 10]) expect(after[i]).toBeCloseTo(before[i], 5);
    expect(Math.hypot(after[12] - before[12], after[13] - before[13], after[14] - before[14])).toBeGreaterThan(1);
    await browser.pause(500);
    expect(await camera()).toEqual(after);
  });

  it("F frames the pick, the views turn to each side, and the viewer's keys orbit and zoom only when it has focus", async () => {
    const before = await camera();
    await browser.keys("f");
    await browser.waitUntil(async () => JSON.stringify(await camera()) !== JSON.stringify(before), { timeoutMsg: "F didn't frame the pick" });
    await $("#view-front").click();
    const front = await camera();
    // Looking along +X (the view's direction is the camera's −Z column).
    expect(front[8]).toBeLessThan(-0.99);
    await $("#viewer").click();
    await browser.keys("ArrowLeft");
    await browser.waitUntil(async () => JSON.stringify(await camera()) !== JSON.stringify(front), { timeoutMsg: "← didn't orbit the viewer" });
    const turned = await camera();
    await browser.keys("-");
    await browser.waitUntil(async () => JSON.stringify(await camera()) !== JSON.stringify(turned), { timeoutMsg: "− didn't zoom out" });
    // In the Steps tree, the arrows move through the tree and the camera stays.
    await $("#tab-steps").click();
    const still = await restingCamera();
    await (await $("#steps-tree [role='treeitem']")).click();
    await browser.keys("ArrowDown");
    await browser.pause(200);
    expect(await camera()).toEqual(still);
    await $("#view-iso").click();
  });

  it("a step shows the build up to it, outlines its new parts, and draws its parts box", async () => {
    const { scene } = await invoke("ui_state");
    const step = scene.tree.steps.find((s) => s.n > 3 && s.items.length >= 2);
    await $("#tab-steps").click();
    await (await $(`#steps-tree [role='treeitem'][data-n='${step.n}']`)).click();
    await browser.waitUntil(async () => (await stats()).step === step.n, { timeoutMsg: "picking a step didn't show it" });
    const s = await stats();
    expect(s.hidden).toBeGreaterThan(0);
    expect(s.outlined).toBe(step.items.reduce((n, i) => n + i.count, 0));
    await expect($("#step-view .step-number")).toHaveText(String(step.n));
    await expect($$("#parts-box .parts-box-item")).toBeElementsArrayOfSize(step.items.length);
    await browser.waitUntil(async () => (await $$("#parts-box img")).length === step.items.length, { timeoutMsg: "the parts box's pictures never drew" });
    await screenshots("viewer-step");
  });

  it("a step that puts on a sub-assembly draws its box with numbered sub-steps and a leader to the copies", async () => {
    await open(town);
    const { scene } = await invoke("ui_state");
    const step = scene.tree.steps.find((s) => s.subs.length);
    await $("#tab-steps").click();
    await (await $(`#steps-tree [role='treeitem'][data-n='${step.n}']`)).click();
    await browser.waitUntil(async () => (await stats()).step === step.n);
    const sub = scene.tree.submodels.find((m) => m.file === step.subs[0].file);
    await expect($("#step-view .sub-assembly")).toBeDisplayed();
    await expect($$("#step-view .sub-steps li")).toBeElementsArrayOfSize(sub.steps.length);
    await browser.waitUntil(async () => (await $$("#step-view .sub-steps img")).length === sub.steps.length, { timeoutMsg: "the sub-steps never drew" });
    await expect($("#step-leader")).toBeExisting();
    await screenshots("viewer-step-sub-assembly");
    // The sub-assembly's parts show at the step that places the copy, not at their own sub-steps: all of them there,
    // and none a step before (a filter by their own step showed them from step 2).
    expect((await stats()).outlined).toBe(step.subs[0].count * sub.parts);
    const shownIds = () => browser.execute(() => window.__emmet.viewer().idsInBox({ x: -1e5, y: -1e5, width: 2e5, height: 2e5 }));
    const copy = `${step.subs[0].ids[0]}/`;
    expect((await shownIds()).filter((id) => id.startsWith(copy)).length).toBe(sub.parts);
    await (await $(`#steps-tree [role='treeitem'][data-n='${step.n - 1}']`)).click();
    await browser.waitUntil(async () => (await stats()).step === step.n - 1);
    expect((await shownIds()).filter((id) => id.startsWith(copy))).toEqual([]);
  });

  it("the cut-away hides what's above a layer, its bar lowers and raises it, and Show all brings it back", async () => {
    await open(cottage);
    await $("#viewer").click();
    await browser.keys(["Alt", "ArrowDown"]);
    await $("#cut-bar").waitForDisplayed({ timeoutMsg: "⌥↓ didn't start the cut-away" });
    const layers = await browser.execute(() => window.__emmet.viewer().layers());
    expect((await stats()).cut).toBe(layers - 1);
    await expect($("#cut-bar")).toHaveText(expect.stringContaining(`Showing layers 1–${layers - 1} of ${layers}`));
    for (let i = 0; i < 10; i++) await $("#cut-down").click();
    await screenshots("viewer-cut-away");
    const low = await stats();
    expect(low.cut).toBe(layers - 11);
    expect(low.hidden).toBeGreaterThan(0);
    await $("#cut-up").click();
    expect((await stats()).cut).toBe(layers - 10);
    await $("#cut-off").click();
    await expect($("#cut-bar")).not.toBeExisting();
    expect((await stats()).hidden).toBe(0);
  });

  it("Screenshot puts a picture of the view on the clipboard, and the toast saves it to a file", async () => {
    // A retried spec finds the first try's file, and the next would be "Interact 2.png".
    rmSync(join(agentHome, "Downloads", "Interact.png"), { force: true });
    // The clipboard is the system's, which CI's own Mac shares: the spec records what the app hands it.
    await browser.execute(() => {
      window.__clip = null;
      navigator.clipboard.write = async (items) => {
        const blob = await items[0].getType("image/png");
        const bytes = new Uint8Array(await blob.arrayBuffer());
        window.__clip = Array.from(bytes.subarray(0, 8));
      };
    });
    await $("#screenshot").click();
    await expect($("#toast")).toHaveText(expect.stringContaining("Copied a picture of the model"));
    expect(await browser.execute(() => window.__clip)).toEqual([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
    await $("#toast-action").click();
    await expect($("#toast")).toHaveText(expect.stringContaining("Saved “Interact.png” to Downloads"));
    const file = join(agentHome, "Downloads", "Interact.png");
    expect(existsSync(file)).toBe(true);
    expect([...readFileSync(file).subarray(0, 4)]).toEqual([0x89, 0x50, 0x4e, 0x47]);
    // The host writes only a PNG: anything else the window sends is refused, and nothing is written.
    const files = readdirSync(join(agentHome, "Downloads")).length;
    for (const png of [btoa("not a png"), "%%%"]) {
      const refused = await browser.execute((b64) => window.__TAURI_INTERNALS__.invoke("save_screenshot", { png: b64 }).then(() => null, (e) => e.message ?? String(e)), png);
      expect(refused).toContain("wasn't a picture Emmet could save");
    }
    expect(readdirSync(join(agentHome, "Downloads")).length).toBe(files);
  });

  it("picks the parts a step hid once they show again", async () => {
    // Picking keeps each part's bounds; the parts a step hid must be pickable after it.
    const roof = await browser.execute(async () => {
      const v = window.__emmet.viewer();
      v.showSteps(1);
      await v.nextFrame();
      v.pickAt(10, 10);
      v.showSteps(null);
      await v.nextFrame();
      const { scene } = await window.__TAURI_INTERNALS__.invoke("ui_state");
      const tile = scene.placements.find((p) => p.part === "3037" && p.submodel.some((s) => /roof/i.test(s)) && p.id.includes("front"));
      const at = v.screenPoint([tile.id]);
      // Just below the top of its box, which is on the slope.
      return v.pickAt(at.x, at.y + 3);
    });
    expect(roof).toMatch(/^roof\//);
  });

  it("the cut-away counts from the model's new bottom after an update", async () => {
    const r = await browser.execute(async () => {
      const v = window.__emmet.viewer();
      const { scene } = await window.__TAURI_INTERNALS__.invoke("ui_state");
      const c = document.querySelector("#viewer canvas");
      const seen = () => {
        const ids = new Set();
        for (let y = 0; y < c.clientHeight; y += 9) for (let x = 0; x < c.clientWidth; x += 9) ids.add(v.pickAt(x, y));
        return ids.size;
      };
      v.cutAway(6);
      // A plate under the whole model: the bottom moves down a plate.
      const under = { id: "under", part: "3958", color: 71, matrix: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 200, 0, 1], step: 1, submodel: [] };
      await v.update([...scene.placements, under]);
      await v.nextFrame();
      const after = seen();
      // Cut again from scratch: what's drawn must be what the update drew.
      v.cutAway(null);
      v.cutAway(6);
      await v.nextFrame();
      const again = seen();
      v.cutAway(null);
      await v.update(scene.placements);
      return { after, again };
    });
    expect(r.after).toBe(r.again);
  });

  it("a part picture that can't be drawn doesn't stop the others", async () => {
    const [missing, after] = await browser.execute(async () => {
      const missing = await window.__emmet.partPicture("no-such-part", 4);
      const after = await window.__emmet.partPicture("3001", 1);
      return [missing, after?.slice(0, 22) ?? null];
    });
    expect(missing).toBe(null);
    expect(after).toBe("data:image/png;base64,");
  });

  it("a finding's pin sits at its parts and opens it in Checks", async () => {
    await open(floating);
    const pin = await $(".viewer-pin");
    await pin.waitForDisplayed({ timeoutMsg: "the finding has no pin" });
    await expect(pin).toHaveText(expect.stringMatching(/parts? floats?/));
    await screenshots("viewer-pin");
    const before = await restingCamera();
    await pin.click();
    await expect($("#tab-checks")).toHaveAttribute("aria-selected", "true");
    const s = await stats();
    expect(s.mode).toBe("issue");
    expect(s.highlighted).toBeGreaterThan(0);
    expect(await camera()).toEqual(before);
  });

  it("findings of one rule share a pin, and a step's size gets none", async () => {
    // Two floating bricks far apart and a step of ten: one "2 parts float" pin, not two, and no step-size pin stacked
    // over the model (the step-size warning is in Checks).
    await open(pins);
    const pin = await $(".viewer-pin");
    await pin.waitForDisplayed({ timeoutMsg: "the floating findings have no pin" });
    await expect($$(".viewer-pin")).toBeElementsArrayOfSize(1);
    await expect(pin).toHaveAttribute("data-pin", "floating");
    await expect(pin).toHaveText(expect.stringContaining("2 parts float"));
  });
});
