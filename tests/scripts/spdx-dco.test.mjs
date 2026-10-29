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

test("spdx-check wants Apache-2.0 under data/techniques and templates, and AGPL elsewhere in data", () => {
  assert.equal(spdx({ "templates/starter/model.py": APACHE }).code, 0);
  // A technique's candidate sketch becomes data/techniques/<name>.py when it's accepted (3C).
  assert.equal(spdx({ "docs/techniques/candidates/batch-1/sill.py": APACHE }).code, 0);
  assert.match(spdx({ "docs/spec/x.py": APACHE }).out, /docs\/spec\/x.py: has Apache-2.0, needs SPDX-License-Identifier: AGPL-3.0-or-later/);
  assert.match(spdx({ "data/techniques/gable.py": "# SPDX-License-Identifier: AGPL-3.0-or-later\n" }).out, /data\/techniques\/gable.py: has AGPL-3.0-or-later, needs SPDX-License-Identifier: Apache-2.0/);
  assert.match(spdx({ "data/other/x.py": APACHE }).out, /data\/other\/x.py: has Apache-2.0, needs SPDX-License-Identifier: AGPL-3.0-or-later/);
});

test("spdx-check reads the header from the first five lines only", () => {
  const r = spdx({ "crates/a/src/lib.rs": "\n\n\n\n\n" + AGPL });
  assert.equal(r.code, 1);
  assert.match(r.out, /crates\/a\/src\/lib.rs: missing/);
});

test("spdx-check covers extensionless scripts and hooks", () => {
  const r = spdx({ "scripts/name-check": "#!/usr/bin/env bash\necho hi\n" });
  assert.equal(r.code, 1);
  assert.match(r.out, /scripts\/name-check: missing/);
});

function dco(messages, author) {
  const { dir, git } = tempRepo();
  git("commit", "-q", "--allow-empty", "-m", "base");
  const base = git("rev-parse", "HEAD").trim();
  const as = author ? ["-c", `user.name=${author.name}`, "-c", `user.email=${author.email}`] : [];
  for (const m of messages) git(...as, "commit", "-q", "--allow-empty", "-m", m);
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
  assert.match(r.out, /one has no 'Signed-off-by: Test Author <test@example.com>'/);
});

test("dco-check fails when the range doesn't resolve", () => {
  const { dir, git } = tempRepo();
  git("commit", "-q", "--allow-empty", "-m", "unsigned");
  const r = run("bash", [join(ROOT, "scripts/dco-check.sh"), "1".repeat(40), "HEAD"], { cwd: dir });
  assert.equal(r.code, 1, r.out);
  assert.match(r.out, /dco-check: can't read 1{40}\.\.HEAD/);
  assert.doesNotMatch(r.out, /dco-check: ok/);
});

// What a real Dependabot commit carries: its noreply address as author, support@github.com in the sign-off.
const DEPENDABOT = { name: "dependabot[bot]", email: "49699333+dependabot[bot]@users.noreply.github.com" };

test("dco-check accepts Dependabot's own sign-off", () => {
  const r = dco(["bump\n\nSigned-off-by: dependabot[bot] <support@github.com>"], DEPENDABOT);
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /ok \(1 commits\)/);
});

test("dco-check still fails an unsigned Dependabot commit, and support@github.com from anyone else", () => {
  assert.match(dco(["bump"], DEPENDABOT).out, /bump has no 'Signed-off-by: dependabot\[bot\] <support@github.com>'/);
  const r = dco(["one\n\nSigned-off-by: dependabot[bot] <support@github.com>"]);
  assert.equal(r.code, 1, r.out);
});
