// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/ldraw-mini-check.mjs: passes on the committed fixture, and fails when a license obligation is broken.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempDir, writeFiles } from "./helpers.mjs";

const PART = "0 Brick  1 x  1\r\n0 Name: 3005.dat\r\n0 Author: A Part Author [author]\r\n0 !LDRAW_ORG Part UPDATE 2026-01\r\n0 !LICENSE Licensed under CC BY 4.0 : see CAreadme.txt\r\n1 16 0 0 0 1 0 0 0 1 0 0 0 1 s\\3005s01.dat\r\n";
const sha = (s) => createHash("sha256").update(s).digest("hex");

/** A tiny fixture that passes, with `change` applied to its files first. */
function fixture(change = (f) => f) {
  const files = change({
    "parts/3005.dat": PART,
    "CAreadme.txt": "readme",
    "CAlicense.txt": "CC BY 2.0",
    "CAlicense4.txt": "CC BY 4.0",
  });
  const manifest = Object.entries({ "parts/3005.dat": PART, "CAreadme.txt": "readme", "CAlicense.txt": "CC BY 2.0", "CAlicense4.txt": "CC BY 4.0" })
    .map(([f, c]) => `${sha(c)}  ${f}`)
    .join("\n");
  const dir = tempDir();
  writeFiles(dir, { "MANIFEST.sha256": manifest + "\n", "ATTRIBUTION.md": "| parts/3005.dat | Brick 1 x 1 | A Part Author | CC BY 4.0 |\n", ...files });
  return run("node", [join(ROOT, "scripts/ldraw-mini-check.mjs"), dir]);
}

test("ldraw-mini-check passes on the committed fixture", () => {
  const r = run("node", ["scripts/ldraw-mini-check.mjs"], { cwd: ROOT });
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /ok \(\d+ files, \d+ parts\)/);
});

test("ldraw-mini-check passes on a minimal good fixture", () => {
  const r = fixture();
  assert.equal(r.code, 0, r.out);
});

test("ldraw-mini-check fails when a part's license line is stripped", () => {
  const r = fixture((f) => ({ ...f, "parts/3005.dat": PART.replace(/0 !LICENSE.*\r\n/, "") }));
  assert.equal(r.code, 1);
  assert.match(r.out, /parts\/3005.dat has no accepted !LICENSE line/);
  assert.match(r.out, /parts\/3005.dat was modified/);
});

test("ldraw-mini-check fails when a part's author is removed", () => {
  const r = fixture((f) => ({ ...f, "parts/3005.dat": PART.replace(/0 Author:.*\r\n/, "") }));
  assert.equal(r.code, 1);
  assert.match(r.out, /parts\/3005.dat has no Author line/);
});

test("ldraw-mini-check fails on a file outside the manifest and attribution", () => {
  const r = fixture((f) => ({ ...f, "parts/3004.dat": PART.replace("3005", "3004") }));
  assert.equal(r.code, 1);
  assert.match(r.out, /parts\/3004.dat is not in MANIFEST.sha256/);
  assert.match(r.out, /parts\/3004.dat is not listed in ATTRIBUTION.md/);
});

test("ldraw-mini-check fails when a license text is missing", () => {
  const r = fixture((f) => {
    delete f["CAlicense4.txt"];
    return f;
  });
  assert.equal(r.code, 1);
  assert.match(r.out, /CAlicense4.txt is missing/);
});
