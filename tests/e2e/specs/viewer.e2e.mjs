// SPDX-License-Identifier: AGPL-3.0-or-later
// The live viewer on the real app (session 1B): open a project and see every part; a slider or an external edit
// updates the scene in place without moving the camera; a broken model.py shows the error and its line while the last
// good model stays; fixing it recovers. On the cottage template, opened at launch (lib/projects.mjs).
import { readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { $, expect } from "@wdio/globals";
import { camera, instances, invoke, nextScene, open, screenshots, shown, stats } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const project = projectDir("viewer");
const modelPy = join(project, "model.py");
const original = readFileSync(modelPy, "utf8");
const partCount = async () => Number(await $("#part-count").getText());

describe("the live viewer", () => {
  it("opens the fixture project and draws every part", async () => {
    await open(project);
    const parts = await partCount();
    expect(parts).toBeGreaterThan(50);
    await instances(parts);
    const s = await stats();
    expect(s.failed).toEqual({});
    expect(s.drawCalls).toBeGreaterThan(0);
    await expect($("#status")).toHaveText(expect.stringContaining(`Built ${parts} parts`));
    await screenshots("viewer-open");
  });

  it("dragging the width slider changes the count and keeps the camera", async () => {
    const before = await camera();
    const gen = await shown();
    const count = (await stats()).instances;
    const slider = await $("#param-width");
    const started = Date.now();
    if (process.platform === "darwin") {
      // A real drag, from the middle of the track to the right (the embedded driver doesn't send keys to a range).
      await browser.action("pointer").move({ origin: slider }).down().move({ origin: slider, x: 40, y: 0 }).up().perform();
    } else {
      // On WebView2 the embedded driver synthesizes pointer actions as DOM events, which never move a native range
      // (ADR 0002). Set the value the way a drag does: the native setter, then input and change.
      await browser.execute((el) => {
        Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set.call(el, "11");
        el.dispatchEvent(new Event("input", { bubbles: true }));
        el.dispatchEvent(new Event("change", { bubbles: true }));
      }, slider);
    }
    // Sample while the update lands: the old scene stays whole until the new one replaces it (no flicker).
    let lowest = Infinity;
    await browser.waitUntil(
      async () => {
        const s = await stats();
        lowest = Math.min(lowest, s.instances);
        return (await shown()) > gen && s.instances !== count;
      },
      { timeout: 20_000, interval: 25, timeoutMsg: "the slider never changed the model" },
    );
    const ms = Date.now() - started;
    console.log(`slider → updated viewer: ${ms} ms`);
    expect(lowest).toBeGreaterThanOrEqual(count);
    expect((await stats()).instances).toBeGreaterThan(count);
    expect(await camera()).toEqual(before);
    const width = Number(await slider.getValue());
    expect(width).toBeGreaterThan(8);
    expect(JSON.parse(readFileSync(join(project, "params.json"), "utf8")).width).toBe(width);
    // Section 13's budget on M-series; slower hosted runners only report it.
    if (process.platform === "darwin" && process.env.RUNNER_ENVIRONMENT !== "github-hosted") expect(ms).toBeLessThan(1000);
  });

  it("an external edit to model.py updates the viewer in place", async () => {
    const before = await camera();
    const gen = await shown();
    const count = (await stats()).instances;
    writeFileSync(modelPy, original.replace("chimney: bool = True", "chimney: bool = False"));
    await nextScene(gen);
    await browser.waitUntil(async () => (await stats()).instances < count, { timeoutMsg: "the chimney didn't go" });
    expect(await camera()).toEqual(before);
    // The params drawer reads the schema again: the chimney's new default.
    await browser.waitUntil(async () => !(await $("#param-chimney").isSelected()), { timeoutMsg: "the chimney toggle kept the old default" });
  });

  it("a broken model.py shows the error and its line, and the last good model stays", async () => {
    const count = (await stats()).instances;
    const lines = original.split("\n");
    const at = lines.findIndex((l) => l.includes("m = Model()"));
    lines.splice(at + 1, 0, "    print(widht)");
    writeFileSync(modelPy, lines.join("\n"));
    await $("#error-panel").waitForDisplayed({ timeout: 20_000 });
    await expect($("#error-location")).toHaveText(`model.py:${at + 2}`);
    await expect($("#error-panel .message")).toHaveText(expect.stringContaining("widht"));
    await expect($("#status")).toHaveText(expect.stringContaining("showing the last good model"));
    expect((await stats()).instances).toBe(count);
    await screenshots("viewer-error");
  });

  it("fixing model.py recovers", async () => {
    const gen = await shown();
    writeFileSync(modelPy, original);
    await nextScene(gen);
    await expect($("#error-panel")).not.toBeExisting();
    await instances(await partCount());
  });

  it("clicking a part in the tree highlights it", async () => {
    const button = await $("button*=3004 ×");
    await button.click();
    await expect(button).toHaveAttribute("aria-pressed", "true");
    await button.click();
    await expect(button).toHaveAttribute("aria-pressed", "false");
  });

  it("emmet-lib serves the library and refuses paths outside it", async () => {
    const statuses = await browser.execute(async () => {
      const base = window.__TAURI_INTERNALS__.convertFileSrc("", "emmet-lib");
      const get = async (p) => (await fetch(base + p)).status;
      return {
        part: await get("parts/3001.dat"),
        upper: await get("PARTS/3001.DAT"),
        dotdot: await get("parts/..%2F..%2Flibrary.json"),
        encoded: await get("%2e%2e/%2e%2e/machine-id"),
        backslash: await get("parts%5C..%5C..%5Cmachine-id"),
        absolute: await get("%2Fetc%2Fhosts"),
      };
    });
    expect(statuses).toEqual({ part: 200, upper: 200, dotdot: 404, encoded: 404, backslash: 404, absolute: 404 });
  });

  it("deleting build/ keeps the model on screen, and the next change rebuilds it", async () => {
    const count = (await stats()).instances;
    rmSync(join(project, "build"), { recursive: true, force: true });
    await browser.pause(500);
    expect((await stats()).instances).toBe(count);
    await expect($("#error-panel")).not.toBeExisting();
    const gen = await shown();
    await invoke("set_params", { values: { width: 8 } });
    await nextScene(gen);
    expect(existsSync(join(project, "build", "placements.json"))).toBe(true);
    await instances(await partCount());
  });
});
