#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Ranks OMR sets as rung 3 candidates (plan section 02: 150-400 parts, a Rebrickable inventory, mostly common parts).
//
//   node tests/spikes/omr/shortlist.mjs <omr-sets.json> <rebrickable dir> [limit]
//
// "Common" is the share of a set's pieces whose part and color appear together in at least 100 sets. Printed parts
// and minifig parts count against a set. Output: one line per set, best first.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { gunzipSync } from "node:zlib";
import { csv } from "../../../scripts/lib/csv.mjs";

const [omrFile, rbDir, limit = "30"] = process.argv.slice(2);
const load = (f) => csv(gunzipSync(readFileSync(join(rbDir, `${f}.csv.gz`))).toString("utf8"));
const omr = JSON.parse(readFileSync(omrFile, "utf8"));
const sets = new Map(load("sets").map((s) => [s.set_num, s]));
const themes = new Map(load("themes").map((t) => [t.id, t]));
const themePath = (id) => (themes.get(id)?.parent_id ? `${themePath(themes.get(id).parent_id)} > ` : "") + (themes.get(id)?.name ?? "?");
const categories = new Map(load("parts").map((p) => [p.part_num, p.part_cat_id]));
const inventories = load("inventories");
const firstInv = new Map();
for (const i of inventories.sort((a, b) => Number(a.version) - Number(b.version))) if (!firstInv.has(i.set_num)) firstInv.set(i.set_num, i.id);

// How many sets each part × color appears in.
const lines = load("inventory_parts").filter((l) => l.is_spare === "False");
const setOfInv = new Map(inventories.map((i) => [i.id, i.set_num]));
const spread = new Map();
for (const l of lines) {
  const k = `${l.part_num}|${l.color_id}`;
  if (!spread.has(k)) spread.set(k, new Set());
  spread.get(k).add(setOfInv.get(l.inventory_id));
}
const byInv = new Map();
for (const l of lines) (byInv.get(l.inventory_id) ?? byInv.set(l.inventory_id, []).get(l.inventory_id)).push(l);

const MINIFIG_CATS = new Set(["13", "27", "59", "60", "61", "62", "63", "64", "65", "70", "71", "72", "73"]); // part_categories.csv: minifig and minidoll parts
const rows = [];
for (const o of omr) {
  const s = sets.get(o.set_num);
  const inv = firstInv.get(o.set_num);
  if (!s || !inv || !byInv.has(inv)) continue;
  const own = byInv.get(inv);
  const pcs = own.reduce((n, l) => n + Number(l.quantity), 0);
  if (pcs < 150 || pcs > 400) continue;
  const share = (f) => own.filter(f).reduce((n, l) => n + Number(l.quantity), 0) / pcs;
  const common = share((l) => spread.get(`${l.part_num}|${l.color_id}`).size >= 100);
  const printed = share((l) => /pr\d|pat|p\d{2,}$/i.test(l.part_num));
  const minifig = share((l) => MINIFIG_CATS.has(categories.get(l.part_num)));
  rows.push({ ...o, pieces: pcs, lines: own.length, common, printed, minifig, theme_rb: themePath(s.theme_id), score: common - printed - minifig });
}
rows.sort((a, b) => b.score - a.score);
const pct = (x) => `${(100 * x).toFixed(0)}%`.padStart(4);
for (const r of rows.slice(0, Number(limit)))
  console.log(`${r.set_num.padEnd(9)} ${String(r.pieces).padStart(3)} pcs ${String(r.lines).padStart(3)} lines  common ${pct(r.common)} printed ${pct(r.printed)} minifig ${pct(r.minifig)}  ${r.year}  ${r.name}  [${r.theme_rb}] omr:${r.omr_id}`);
console.log(`shortlist: ${rows.length} OMR sets with 150-400 pieces and a Rebrickable inventory`);
