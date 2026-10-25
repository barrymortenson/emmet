#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Prints a recorded transcript as one line per protocol message, skipping streaming deltas (ADR 0001).
//   node tests/spikes/agents/summarize.mjs <transcript.jsonl>...

import { readFileSync } from "node:fs";

const clip = (s, n = 150) => (s.length > n ? s.slice(0, n) + "…" : s);
const DELTAS = new Set(["stream_event", "item/agentMessage/delta", "item/reasoning/summaryTextDelta", "item/reasoning/textDelta", "item/commandExecution/outputDelta", "item/reasoning/summaryPartAdded"]);

function line(dir, m) {
  if (typeof m !== "object" || m === null) return `${dir} ${clip(String(m))}`;
  if (m.type === "stream_event" || DELTAS.has(m.method)) return null;
  if (dir === "note" || dir === "stub") return `${dir} ${clip(JSON.stringify(m), 220)}`;
  // Claude stream-JSON
  if (m.type === "system") {
    const keep = m.subtype === "init" ? { apiKeySource: m.apiKeySource, model: m.model, permissionMode: m.permissionMode, version: m.claude_code_version, session_id: m.session_id } : { ...m, uuid: undefined, session_id: undefined };
    return `${dir} system/${m.subtype} ${clip(JSON.stringify(keep), 220)}`;
  }
  if (m.type === "assistant" || m.type === "user") {
    const parts = (Array.isArray(m.message?.content) ? m.message.content : [{ type: "text", text: m.message?.content }]).map((c) =>
      c.type === "text" ? `text:${JSON.stringify(clip(c.text ?? "", 80))}` : c.type === "tool_use" ? `tool_use:${c.name}(${clip(JSON.stringify(c.input), 80)})` : c.type === "tool_result" ? `tool_result${c.is_error ? "(error)" : ""}:${clip(JSON.stringify(c.content), 100)}` : c.type,
    );
    return `${dir} ${m.type} ${parts.join(" | ")}`;
  }
  if (m.type === "result") return `${dir} result/${m.subtype} ${clip(JSON.stringify({ result: m.result, is_error: m.is_error, turns: m.num_turns, cost: m.total_cost_usd, ms: m.duration_ms, errors: m.errors }), 220)}`;
  if (m.type) return `${dir} ${m.type} ${clip(JSON.stringify({ ...m, type: undefined, uuid: undefined, session_id: undefined }), 200)}`;
  // JSON-RPC (Codex)
  if (m.method) return `${dir} ${m.id !== undefined ? `req#${m.id} ` : ""}${m.method} ${clip(JSON.stringify(m.params ?? {}), 200)}`;
  if (m.id !== undefined) return `${dir} res#${m.id} ${clip(JSON.stringify(m.result ?? m.error), 200)}`;
  return `${dir} ${clip(JSON.stringify(m))}`;
}

for (const f of process.argv.slice(2)) {
  console.log(`=== ${f}`);
  for (const l of readFileSync(f, "utf8").split("\n").filter(Boolean)) {
    const { t, dir, msg } = JSON.parse(l);
    const s = line(dir, msg);
    if (s) console.log(`${String(t).padStart(7)} ${s}`);
  }
}
