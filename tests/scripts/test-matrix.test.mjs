// SPDX-License-Identifier: AGPL-3.0-or-later
// docs/spec/test-matrix.md (session 5A): no row without a test, every named test exists, and CI runs it. It also
// checks the matrix against the plan, so a new error code, CLI command, MCP tool, or validator rule needs a row.
import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { AREAS, CORE, GATE, WINDOWS } from "../e2e/tiers.mjs";
import { ROOT } from "./helpers.mjs";

const read = (path) => readFileSync(join(ROOT, path), "utf8");
const matrix = read("docs/spec/test-matrix.md");

/** The matrix's rows: { section, behavior, tests: [{ file, name }], runsOn }. */
function rows() {
  const out = [];
  let section = "";
  let inTable = false;
  for (const line of matrix.split("\n")) {
    if (line.startsWith("## ")) section = line.slice(3);
    if (line.startsWith("| Behavior | Test | Runs on |")) inTable = true;
    else if (!line.startsWith("|")) inTable = false;
    else if (inTable && !line.startsWith("|---")) {
      const cells = line.split(" | ").map((c) => c.replace(/^\| ?| ?\|$/g, "").trim());
      const tests = [...cells[1].matchAll(/`([^`]+)` › `([^`]+)`/g)].map((m) => ({ file: m[1], name: m[2] }));
      out.push({ section, behavior: cells[0], tests, runsOn: cells[2] });
    }
  }
  return out;
}

const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** Why `name` in `file` isn't a test CI runs, or null when it is. */
function problem(file, name) {
  if (!existsSync(join(ROOT, file))) return "the file doesn't exist";
  const text = read(file);
  if (file.endsWith(".rs")) {
    const m = text.match(new RegExp(`((?:#\\[[^\\]]*\\]\\s*)+)(?:pub )?(?:async )?fn ${escape(name)}\\s*\\(`));
    if (!m || !/#\[(tokio::)?test/.test(m[1])) return "no #[test] of that name";
    // The nightly workflow runs the ignored corpus and bench tests by name; nothing runs any other ignored test.
    if (m[1].includes("#[ignore") && !read(".github/workflows/nightly.yml").includes(name)) return "the test is ignored, so CI never runs it";
    if (file.startsWith("tests/it/tests/")) {
      const mod = file.slice("tests/it/tests/".length, -3);
      if (!new RegExp(`^mod ${mod};$`, "m").test(read("tests/it/tests/main.rs"))) return "its module isn't in tests/it/tests/main.rs";
      return null;
    }
    return file.startsWith("apps/desktop/src-tauri/tests/") ? null : "not a test folder `just test` runs";
  }
  if (!file.endsWith(".mjs")) return "not a test file";
  const call = text.match(new RegExp(`(\\b[A-Za-z.]+)\\(\\s*(["'\`])${escape(name)}\\2`));
  if (!call) return "no test of that name";
  if (!["it", "test"].includes(call[1])) return `it's ${call[1]}(…), which CI doesn't run`;
  if (file.startsWith("tests/e2e/specs/")) {
    const spec = file.slice("tests/e2e/specs/".length).replace(/\.e2e\.mjs$/, "");
    const inCi = [GATE, ...CORE, ...Object.values(AREAS).flatMap((a) => a.specs)];
    return inCi.includes(spec) ? null : "its spec is in the by-hand tier (tests/e2e/tiers.mjs)";
  }
  return file.startsWith("tests/scripts/") || file.startsWith("packages/viewer/test/") ? null : "not a test folder `just test` runs";
}

/** Where a row's tests run, as the matrix's last column says it. */
function runsOn(tests) {
  const on = new Set();
  for (const { file } of tests) {
    on.add("macOS");
    if (file.startsWith("tests/scripts/")) on.add("Linux");
    else if (!file.startsWith("tests/e2e/specs/")) on.add("Windows");
    else if (WINDOWS.includes(file.slice("tests/e2e/specs/".length).replace(/\.e2e\.mjs$/, ""))) on.add("Windows");
  }
  return ["macOS", "Windows", "Linux"].filter((o) => on.has(o)).join(", ");
}

test("the matrix has no row without a test, every test it names exists, and CI runs each one", () => {
  const all = rows();
  assert.ok(all.length >= 150, `only ${all.length} rows were read: has the table's shape changed?`);
  const bad = [];
  for (const row of all) {
    if (row.tests.length === 0) bad.push(`${row.section} / ${row.behavior}: no test`);
    for (const { file, name } of row.tests) {
      const why = problem(file, name);
      if (why) bad.push(`${row.section} / ${row.behavior}: ${file} › ${name}: ${why}`);
    }
    if (row.tests.length && row.runsOn !== runsOn(row.tests)) bad.push(`${row.section} / ${row.behavior}: runs on "${runsOn(row.tests)}", not "${row.runsOn}"`);
  }
  assert.deepEqual(bad, []);
});

test("the check itself catches a missing file, a missing test, a by-hand spec, and a by-hand test", () => {
  assert.equal(problem("tests/it/tests/nothing.rs", "x"), "the file doesn't exist");
  assert.equal(problem("tests/it/tests/cli.rs", "no_such_test"), "no #[test] of that name");
  assert.match(problem("tests/e2e/specs/screens.e2e.mjs", "offline: what still works"), /by-hand tier/);
  assert.match(problem("tests/e2e/specs/chat.e2e.mjs", "stays smooth at 1,000 messages"), /extraIt/);
  assert.match(problem("tests/it/tests/validator_corpus.rs", "explain"), /ignored/);
  assert.equal(problem("tests/it/tests/validator_corpus.rs", "corpus_nightly"), null);
  assert.equal(problem("tests/it/tests/cli.rs", "version_json_is_the_success_envelope"), null);
});

/** One section of the plan, as HTML. */
function planSection(n) {
  const plan = read("docs/plan.html");
  const start = plan.indexOf(`<section id="s${n}">`);
  return plan.slice(start, plan.indexOf("</section>", start));
}

/** The <code> names in the first cell of each row of the table after `heading`. */
function firstCells(html, heading) {
  const from = html.indexOf(heading);
  assert.ok(from >= 0, `the plan has no "${heading}"`);
  const table = html.slice(from, html.indexOf("</table>", from));
  return [...table.matchAll(/<tr><td>(.*?)<\/td>/g)].map((m) => m[1]);
}

const codes = (cell) => [...cell.matchAll(/<code>([^<]+)<\/code>/g)].map((m) => m[1]);

test("every error code, CLI command, MCP tool, and validator rule in the plan has a row", () => {
  const s09 = planSection("09");
  const behaviors = rows().map((r) => r.behavior).join("\n");
  const missing = [];
  // Error codes: one or two per row ("MODEL_TIMEOUT / MODEL_TOO_LARGE", "IO_ERROR · INTERNAL").
  const errors = firstCells(s09, "<h3>Error codes</h3>").flatMap(codes);
  assert.ok(errors.length >= 14, `${errors.length} error codes read from the plan`);
  for (const code of errors) if (!new RegExp(`^${code}$`, "m").test(behaviors)) missing.push(`error code ${code}`);
  // CLI: each `emmet <command>` the table's first cells name.
  const commands = new Set(firstCells(s09, "<h3>CLI</h3>").flatMap(codes).flatMap((c) => [...c.matchAll(/^emmet ([a-z-]+)/g)].map((m) => m[1])));
  assert.ok(commands.size >= 20, `${commands.size} commands read from the plan`);
  for (const c of commands) if (!new RegExp("`emmet " + c + "\\b").test(behaviors)) missing.push(`emmet ${c}`);
  // MCP tools: the plan's two tables, and every schema the server writes.
  const tools = new Set([
    ...firstCells(s09, "<h3>MCP tools").flatMap(codes),
    ...firstCells(s09, "<tr><th>Tool (<code>emmet_host</code>)").flatMap(codes),
    ...readdirSync(join(ROOT, "docs/spec/mcp")).map((f) => f.replace(/\.schema\.json$/, "")),
  ]);
  assert.ok(tools.size >= 18, `${tools.size} tools read from the plan and docs/spec/mcp`);
  for (const t of tools) if (!behaviors.includes("`" + t + "`")) missing.push(`MCP tool ${t}`);
  // Validator rules.
  const rules = firstCells(planSection("11"), "<h3>Rules</h3>").flatMap(codes);
  assert.ok(rules.length >= 13, `${rules.length} rules read from the plan`);
  for (const r of rules) if (!behaviors.includes("`" + r + "`")) missing.push(`rule ${r}`);
  assert.deepEqual(missing, []);
});

test("every section 14 state and section 15 threat has a row or waits for a named session", () => {
  const behaviors = rows();
  const count = (prefix) => behaviors.filter((r) => r.section.startsWith(prefix)).length;
  // Section 14 lists 17 states; "agent thinking" and "building" share rows with their neighbors, and the plan limit
  // has its own.
  assert.ok(count("14 · States") >= 17, "a row per state of plan section 14");
  assert.ok(count("14 · Key flows") >= 9, "a row per key flow of plan section 14");
  // Section 15's table: each threat has rows here or a line under "Waiting for another session".
  const threats = firstCells(planSection("15"), "<tr><th>Threat</th>").length;
  const waiting = [...matrix.matchAll(/^\| (15 · [^|]+) \| ([0-9][A-Z]) \|$/gm)];
  assert.ok(count("15 ·") + waiting.length >= threats, `${threats} threats in the plan`);
});

test("a row waits only for a session that isn't done yet", () => {
  // The repo's own status file, read the way scripts/plan-check.mjs reads it.
  const window = {};
  runInNewContext(read("docs/plan-status.js"), { window });
  const waiting = [...matrix.matchAll(/^\| ([^|]+) \| ([0-9][A-Z]) \|$/gm)].map((m) => ({ behavior: m[1], session: m[2] }));
  assert.ok(waiting.length > 0 || !matrix.includes("| Behavior | Session |\n|---|---|\n|"), "the waiting table didn't parse");
  const overdue = waiting.filter((w) => ["done", "skipped"].includes(window.EMMET_STATUS.sessions[w.session]?.status));
  assert.deepEqual(overdue, [], "these sessions are done: give each row its test and move it into a table above");
});
