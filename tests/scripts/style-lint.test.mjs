// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/style-lint.mjs (plan session 4A, What good looks like): the app passes, and a raw color, a pixel size, or a
// literal inline style each fail, while tokens, comments, and values computed from data pass.
import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";
import { lint } from "../../scripts/style-lint.mjs";
import { ROOT, run, tempDir, writeFiles } from "./helpers.mjs";

test("the app's styles use only tokens", () => {
  const r = run("node", [join(ROOT, "scripts/style-lint.mjs")]);
  assert.equal(r.code, 0, r.out);
});

test("raw colors and pixel sizes fail; tokens, comments, and computed values pass", () => {
  const css = (s) => lint("x.css", s).map(([, what]) => what);
  assert.deepEqual(css(".a { color: #fff; }"), ["a raw color"]);
  assert.deepEqual(css(".a { background: rgba(0, 0, 0, 0.5); }"), ["a raw color"]);
  assert.deepEqual(css(".a { border-color: white; }"), ["a raw color"]);
  assert.deepEqual(css(".a { width: 12px; }"), ["a pixel size"]);
  assert.deepEqual(css(".a { color: var(--fg-text); width: var(--control-md); } /* 12px, #fff */"), []);
  const tsx = (s) => lint("x.tsx", s).map(([, what]) => what);
  assert.deepEqual(tsx('<i style={{ background: "#c2410c" }} />'), ["a raw color in an inline style"]);
  assert.deepEqual(tsx("<i style={{ width: 20 }} />"), ["a pixel size in an inline style"]);
  assert.deepEqual(tsx('<i style={{ width: "20px" }} />'), ["a pixel size in an inline style"]);
  assert.deepEqual(tsx("<i style={{ background: part.rgb, width: `${pct}%` }} />"), []);
});

test("a stylesheet with a one-off style fails the run", () => {
  const dir = tempDir();
  writeFiles(dir, { "styles/tokens.css": ":root { --x: 4px; }", "styles/app.css": ".a { margin: 3px; }" });
  const r = run("node", [join(ROOT, "scripts/style-lint.mjs"), dir]);
  assert.equal(r.code, 1);
  assert.match(r.out, /styles\/app\.css:1: a pixel size/);
});
