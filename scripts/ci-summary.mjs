#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// A CI job's last step (plan section 16): its time against its budget, the E2E spec files that needed their one
// retry, and any quarantined test. `node scripts/ci-summary.mjs <job> <seconds>` writes to the job summary
// ($GITHUB_STEP_SUMMARY, or stdout), warns when the job is over its budget, and fails when it's 50 % over. A hosted
// runner's speed varies from run to run, which is what the 50 % is for; a job that's over it has grown, and a test
// moves to the by-hand tier or gets faster.
import { appendFileSync, existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");

/** Each job's budget in minutes: 10 for a PR's macOS jobs, 15 for Windows. The E2E shards (mac-e2e-core,
 * mac-e2e-areas) are jobs of their own. */
export const BUDGET = { mac: 10, "mac-e2e": 10, windows: 15, "windows-e2e": 15 };

const clock = (s) => `${Math.floor(s / 60)}m ${Math.round(s % 60)}s`;

/** The budget for `job` ("mac-e2e-core" has mac-e2e's), or undefined for a job without one. */
export function budgetOf(job) {
  const name = Object.keys(BUDGET)
    .sort((a, b) => b.length - a.length)
    .find((k) => job === k || job.startsWith(`${k}-`));
  return name && BUDGET[name];
}

/** Where `seconds` puts `job`: ok, warn (over its budget), or fail (over it by more than half). */
export function verdict(job, seconds) {
  const minutes = budgetOf(job);
  if (minutes === undefined) return { level: "ok", line: `${job} job: ${clock(seconds)} (no budget)` };
  const budget = minutes * 60;
  const level = seconds > budget * 1.5 ? "fail" : seconds > budget ? "warn" : "ok";
  const words = { ok: "within its budget", warn: "over its budget", fail: "more than 50 % over its budget" }[level];
  return { level, line: `${job} job: ${clock(seconds)}, ${words} of ${minutes}m` };
}

/** The spec files that ran a second time, from tests/e2e/out/retries.jsonl (wdio.conf.mjs writes it). */
export function retries(text) {
  return text
    .split("\n")
    .filter((l) => l.trim())
    .map((l) => JSON.parse(l))
    .map((r) => `${r.spec}: ${r.passed ? "passed on its retry" : "failed twice"}`);
}

/** Every quarantined test in the E2E specs: [{ spec, issue, title }]. An `issue` that isn't a string is a call the
 * check in tests/scripts/ci.test.mjs fails. */
export function quarantined(dir = join(ROOT, "tests/e2e/specs")) {
  const out = [];
  for (const file of readdirSync(dir).filter((f) => f.endsWith(".e2e.mjs")).sort()) {
    const text = readFileSync(join(dir, file), "utf8");
    for (const m of text.matchAll(/\bquarantine\(\s*(?:(["'`])(.*?)\1\s*,\s*(["'`])(.*?)\3)?/g)) {
      out.push({ spec: file.replace(/\.e2e\.mjs$/, ""), issue: m[2] ?? null, title: m[4] ?? null });
    }
  }
  return out;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [job, secs] = process.argv.slice(2);
  const seconds = Number(secs);
  if (!job || !Number.isFinite(seconds)) {
    console.error("usage: node scripts/ci-summary.mjs <job> <seconds>");
    process.exit(2);
  }
  const { level, line } = verdict(job, seconds);
  const lines = [line];
  if (job.includes("e2e")) {
    const file = join(process.env.EMMET_E2E_OUT ?? join(ROOT, "tests/e2e/out"), "retries.jsonl");
    // wdio makes the file when it starts (tests/e2e/wdio.conf.mjs): without it, no spec ran, which isn't "none".
    if (!existsSync(file)) lines.push("E2E retries: not recorded (the specs didn't start)");
    else {
      const again = retries(readFileSync(file, "utf8"));
      lines.push(again.length ? `E2E retries (one is allowed, plan section 16): ${again.join("; ")}` : "E2E retries: none");
    }
    const q = quarantined();
    lines.push(q.length ? `Quarantined: ${q.map((t) => `${t.spec} › ${t.title} (${t.issue})`).join("; ")}` : "Quarantined: none");
  }
  const text = lines.map((l) => `${l}\n`).join("\n");
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${text}\n`);
  else process.stdout.write(text);
  if (level === "warn") console.log(`::warning title=CI time budget::${line}`);
  if (level === "fail") {
    console.log(`::error title=CI time budget::${line}`);
    process.exit(1);
  }
}
