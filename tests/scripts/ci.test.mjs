// SPDX-License-Identifier: AGPL-3.0-or-later
// .github/workflows: the mac job reaches the self-hosted runner only for trusted events (D-13), and every action is
// pinned by SHA. Evaluates the real runs-on expression from ci.yml.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT } from "./helpers.mjs";

const workflows = join(ROOT, ".github/workflows");
const ci = readFileSync(join(workflows, "ci.yml"), "utf8");
const expr = ci.match(/^ {2}mac:\n(?: {4}#.*\n)*? {4}runs-on: \$\{\{ (.+) \}\}$/m)?.[1];

// GitHub expressions in this one line use only ==, &&, ||, parentheses, '...' strings, and fromJSON. JS evaluates them
// the same way (&& and || return an operand) once == becomes eq(), which ignores case as GitHub's == does.
function runsOn(github) {
  const js = expr
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

test("the aggregate ci check needs every other job, runs always, and passes only when each succeeded", () => {
  const others = Object.keys(jobs).filter((j) => j !== "ci");
  const agg = jobs.ci;
  assert.match(agg, /^ {4}if: always\(\)$/m, "without always() a failed job skips ci, and a skipped required check passes");
  assert.deepEqual(agg.match(/needs: \[(.*)\]/)[1].split(/,\s*/).sort(), [...others].sort());
  for (const j of others) assert.match(agg, new RegExp(`\\$\\{\\{ needs\\.${j}\\.result \\}\\}`));
  assert.match(agg, new RegExp(`== "${others.map(() => "success").join(" ")}"`));
});

test("DCO runs on every event but a push to main, so the hosted fallback's ci check covers sign-offs", () => {
  const plan = jobs.plan;
  assert.match(plan, /- name: DCO\n {8}if: github\.event_name != 'push'\n/);
  assert.match(plan, /BASE: \$\{\{ github\.event\.pull_request\.base\.sha \|\| 'origin\/main' \}\}/);
});

test("each OS job skips its work when the same code already passed on that OS, and saves the marker only when it ran", () => {
  const always = { mac: ["Runner guard", "Install gitleaks", "Timing"], windows: ["Timing"] };
  for (const job of ["mac", "windows"]) {
    const steps = jobs[job].split(/\n(?= {6}- )/).slice(1);
    const at = steps.findIndex((s) => s.includes("name: Already passed?"));
    assert.ok(at > 0, `${job}: no pass-marker lookup`);
    assert.match(steps[at], /key: ci-pass-\$\{\{ runner\.os \}\}-\$\{\{ steps\.inputs\.outputs\.hash \}\}/);
    assert.match(steps[at], /lookup-only: true/);
    assert.match(steps[at - 1], /run: echo "hash=\$\(scripts\/ci-inputs-hash\)" >> "\$GITHUB_OUTPUT"/);
    for (const step of steps.slice(at + 1)) {
      const name = step.match(/name: (.+)/)?.[1];
      if (always[job].includes(name) || name === "Secrets") continue;
      assert.match(step, /if: .*steps\.passed\.outputs\.cache-hit != 'true'/, `${job}: this step runs even when skipped:\n${step}`);
    }
    const save = steps.find((s) => s.includes("actions/cache/save@"));
    assert.ok(save && /key: ci-pass-\$\{\{ runner\.os \}\}-\$\{\{ steps\.inputs\.outputs\.hash \}\}/.test(save), `${job}: no marker save`);
    assert.ok(steps.indexOf(save) > steps.findIndex((s) => /run: just (ci|e2e)/.test(s)), `${job}: the marker is saved before the work`);
  }
  // A skipped mac run still scans for secrets.
  assert.match(jobs.mac, /name: Secrets\n {8}if: steps\.passed\.outputs\.cache-hit == 'true'\n {8}run: gitleaks git/);
});
