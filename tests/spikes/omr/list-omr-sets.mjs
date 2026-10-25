#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Lists every set in the LDraw Official Model Repository (OMR) from its paged index, one request a second, for the
// rung 3 shortlist (docs/spec/ladder.md). Only the index pages are read.
//
//   node tests/spikes/omr/list-omr-sets.mjs <out.json>

import { writeFileSync } from "node:fs";

const BASE = "https://library.ldraw.org/omr/sets";
const clean = (s) => s.replace(/<[^>]+>/g, " ").replace(/&amp;/g, "&").replace(/&#039;|&#39;/g, "'").replace(/&quot;/g, '"').replace(/&gt;/g, ">").replace(/\s+/g, " ").trim();

async function page(n) {
  const res = await fetch(`${BASE}?page=${n}`, { headers: { "user-agent": "emmet-0b-spike (one-time OMR index read)" } });
  if (!res.ok) throw new Error(`page ${n}: HTTP ${res.status}`);
  const html = await res.text();
  const rows = [...html.matchAll(/<tr[^>]*>([\s\S]*?)<\/tr>/g)].slice(1);
  return rows.map(([, r]) => {
    const cells = [...r.matchAll(/<td[^>]*>([\s\S]*?)<\/td>/g)].map(([, c]) => clean(c));
    const id = r.match(/omr\/sets\/(\d+)/)?.[1];
    return { omr_id: Number(id), set_num: cells[1], name: cells[2], theme: cells[3], year: Number(cells[4]), models: Number(cells[5]) };
  });
}

const out = process.argv[2];
if (!out) {
  console.error("usage: list-omr-sets.mjs <out.json>");
  process.exit(2);
}
const sets = [];
for (let n = 1; ; n++) {
  const rows = await page(n);
  if (!rows.length) break;
  sets.push(...rows);
  await new Promise((r) => setTimeout(r, 1000));
}
writeFileSync(out, JSON.stringify(sets, null, 1));
console.log(`list-omr-sets: ${sets.length} sets → ${out}`);
