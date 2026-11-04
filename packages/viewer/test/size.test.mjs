// SPDX-License-Identifier: AGPL-3.0-or-later
// The viewer's own code stays under 1 MB gzipped, three.js aside (session 4C, What good looks like 2): its built
// modules in dist/, as the app bundles them.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { gzipSync } from "node:zlib";

test("the viewer's built code is under 1 MB gzipped, excluding three.js", () => {
  const dist = join(import.meta.dirname, "..", "dist");
  const files = readdirSync(dist).filter((f) => f.endsWith(".js"));
  assert.ok(files.includes("index.js"), "build the viewer first");
  const bytes = files.reduce((n, f) => n + gzipSync(readFileSync(join(dist, f))).length, 0);
  console.log(`viewer: ${files.length} modules, ${(bytes / 1024).toFixed(1)} KB gzipped`);
  assert.ok(bytes < 1024 * 1024, `${bytes} bytes gzipped`);
});
