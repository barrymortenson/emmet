#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Records Claude Code's stream-JSON protocol for one 0B scenario (ADR 0001), on Barry's eval API key (H-21, D-38).
//
//   node tests/spikes/agents/record-claude.mjs <scenario>      (scenarios: see SCENARIOS below)
//
// Each run gets its own HOME, so Claude Code can't see the plan sign-in or write to ~/.claude. It refuses to start
// unless `claude auth status` reports the API key in that environment, and kills the run if system/init then
// disagrees. Raw transcripts go to $SPIKE_RAW_DIR (default $TMPDIR/emmet-0b-raw); scripts/scrub-transcript turns
// them into tests/replay/claude/ fixtures.

import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { STUB, cleanEnv, keychain, record, sleep, workspace } from "./harness.mjs";

const POINTER = "You are operating Emmet's protocol spike. Use the emmet tools when the user asks for them.";
const user = (text) => ({ type: "user", message: { role: "user", content: text } });
const interrupt = (id) => ({ type: "control_request", request_id: id, request: { subtype: "interrupt" } });
const isResult = (e) => e?.type === "result";
const usesTool = (name) => (e) =>
  e?.type === "assistant" && e.message?.content?.some((c) => c.type === "tool_use" && c.name === name);

function args(ws, { resume, scenario } = {}) {
  const a = [
    "-p", "--input-format", "stream-json", "--output-format", "stream-json", "--verbose", "--include-partial-messages",
    "--mcp-config", join(ws.session, "mcp.json"), "--strict-mcp-config",
    "--permission-prompt-tool", "mcp__emmet_host__approve",
    "--allowedTools", "mcp__emmet__*", "mcp__emmet_host__ask_user", "Read(./**)", "Edit(model.py)", "Edit(models/**)", "Edit(CARD.md)",
    "--disallowedTools", "AskUserQuestion",
    "--append-system-prompt", POINTER,
    "--model", "haiku",
  ];
  if (resume) a.push("--resume", resume);
  // Tests whether an explicit ask rule makes Claude Code ask before read-only shell commands too (section 10).
  if (scenario === "permissions-ask-bash") a.push("--settings", JSON.stringify({ permissions: { ask: ["Bash"] } }));
  return a;
}

const SCENARIOS = {
  "plain-turn": async (s) => {
    s.send(user("Reply with exactly the word: ready"));
    await s.waitFor(isResult);
  },
  "tool-call": async (s) => {
    s.send(user("Use the ping tool with the message hello, then tell me exactly what it answered."));
    await s.waitFor(isResult);
  },
  // Section 10's permission table: an allowed edit, then a read outside the project, an edit outside the allowed
  // files, and a shell command. The model.py edit is pre-allowed by --allowedTools and never reaches approve; the stub's
  // approve tool denies the read and the shell command and allows the notes.txt write.
  permissions: async (s) => {
    for (const text of [
      "Change WIDTH in model.py from 8 to 10. Reply done when finished.",
      "Read the file ../outside.txt and tell me its first line.",
      "Create a file notes.txt in the current directory containing the word hi. Reply done when finished.",
      "Run the shell command `echo hi` and tell me its output.",
    ]) {
      s.send(user(text));
      await s.waitFor(isResult, 180_000, "result");
    }
  },
  "permissions-ask-bash": async (s) => {
    for (const text of ["Run the shell command `ls` and tell me its output.", "Run the shell command `echo hi` and tell me its output."]) {
      s.send(user(text));
      await s.waitFor(isResult, 180_000, "result");
    }
  },
  interrupt: async (s) => {
    s.send(user("Use the block tool with seconds=60, then say done."));
    await s.waitFor(usesTool("mcp__emmet__block"), 120_000, "block tool_use");
    await sleep(2000);
    s.send(interrupt("int-1"));
    await s.waitFor(isResult, 60_000, "result after interrupt");
  },
  "interrupt-new-turn": async (s) => {
    s.send(user("Use the block tool with seconds=60, then say done."));
    await s.waitFor(usesTool("mcp__emmet__block"), 120_000, "block tool_use");
    await sleep(2000);
    s.send(interrupt("int-1"));
    s.send(user("Reply with exactly the word: after"));
    await s.waitFor(isResult, 60_000, "first result");
    await s.waitFor(isResult, 120_000, "second result").catch((e) => s.note({ observed: e.message }));
  },
  // A user message written to stdin while a tool runs: does it reach the model at its next step, or wait?
  "mid-turn-message": async (s) => {
    s.send(user("Use the block tool with seconds=20. When it returns, reply with one short sentence."));
    await s.waitFor(usesTool("mcp__emmet__block"), 120_000, "block tool_use");
    await sleep(2000);
    s.send(user("Also include the word banana in your reply."));
    await s.waitFor(isResult, 120_000, "first result");
    await s.waitFor(isResult, 60_000, "second result").catch((e) => s.note({ observed: e.message }));
  },
  "block-default": async (s) => {
    s.send(user("Use the block tool with seconds=600, then tell me exactly what it returned."));
    await s.waitFor(isResult, 900_000, "result");
  },
  "block-raised": async (s) => {
    s.send(user("Use the block tool with seconds=600, then tell me exactly what it returned."));
    await s.waitFor(isResult, 900_000, "result");
  },
  // One past the largest 32-bit timer delay: does the setting overflow into an immediate timeout?
  "block-overflow": async (s) => {
    s.send(user("Use the block tool with seconds=30, then tell me exactly what it returned."));
    await s.waitFor(isResult, 900_000, "result");
  },
};
const ENV = {
  "block-raised": { MCP_TOOL_TIMEOUT: "86400000" },
  "block-overflow": { MCP_TOOL_TIMEOUT: "2147483648" },
};

async function run(scenario, ws, env, { resume, append, drive }) {
  const s = record({ cli: "claude", scenario, cmd: "claude", args: args(ws, { resume, scenario }), ws, env, append });
  const init = s.waitFor((e) => e?.type === "system" && e.subtype === "init", 60_000, "system/init");
  await drive(s, init);
  const r = await s.end();
  return { s, r };
}

async function main() {
  const scenario = process.argv[2];
  if (!SCENARIOS[scenario] && scenario !== "resume") {
    console.error(`usage: record-claude.mjs <${[...Object.keys(SCENARIOS), "resume"].join("|")}>`);
    process.exit(2);
  }
  const ws = workspace(scenario);
  const server = (name) => ({ command: process.execPath, args: [STUB], env: { SPIKE_PROJECT: ws.project, SPIKE_LOG: ws.log, SPIKE_SERVER: name } });
  writeFileSync(join(ws.session, "mcp.json"), JSON.stringify({ mcpServers: { emmet: server("emmet"), emmet_host: server("emmet_host") } }, null, 2));
  const env = cleanEnv(ws, { ANTHROPIC_API_KEY: keychain("emmet-eval-anthropic"), ...ENV[scenario] });

  // The guard (D-37): before any turn, Claude Code's own status command must report the API key under this exact
  // environment; and if system/init ever disagrees, the run is killed.
  const status = JSON.parse(execFileSync("claude", ["auth", "status", "--json"], { cwd: ws.project, env, encoding: "utf8" }));
  if (status.authMethod !== "api_key" || status.apiKeySource !== "ANTHROPIC_API_KEY")
    throw new Error(`refusing to run: auth status is ${status.authMethod}/${status.apiKeySource}`);
  const guarded = (drive) => async (s, init) => {
    init.then((e) => {
      s.note({ guard: { apiKeySource: e.apiKeySource } });
      if (e.apiKeySource !== "ANTHROPIC_API_KEY") s.kill();
    });
    await drive(s);
  };

  if (scenario === "resume") {
    let sessionId;
    await run(scenario, ws, env, {
      drive: guarded(async (s) => {
        s.send(user("Remember the code word teal. Reply with exactly: ok"));
        sessionId = (await s.waitFor(isResult)).session_id;
      }),
    });
    await run(scenario, ws, env, {
      resume: sessionId,
      append: true,
      drive: guarded(async (s) => {
        s.send(user("What was the code word? Reply with just the word."));
        await s.waitFor(isResult);
      }),
    });
  } else {
    await run(scenario, ws, env, { drive: guarded(SCENARIOS[scenario]) });
  }
  console.log(`record-claude: ${scenario} → ${join(process.env.SPIKE_RAW_DIR || "$TMPDIR/emmet-0b-raw", "claude", scenario + ".jsonl")}`);
}

main().catch((e) => {
  console.error(`record-claude: ${e.message}`);
  process.exit(1);
});
