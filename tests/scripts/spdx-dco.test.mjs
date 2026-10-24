// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/spdx-check.mjs and scripts/dco-check.sh: each passes on good input and fails when broken.
import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempRepo, writeFiles } from "./helpers.mjs";

const AGPL = "// SPDX-License-Identifier: AGPL-3.0-or-later\n";
const APACHE = "# SPDX-License-Identifier: Apache-2.0\n";

function spdx(files) {
  const { dir, git } = tempRepo();
  writeFiles(dir, files);
  git("add", "-A");
  return run("node", [join(ROOT, "scripts/spdx-check.mjs"), dir]);
}

test("spdx-check passes on the repo itself", () => {
  const r = run("node", ["scripts/spdx-check.mjs"], { cwd: ROOT });
  assert.equal(r.code, 0, r.out);
});

test("spdx-check passes when each file has the header for its path", () => {
  const r = spdx({ "crates/a/src/lib.rs": AGPL, "python/emmet_dsl/src/x.py": APACHE, "README.md": "no header needed" });
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /ok \(2 source files\)/);
});

test("spdx-check fails on a source file without a header", () => {
  const r = spdx({ "crates/a/src/lib.rs": "fn main() {}\n" });
  assert.equal(r.code, 1);
  assert.match(r.out, /crates\/a\/src\/lib.rs: missing SPDX-License-Identifier: AGPL-3.0-or-later/);
});

test("spdx-check fails on the wrong license for the path", () => {
  const r = spdx({ "python/emmet_dsl/src/x.py": "# SPDX-License-Identifier: AGPL-3.0-or-later\n" });
  assert.equal(r.code, 1);
  assert.match(r.out, /has AGPL-3.0-or-later, needs SPDX-License-Identifier: Apache-2.0/);
});

test("spdx-check covers extensionless scripts and hooks", () => {
  const r = spdx({ "scripts/name-check": "#!/usr/bin/env bash\necho hi\n" });
  assert.equal(r.code, 1);
  assert.match(r.out, /scripts\/name-check: missing/);
});

function dco(messages) {
  const { dir, git } = tempRepo();
  git("commit", "-q", "--allow-empty", "-m", "base");
  const base = git("rev-parse", "HEAD").trim();
  for (const m of messages) git("commit", "-q", "--allow-empty", "-m", m);
  return run("bash", [join(ROOT, "scripts/dco-check.sh"), base, "HEAD"], { cwd: dir });
}

test("dco-check passes when every commit is signed off by its author", () => {
  const r = dco(["one\n\nSigned-off-by: Test Author <test@example.com>", "two\n\nSigned-off-by: Test Author <test@example.com>"]);
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /ok \(2 commits\)/);
});

test("dco-check fails on a commit without a sign-off", () => {
  const r = dco(["one\n\nSigned-off-by: Test Author <test@example.com>", "two"]);
  assert.equal(r.code, 1);
  assert.match(r.out, /two has no 'Signed-off-by: Test Author <test@example.com>'/);
  assert.match(r.out, /1 of 2 commits/);
});

test("dco-check fails on a sign-off that doesn't match the author", () => {
  const r = dco(["one\n\nSigned-off-by: Someone Else <else@example.com>"]);
  assert.equal(r.code, 1);
});
