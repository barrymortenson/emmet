// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/ci-inputs-hash: the key for "this code already passed on this OS". Progress-only files leave it alone;
// anything a test can read changes it.
import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempRepo, writeFiles } from "./helpers.mjs";

const script = join(ROOT, "scripts/ci-inputs-hash");

function repo() {
  const r = tempRepo();
  writeFiles(r.dir, {
    "crates/a.rs": "fn main() {}\n",
    "docs/plan-status.js": "window.EMMET_STATUS = {};\n",
    "docs/decisions.md": "# Decisions\n",
    "docs/spec/dsl.md": "# DSL\n",
    "CHANGELOG.md": "# Changelog\n",
  });
  r.git("add", "-A");
  r.git("commit", "-qm", "start");
  const hash = () => {
    const h = run(script, [r.dir]);
    assert.equal(h.code, 0, h.out);
    return h.out.trim();
  };
  return { ...r, hash };
}

test("the hash is a git object id and stable", () => {
  const r = repo();
  assert.match(r.hash(), /^[0-9a-f]{40}$/);
  assert.equal(r.hash(), r.hash());
});

test("status, decision, and changelog edits keep the hash", () => {
  const r = repo();
  const before = r.hash();
  writeFiles(r.dir, {
    "docs/plan-status.js": "window.EMMET_STATUS = { review: 9 };\n",
    "docs/decisions.md": "# Decisions\n- one more\n",
    "CHANGELOG.md": "# Changelog\n- one more\n",
  });
  r.git("add", "-A");
  assert.equal(r.hash(), before);
});

test("a source, a doc a test reads, a new file, a rename, or a mode change changes the hash", () => {
  const r = repo();
  const seen = new Set([r.hash()]);
  const step = (files, ...git) => {
    if (files) writeFiles(r.dir, files);
    r.git("add", "-A");
    if (git.length) r.git(...git);
    const h = r.hash();
    assert.ok(!seen.has(h), `the hash didn't change after ${JSON.stringify(files ?? git)}`);
    seen.add(h);
  };
  step({ "crates/a.rs": "fn main() { }\n" });
  step({ "docs/spec/dsl.md": "# DSL v0\n" });
  step({ "crates/b.rs": "\n" });
  step(null, "mv", "crates/b.rs", "crates/c.rs");
  step(null, "update-index", "--chmod=+x", "crates/a.rs");
});
