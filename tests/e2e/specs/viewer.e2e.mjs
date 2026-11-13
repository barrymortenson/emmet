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

  it("a software renderer gets the Fast preset, and a graphics card a better one", async () => {
    // Windows without a usable GPU (a VM, remote desktop, a blocked driver) draws WebGL in software: SwiftShader or
    // the Microsoft Basic Render Driver. GitHub's Windows runners are such machines.
    const { gpu, quality } = await stats();
    console.log(`gpu: ${gpu}; quality: ${quality}`);
    const software = /swiftshader|llvmpipe|software|basic render/i.test(gpu);
    expect(quality === "fast").toBe(software);
    if (software) expect((await stats()).pixelRatio).toBe(1);
  });

  it("the canvas follows the screen's scale: 100, 150, and 200 %", async () => {
    // A second viewer at Balanced (Fast always draws 1:1), told of each scale the way the browser tells a page: the
    // ratio changes and the window lays out again.
    const sizes = await browser.execute(async () => {
      const real = Object.getOwnPropertyDescriptor(window, "devicePixelRatio");
      const v = window.__emmet.newViewer();
      const canvas = document.body.lastElementChild;
      v.setQuality("balanced");
      const out = [];
      try {
        for (const ratio of [1, 1.5, 2, 3]) {
          Object.defineProperty(window, "devicePixelRatio", { value: ratio, configurable: true });
          canvas.style.width = `${800 + ratio * 2}px`;
          await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
          await v.nextFrame();
          out.push([v.stats().pixelRatio, canvas.width, canvas.clientWidth]);
        }
      } finally {
        if (real) Object.defineProperty(window, "devicePixelRatio", real);
        else delete window.devicePixelRatio;
        v.dispose();
        canvas.remove();
      }
      return out;
    });
    // Past 200 % the canvas stays at 2 pixels a CSS pixel: more costs frame time nobody sees.
    expect(sizes.map(([r]) => r)).toEqual([1, 1.5, 2, 2]);
    for (const [ratio, width, css] of sizes) expect(width).toBe(Math.floor(css * ratio));
  });

  it("draws again after the graphics context is lost and comes back", async () => {
    // A driver update, a laptop waking, or a remote desktop session connecting takes the WebGL context away. On a
    // viewer of its own with the same model, so the window's viewer keeps its context for the specs after this one.
    const placements = JSON.parse(readFileSync(join(project, "build", "placements.json"), "utf8")).parts;
    const result = await browser.execute(async (list) => {
      const v = window.__emmet.newViewer();
      const canvas = document.body.lastElementChild;
      canvas.style.visibility = "visible";
      canvas.style.zIndex = "-1";
      try {
        await v.load(list);
        v.frame();
        const gl = canvas.getContext("webgl2");
        const lose = gl.getExtension("WEBGL_lose_context");
        if (!lose) return { skipped: true };
        // An event that never comes says which one, in seconds, instead of timing the test out.
        const within = (what, p) => Promise.race([p, new Promise((_, no) => setTimeout(() => no(new Error(`${what} never came`)), 5000))]);
        const event = (name) => within(name, new Promise((r) => canvas.addEventListener(name, r, { once: true })));
        // The canvas's own pixels, read in the same turn as the frame that drew them, at a size quick to compare.
        const picture = async () => {
          await within("a frame", v.nextFrame());
          const g = new OffscreenCanvas(160, 120).getContext("2d");
          g.drawImage(canvas, 0, 0, 160, 120);
          return g.getImageData(0, 0, 160, 120).data;
        };
        const colors = (data) => {
          const seen = new Set();
          for (let i = 0; i < data.length; i += 4) seen.add((data[i] << 16) | (data[i + 1] << 8) | data[i + 2]);
          return seen.size;
        };
        const differs = (a, b) => {
          let n = 0;
          for (let i = 0; i < a.length; i += 4) if (Math.abs(a[i] - b[i]) + Math.abs(a[i + 1] - b[i + 1]) + Math.abs(a[i + 2] - b[i + 2]) > 24) n++;
          return n / (a.length / 4);
        };
        // Settled first: the viewer picks each part's stud detail a frame or two after the camera stops.
        let was = await picture();
        for (let i = 0; i < 20; i++) {
          const next = await picture();
          const same = differs(was, next) === 0;
          was = next;
          if (same) break;
        }
        const before = v.stats();
        const lost = event("webglcontextlost");
        lose.loseContext();
        await lost;
        // Only once the lost event is over: until then the browser hasn't recorded that the page handled it, and
        // won't give the context back.
        await new Promise((r) => setTimeout(r, 100));
        const restored = event("webglcontextrestored");
        lose.restoreContext();
        await restored;
        // No one asks for a frame here: the viewer must draw the scene again by itself.
        for (let i = 0; i < 120 && v.stats().framesDrawn === before.framesDrawn; i++) await new Promise((r) => requestAnimationFrame(r));
        const drew = v.stats().framesDrawn - before.framesDrawn;
        // The same picture as before: every part, its colors, and the room's light on them.
        let now = await picture();
        for (let i = 0; i < 20 && differs(was, now) > 0; i++) now = await picture();
        await new Promise((r) => setTimeout(r, 500));
        return { lost: gl.isContextLost(), drew, instances: [before.instances, v.stats().instances], differ: differs(was, now), colors: [colors(was), colors(now)] };
      } finally {
        v.dispose();
        canvas.remove();
      }
    }, placements);
    if (result.skipped) return console.log("no WEBGL_lose_context here");
    console.log(`after the context came back: ${JSON.stringify(result)}`);
    expect(result.lost).toBe(false);
    expect(result.drew).toBeGreaterThan(0);
    expect(result.instances[1]).toBe(result.instances[0]);
    expect(result.colors[0]).toBeGreaterThan(50);
    expect(result.colors[1]).toBeGreaterThan(50);
    expect(result.differ).toBeLessThan(0.01);
  });

  it("Copy diagnostics says what draws the window and how fast it has been", async () => {
    // What a bug report from a slow machine needs, and how section 13's budgets are read off an installed build.
    await browser.execute(async () => {
      const v = window.__emmet.viewer();
      for (let i = 0; i < 12; i++) {
        v.orbit(0.03);
        await v.nextFrame();
      }
    });
    const text = await browser.execute(() => window.__emmet.diagnostics());
    const { gpu } = await stats();
    expect(text).toContain(`Graphics: ${gpu} (`);
    expect(text).toMatch(/^Start: ready \d+ ms after launch$/m);
    expect(text).toMatch(/^Last build shown: \d+ parts.*, run and check (\d+|\?) ms, drawn in \d+ ms$/m);
    expect(text).toMatch(/^Last motion: \d+\.\d ms a frame, \d+ draw calls$/m);
    expect(text).toMatch(/^Emmet \d/);
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
