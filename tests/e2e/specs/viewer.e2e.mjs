// SPDX-License-Identifier: AGPL-3.0-or-later
// The live viewer on the real app (session 1B): open a project and see every part; a slider or an external edit
// updates the scene in place without moving the camera; a broken model.py shows the error and its line while the last
// good model stays; fixing it recovers. On the cottage template, opened with open_recent (lib/projects.mjs).
import { readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { $, expect } from "@wdio/globals";
import { camera, instances, invoke, nextScene, open, screenshots, shown, stats } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const project = projectDir("viewer");
const modelPy = join(project, "model.py");
const original = readFileSync(modelPy, "utf8");
/** The part count of the build on screen, as the host has it. */
const partCount = async () => (await invoke("ui_state")).scene.parts;

describe("the live viewer", () => {
  it("opens the fixture project and draws every part", async () => {
    await open(project);
    const parts = await partCount();
    expect(parts).toBeGreaterThan(50);
    await instances(parts);
    const s = await stats();
    expect(s.failed).toEqual({});
    expect(s.drawCalls).toBeGreaterThan(0);
    await expect($("#model-state .panel-head-text")).toHaveText("No problems");
    await expect($("#tab-parts")).toHaveText(`Parts${parts}`);
    await screenshots("viewer-open");
  });

  it("a wider cottage changes the count in place and keeps the camera", async () => {
    const before = await camera();
    const gen = await shown();
    const count = (await stats()).instances;
    const started = Date.now();
    // Width isn't one of the cottage's knobs (they change its character, @model(knobs=...)), so it's set as the CLI
    // and the agent set it.
    await invoke("set_params", { values: { width: 18 } });
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
    const width = 18;
    expect(JSON.parse(readFileSync(join(project, "params.json"), "utf8")).width).toBe(width);
    // 1B's budget (What good looks like: ≤ 1.0 s on M-series); slower hosted runners only report it.
    if (process.platform === "darwin" && process.env.RUNNER_ENVIRONMENT !== "github-hosted") expect(ms).toBeLessThan(1000);
  });

  it("an external edit to model.py updates the viewer in place", async () => {
    const before = await camera();
    const gen = await shown();
    const count = (await stats()).instances;
    writeFileSync(modelPy, original.replace("roof: bool = True", "roof: bool = False"));
    await nextScene(gen);
    await browser.waitUntil(async () => (await stats()).instances < count, { timeoutMsg: "the roof didn't come off" });
    expect(await camera()).toEqual(before);
    // The knobs read the schema again: the roof's new default.
    await browser.waitUntil(async () => (await $("#param-roof-switch").getAttribute("aria-checked")) === "false", { timeoutMsg: "the roof switch kept the old default" });
  });

  it("a broken model.py says the model didn't build, with its line for the agent, and the last good model stays", async () => {
    const count = (await stats()).instances;
    const lines = original.split("\n");
    const at = lines.findIndex((l) => l.includes("m = Model()"));
    lines.splice(at + 1, 0, "    print(widht)");
    writeFileSync(modelPy, lines.join("\n"));
    await $("#build-failed").waitForDisplayed({ timeout: 20_000 });
    await expect($("#build-failed")).toHaveText(expect.stringContaining("The model didn't build"));
    await expect($("#build-failed")).toHaveText(expect.stringContaining("The last good model is still shown."));
    // The file, line, and Python's words go to the agent with Ask agent to fix, not to the screen (law 2).
    expect(await (await $("#build-failed")).getText()).not.toMatch(/widht|model\.py/);
    const error = await browser.execute(async () => {
      const { error: e } = await window.__TAURI_INTERNALS__.invoke("ui_state");
      return { file: e.file, line: e.line, message: e.message };
    });
    expect([error.file, error.line]).toEqual(["model.py", at + 2]);
    expect(error.message).toContain("widht");
    expect((await stats()).instances).toBe(count);
    await screenshots("viewer-error");
  });

  it("fixing model.py recovers", async () => {
    const gen = await shown();
    writeFileSync(modelPy, original);
    await nextScene(gen);
    await expect($("#build-failed")).not.toBeExisting();
    await instances(await partCount());
  });

  it("clicking a part in the Parts list highlights it", async () => {
    await $("#tab-parts").click();
    const row = await $("#parts-list tr[data-part='3004']");
    await row.click();
    await expect(row).toHaveAttribute("aria-selected", "true");
    await row.click();
    expect(await row.getAttribute("aria-selected")).toBe(null);
    await $("#tab-options").click();
  });

  it("emmet-lib serves the library and refuses paths outside it", async () => {
    // The targets exist, so a 404 means refused, not missing: the library is <data>/ldraw.
    const data = process.env.EMMET_DATA_DIR;
    expect(existsSync(join(data, "library.json")) && existsSync(join(data, "machine-id"))).toBe(true);
    const statuses = await browser.execute(async () => {
      const base = window.__TAURI_INTERNALS__.convertFileSrc("", "emmet-lib");
      const get = async (p) => (await fetch(base + p)).status;
      return {
        part: await get("parts/3001.dat"),
        upper: await get("PARTS/3001.DAT"),
        dotdot: await get("parts/..%2F..%2Flibrary.json"),
        encoded: await get("%2e%2e/machine-id"),
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
    await expect($("#build-failed")).not.toBeExisting();
    const gen = await shown();
    await invoke("set_params", { values: { width: 14 } });
    await nextScene(gen);
    expect(existsSync(join(project, "build", "placements.json"))).toBe(true);
    await instances(await partCount());
  });

  it("open_recent refuses a folder that isn't on the list", async () => {
    const refused = await browser.execute(async () => {
      try {
        await window.__TAURI_INTERNALS__.invoke("open_recent", { path: "/" });
        return false;
      } catch {
        return true;
      }
    });
    expect(refused).toBe(true);
  });

  it("two parameter changes at once both land", async () => {
    const gen = await shown();
    await browser.execute(() =>
      Promise.all([
        window.__TAURI_INTERNALS__.invoke("set_params", { values: { depth: 12 } }),
        window.__TAURI_INTERNALS__.invoke("set_params", { values: { weathering: 7 } }),
      ]),
    );
    await nextScene(gen);
    const saved = JSON.parse(readFileSync(join(project, "params.json"), "utf8"));
    expect([saved.depth, saved.weathering]).toEqual([12, 7]);
  });

  it("reopening a project whose build lost its placements builds it again", async () => {
    rmSync(join(project, "build", "placements.json"));
    const root = await open(project);
    expect(root).toBeTruthy();
    expect(existsSync(join(project, "build", "placements.json"))).toBe(true);
    await instances(await partCount());
  });

  it("a part that failed to load is tried again, and drawn when it loads", async () => {
    const result = await browser.execute(async (placements) => {
      // The app's part source, failing its first request, as a dropped fetch would.
      let calls = 0;
      const lib = window.__emmet.library;
      const flaky = {
        parts: (ids) => (calls++ === 0 ? Promise.reject(new Error("offline")) : lib.parts(ids)),
        colors: () => lib.colors(),
      };
      const v = window.__emmet.newViewer(flaky);
      await v.load(placements);
      const first = v.stats();
      const refused = await v.render({ size: 64 }).then(() => false, () => true);
      await v.update(placements);
      await v.nextFrame();
      const second = v.stats();
      const rendered = await v.render({ size: 64 }).then(() => true, () => false);
      v.dispose();
      return { failedFirst: Object.keys(first.failed).length, refused, failedAfter: Object.keys(second.failed).length, instances: second.instances, rendered };
    }, (await invoke("ui_state")).scene.placements);
    expect(result.failedFirst).toBeGreaterThan(0);
    expect(result.refused).toBe(true);
    expect(result.failedAfter).toBe(0);
    expect(result.instances).toBe(await partCount());
    expect(result.rendered).toBe(true);
  });
});
