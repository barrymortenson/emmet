// SPDX-License-Identifier: AGPL-3.0-or-later
// `just test` runs the real recipe under /bin/bash (bash 3.2 on macOS), with stub node, uv, pnpm, and cargo that log
// their arguments: every suite runs, --changed picks the right ones, and a missing test dir fails instead of passing.
import assert from "node:assert/strict";
import { chmodSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempDir, tempRepo, writeFiles } from "./helpers.mjs";

const skip = process.platform === "win32" && "the recipe tests need a Unix /bin/bash";

function project(change) {
  const { dir, git } = tempRepo();
  writeFiles(dir, {
    "tests/scripts/a.test.mjs": "",
    "packages/viewer/test/a.test.mjs": "",
    "apps/desktop/test/a.test.mjs": "",
    ".github/workflows/ci.yml": "name: ci\n",
    "rust-toolchain.toml": "",
  });
  git("add", "-A");
  git("commit", "-q", "-m", "base");
  if (change) writeFiles(dir, { [change]: "changed\n" });
  return dir;
}

function justTest(dir, ...args) {
  const bin = tempDir();
  const log = join(bin, "calls.log");
  writeFileSync(log, "");
  symlinkSync("/bin/bash", join(bin, "bash"));
  for (const tool of ["node", "uv", "pnpm", "cargo"]) {
    writeFileSync(join(bin, tool), `#!/bin/sh\necho "${tool} $*" >> "${log}"\n`);
    chmodSync(join(bin, tool), 0o755);
  }
  const just = run("sh", ["-c", "command -v just"]).out.trim();
  const r = run(just, ["--justfile", join(ROOT, "justfile"), "--working-directory", dir, "--no-deps", "test", ...args], {
    env: { PATH: `${bin}:${process.env.PATH}`, EMMET_CI_BASE: "" },
  });
  return { ...r, calls: readFileSync(log, "utf8") };
}

test("just test runs every suite under /bin/bash", { skip }, () => {
  const r = justTest(project());
  assert.equal(r.code, 0, r.out);
  assert.match(r.calls, /^node --test tests\/scripts\/\*\.test\.mjs$/m);
  assert.match(r.calls, /^uv run .* unittest discover -s python\/emmet_dsl\/tests$/m);
  assert.match(r.calls, /^pnpm --filter @emmet\/viewer --fail-if-no-match exec node --test test\/\*\.test\.mjs$/m);
  assert.match(r.calls, /^pnpm --filter @emmet\/desktop --fail-if-no-match exec node --test test\/\*\.test\.mjs$/m);
  assert.match(r.calls, /^cargo test --workspace --locked --$/m);
});

test("just test passes a name filter to every runner", { skip }, () => {
  const r = justTest(project(), "envelope");
  assert.equal(r.code, 0, r.out);
  assert.match(r.calls, /^node --test --test-name-pattern envelope tests/m);
  assert.match(r.calls, / -k envelope$/m);
  assert.match(r.calls, /^cargo test --workspace --locked -- envelope$/m);
});

for (const [change, picked] of [
  [null, "nothing"],
  [".github/workflows/ci.yml", "scripts"],
  ["rust-toolchain.toml", "scripts python viewer desktop rust"],
  ["crates/emmet-core/src/lib.rs", "rust"],
  ["docs/plan-status.js", "scripts"],
]) {
  test(`just test --changed with ${change ?? "no change"} runs ${picked}`, { skip }, () => {
    const r = justTest(project(change), "--changed");
    assert.equal(r.code, 0, r.out);
    assert.match(r.out, new RegExp(`test --changed: ${picked}\n`));
  });
}

test("just test --changed fails on a base that isn't a commit, instead of running nothing", { skip }, () => {
  const bin = tempDir();
  const r = run(run("sh", ["-c", "command -v just"]).out.trim(), ["--justfile", join(ROOT, "justfile"), "--working-directory", project(), "--no-deps", "test", "--changed"], {
    env: { PATH: `${bin}:${process.env.PATH}`, EMMET_CI_BASE: "0000000000000000000000000000000000000000" },
  });
  assert.notEqual(r.code, 0, r.out);
  assert.doesNotMatch(r.out, /test --changed: nothing/);
});

test("just test fails when a suite's tests are missing", { skip }, () => {
  const dir = project();
  rmSync(join(dir, "packages/viewer/test"), { recursive: true });
  const r = justTest(dir);
  assert.notEqual(r.code, 0, r.out);
  assert.match(r.out, /test: no tests match packages\/viewer\/test\/\*\.test\.mjs/);
  assert.doesNotMatch(r.calls, /@emmet\/viewer/);
});
