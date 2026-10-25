#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Checks that tests/fixtures/ldraw-mini/ still meets the LDraw library's license (ADR 0003) and is complete: every file
// is listed in MANIFEST.sha256 and unmodified, every .dat file keeps its Author and !LICENSE lines and is listed in
// ATTRIBUTION.md, the license texts are present, and every part PARTS.txt names is there. Runs in CI.
// Usage: node scripts/ldraw-mini-check.mjs [fixture dir]

import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const dir = process.argv[2] || "tests/fixtures/ldraw-mini";
const OWN = new Set(["PARTS.txt", "ATTRIBUTION.md", "MANIFEST.sha256"]);
const LICENSES = [
  "0 !LICENSE Licensed under CC BY 4.0 : see CAreadme.txt",
  "0 !LICENSE Licensed under CC BY 2.0 and CC BY 4.0 : see CAreadme.txt",
  "0 !LICENSE Redistributable under CCAL version 2.0 : see CAreadme.txt",
];
const errors = [];

const walk = (d) => readdirSync(d).flatMap((n) => (statSync(join(d, n)).isDirectory() ? walk(join(d, n)) : [join(d, n)]));
const files = walk(dir).map((f) => relative(dir, f).split("\\").join("/")).filter((f) => !OWN.has(f));

for (const f of ["CAreadme.txt", "CAlicense.txt", "CAlicense4.txt", "ATTRIBUTION.md", "MANIFEST.sha256", "PARTS.txt"])
  if (!existsSync(join(dir, f))) errors.push(`${f} is missing`);

const manifest = new Map(
  (existsSync(join(dir, "MANIFEST.sha256")) ? readFileSync(join(dir, "MANIFEST.sha256"), "utf8") : "")
    .split("\n")
    .filter(Boolean)
    .map((l) => [l.slice(66), l.slice(0, 64)]),
);
const attribution = existsSync(join(dir, "ATTRIBUTION.md")) ? readFileSync(join(dir, "ATTRIBUTION.md"), "utf8") : "";

for (const f of files) {
  const buf = readFileSync(join(dir, f));
  if (!manifest.has(f)) errors.push(`${f} is not in MANIFEST.sha256`);
  else if (createHash("sha256").update(buf).digest("hex") !== manifest.get(f)) errors.push(`${f} was modified (sha256 differs from MANIFEST.sha256)`);
  if (!/\.(dat|ldr)$/i.test(f) || f === "LDConfig.ldr") continue;
  const head = buf.toString("latin1").split(/\r?\n/);
  if (!head.some((l) => /^0 Author: \S/.test(l))) errors.push(`${f} has no Author line`);
  if (!head.some((l) => LICENSES.includes(l.trim()))) errors.push(`${f} has no accepted !LICENSE line`);
  if (!attribution.includes(`| ${f} |`)) errors.push(`${f} is not listed in ATTRIBUTION.md`);
}
for (const f of manifest.keys()) if (!files.includes(f)) errors.push(`${f} is in MANIFEST.sha256 but missing`);

const parts = files.filter((f) => /^parts\/[^/]+\.dat$/i.test(f));
if (!parts.length) errors.push("the fixture has no parts");
const listed = existsSync(join(dir, "PARTS.txt"))
  ? readFileSync(join(dir, "PARTS.txt"), "utf8").split("\n").flatMap((l) => l.replace(/#.*/, "").trim().split(/\s+/)).filter(Boolean)
  : [];
const have = new Set(parts.map((f) => f.slice(6, -4).toLowerCase()));
for (const id of listed) if (!have.has(id.toLowerCase())) errors.push(`PARTS.txt lists ${id}, but parts/${id}.dat is missing`);

for (const e of errors) console.error(`error: ${e}`);
if (errors.length) {
  console.error(`ldraw-mini-check: ${errors.length} problems in ${dir}`);
  process.exit(1);
}
console.log(`ldraw-mini-check: ok (${files.length} files, ${parts.length} parts)`);
