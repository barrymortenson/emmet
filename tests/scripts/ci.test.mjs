// SPDX-License-Identifier: AGPL-3.0-or-later
// .github/workflows: the mac job reaches the self-hosted runner only for trusted events (D-13), every action is
// pinned by SHA, and scripts/ci-scope.sh runs each OS job only when a change can break it. Evaluates the real runs-on
// expression from ci.yml and runs the real scope script in a temp repo.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempRepo, writeFiles } from "./helpers.mjs";

const workflows = join(ROOT, ".github/workflows");
const ci = readFileSync(join(workflows, "ci.yml"), "utf8");
const expr = ci.match(/^ {2}mac:\n(?: {4}#.*\n)*? {4}runs-on: \$\{\{ (.+) \}\}$/m)?.[1];

// GitHub expressions in this one line use only ==, &&, ||, parentheses, '...' strings, and fromJSON. JS evaluates them
// the same way (&& and || return an operand) once == becomes eq(), which ignores case as GitHub's == does.
function runsOn(github, line = expr) {
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

test("ci.yml has a mac runs-on expression", () => assert.ok(expr, "mac job runs-on not found"));

test("Barry's PR from a branch in this repo runs on the self-hosted Mac", () => {
  assert.deepEqual(runsOn(pr(REPO, BARRY)), SELF);
});

test("a push to main by Barry runs on the self-hosted Mac", () => {
  assert.deepEqual(runsOn({ repository: REPO, actor: BARRY, event_name: "push", ref: "refs/heads/main", event: {} }), SELF);
});

test("GitHub's case-insensitive == also sends Barry's login in another case to the Mac, where the guard refuses it", () => {
  assert.deepEqual(runsOn(pr(REPO, "BarryMortenson")), SELF);
  assert.match(readFileSync(join(ROOT, "scripts/runner-guard.sh"), "utf8"), /\[\[ "\$\{GITHUB_ACTOR:-\}" == "\$owner" \]\]/);
});

for (const [why, github] of [
  ["a fork PR", pr("someone/emmet", "someone")],
  ["a fork PR Barry pushes to", pr("someone/emmet", "someone", BARRY)],
  ["a Dependabot PR", pr(REPO, "dependabot[bot]")],
  ["Barry's PR that someone else pushed to", pr(REPO, BARRY, "someone")],
  ["a manual run", { repository: REPO, actor: BARRY, event_name: "workflow_dispatch", ref: "refs/heads/main", event: {} }],
  ["a fork's own push", { repository: "someone/emmet", actor: BARRY, event_name: "push", ref: "refs/heads/main", event: {} }],
]) {
  test(`${why} runs on hosted macos-latest`, () => assert.equal(runsOn(github), "macos-latest"));
}

// nightly.yml (3A): the corpus job reaches the Mac for the schedule and Barry's own PRs, as ci.yml's mac job does.
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

test("the self-hosted mac job runs the runner guard, with the PR fields, right after checkout", () => {
  const steps = jobs.mac.split(/\n {6}- /).slice(1);
  const i = steps.findIndex((step) => /\n {10}scripts\/runner-guard\.sh\n/.test(step));
  assert.ok(i > 0, "no step runs scripts/runner-guard.sh");
  assert.match(steps[i], /if: runner\.environment == 'self-hosted'/);
  assert.match(steps[i], /EMMET_PR_HEAD_REPO: \$\{\{ github\.event\.pull_request\.head\.repo\.full_name \}\}/);
  assert.match(steps[i], /EMMET_PR_AUTHOR: \$\{\{ github\.event\.pull_request\.user\.login \}\}/);
  for (const step of steps.slice(0, i)) assert.match(step, /^uses: actions\/checkout@/, `a step runs before the guard: ${step}`);
});

test("the aggregate ci check needs every other job, runs always, and lets only the OS jobs be skipped", () => {
  const others = Object.keys(jobs).filter((j) => j !== "ci");
  const agg = jobs.ci;
  assert.match(agg, /^ {4}if: always\(\)$/m, "without always() a failed job skips ci, and a skipped required check passes");
  assert.deepEqual(agg.match(/needs: \[(.*)\]/)[1].split(/,\s*/).sort(), [...others].sort());
  assert.match(agg, /RESULTS: \$\{\{ needs\.scope\.result \}\} \$\{\{ needs\.mac\.result \}\} \$\{\{ needs\.windows\.result \}\} \$\{\{ needs\.plan\.result \}\} \$\{\{ needs\.nsis\.result \}\}\n/);
  const pattern = new RegExp(agg.match(/=~ (.+) \]\]/)[1].replaceAll("\\ ", " "));
  for (const ok of ["success success success success success", "success skipped skipped success skipped", "success success skipped success skipped"]) {
    assert.match(ok, pattern, ok);
  }
  for (const bad of ["skipped skipped skipped success skipped", "success failure skipped success skipped", "success skipped cancelled success skipped", "success success success skipped skipped", "success success success success failure"]) {
    assert.doesNotMatch(bad, pattern, bad);
  }
  for (const job of ["mac", "windows", "nsis"]) {
    assert.match(jobs[job], /^ {4}needs: scope\n {4}if: needs\.scope\.outputs\.\w+ == 'true'$/m, job);
  }
});

/** scripts/ci-scope.sh's answers after committing a base, then changing `change` (null: nothing). */
function scope(change, env = {}) {
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
  const out = Object.fromEntries(r.out.trim().split("\n").map((l) => l.split("=")));
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
  ["Cargo.lock", "full mac deps e2e_mac build_app bench"],
  [".github/workflows/ci.yml", "full mac deps e2e_mac build_app bench"],
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
  assert.equal(scope(null, { EMMET_CI_FULL: "1" }), "full mac deps e2e_mac build_app bench");
  assert.equal(
    scope(null, { EMMET_CI_FULL: "1", EMMET_CI_WINDOWS: "1" }),
    "full mac windows deps e2e_mac e2e_windows build_app nsis bench",
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
