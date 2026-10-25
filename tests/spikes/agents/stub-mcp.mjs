#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Throwaway stdio MCP server for the 0B protocol spikes (ADR 0001). No dependencies: newline-delimited JSON-RPC.
//
// Tools:
//   ping     {message?}            → "pong: <message>"
//   approve  {tool_name, input}    → Claude Code's --permission-prompt-tool answer: allow, except Bash and any path
//                                    outside SPIKE_PROJECT, which are denied
//   block    {seconds}             → sleeps, then "blocked for N s" (the tool-timeout tests)
//
// Every call is appended to $SPIKE_LOG as one JSON line ({at, server, tool, args}, plus `answer` for approve and
// `phase` start and end for block) so a recording can show what the CLI asked, independent of what it streamed.

import { appendFileSync } from "node:fs";
import { createInterface } from "node:readline";
import { isAbsolute, relative, resolve } from "node:path";

const project = process.env.SPIKE_PROJECT || process.cwd();
const log = (entry) => process.env.SPIKE_LOG && appendFileSync(process.env.SPIKE_LOG, JSON.stringify({ at: Date.now(), server: process.env.SPIKE_SERVER, ...entry }) + "\n");
const send = (msg) => process.stdout.write(JSON.stringify({ jsonrpc: "2.0", ...msg }) + "\n");

const TOOLS = [
  {
    name: "ping",
    description: "Answers pong. Use it when asked to ping the spike server.",
    inputSchema: { type: "object", properties: { message: { type: "string" } } },
  },
  {
    name: "approve",
    description: "Permission prompt handler for the spike. Not for direct use.",
    inputSchema: {
      type: "object",
      properties: { tool_name: { type: "string" }, input: { type: "object" }, tool_use_id: { type: "string" } },
      required: ["tool_name", "input"],
    },
  },
  {
    name: "block",
    description: "Waits the given number of seconds, then returns. Use it when asked to block or wait.",
    inputSchema: { type: "object", properties: { seconds: { type: "number" } }, required: ["seconds"] },
  },
];

const outside = (p) => {
  if (typeof p !== "string") return false;
  const rel = relative(project, isAbsolute(p) ? p : resolve(project, p));
  return rel.startsWith("..") || isAbsolute(rel);
};

async function call(name, args) {
  if (name === "ping") return `pong: ${args.message ?? ""}`;
  if (name === "approve") {
    const paths = [args.input?.file_path, args.input?.path, args.input?.notebook_path];
    const deny =
      args.tool_name === "Bash"
        ? "Shell commands are denied in this spike."
        : paths.some(outside)
          ? "Paths outside the project are denied in this spike."
          : null;
    const answer = deny ? { behavior: "deny", message: deny } : { behavior: "allow", updatedInput: args.input };
    log({ tool: name, args, answer });
    return JSON.stringify(answer);
  }
  if (name === "block") {
    const s = Number(args.seconds) || 0;
    log({ tool: name, args, phase: "start" });
    await new Promise((r) => setTimeout(r, s * 1000));
    log({ tool: name, args, phase: "end" });
    return `blocked for ${s} s`;
  }
  throw new Error(`unknown tool ${name}`);
}

createInterface({ input: process.stdin }).on("line", async (line) => {
  if (!line.trim()) return;
  let msg;
  try {
    msg = JSON.parse(line);
  } catch {
    return send({ id: null, error: { code: -32700, message: "parse error" } });
  }
  const { id, method, params } = msg;
  if (id === undefined) return; // notifications (initialized, cancelled) need no answer
  if (method === "initialize")
    return send({
      id,
      result: {
        protocolVersion: params?.protocolVersion ?? "2025-06-18",
        capabilities: { tools: {} },
        serverInfo: { name: "spike", version: "0.0.0" },
      },
    });
  if (method === "ping") return send({ id, result: {} });
  if (method === "tools/list") return send({ id, result: { tools: TOOLS } });
  if (method === "tools/call") {
    const { name, arguments: args = {} } = params;
    if (name !== "approve" && name !== "block") log({ tool: name, args });
    try {
      return send({ id, result: { content: [{ type: "text", text: await call(name, args) }] } });
    } catch (e) {
      return send({ id, result: { content: [{ type: "text", text: String(e.message) }], isError: true } });
    }
  }
  send({ id, error: { code: -32601, message: `method not found: ${method}` } });
});
