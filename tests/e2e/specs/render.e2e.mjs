// SPDX-License-Identifier: AGPL-3.0-or-later
// emmet render and the MCP render and highlight tools with the app open (1B Done when 3): they reach the open window
// over emmet-bus and render through its viewer, without moving its camera. tests/it/tests/render.rs covers the app
// closed (render mode).
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { expect } from "@wdio/globals";
import { camera, open, stats } from "../lib/app.mjs";
import { mcp } from "../lib/mcp.mjs";
import { bin, out } from "../lib/paths.mjs";
import { decode, isBlank } from "../lib/png.mjs";
import { projectDir } from "../lib/projects.mjs";

const project = projectDir("render");

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
});
