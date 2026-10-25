#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// ADR 0004: how well LDraw part ids and colors map to Rebrickable, measured on the pinned archives.
//
//   node tests/spikes/data/coverage.mjs <rebrickable dir> <ldraw complete.zip> [set_num ...]
//
// A part maps through an LD row in part_mappings.csv (ext_id = the LDraw id), otherwise through an identical
// part_num in parts.csv (plan section 12). For each set_num, it reports how many of the set's Rebrickable inventory
// lines (and pieces) resolve to an LDraw part, the other way round. The zip is read through `unzip` only.

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { gunzipSync } from "node:zlib";
import { csv } from "./csv.mjs";

const [rbDir, zip, ...sets] = process.argv.slice(2);
if (!rbDir || !zip) {
  console.error("usage: coverage.mjs <rebrickable dir> <complete.zip> [set_num ...]");
  process.exit(2);
}
const load = (f) => csv(gunzipSync(readFileSync(join(rbDir, `${f}.csv.gz`))).toString("utf8"));
const parts = load("parts");
const partMap = load("part_mappings");
const colors = load("colors");
const colorMap = load("color_mappings");

const rbParts = new Set(parts.map((p) => p.part_num));
const ldToRb = new Map();
const rbToLd = new Map();
for (const m of partMap.filter((m) => m.ext_system === "LD")) {
  if (!ldToRb.has(m.ext_id) || m.is_import_id === "True") ldToRb.set(m.ext_id, m.part_num);
  if (!rbToLd.has(m.part_num) || m.is_export_id === "True") rbToLd.set(m.part_num, m.ext_id);
}

// LDraw library: top-level parts (not subparts) and their titles, from one `unzip -c` pass (each file follows an
// "inflating: <name>" line, and its first line is its title).
const titles = new Map();
let current = null;
const dump = execFileSync("unzip", ["-c", zip, "ldraw/parts/*.dat", "-x", "ldraw/parts/s/*"], { encoding: "latin1", maxBuffer: 1 << 30 });
for (const line of dump.split(/\r?\n/)) {
  const m = line.match(/^\s+inflating: \S*?parts\/([^/]+)\.dat\s*$/i); // macOS unzip prints "draw/parts/…"
  if (m) current = m[1].toLowerCase();
  else if (current && line.startsWith("0 ")) titles.set(current, line.slice(2).trim()), (current = null);
}
const ldIds = [...titles.keys()];
const ldSet = new Set(ldIds);
const title = (id) => titles.get(id) ?? "?";
// "~Moved to X" files: the old id is an alias of X. Rebrickable's LD rows sometimes still name the old id.
const movedFrom = new Map();
for (const [id, t] of titles) {
  const m = t.match(/^~Moved to (\S+)/i);
  if (m) movedFrom.set(m[1].toLowerCase(), [...(movedFrom.get(m[1].toLowerCase()) ?? []), id]);
}

function mapPart(id) {
  if (ldToRb.has(id)) return { via: "LD row", rb: ldToRb.get(id) };
  for (const old of movedFrom.get(id) ?? []) if (ldToRb.has(old)) return { via: "LD row for a moved id", rb: ldToRb.get(old) };
  if (rbParts.has(id)) return { via: "identical id", rb: id };
  const base = id.replace(/-f\d+$/, ""); // LDraw's formed variants of a flexible part (75541-f1, -f2)
  if (base !== id) return mapPart(base).via ? { via: "base of a formed variant", rb: mapPart(base).rb } : { via: null };
  return { via: null };
}
const pct = (a, b) => `${((100 * a) / b).toFixed(1)}%`;

function report(label, ids, detail) {
  const r = { none: [] };
  for (const id of ids) {
    const m = mapPart(id);
    if (m.via) r[m.via] = (r[m.via] ?? 0) + 1;
    else r.none.push(id);
  }
  const ok = ids.length - r.none.length;
  const how = Object.entries(r).filter(([k]) => k !== "none").map(([k, n]) => `${n} ${k}`).join(", ");
  console.log(`${label}: ${ok} of ${ids.length} map (${pct(ok, ids.length)}): ${how}`);
  if (detail) for (const id of r.none) console.log(`  unmapped: ${id}  ${title(id)}`);
  return r;
}
// A real part: not an alias, a moved file, an obsolete or internal part (~), or a physical-colour alias (= or _).
const isReal = (id) => !/^[~=_]/.test(title(id));

// 1. ldraw-mini's top-level parts.
const mini = readFileSync(new URL("../../fixtures/ldraw-mini/MANIFEST.sha256", import.meta.url), "utf8")
  .split("\n")
  .map((l) => l.slice(66))
  .filter((f) => /^parts\/[^/]+\.dat$/.test(f))
  .map((f) => f.slice(6, -4).toLowerCase());
report("ldraw-mini parts (real parts only)", mini.filter(isReal), true);
console.log(`  (ldraw-mini also has ${mini.filter((id) => !isReal(id)).length} aliases and internal ~ parts, which resolve to the real parts above)`);

// 2. The whole library.
report("LDraw library, real parts", ldIds.filter(isReal), false);
report("LDraw library, every parts/*.dat", ldIds, false);

// 3. Colors: LDraw codes in LDConfig.ldr against color_mappings.csv LD rows, and Rebrickable colors with a BrickLink id.
const ldconfig = execFileSync("unzip", ["-p", zip, "ldraw/LDConfig.ldr"]).toString("utf8");
const ldColors = [...ldconfig.matchAll(/^0 !COLOUR (\S+)\s+CODE\s+(\d+)/gm)].map(([, name, code]) => ({ name, code }));
const ldColorToRb = new Map(colorMap.filter((m) => m.ext_system === "LD").map((m) => [m.ext_id, m.id]));
const rbWithBl = new Set(colorMap.filter((m) => m.ext_system === "BL").map((m) => m.id));
const mapped = ldColors.filter((c) => ldColorToRb.has(c.code));
console.log(`LDraw colors (LDConfig.ldr): ${mapped.length} of ${ldColors.length} map to a Rebrickable color (${pct(mapped.length, ldColors.length)})`);
for (const c of ldColors.filter((c) => !ldColorToRb.has(c.code))) console.log(`  unmapped LDraw color: ${c.code} ${c.name}`);
const solid = colors.filter((c) => Number(c.id) >= 0 && Number(c.num_parts) > 0);
console.log(`Rebrickable colors in use: ${solid.filter((c) => [...ldColorToRb.values()].includes(c.id)).length} of ${solid.length} have an LDraw code; ${solid.filter((c) => rbWithBl.has(c.id)).length} have a BrickLink id`);

// 4. Sample sets: Rebrickable inventory → LDraw.
if (sets.length) {
  const inventories = load("inventories");
  const lines = load("inventory_parts");
  for (const s of sets) {
    const inv = inventories.filter((i) => i.set_num === s).sort((a, b) => Number(a.version) - Number(b.version))[0];
    if (!inv) {
      console.log(`set ${s}: no inventory`);
      continue;
    }
    const own = lines.filter((l) => l.inventory_id === inv.id && l.is_spare === "False");
    const toLd = (p) => (rbToLd.has(p) && ldSet.has(rbToLd.get(p).toLowerCase()) ? rbToLd.get(p) : ldSet.has(p.toLowerCase()) ? p : null);
    const hit = own.filter((l) => toLd(l.part_num));
    const pcs = (ls) => ls.reduce((n, l) => n + Number(l.quantity), 0);
    console.log(`set ${s}: ${hit.length} of ${own.length} inventory lines map to LDraw (${pct(hit.length, own.length)}), ${pcs(hit)} of ${pcs(own)} pieces (${pct(pcs(hit), pcs(own))})`);
    for (const l of own.filter((l) => !toLd(l.part_num))) console.log(`  unmapped: ${l.part_num} ×${l.quantity}`);
  }
}

// 5. What the unmapped real parts are, by kind (from the title and id), so "availability unknown" has a known scope.
{
  const kinds = [
    ["sticker", (id, t) => /sticker/i.test(t)],
    ["printed or patterned", (id, t) => /p[0-9a-z]{2,}$/i.test(id) || /pattern|print/i.test(t)],
    ["minifig and figure parts", (id, t) => /minifig|figure|torso|legs|hips/i.test(t)],
    ["Duplo, Primo, Quatro, Technic figures", (id, t) => /duplo|primo|quatro|fabuland|scala|belville/i.test(t)],
    ["assemblies (cNN)", (id) => /c\d+$/i.test(id)],
    ["other", () => true],
  ];
  const counts = new Map(kinds.map(([k]) => [k, 0]));
  for (const id of ldIds.filter(isReal).filter((id) => !mapPart(id).via)) {
    const [k] = kinds.find(([, f]) => f(id, title(id)));
    counts.set(k, counts.get(k) + 1);
  }
  console.log(`Unmapped real parts by kind: ${[...counts].map(([k, n]) => `${k} ${n}`).join(", ")}`);
}
