// SPDX-License-Identifier: AGPL-3.0-or-later
// A minimal MCP client over stdio for the E2E specs: start `emmet mcp`, initialize, call tools.
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { bin } from "./paths.mjs";

export async function mcp(project) {
  const child = spawn(bin("emmet"), ["--data-dir", process.env.EMMET_DATA_DIR, "mcp", "--project", project], {
    stdio: ["pipe", "pipe", "inherit"],
  });
  const waiting = new Map();
  createInterface({ input: child.stdout }).on("line", (line) => {
    const msg = JSON.parse(line);
    waiting.get(msg.id)?.(msg);
    waiting.delete(msg.id);
  });
  let id = 0;
  const request = (method, params) =>
    new Promise((resolve, reject) => {
      const n = ++id;
      waiting.set(n, (m) => (m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result)));
      child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id: n, method, params })}\n`);
    });
  await request("initialize", {
    protocolVersion: "2025-06-18",
    capabilities: {},
    clientInfo: { name: "emmet-e2e", version: "0" },
  });
  child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
  return {
    call: (name, args) => request("tools/call", { name, arguments: args }),
    close: () => child.kill(),
  };
}
