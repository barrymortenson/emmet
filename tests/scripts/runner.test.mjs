// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/runner-guard.sh refuses untrusted runs (plan section 17); scripts/runner-env.sh writes the runner's .path.
import assert from "node:assert/strict";
import { chmodSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempDir, writeFiles } from "./helpers.mjs";

const temp = tempDir();
const trusted = {
  GITHUB_REPOSITORY: "barrymortenson/emmet",
  GITHUB_ACTOR: "barrymortenson",
  GITHUB_EVENT_NAME: "pull_request",
  GITHUB_REF: "refs/pull/1/merge",
  EMMET_PR_HEAD_REPO: "barrymortenson/emmet",
  EMMET_PR_AUTHOR: "barrymortenson",
  RUNNER_TEMP: temp,
  EMMET_DATA_DIR: join(temp, "data"),
  EMMET_TEST_HOME: join(temp, "home"),
};
const guard = (env) => run("bash", [join(ROOT, "scripts/runner-guard.sh")], { env: { ...trusted, ...env } });

test("runner-guard allows Barry's own PR", () => {
  const r = guard({});
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /runner-guard: ok/);
});

for (const [why, env] of [
  ["a push to main", { GITHUB_EVENT_NAME: "push", GITHUB_REF: "refs/heads/main" }],
  ["a tag push", { GITHUB_EVENT_NAME: "push", GITHUB_REF: "refs/tags/v0.1.0" }],
  ["the nightly schedule", { GITHUB_EVENT_NAME: "schedule", GITHUB_REF: "refs/heads/main" }],
]) {
  test(`runner-guard allows ${why}`, () => {
    const r = guard({ ...env, EMMET_PR_HEAD_REPO: "", EMMET_PR_AUTHOR: "" });
    assert.equal(r.code, 0, r.out);
  });
}

for (const [why, env, message] of [
  ["a wrong actor", { GITHUB_ACTOR: "someone-else" }, /the actor is 'someone-else'/],
  ["Dependabot", { GITHUB_ACTOR: "dependabot[bot]" }, /the actor is 'dependabot\[bot\]'/],
  ["a fork", { EMMET_PR_HEAD_REPO: "someone-else/emmet" }, /the PR comes from 'someone-else\/emmet'/],
  ["a PR someone else opened", { EMMET_PR_AUTHOR: "someone-else" }, /the PR was opened by 'someone-else'/],
  ["another repository", { GITHUB_REPOSITORY: "someone-else/emmet" }, /the repository is 'someone-else\/emmet'/],
  ["an untrusted event", { GITHUB_EVENT_NAME: "pull_request_target" }, /the event 'pull_request_target' isn't trusted/],
  ["a data dir outside the temp dir", { EMMET_DATA_DIR: "/Users/someone/Library" }, /EMMET_DATA_DIR .* isn't inside RUNNER_TEMP/],
  ["a test HOME outside the temp dir", { EMMET_TEST_HOME: "" }, /EMMET_TEST_HOME .* isn't inside RUNNER_TEMP/],
  // A template string, not join(), which would normalize the .. away.
  ["a test HOME that escapes with ..", { EMMET_TEST_HOME: `${temp}/../home` }, /EMMET_TEST_HOME \('.*\/\.\.\/home'\) isn't inside RUNNER_TEMP/],
  ["a data dir in a sibling of the temp dir", { EMMET_DATA_DIR: `${temp}-evil/data` }, /EMMET_DATA_DIR .* isn't inside RUNNER_TEMP/],
  ["an empty RUNNER_TEMP", { RUNNER_TEMP: "" }, /RUNNER_TEMP isn't a directory/],
  ["a RUNNER_TEMP that doesn't exist", { RUNNER_TEMP: join(temp, "missing"), EMMET_DATA_DIR: join(temp, "missing/data"), EMMET_TEST_HOME: join(temp, "missing/home") }, /RUNNER_TEMP isn't a directory/],
  ["a push to another branch", { GITHUB_EVENT_NAME: "push", GITHUB_REF: "refs/heads/feature" }, /a push to 'refs\/heads\/feature', not main or a tag/],
  ["a manual run", { GITHUB_EVENT_NAME: "workflow_dispatch" }, /the event 'workflow_dispatch' isn't trusted/],
]) {
  test(`runner-guard refuses ${why}`, () => {
    const r = guard(env);
    assert.equal(r.code, 1, r.out);
    assert.match(r.out, /runner-guard: refusing to run:/);
    assert.match(r.out, message);
  });
}

test("runner-env writes the real tool directories into the runner's .path", { skip: process.platform === "win32" }, () => {
  const runner = tempDir();
  const bin = tempDir();
  const shims = join(tempDir(), "shims");
  const tools = ["cargo", "pnpm", "uv", "just", "gitleaks", "cargo-deny", "gh"];
  writeFiles(bin, Object.fromEntries(tools.map((t) => [t, "#!/bin/sh\n"])));
  writeFiles(shims, { pnpm: "#!/bin/sh\n" });
  for (const t of tools) chmodSync(join(bin, t), 0o755);
  chmodSync(join(shims, "pnpm"), 0o755);
  const nodeDir = join(process.execPath, "..");
  const r = run("bash", [join(ROOT, "scripts/runner-env.sh"), runner], { env: { PATH: `${shims}:${bin}:${nodeDir}:/usr/bin:/bin` } });
  assert.equal(r.code, 0, r.out);
  const path = readFileSync(join(runner, ".path"), "utf8").trim().split(":");
  assert.ok(path.includes(bin), path.join(":"));
  assert.ok(path.includes(nodeDir), "node's real directory");
  assert.ok(!path.includes(shims), "shims are skipped");
  assert.ok(path.includes("/usr/bin"));
});

test("runner-env refuses a runner dir that doesn't exist", { skip: process.platform === "win32" }, () => {
  const r = run("bash", [join(ROOT, "scripts/runner-env.sh"), join(tempDir(), "missing")]);
  assert.equal(r.code, 1);
  assert.match(r.out, /register the runner first/);
});
