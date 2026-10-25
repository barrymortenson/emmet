// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/terms-check: finds each recorded phrase on its page (through tags, entities, and curly quotes), reports a
// phrase that's gone and a page that won't load, and lists "manual" pages without fetching them. A local server
// stands in for the providers, so the test never uses the network.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, tempDir } from "./helpers.mjs";

const PAGE = "<html><body><p>Running Claude Code <b>in your</b> products is fine.</p><p>It&rsquo;s free &amp; open.</p></body></html>";

async function check(pages) {
  const hits = [];
  const server = createServer((req, res) => {
    hits.push(req.url);
    if (req.url === "/page") res.end(PAGE);
    else res.writeHead(500).end();
  });
  await new Promise((r) => server.listen(0, "127.0.0.1", r));
  const base = `http://127.0.0.1:${server.address().port}`;
  const file = join(tempDir(), "quotes.json");
  writeFileSync(file, JSON.stringify({ checked: "2026-10-24", pages: pages(base) }));
  const child = spawn("node", [join(ROOT, "scripts/terms-check"), file]);
  let out = "";
  child.stdout.on("data", (d) => (out += d));
  child.stderr.on("data", (d) => (out += d));
  const code = await new Promise((r) => child.on("exit", r));
  server.close();
  return { code, out, hits };
}

const page = (url, quotes, check = "fetch") => ({ provider: "P", title: "T", url, check, quotes: quotes.map((text) => ({ text, basis: "b" })) });

test("terms-check passes when every phrase is on its page", async () => {
  const r = await check((base) => [page(`${base}/page`, ["running Claude Code in your products", "It's free & open"])]);
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /terms-check: ok/);
});

test("terms-check fails on a phrase that's gone", async () => {
  const r = await check((base) => [page(`${base}/page`, ["running Claude Code in your products", "resell Claude usage"])]);
  assert.equal(r.code, 1);
  assert.match(r.out, /GONE "resell Claude usage"/);
  assert.match(r.out, /1 phrases are gone or unchecked/);
});

test("terms-check fails when a page won't load", async () => {
  const r = await check((base) => [page(`${base}/broken`, ["anything"])]);
  assert.equal(r.code, 1);
  assert.match(r.out, /could not fetch: HTTP 500/);
});

test("terms-check lists manual pages without fetching them", async () => {
  const r = await check((base) => [page(`${base}/manual`, ["no automated traffic"], "manual")]);
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /check by hand/);
  assert.match(r.out, /\[ \] "no automated traffic"/);
  assert.deepEqual(r.hits, []);
});
