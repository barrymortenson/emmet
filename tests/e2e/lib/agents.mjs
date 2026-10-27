// SPDX-License-Identifier: AGPL-3.0-or-later
// The fake agents for the chat and setup specs (tests/fake-agents, plan section 16): the app finds them through
// EMMET_CLAUDE_BIN, EMMET_CODEX_BIN, and EMMET_AGY_BIN, they read their transcript from EMMET_FAKE_DIR, and Codex's API
// key comes from a file keychain.
// The app runs with HOME (USERPROFILE on Windows) in a test home holding sentinel agent configs, which the chat and
// setup specs hash before and after (D-26): Emmet must never write a user's .claude, .codex, or .gemini.
//
// The fake CLIs sit in the test home where EMMET_CLAUDE_BIN and EMMET_CODEX_BIN point, so the setup spec can remove
// one ("not installed") and have fake-install (EMMET_FAKE_INSTALL) put it back; the app never looks on PATH for them.
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import { bin, out, repo } from "./paths.mjs";

export const fakeDir = join(out, "fake");
export const agentHome = join(out, "agent-home");

const SENTINELS = {
  ".claude/settings.json": '{"permissions":{"allow":["Bash(ls)"]}}',
  ".claude/CLAUDE.md": "# Sentinel\n",
  ".claude.json": '{"mcpServers":{}}',
  ".codex/config.toml": 'model = "sentinel"\n',
  ".codex/AGENTS.md": "# Sentinel\n",
  ".gemini/config/mcp_config.json": '{"mcpServers":{}}',
};

const exe = process.platform === "win32" ? ".exe" : "";
/** Where the fake CLIs are "installed", and Antigravity's agy (a copy of fake-codex, which answers --version). */
export const installed = {
  claude: join(agentHome, ".local", "bin", `claude${exe}`),
  codex: join(agentHome, ".local", "bin", `codex${exe}`),
  agy: join(agentHome, ".local", "bin", `agy${exe}`),
};

/** Puts the fake `cli` where the app looks for it. */
export function install(cli) {
  mkdirSync(join(agentHome, ".local", "bin"), { recursive: true });
  copyFileSync(bin(cli === "agy" ? "fake-codex" : `fake-${cli}`), installed[cli]);
}

/** Sets the app's environment for the fakes (wdio's onPrepare, before the service starts the app). */
export function prepareAgents() {
  rmSync(fakeDir, { recursive: true, force: true });
  rmSync(agentHome, { recursive: true, force: true });
  mkdirSync(fakeDir, { recursive: true });
  for (const [rel, text] of Object.entries(SENTINELS)) {
    const p = join(agentHome, rel);
    mkdirSync(join(p, ".."), { recursive: true });
    writeFileSync(p, text);
  }
  for (const cli of ["claude", "codex", "agy"]) install(cli);
  const keychain = join(fakeDir, "keychain");
  writeFileSync(keychain, "sk-e2e-not-a-real-key");
  Object.assign(process.env, {
    EMMET_CLAUDE_BIN: installed.claude,
    EMMET_CODEX_BIN: installed.codex,
    EMMET_AGY_BIN: installed.agy,
    EMMET_FAKE_INSTALL: bin("fake-install"),
    // The fakes aren't signed by Anthropic; the integration tests check a real refusal.
    EMMET_FAKE_SIGNATURE: "ok",
    EMMET_FAKE_DIR: fakeDir,
    EMMET_FAKE_KEYCHAIN: keychain,
    // The agents get the app's own environment, so they see the variables above.
    EMMET_AGENT_ENV: "inherit",
    HOME: agentHome,
    USERPROFILE: agentHome,
  });
  // Claude's usage probe (D-36) plays 0B's /usage report on a Max plan.
  writeFileSync(join(fakeDir, "claude-usage.scenario"), join(repo, "tests", "replay", "claude", "usage-report.jsonl"));
  process.env.EMMET_E2E_SENTINELS = JSON.stringify(agentConfigHashes());
}

/** Points the next start of `cli`'s fake at tests/replay/<cli>/<name>.jsonl, as its first start. */
export function scenario(cli, name) {
  writeFileSync(join(fakeDir, `${cli}.scenario`), join(repo, "tests", "replay", cli, `${name}.jsonl`));
  writeFileSync(join(fakeDir, `${cli}.count`), "0");
}

/** The fake keychain's Sign in with ChatGPT record for `id`, put back together when it's split into parts (ADR 0011). */
export function chatgptRecord(id) {
  const head = readFileSync(join(fakeDir, `keychain.chatgpt-${id}`), "utf8");
  const n = Number(head.match(/^emmet-parts:(\d+)$/)?.[1] ?? 0);
  const text = n ? Array.from({ length: n }, (_, i) => readFileSync(join(fakeDir, `keychain.chatgpt-${id}-${i + 1}`), "utf8")).join("") : head;
  return JSON.parse(text);
}

/** Saves `record` for `id` whole in the entry (the fake reads any length; only saving checks the limit). */
export function setChatgptRecord(id, record) {
  for (const f of readdirSync(fakeDir)) if (f.startsWith(`keychain.chatgpt-${id}-`)) rmSync(join(fakeDir, f));
  writeFileSync(join(fakeDir, `keychain.chatgpt-${id}`), JSON.stringify(record));
}

/** sha256 of every file in the test home's agent configs (.claude, .claude.json, .codex, .gemini). */
export function agentConfigHashes() {
  const out = {};
  const walk = (p) => {
    if (!existsSync(p)) return;
    if (statSync(p).isDirectory()) for (const n of readdirSync(p)) walk(join(p, n));
    else out[relative(agentHome, p).replaceAll("\\", "/")] = createHash("sha256").update(readFileSync(p)).digest("hex");
  };
  for (const top of [".claude", ".claude.json", ".codex", ".gemini"]) walk(join(agentHome, top));
  return out;
}

// Sign in with ChatGPT (1E): fake-openai stands in for OpenAI's sign-in endpoints (EMMET_FAKE_OPENAI), and the fake
// browser (EMMET_FAKE_BROWSER) follows the authorize redirect to the app's loopback callback and notes each URL the app
// opens in fakeDir/opened.txt.
export const openaiDir = join(out, "openai");
let openai = null;

/** Starts fake-openai for the run and points the app at it (wdio's onPrepare). */
export async function startOpenAi() {
  rmSync(openaiDir, { recursive: true, force: true });
  mkdirSync(openaiDir, { recursive: true });
  openai = spawn(bin("fake-openai"), [openaiDir], { stdio: ["ignore", "pipe", "inherit"] });
  const base = await new Promise((resolve, reject) => {
    openai.stdout.once("data", (d) => resolve(String(d).trim()));
    openai.once("error", reject);
  });
  Object.assign(process.env, { EMMET_FAKE_OPENAI: base, EMMET_FAKE_BROWSER: fakeDir });
}

export function stopOpenAi() {
  openai?.kill();
}

/** How fake-openai answers from now on (its docs list the settings). */
export function setOpenAi(cfg) {
  writeFileSync(join(openaiDir, "openai.json"), JSON.stringify(cfg));
}

/** Every token fake-openai issued in this run. */
export function issuedTokens() {
  const file = join(openaiDir, "issued.txt");
  return existsSync(file) ? readFileSync(file, "utf8").split("\n").filter(Boolean) : [];
}

/** The URLs the app opened in the (fake) browser. */
export function opened() {
  const file = join(fakeDir, "opened.txt");
  return existsSync(file) ? readFileSync(file, "utf8").split("\n").filter(Boolean) : [];
}
