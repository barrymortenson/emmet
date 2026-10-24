// SPDX-License-Identifier: AGPL-3.0-or-later
// Checks the built frontend (dist/) that Tauri embeds: the app name and the package version are in it.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { test } from "node:test";

const dist = new URL("../dist/", import.meta.url);
const pkg = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));

test("the built app shows Emmet and its version", () => {
  const html = readFileSync(new URL("index.html", dist), "utf8");
  assert.match(html, /<title>Emmet<\/title>/);
  const js = readdirSync(new URL("assets/", dist)).filter((f) => f.endsWith(".js"));
  const code = js.map((f) => readFileSync(new URL(`assets/${f}`, dist), "utf8")).join("\n");
  // The minifier may quote strings with ", ', or `.
  const literal = (text) => new RegExp(`["'\`]${text.replaceAll(".", "\\.")}["'\`]`);
  assert.match(code, literal(pkg.version), `version ${pkg.version} not in the bundle`);
  assert.doesNotMatch(code, literal("DEV"), "the DEV badge is dev-only");
});
