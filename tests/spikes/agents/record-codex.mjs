#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Records the Codex app-server JSON-RPC protocol for one 0B scenario (ADR 0001), on Barry's eval API key (H-21, D-38).
//
//   node tests/spikes/agents/record-codex.mjs <scenario> [--policy on-request|untrusted]
//
// The app-server runs with section 10's API-key provider block and -c mcp_servers overrides, plus --strict-config so a
// misspelled key fails instead of being ignored. Each run gets its own HOME (so CODEX_HOME is <HOME>/.codex, never
// ~/.codex), and the run stops before any turn unless initialize reports that codexHome and account/read shows no
// ChatGPT account. Approvals are answered like the earlier ask-first table: a file change whose paths are all inside
// the project is accepted, any other file change and every command is declined, MCP tool approvals for the spike's own
// two servers are accepted, and a server request the spike doesn't know gets a JSON-RPC error.

import { isAbsolute, join, relative } from "node:path";
import { STUB, cleanEnv, keychain, record, sleep, workspace } from "./harness.mjs";

const MODEL = "gpt-6-luna";
const POINTER = "You are operating Emmet's protocol spike. Use the emmet tools when the user asks for them.";

function args(ws, policy, scenario) {
  const toml = (v) => JSON.stringify(v);
  const server = (name, extra = []) => [
    "-c", `mcp_servers.${name}.command=${toml(process.execPath)}`,
    "-c", `mcp_servers.${name}.args=[${toml(STUB)}]`,
    "-c", `mcp_servers.${name}.env={ SPIKE_PROJECT = ${toml(ws.project)}, SPIKE_LOG = ${toml(ws.log)}, SPIKE_SERVER = ${toml(name)} }`,
    ...extra,
  ];
  return [
    "app-server", "--listen", "stdio://", "--strict-config",
    "-c", 'model_provider="emmet_api_key"', "-c", 'model_providers.emmet_api_key.name="OpenAI API key"',
    "-c", 'model_providers.emmet_api_key.base_url="https://api.openai.com/v1"', "-c", 'model_providers.emmet_api_key.env_key="OPENAI_API_KEY"',
    "-c", 'model_providers.emmet_api_key.wire_api="responses"', "-c", "model_providers.emmet_api_key.requires_openai_auth=false",
    ...server("emmet"),
    ...server("emmet_host", ["-c", "mcp_servers.emmet_host.tool_timeout_sec=3600"]),
    "-c", 'sandbox_mode="workspace-write"',
    // Codex 0.160.1 refuses approval_policy = "untrusted" in config; that run passes it to thread/start instead.
    ...(policy === "untrusted" ? [] : ["-c", `approval_policy=${toml(policy)}`]),
    // workspace-write lets commands write to /tmp and $TMPDIR by default, and the spike's workspace is under /tmp, so
    // the permission runs turn that off to test a write outside the project.
    ...(scenario === "permissions"
      ? ["-c", "sandbox_workspace_write.exclude_slash_tmp=true", "-c", "sandbox_workspace_write.exclude_tmpdir_env_var=true"]
      : []),
  ];
}

/** A JSON-RPC client over the recorder: requests, responses by id, and auto-answers to server requests. */
function rpc(s, ws) {
  let next = 1;
  // File-change approval requests carry only the item id; the paths come with the item's item/started event.
  const changePaths = new Map();
  const inside = (p) => {
    const rel = relative(ws.project, isAbsolute(p) ? p : join(ws.project, p));
    return rel !== "" && !rel.startsWith("..") && !isAbsolute(rel);
  };
  const decide = (m) => {
    const p = m.params ?? {};
    if (m.method === "item/commandExecution/requestApproval" || m.method === "execCommandApproval") return { decision: "decline" };
    if (m.method === "item/fileChange/requestApproval" || m.method === "applyPatchApproval") {
      const paths = changePaths.get(p.itemId) ?? Object.keys(p.fileChanges ?? {});
      return { decision: paths.length && paths.every(inside) ? "accept" : "decline" };
    }
    if (m.method === "mcpServer/elicitation/request")
      return /^emmet(_host)?$/.test(p.serverName) ? { action: "accept", content: {} } : { action: "decline" };
    if (m.method === "item/permissions/requestApproval") return { decision: "decline" };
    return null;
  };
  s.on((m) => {
    const item = m?.method === "item/started" ? m.params?.item : null;
    if (item?.type === "fileChange") changePaths.set(item.id, (item.changes ?? []).map((c) => c.path));
    if (m?.id === undefined || !m.method) return; // only server→client requests
    const result = decide(m);
    s.send(result ? { id: m.id, result } : { id: m.id, error: { code: -32601, message: `spike doesn't answer ${m.method}` } });
  });
  return {
    async call(method, params, ms = 120_000) {
      const id = next++;
      s.send({ id, method, params });
      const r = await s.waitFor((m) => m?.id === id && !m.method, ms, `${method} response`);
      if (r.error) throw new Error(`${method}: ${JSON.stringify(r.error)}`);
      return r.result;
    },
    notify(method, params) {
      s.send(params === undefined ? { method } : { method, params });
    },
  };
}

const text = (t) => [{ type: "text", text: t }];
const completed = (s, ms = 300_000) => s.waitFor((m) => m?.method === "turn/completed", ms, "turn/completed");
const toolStarted = (s, server) =>
  s.waitFor((m) => m?.method === "item/started" && m.params?.item?.type === "mcpToolCall" && (!server || m.params.item.server === server), 180_000, "mcpToolCall started");

async function start(s, c, ws) {
  const init = await c.call("initialize", { clientInfo: { name: "emmet", title: "Emmet", version: "0.0.0" } });
  c.notify("initialized");
  const account = await c.call("account/read", {});
  s.note({ guard: { codexHome: init.codexHome, account: account.account, requiresOpenaiAuth: account.requiresOpenaiAuth } });
  if (init.codexHome !== join(ws.home, ".codex") || account.account?.type === "chatgpt")
    throw new Error(`refusing to run: codexHome ${init.codexHome}, account ${JSON.stringify(account.account)}`);
}

async function thread(c, ws) {
  const policy = ws.policy === "untrusted" ? { approvalPolicy: "untrusted" } : {};
  const r = await c.call("thread/start", { cwd: ws.project, model: MODEL, developerInstructions: POINTER, ...policy });
  return r.thread.id;
}

async function turn(s, c, threadId, t, ms) {
  const r = await c.call("turn/start", { threadId, input: text(t), effort: "low" });
  return { turnId: r.turn.id, done: completed(s, ms) };
}

const SCENARIOS = {
  "plain-turn": async (s, c, ws) => {
    const th = await thread(c, ws);
    await (await turn(s, c, th, "Reply with exactly the word: ready")).done;
  },
  "tool-call": async (s, c, ws) => {
    const th = await thread(c, ws);
    await (await turn(s, c, th, "Use the emmet server's ping tool with the message hello, then tell me exactly what it answered.")).done;
  },
  permissions: async (s, c, ws) => {
    const th = await thread(c, ws);
    for (const t of [
      "Change WIDTH in model.py from 8 to 10. Reply done when finished.",
      "Read the file ../outside.txt and tell me its first line.",
      "Create a file notes.txt in the current directory containing the word hi. Reply done when finished.",
      "Append the line bye to the file ../outside.txt. Reply done when finished.",
      "Run the shell command `echo hi` and tell me its output.",
    ])
      await (await turn(s, c, th, t)).done;
  },
  interrupt: async (s, c, ws) => {
    const th = await thread(c, ws);
    const { turnId, done } = await turn(s, c, th, "Use the emmet server's block tool with seconds=60, then say done.");
    await toolStarted(s, "emmet");
    await sleep(2000);
    await c.call("turn/interrupt", { threadId: th, turnId });
    await done;
  },
  "interrupt-new-turn": async (s, c, ws) => {
    const th = await thread(c, ws);
    const { turnId, done } = await turn(s, c, th, "Use the emmet server's block tool with seconds=60, then say done.");
    await toolStarted(s, "emmet");
    await sleep(2000);
    await c.call("turn/interrupt", { threadId: th, turnId });
    const next = await turn(s, c, th, "Reply with exactly the word: after").catch((e) => (s.note({ observed: e.message }), null));
    await done;
    if (next) await next.done;
  },
  // turn/steer while a tool runs: does the note reach the model at its next step?
  "mid-turn-steer": async (s, c, ws) => {
    const th = await thread(c, ws);
    const { turnId, done } = await turn(s, c, th, "Use the emmet server's block tool with seconds=20. When it returns, reply with one short sentence.");
    await toolStarted(s, "emmet");
    await sleep(2000);
    await c.call("turn/steer", { threadId: th, expectedTurnId: turnId, input: text("Also include the word banana in your reply.") }).catch((e) => s.note({ observed: e.message }));
    await done;
  },
  // turn/start while a turn is active: rejected, queued, or folded into the running turn?
  "mid-turn-start": async (s, c, ws) => {
    const th = await thread(c, ws);
    const { done } = await turn(s, c, th, "Use the emmet server's block tool with seconds=20. When it returns, reply with one short sentence.");
    await toolStarted(s, "emmet");
    await sleep(2000);
    const second = await turn(s, c, th, "Also include the word banana in your reply.").catch((e) => (s.note({ observed: e.message }), null));
    await done;
    if (second) await Promise.race([second.done, sleep(60_000).then(() => s.note({ observed: "no second turn/completed within 60 s" }))]);
  },
  resume: async (s, c, ws, state) => {
    if (!state.threadId) {
      state.threadId = await thread(c, ws);
      await (await turn(s, c, state.threadId, "Remember the code word teal. Reply with exactly: ok")).done;
    } else {
      await c.call("thread/resume", { threadId: state.threadId, cwd: ws.project });
      await (await turn(s, c, state.threadId, "What was the code word? Reply with just the word.")).done;
    }
  },
  "block-default": async (s, c, ws) => {
    const th = await thread(c, ws);
    await (await turn(s, c, th, "Use the emmet server's block tool with seconds=600, then tell me exactly what it returned.", 900_000)).done;
  },
  "block-raised": async (s, c, ws) => {
    const th = await thread(c, ws);
    await (await turn(s, c, th, "Use the emmet_host server's block tool with seconds=600, then tell me exactly what it returned.", 900_000)).done;
  },
};

async function main() {
  const [scenario, flag, value] = process.argv.slice(2);
  const policy = flag === "--policy" ? value : "on-request";
  if (!SCENARIOS[scenario] || !["on-request", "untrusted"].includes(policy)) {
    console.error(`usage: record-codex.mjs <${Object.keys(SCENARIOS).join("|")}> [--policy on-request|untrusted]`);
    process.exit(2);
  }
  const name = policy === "on-request" ? scenario : `${scenario}-${policy}`;
  const ws = workspace(name);
  ws.policy = policy;
  const env = cleanEnv(ws, { OPENAI_API_KEY: keychain("emmet-eval-openai") });
  const state = {};
  for (let part = 0; part < (scenario === "resume" ? 2 : 1); part++) {
    const s = record({ cli: "codex", scenario: name, cmd: "codex", args: args(ws, policy, scenario), ws, env, append: part > 0, meta: { model: MODEL, policy } });
    const c = rpc(s, ws);
    try {
      await start(s, c, ws);
      await SCENARIOS[scenario](s, c, ws, state);
    } catch (e) {
      s.note({ error: e.message });
      await s.end(5_000);
      throw e;
    }
    await s.end(10_000);
  }
  console.log(`record-codex: ${name} → ${join(process.env.SPIKE_RAW_DIR || "$TMPDIR/emmet-0b-raw", "codex", name + ".jsonl")}`);
}

main().catch((e) => {
  console.error(`record-codex: ${e.message}`);
  process.exit(1);
});
