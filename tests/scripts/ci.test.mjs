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

// GitHub expressions in this one line use only ==, &&, ||, parentheses, '...' strings, and fromJSON; JS evaluates them
// the same way (&& and || return an operand).
function runsOn(github) {
  const js = expr.replaceAll("==", "===").replaceAll("fromJSON", "JSON.parse");
  return new Function("github", `return ${js};`)(github);
}

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
