// SPDX-License-Identifier: AGPL-3.0-or-later
// emmet render and the MCP render and highlight tools with the app open (1B Done when 3): they reach the open window
// over emmet-bus and render through its viewer, without moving its camera. tests/it/tests/render.rs covers the app
// closed (render mode).
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect } from "@wdio/globals";
import pixelmatch from "pixelmatch";
import { camera, nextScene, open, render, shown, stats } from "../lib/app.mjs";
import { mcp } from "../lib/mcp.mjs";
import { bin, os, out, repo } from "../lib/paths.mjs";
import { decode, isBlank } from "../lib/png.mjs";
import { projectDir } from "../lib/projects.mjs";

const project = projectDir("render");

const emmetJson = (project, args) =>
  JSON.parse(
    execFileSync(bin("emmet"), ["--json", "--data-dir", process.env.EMMET_DATA_DIR, "--project", project, ...args], {
      encoding: "utf8",
    }),
  );

function checkPng(bytes, size) {
  const img = decode(bytes);
  expect([img.width, img.height]).toEqual([size, size]);
  expect(isBlank(img)).toBe(false);
}

describe("rendering with the app open", () => {
  before(async () => {
    await open(project);
  });

  it("emmet render renders through the open window", async () => {
    const before = await camera();
    const file = join(out, "render-app.png");
    const env = JSON.parse(
      execFileSync(bin("emmet"), ["--json", "--data-dir", process.env.EMMET_DATA_DIR, "--project", project, "render", "--view", "front", "--size", "256", "--out", file], {
        encoding: "utf8",
      }),
    );
    expect(env.ok).toBe(true);
    expect(env.data.via).toBe("app");
    expect(env.data.camera.view).toBe("front");
    checkPng(readFileSync(file), 256);
    expect(await camera()).toEqual(before);
  });

  it("MCP render returns image content from the open window", async () => {
    const client = await mcp(project);
    try {
      const r = await client.call("render", { view: "iso", size: 192, highlight: ["door"] });
      expect(r.isError).not.toBe(true);
      expect(r.structuredContent.data.via).toBe("app");
      expect(r.structuredContent.data.highlighted).toEqual(["door"]);
      const image = r.content.find((c) => c.type === "image");
      expect(image.mimeType).toBe("image/png");
      checkPng(Buffer.from(image.data, "base64"), 192);
      // The render's highlight was for the render only.
      expect((await stats()).highlighted).toBe(0);
    } finally {
      client.close();
    }
  });

  it("MCP highlight reaches the open viewer", async () => {
    const client = await mcp(project);
    try {
      const r = await client.call("highlight", { ids: ["door", "base"] });
      expect(r.structuredContent.data.delivered).toBe(true);
      await browser.waitUntil(async () => (await stats()).highlighted === 2, { timeoutMsg: "no highlight in the viewer" });
      await client.call("highlight", { ids: [] });
      await browser.waitUntil(async () => (await stats()).highlighted === 0);
    } finally {
      client.close();
    }
  });

  it("a render shows only its own highlight, never the window's", async () => {
    const plain = await render({ view: "iso", size: 256 });
    const client = await mcp(project);
    try {
      await client.call("highlight", { ids: ["door"] });
      await browser.waitUntil(async () => (await stats()).highlighted === 1);
      expect(Buffer.compare(await render({ view: "iso", size: 256 }), plain)).toBe(0);
      expect((await stats()).highlighted).toBe(1);
      await client.call("highlight", { ids: [] });
    } finally {
      client.close();
    }
  });

  it("a project the app doesn't have open renders in render mode, the same as the window would", async () => {
    const other = projectDir("render-other");
    const file = join(out, "render-other.png");
    const env = emmetJson(other, ["render", "--view", "iso", "--size", "512", "--out", file]);
    expect(env.ok).toBe(true);
    expect(env.data.via).toBe("render-mode");
    const actual = decode(readFileSync(file));
    const baseline = decode(readFileSync(join(repo, "tests", "e2e", "baselines", os, "colors.png")));
    const bad = pixelmatch(baseline.data, actual.data, null, 512, 512, { threshold: 0.1 });
    expect(bad / (512 * 512)).toBeLessThanOrEqual(0.001);
  });

  it("a render right after an edit shows the edited model", async () => {
    const before = await render({ view: "iso", size: 256 });
    const modelPy = join(project, "model.py");
    const text = readFileSync(modelPy, "utf8");
    writeFileSync(modelPy, text.replace("chimney: bool = True", "chimney: bool = False"));
    try {
      // emmet render runs the stale model itself and shows the result in the window before rendering it.
      const env = emmetJson(project, ["render", "--view", "iso", "--size", "256", "--out", join(out, "render-edit.png")]);
      expect(env.data.via).toBe("app");
      const edited = readFileSync(join(out, "render-edit.png"));
      expect(Buffer.compare(edited, before)).not.toBe(0);
      // The app's watcher runs the same edit too, so its run can land just after the render, one generation on and
      // built from the same file: the window catches up to it, and the render was already that model.
      const manifest = JSON.parse(readFileSync(join(project, "build", "manifest.json"), "utf8"));
      await browser.waitUntil(async () => (await shown()) >= manifest.run.generation, {
        timeoutMsg: `the window never showed generation ${manifest.run.generation}`,
      });
      expect(Buffer.compare(await render({ view: "iso", size: 256 }), edited)).toBe(0);
    } finally {
      const gen = await shown();
      writeFileSync(modelPy, text);
      await nextScene(gen);
    }
  });
});
