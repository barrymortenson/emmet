// SPDX-License-Identifier: AGPL-3.0-or-later
// .github/workflows: ci.yml's Mac jobs run on GitHub's pinned macos-15 (Barry, 2026-11-06), nightly's corpus job reaches
// the self-hosted runner only for trusted events (D-13), every action is pinned by SHA, and scripts/ci-scope.sh runs
// each OS job only when a change can break it. Evaluates nightly's real runs-on expression and runs the real scope
// script in a temp repo.
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempRepo, writeFiles } from "./helpers.mjs";
import { BUDGET, budgetOf, quarantined, retries, verdict } from "../../scripts/ci-summary.mjs";
import { CORE as CORE_SPECS, ISSUE, SHARDS, retried, shard } from "../e2e/tiers.mjs";

const workflows = join(ROOT, ".github/workflows");
const ci = readFileSync(join(workflows, "ci.yml"), "utf8");
// GitHub expressions in this one line use only ==, &&, ||, parentheses, '...' strings, and fromJSON. JS evaluates them
// the same way (&& and || return an operand) once == becomes eq(), which ignores case as GitHub's == does.
function runsOn(github, line) {
  const js = line
    .replace(/([\w.]+|'[^']*') == ([\w.]+|'[^']*')/g, "eq($1, $2)")
    .replaceAll("fromJSON", "JSON.parse");
  const eq = (a, b) => String(a).toLowerCase() === String(b).toLowerCase();
  return new Function("github", "eq", `return ${js};`)(github, eq);
}

/** Each top-level job's id and its lines of YAML. */
const jobs = Object.fromEntries(
  ci.slice(ci.indexOf("\njobs:\n")).split(/\n(?= {2}[\w-]+:\n)/).slice(1).map((block) => [block.match(/^ {2}([\w-]+):/)[1], block]),
);

const BARRY = "barrymortenson";
const REPO = "barrymortenson/emmet";
const pr = (headRepo, author, actor = author) => ({
  repository: REPO,
  actor,
  event_name: "pull_request",
  ref: "refs/pull/1/merge",
  event: { pull_request: { head: { repo: { full_name: headRepo } }, user: { login: author } } },
});
const SELF = ["self-hosted", "macOS", "emmet"];

// A hosted runner takes every event, so no event of anyone's reaches Barry's Mac through ci.yml.
for (const job of ["mac", "mac-e2e"]) {
  test(`ci.yml's ${job} job runs on hosted macos-15, never the self-hosted Mac`, () => {
    assert.match(jobs[job], /^ {4}runs-on: macos-15$/m);
    assert.doesNotMatch(jobs[job], /self-hosted|runner-guard/);
  });
}
test("no ci.yml job runs on the self-hosted Mac", () => assert.doesNotMatch(ci, /self-hosted/));

// nightly.yml (3A): the corpus job reaches the Mac for the schedule and Barry's own PRs.
const nightly = readFileSync(join(workflows, "nightly.yml"), "utf8");
const nightlyExpr = nightly.match(/^ {2}corpus:\n(?: {4}#.*\n)*? {4}runs-on: \$\{\{ (.+) \}\}$/m)?.[1];
test("nightly.yml's corpus job has a runs-on expression", () => assert.ok(nightlyExpr, "corpus job runs-on not found"));
// Without bash's pipefail, `cargo test … | tee` passes whatever the test does.
test("nightly.yml's steps run in bash with pipefail", () =>
  assert.match(nightly, /^ {4}defaults:\n {6}run:\n {8}shell: bash$/m));
test("the nightly schedule and Barry's PR run on the self-hosted Mac", () => {
  assert.deepEqual(runsOn({ repository: REPO, actor: BARRY, event_name: "schedule", ref: "refs/heads/main", event: {} }, nightlyExpr), SELF);
  assert.deepEqual(runsOn(pr(REPO, BARRY), nightlyExpr), SELF);
});
for (const [why, github] of [
  ["a fork PR", pr("someone/emmet", "someone")],
  ["Barry's PR that someone else pushed to", pr(REPO, BARRY, "someone")],
  ["a manual run", { repository: REPO, actor: BARRY, event_name: "workflow_dispatch", ref: "refs/heads/main", event: {} }],
  ["a fork's schedule", { repository: "someone/emmet", actor: BARRY, event_name: "schedule", ref: "refs/heads/main", event: {} }],
]) {
  test(`nightly: ${why} runs on hosted macos-latest`, () => assert.equal(runsOn(github, nightlyExpr), "macos-latest"));
}

// No YAML parser without a new dependency, so this checks the one rule that bit us: a plain (unquoted) scalar can't
// contain ": ". GitHub rejected ci.yml for it on 2026-10-24.
test("no workflow or Dependabot value is a plain scalar containing ': '", () => {
  const files = [...readdirSync(workflows).map((f) => join(workflows, f)), join(ROOT, ".github/dependabot.yml")];
  for (const file of files) {
    readFileSync(file, "utf8").split("\n").forEach((line, i) => {
      const value = line.match(/^\s*(?:- )?[\w-]+: (.*)$/)?.[1]?.replace(/\s+#.*$/, "");
      if (!value || /^["'|>{[]/.test(value)) return;
      assert.ok(!value.includes(": "), `${file}:${i + 1}: quote this value or use a block scalar`);
    });
  }
});

test("every action in every workflow is pinned by a full commit SHA", () => {
  for (const file of readdirSync(workflows)) {
    const text = readFileSync(join(workflows, file), "utf8");
    for (const [, ref] of text.matchAll(/uses: ([^\s#]+)/g)) {
      if (ref.startsWith("./")) continue;
      assert.match(ref, /@[0-9a-f]{40}$/, `${file}: ${ref}`);
    }
  }
});

test("the aggregate ci check needs every other job, runs always, and lets only the OS jobs be skipped", () => {
  const others = Object.keys(jobs).filter((j) => j !== "ci");
  const agg = jobs.ci;
  assert.match(agg, /^ {4}if: always\(\)$/m, "without always() a failed job skips ci, and a skipped required check passes");
  assert.deepEqual(agg.match(/needs: \[(.*)\]/)[1].split(/,\s*/).sort(), [...others].sort());
  const order = ["scope", "mac", "mac-e2e", "windows", "windows-e2e", "plan", "nsis"];
  assert.equal(agg.match(/RESULTS: (.*)\n/)[1], order.map((j) => `\${{ needs.${j}.result }}`).join(" "));
  const pattern = new RegExp(agg.match(/=~ (.+) \]\]/)[1].replaceAll("\\ ", " "));
  for (const ok of ["success success success success success success success", "success skipped skipped skipped skipped success skipped", "success success skipped success skipped success skipped"]) {
    assert.match(ok, pattern, ok);
  }
  for (const bad of ["skipped skipped skipped skipped skipped success skipped", "success failure skipped skipped skipped success skipped", "success success failure skipped skipped success skipped", "success skipped skipped cancelled skipped success skipped", "success success success success failure success skipped", "success success success success success skipped skipped", "success success success success success success failure"]) {
    assert.doesNotMatch(bad, pattern, bad);
  }
  for (const job of ["mac", "mac-e2e", "windows", "windows-e2e", "nsis"]) {
    assert.match(jobs[job], /^ {4}needs: scope\n {4}if: needs\.scope\.outputs\.\w+ == 'true'$/m, job);
  }
});

/** scripts/ci-scope.sh's output after committing a base, then changing `change` (null: nothing). */
function scopeOut(change, env = {}) {
  const { dir, git } = tempRepo();
  writeFiles(dir, { "README.md": "", "crates/a.rs": "" });
  git("add", "-A");
  git("commit", "-q", "-m", "base");
  if (change) writeFiles(dir, { [change]: "changed\n" });
  const r = run("bash", [join(ROOT, "scripts/ci-scope.sh")], {
    cwd: dir,
    env: { EMMET_CI_BASE: "", EMMET_CI_FULL: "", EMMET_CI_WINDOWS: "", ...env },
  });
  assert.equal(r.code, 0, r.out);
  return Object.fromEntries(r.out.trim().split("\n").map((l) => l.split("=")));
}

/** The jobs and steps scripts/ci-scope.sh turns on for `change`. */
function scope(change, env = {}) {
  const out = scopeOut(change, env);
  return Object.keys(out).filter((k) => out[k] === "true").join(" ") || "nothing";
}

for (const [change, runs] of [
  [null, "nothing"],
  ["docs/plan-status.js", "nothing"],
  ["README.md", "nothing"],
  ["docs/spec/mcp/tools.json", "mac"],
  ["docs/design/frozen/components.css", "mac"],
  ["docs/design/components.md", "mac"],
  ["scripts/plan-check.mjs", "mac"],
  ["apps/desktop/src/App.tsx", "mac e2e_mac"],
  ["packages/viewer/src/index.ts", "mac e2e_mac"],
  ["crates/emmet-core/src/lib.rs", "mac e2e_mac"],
  ["python/emmet_dsl/src/emmet_dsl/model.py", "mac e2e_mac"],
  ["apps/desktop/src-tauri/src/main.rs", "mac e2e_mac build_app"],
  ["tests/e2e/specs/viewer.e2e.mjs", "mac e2e_mac"],
  ["crates/emmet-agent/src/session.rs", "mac e2e_mac"],
  ["tests/fake-agents/src/lib.rs", "mac e2e_mac"],
  ["tests/replay/claude/plain-turn.jsonl", "mac e2e_mac"],
  ["deny.toml", "mac deps"],
  ["scripts/release-dry.sh", "mac"],
  ["apps/desktop/src-tauri/tauri.release.conf.json", "mac e2e_mac build_app"],
  ["Cargo.lock", "full mac deps e2e_mac build_app"],
  // How CI runs: the checks and core E2E, not everything (plan section 16).
  [".github/workflows/ci.yml", "mac e2e_mac"],
  ["justfile", "mac e2e_mac"],
]) {
  test(`ci scope: ${change ?? "no change"} runs ${runs}`, { skip: process.platform === "win32" }, () => {
    assert.equal(scope(change), runs);
  });
}

// Windows is paused (D-43): the same rules, only with EMMET_CI_WINDOWS=1.
for (const [change, runs] of [
  ["crates/emmet-core/src/lib.rs", "mac windows e2e_mac"],
  ["crates/emmet-agent/src/session.rs", "mac windows e2e_mac e2e_windows"],
  ["scripts/release-dry.sh", "mac nsis"],
]) {
  test(`ci scope with Windows on: ${change} runs ${runs}`, { skip: process.platform === "win32" }, () => {
    assert.equal(scope(change, { EMMET_CI_WINDOWS: "1" }), runs);
  });
}

test("ci scope: EMMET_CI_FULL=1 runs everything, and EMMET_CI_BASE limits the changes to those after it", { skip: process.platform === "win32" }, () => {
  assert.equal(scope(null, { EMMET_CI_FULL: "1" }), "full mac deps e2e_mac build_app");
  assert.equal(
    scope(null, { EMMET_CI_FULL: "1", EMMET_CI_WINDOWS: "1" }),
    "full mac windows deps e2e_mac e2e_windows build_app nsis",
  );
  const { dir, git } = tempRepo();
  writeFiles(dir, { "crates/a.rs": "" });
  git("add", "-A");
  git("commit", "-q", "-m", "base");
  const base = git("rev-parse", "HEAD").trim();
  writeFiles(dir, { "crates/a.rs": "changed\n" });
  git("commit", "-q", "-am", "rust");
  const green = git("rev-parse", "HEAD").trim();
  writeFiles(dir, { "docs/plan-status.js": "" });
  git("add", "-A");
  git("commit", "-q", "-m", "status");
  const at = (b) =>
    run("bash", [join(ROOT, "scripts/ci-scope.sh")], { cwd: dir, env: { EMMET_CI_BASE: b, EMMET_CI_FULL: "" } }).out;
  assert.match(at(base), /^mac=true$/m);
  assert.match(at(green), /^mac=false$/m);
  const bad = run("bash", [join(ROOT, "scripts/ci-scope.sh")], { cwd: dir, env: { EMMET_CI_BASE: "0".repeat(40), EMMET_CI_FULL: "" } });
  assert.notEqual(bad.code, 0, "a base that isn't a commit must fail, not scope to nothing");
});

// The E2E specs come from tests/e2e/tiers.mjs: core for anything that reaches the app, plus the areas that changed.
const CORE = "hello,viewer,panel,chat,checks,trust,runs,versions,import,render,visual,states";
for (const [change, specs] of [
  ["README.md", ""],
  ["scripts/plan-check.mjs", ""],
  ["packages/viewer/src/index.ts", `${CORE},interact,animate,perf,instructions`],
  ["crates/emmet-core/src/instructions/layout.rs", `${CORE},instructions`],
  ["apps/desktop/src-tauri/src/projects.rs", `${CORE},instructions`],
  ["crates/emmet-core/src/lib.rs", CORE],
  ["crates/emmet-agent/src/session.rs", `${CORE},chatgpt,setup,bus,photo,dock`],
  ["apps/desktop/src/Panel.tsx", `${CORE},keyboard`],
  ["apps/desktop/src/FirstRun.tsx", `${CORE},keyboard,first-run`],
  ["tests/e2e/specs/leak.e2e.mjs", `${CORE},leak`],
  [".github/workflows/ci.yml", CORE],
]) {
  test(`ci scope: ${change} runs E2E ${specs || "nothing"}`, { skip: process.platform === "win32" }, () => {
    assert.equal(scopeOut(change).e2e_specs, specs);
  });
}

test("ci scope: a full run runs core and every area, not the by-hand tier", { skip: process.platform === "win32" }, () => {
  const specs = scopeOut(null, { EMMET_CI_FULL: "1" }).e2e_specs.split(",");
  for (const s of ["hello", "viewer", "interact", "perf", "chat", "chatgpt", "keyboard", "first-run"]) assert.ok(specs.includes(s), s);
  for (const s of ["leak", "screens", "bench"]) assert.ok(!specs.includes(s), s);
});

test("DCO runs on every event but a push to main, so the hosted fallback's ci check covers sign-offs", () => {
  const plan = jobs.plan;
  assert.match(plan, /- name: DCO\n {8}if: github\.event_name != 'push'\n/);
  assert.match(plan, /BASE: \$\{\{ github\.event\.pull_request\.base\.sha \|\| 'origin\/main' \}\}/);
});

test("both OS jobs keep the integration tests' data dir and home in the runner's temp dir, out of the rust cache", () => {
  for (const job of ["mac", "windows"]) {
    assert.match(jobs[job], /EMMET_DATA_DIR=%s\\nEMMET_TEST_HOME=%s\\n' "\$RUNNER_TEMP\/emmet-data" "\$RUNNER_TEMP\/home"/, job);
  }
});

// The flake policy and the time budgets (plan section 16, session 5A).
test("a job is within its budget up to it, warned over it, and failed 50 % over it", () => {
  assert.deepEqual(BUDGET, { mac: 10, "mac-e2e": 10, windows: 15, "windows-e2e": 15 });
  assert.equal(verdict("mac", 10 * 60).level, "ok");
  assert.equal(verdict("mac", 10 * 60 + 1).level, "warn");
  assert.equal(verdict("mac", 15 * 60).level, "warn");
  assert.equal(verdict("mac", 15 * 60 + 1).level, "fail");
  assert.equal(verdict("windows", 22 * 60).level, "warn");
  assert.equal(verdict("windows", 23 * 60).level, "fail");
  // An E2E shard is a job of its own, with the E2E job's budget; windows-e2e isn't a shard of windows.
  assert.equal(budgetOf("mac-e2e-core"), 10);
  assert.equal(budgetOf("mac-e2e-areas"), 10);
  assert.equal(budgetOf("windows-e2e"), 15);
  assert.equal(verdict("mac-e2e-areas", 9 * 60 + 30).line, "mac-e2e-areas job: 9m 30s, within its budget of 10m");
});

test("the summary step warns over the budget, fails 50 % over it, and says so in the job summary", () => {
  const dir = mkdtempSync(join(tmpdir(), "emmet-ci-summary-"));
  const summary = (job, seconds, e2eOut = dir) => {
    const file = join(dir, `${job}-${seconds}.md`);
    writeFileSync(file, "");
    const r = run("node", [join(ROOT, "scripts/ci-summary.mjs"), job, String(seconds)], { env: { GITHUB_STEP_SUMMARY: file, EMMET_E2E_OUT: e2eOut } });
    return { ...r, text: readFileSync(file, "utf8") };
  };
  const ok = summary("mac", 200);
  assert.equal(ok.code, 0);
  assert.match(ok.text, /^mac job: 3m 20s, within its budget of 10m$/m);
  assert.doesNotMatch(ok.out, /::warning|::error/);
  const warn = summary("mac", 700);
  assert.equal(warn.code, 0, "over the budget is a warning, not a failure");
  assert.match(warn.out, /^::warning title=CI time budget::mac job: 11m 40s, over its budget of 10m$/m);
  const fail = summary("mac", 901);
  assert.equal(fail.code, 1);
  assert.match(fail.out, /^::error title=CI time budget::/m);
  assert.match(fail.text, /more than 50 % over its budget of 10m/);
  // An E2E job also says which spec files ran again and what's quarantined: today, nothing.
  // No retries file: wdio never started (a failed build, a locked screen), which isn't "none".
  const e2e = summary("mac-e2e-core", 300);
  assert.match(e2e.text, /^E2E retries: not recorded \(the specs didn't start\)$/m);
  assert.match(e2e.text, /^Quarantined: none$/m);
  // wdio makes the file when it starts; empty is a clean run, and a line is a retry.
  writeFileSync(join(dir, "retries.jsonl"), "");
  assert.match(summary("mac-e2e-core", 301).text, /^E2E retries: none$/m);
  writeFileSync(join(dir, "retries.jsonl"), '{"spec":"chat","passed":true}\n');
  assert.match(summary("mac-e2e-core", 302).text, /^E2E retries \(one is allowed, plan section 16\): chat: passed on its retry$/m);
});

test("every OS job ends with the summary step, and the E2E jobs' retries reach it", () => {
  for (const job of ["mac", "mac-e2e", "windows", "windows-e2e"]) {
    assert.match(jobs[job], /- name: Summary\n {8}if: always\(\)\n {8}run: node scripts\/ci-summary\.mjs .+ \$\(\( \$\(date \+%s\) - CI_START \)\)\n/, job);
  }
  assert.deepEqual(retries('{"spec":"chat","passed":true}\n{"spec":"panel","passed":false}\n'), ["chat: passed on its retry", "panel: failed twice"]);
  // One retry, in E2E only: wdio retries a spec file once, and nothing retries a Rust, script, or viewer test.
  const wdio = readFileSync(join(ROOT, "tests/e2e/wdio.conf.mjs"), "utf8");
  assert.match(wdio, /specFileRetries: bench \|\| firstRun \|\| gate \? 0 : 1,/);
  assert.match(wdio, /onWorkerEnd\(_cid, exitCode, specs, retries\)/);
  assert.match(wdio, /retried\(failedOnce, basename\(specs\[0\] \?\? "", "\.e2e\.mjs"\), exitCode, retries\)/);
  // wdio hands the hook the retries a spec file has left, a number: 1 on a first run, 0 on the run after a failure,
  // and always 0 for a spec that gets no retry.
  const failedOnce = new Set();
  assert.equal(retried(failedOnce, "viewer", 0, 1), null, "passed first time");
  assert.equal(retried(failedOnce, "chat", 1, 1), null, "failed once: wdio runs it again");
  assert.deepEqual(retried(failedOnce, "chat", 0, 0), { spec: "chat", passed: true });
  assert.equal(retried(failedOnce, "panel", 1, 1), null);
  assert.deepEqual(retried(failedOnce, "panel", 1, 0), { spec: "panel", passed: false });
  assert.equal(retried(failedOnce, "hello", 1, 0), null, "the gate gets no retry, so its failure isn't one");
  assert.equal(retried(failedOnce, "chat", 0, 1), null, "a later run of the same spec starts clean");
  assert.doesNotMatch(readFileSync(join(ROOT, "justfile"), "utf8"), /--retries|retry=|nextest/);
});

test("a quarantined test needs its issue's link, and nothing is quarantined now", () => {
  assert.deepEqual(quarantined(), [], "a quarantined test is fixed within the phase; none may be left at a session's merge");
  const dir = mkdtempSync(join(tmpdir(), "emmet-quarantine-"));
  writeFileSync(
    join(dir, "a.e2e.mjs"),
    'quarantine("https://github.com/barrymortenson/emmet/issues/62", "stops the turn at once", async () => {});\nquarantine("flaky on CI", "another", async () => {});\nquarantine(async () => {});\n',
  );
  const found = quarantined(dir);
  assert.deepEqual(
    found.map((q) => [q.spec, q.title, ISSUE.test(q.issue ?? "")]),
    [
      ["a", "stops the turn at once", true],
      ["a", "another", false],
      ["a", null, false],
    ],
  );
  // What this test asserts of the real specs, said once: every quarantine carries an issue of this repo.
  for (const q of quarantined()) assert.match(q.issue ?? "", ISSUE, `${q.spec}: ${q.title}`);
});

test("the Mac E2E shards split a run in two: core, and everything else, each behind hello", { skip: process.platform === "win32" }, () => {
  assert.deepEqual(SHARDS, ["core", "areas"]);
  const full = scopeOut(null, { EMMET_CI_FULL: "1" });
  assert.equal(full.e2e_shards, '["core","areas"]');
  const specs = full.e2e_specs.split(",");
  const [core, areas] = SHARDS.map((s) => shard(specs, s));
  assert.deepEqual(core, ["hello", ...CORE_SPECS]);
  assert.equal(areas[0], "hello");
  assert.equal(areas.at(-1), "first-run");
  // Every spec is in exactly one shard.
  assert.deepEqual([...core.slice(1), ...areas.slice(1)].sort(), specs.filter((s) => s !== "hello").sort());
  // A change that runs only core starts one job, and the other shard has nothing to run.
  assert.equal(scopeOut("crates/emmet-core/src/lib.rs").e2e_shards, '["core"]');
  assert.deepEqual(shard(scopeOut("crates/emmet-core/src/lib.rs").e2e_specs.split(","), "areas"), []);
  assert.equal(scopeOut("crates/emmet-agent/src/session.rs").e2e_shards, '["core","areas"]');
  // The workflow runs one job per shard and hands each its name.
  assert.match(jobs["mac-e2e"], /shard: \$\{\{ fromJSON\(needs\.scope\.outputs\.e2e_shards\) \}\}/);
  assert.match(jobs["mac-e2e"], /EMMET_E2E_SHARD: \$\{\{ matrix\.shard \}\}/);
});
