#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Fails when a tracked source file lacks the SPDX header for its path (CONTRIBUTING.md).
// Usage: node scripts/spdx-check.mjs [repo dir]
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const SOURCE = /\.(rs|ts|tsx|js|mjs|cjs|py|sh|css)$|^(scripts|\.githooks)\/[^/.]+$|(^|\/)justfile$/;
// Apache-2.0: the DSL, the techniques (data/techniques/ and their candidate sketches in docs/techniques/), and the
// templates (agent rule 1).
const APACHE = /^(python\/emmet_dsl|data\/techniques|docs\/techniques)\/|(^|\/)templates\//;

const root = process.argv[2] || ".";
const files = execFileSync("git", ["ls-files", "-z"], { cwd: root, encoding: "utf8" }).split("\0").filter(Boolean);
const bad = [];
let checked = 0;
for (const file of files.filter((f) => SOURCE.test(f))) {
  checked++;
  const want = APACHE.test(file) ? "Apache-2.0" : "AGPL-3.0-or-later";
  const head = readFileSync(join(root, file), "utf8").split("\n", 5).join("\n");
  const got = head.match(/SPDX-License-Identifier: (\S+)/)?.[1];
  if (got !== want) bad.push(`${file}: ${got ? `has ${got}, needs` : "missing"} SPDX-License-Identifier: ${want}`);
}
for (const b of bad) console.error(`error: ${b}`);
if (bad.length) {
  console.error(`spdx-check: ${bad.length} of ${checked} source files need a fix`);
  process.exit(1);
}
console.log(`spdx-check: ok (${checked} source files)`);
