// SPDX-License-Identifier: AGPL-3.0-or-later
// A terminal agent and the open app (session 4B, plan section 10, docs/spec/bus.md): the real `emmet mcp`, started in
// the project folder with no session, as `codex mcp add emmet -- emmet mcp` or Claude Code's own config starts it,
// highlights parts in the app's viewer (named in the bar over it, with Zoom to them and Clear), turns its view, and reads
// what the user has selected there.
import { $, expect } from "@wdio/globals";
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { camera, invoke, open, screenshots, stats } from "../lib/app.mjs";
import { bin } from "../lib/paths.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("bus");

/** A minimal MCP client over stdio: one JSON-RPC message per line, as rmcp's stdio transport speaks it. */
function mcp() {
  const child = spawn(bin("emmet"), ["--data-dir", process.env.EMMET_DATA_DIR, "mcp"], { cwd: dir, stdio: ["pipe", "pipe", "inherit"] });
  const waiting = new Map();
  createInterface({ input: child.stdout }).on("line", (line) => {
    const m = JSON.parse(line);
    if (m.id != null && waiting.has(m.id)) {
      waiting.get(m.id)(m);
      waiting.delete(m.id);
    }
  });
  let next = 0;
  const request = (method, params) =>
    new Promise((resolve) => {
      const id = ++next;
      waiting.set(id, resolve);
      child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
    });
  return {
    async start() {
      await request("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "claude-code", version: "0" } });
      child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
    },
    async call(name, args) {
      const r = await request("tools/call", { name, arguments: args });
      return { text: r.result.content[0].text, data: r.result.structuredContent?.data };
    },
    close: () => child.kill(),
  };
}

describe("a terminal agent and the open app", () => {
  let agent;
  before(async () => {
    await open(dir);
    agent = mcp();
    await agent.start();
  });
  after(() => agent.close());

  it("highlights parts in the app's viewer and says who is pointing", async () => {
    const { scene } = await invoke("ui_state");
    const ids = scene.placements.slice(0, 3).map((p) => p.id);
    const r = await agent.call("highlight", { ids });
    expect(r.data.delivered).toBe(true);
    await browser.waitUntil(async () => JSON.stringify(await browser.execute(() => window.__emmet.viewer().highlighted().sort())) === JSON.stringify([...ids].sort()), {
      timeout: 5_000,
      timeoutMsg: "the viewer didn't highlight the agent's parts",
    });
    // The MCP client's own name (claude-code) names the agent.
    await expect($("#pointing-bar")).toHaveText(expect.stringContaining("Claude is pointing at 3 parts"));
    await screenshots("agent-pointing");
    const before = await camera();
    await $("#pointing-zoom").click();
    await browser.waitUntil(async () => JSON.stringify(await camera()) !== JSON.stringify(before), { timeoutMsg: "Zoom to them didn't move the camera" });
    await $("#pointing-clear").click();
    await expect($("#pointing-bar")).not.toBeExisting();
    expect((await stats()).highlighted).toBe(0);
  });

  it("turns the view and reads it back", async () => {
    const r = await agent.call("camera", { view: "front" });
    expect(r.data.view).toBe("front");
    await expect($('.view-switch-item[aria-checked="true"]')).toHaveText("Front");
    expect((await agent.call("camera", {})).data.view).toBe("front");
    await browser.keys("1");
  });

  it("reads what the user picked", async () => {
    // The user's pick replaces an agent's pointing.
    const { scene } = await invoke("ui_state");
    await agent.call("highlight", { ids: [scene.placements[0].id] });
    await $("#pointing-bar").waitForExist({ timeoutMsg: "the agent's pointing bar never showed" });
    // A row of the parts list selects every copy of that part.
    await (await $("#tab-parts")).click();
    const row = await $("#parts tbody tr");
    await row.click();
    const picked = await browser.execute(() => window.__emmet.viewer().highlighted().sort());
    expect(picked.length).toBeGreaterThan(0);
    await expect($("#pointing-bar")).not.toBeExisting();
    const r = await agent.call("selection", {});
    expect([...r.data.selected].sort()).toEqual(picked);
    expect(r.text).toContain(`Selected (${picked.length})`);
  });
});
