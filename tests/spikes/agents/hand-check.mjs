#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// The plan-only checks for ADR 0001, which Barry runs by hand on his own sign-ins (plan section 21, 0B). Unlike the
// recorders, these use the real HOME, so they see the plan sign-in, and Claude Code may save a session there as it
// would for any of Barry's own runs. The parent agent's variables are removed, as in the recorders.
//
//   node tests/spikes/agents/hand-check.mjs claude-usage        /usage in a -p stream-JSON session, then the same
//                                                               with --no-session-persistence
//   node tests/spikes/agents/hand-check.mjs claude-turn         one short turn on the plan: which credential, and
//                                                               whether a rate_limit_event arrives
//   node tests/spikes/agents/hand-check.mjs claude-turn-key     the same with ANTHROPIC_API_KEY (the eval key) also set
//   node tests/spikes/agents/hand-check.mjs codex-account       Codex app-server account/read on the CLI's own login,
//                                                               without and with OPENAI_API_KEY set (no turn)
//
// Raw streams go to $TMPDIR/emmet-0b-hand/<check>.jsonl; the summary leaves out emails and ids.

import { execFileSync, spawn } from "node:child_process";
import { appendFileSync, existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";
import { createInterface } from "node:readline";

const OUT = join(tmpdir(), "emmet-0b-hand");
mkdirSync(OUT, { recursive: true });
const env = (extra = {}) => {
  const e = {};
  for (const [k, v] of Object.entries(process.env)) if (!/^(CLAUDE|CODEX_|ORCA_|ANTHROPIC_|OPENAI_|MCP_)/.test(k)) e[k] = v;
  return { ...e, ...extra };
};
const key = (service) => execFileSync("security", ["find-generic-password", "-s", service, "-w"], { encoding: "utf8" }).trim();
const redact = (s) => s.replace(/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g, "<email>").replace(/sk-[A-Za-z0-9_*-]{16,}/g, "<key>");
const say = (s) => console.log(redact(s));

/** Runs a CLI in a fresh directory, feeding `lines` (one per prompt) and collecting JSON output until `done`. */
function run(name, cmd, args, { lines, env: e, done, ms = 120_000 }) {
  const cwd = join(OUT, `${name}-cwd`);
  mkdirSync(cwd, { recursive: true });
  const file = join(OUT, `${name}.jsonl`);
  writeFileSync(file, "");
  const child = spawn(cmd, args, { cwd, env: e, stdio: ["pipe", "pipe", "pipe"] });
  const events = [];
  return new Promise((resolve) => {
    const timer = setTimeout(() => child.kill("SIGTERM"), ms);
    createInterface({ input: child.stdout }).on("line", (l) => {
      appendFileSync(file, l + "\n");
      try {
        const m = JSON.parse(l);
        events.push(m);
        if (done(m, events)) child.stdin.end();
      } catch {
        // not JSON
      }
    });
    child.stderr.on("data", (d) => appendFileSync(file, JSON.stringify({ stderr: String(d) }) + "\n"));
    for (const l of lines) child.stdin.write(l + "\n");
    child.on("exit", (code) => {
      clearTimeout(timer);
      resolve({ events, code, cwd, file });
    });
  });
}

/** Session files Claude Code keeps for a working directory, by its encoded folder name. */
const sessions = (cwd) => {
  const dir = join(homedir(), ".claude", "projects", cwd.replace(/[^A-Za-z0-9]/g, "-"));
  return existsSync(dir) ? readdirSync(dir).filter((f) => f.endsWith(".jsonl")) : [];
};
const claudeArgs = (extra = []) => ["-p", "--input-format", "stream-json", "--output-format", "stream-json", "--verbose", "--strict-mcp-config", "--model", "haiku", ...extra];
const user = (text) => JSON.stringify({ type: "user", message: { role: "user", content: text } });
const init = (events) => events.find((e) => e.type === "system" && e.subtype === "init");

async function claudeUsage() {
  for (const [label, extra] of [["claude-usage", []], ["claude-usage-no-persist", ["--no-session-persistence"]]]) {
    const before = sessions(join(OUT, `${label}-cwd`));
    const r = await run(label, "claude", claudeArgs(extra), { lines: [user("/usage")], env: env(), done: (m) => m.type === "result" });
    const after = sessions(r.cwd);
    const result = r.events.find((e) => e.type === "result") ?? {};
    const report = r.events.find((e) => e.usage_report || e.type === "usage_report" || e.subtype === "usage_report");
    say(`\n== ${label} ${extra.join(" ")}`);
    say(`  apiKeySource: ${init(r.events)?.apiKeySource ?? "(no init)"}`);
    say(`  event types: ${[...new Set(r.events.map((e) => (e.subtype ? `${e.type}/${e.subtype}` : e.type)))].join(", ")}`);
    say(`  usage report: ${report ? JSON.stringify(report.usage_report ?? report).slice(0, 900) : "none"}`);
    say(`  result: ${JSON.stringify({ subtype: result.subtype, is_error: result.is_error, result: String(result.result ?? "").slice(0, 300), total_cost_usd: result.total_cost_usd, usage: result.usage, num_turns: result.num_turns })}`);
    say(`  sessions saved for this directory: before ${before.length}, after ${after.length}`);
  }
}

async function claudeTurn(withKey) {
  const label = withKey ? "claude-turn-key" : "claude-turn";
  const e = withKey ? env({ ANTHROPIC_API_KEY: key("emmet-eval-anthropic") }) : env();
  const r = await run(label, "claude", claudeArgs(["--no-session-persistence"]), { lines: [user("Reply with exactly the word: ready")], env: e, done: (m) => m.type === "result" });
  const result = r.events.find((x) => x.type === "result") ?? {};
  say(`\n== ${label}`);
  say(`  apiKeySource: ${init(r.events)?.apiKeySource}`);
  say(`  rate_limit_event: ${JSON.stringify(r.events.filter((x) => x.type === "rate_limit_event")).slice(0, 900) || "none"}`);
  say(`  result: ${JSON.stringify({ subtype: result.subtype, result: result.result, total_cost_usd: result.total_cost_usd })}`);
  say(`  claude auth status: ${execFileSync("claude", ["auth", "status", "--json"], { env: e, encoding: "utf8" }).replace(/\s+/g, " ")}`);
}

async function codexAccount() {
  for (const [label, e] of [["codex-account", env()], ["codex-account-key", env({ OPENAI_API_KEY: key("emmet-eval-openai") })]]) {
    const req = (id, method, params = {}) => JSON.stringify({ id, method, params });
    const r = await run(label, "codex", ["app-server", "--listen", "stdio://"], {
      lines: [req(1, "initialize", { clientInfo: { name: "emmet-hand-check", version: "0.0.0" } }), JSON.stringify({ method: "initialized" }), req(2, "account/read")],
      env: e,
      done: (m) => m.id === 2,
      ms: 30_000,
    });
    const acct = r.events.find((x) => x.id === 2);
    const a = acct?.result?.account;
    say(`\n== ${label}`);
    say(`  account/read: ${JSON.stringify(a ? { type: a.type, planType: a.planType, email: a.email ? "<present>" : a.email } : acct?.result ?? acct?.error)}`);
    say(`  requiresOpenaiAuth: ${acct?.result?.requiresOpenaiAuth}`);
  }
}

const check = process.argv[2];
const checks = { "claude-usage": claudeUsage, "claude-turn": () => claudeTurn(false), "claude-turn-key": () => claudeTurn(true), "codex-account": codexAccount };
if (!checks[check]) {
  console.error(`usage: hand-check.mjs <${Object.keys(checks).join("|")}>`);
  process.exit(2);
}
await checks[check]();
console.log(`\nraw streams: ${OUT}`);
