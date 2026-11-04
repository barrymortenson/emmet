// SPDX-License-Identifier: AGPL-3.0-or-later
// tests/e2e/tiers.mjs: every E2E spec file is in exactly one tier, so a new spec can't fall out of CI unnoticed, and
// what a change runs follows the tiers (plan section 16).
import assert from "node:assert/strict";
import { readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { AREAS, CORE, GATE, EXTRA, WINDOWS, every, forChanges, tiered } from "../e2e/tiers.mjs";
import { ROOT } from "./helpers.mjs";

const specs = readdirSync(join(ROOT, "tests/e2e/specs"))
  .filter((f) => f.endsWith(".e2e.mjs"))
  .map((f) => f.replace(/\.e2e\.mjs$/, ""))
  .sort();

test("every spec file is in exactly one tier, and every tier names a spec file", () => {
  const all = tiered();
  assert.deepEqual([...all].sort(), specs, "add the spec to a tier in tests/e2e/tiers.mjs (or remove it there)");
  assert.equal(new Set(all).size, all.length, "a spec is in two tiers");
});

test("Windows runs specs from core and the areas, never the extra tier", () => {
  for (const s of WINDOWS) assert.ok(tiered().includes(s) && !EXTRA.includes(s), s);
});

test("prose and scripts run no E2E; any app change runs hello and core, hello first", () => {
  assert.deepEqual(forChanges(["docs/plan.html", "scripts/plan-check.mjs", "README.md"]), []);
  for (const path of ["apps/desktop/src/App.tsx", "crates/emmet-core/src/lib.rs", "data/techniques/a.py", "tests/fixtures/x"]) {
    const got = forChanges([path]);
    assert.equal(got[0], GATE, path);
    for (const s of CORE) assert.ok(got.includes(s), `${path}: ${s}`);
    for (const s of EXTRA) assert.ok(!got.includes(s), `${path}: ${s}`);
  }
});

test("an area's change adds its specs, and only its own", () => {
  assert.ok(forChanges(["crates/emmet-agent/src/claude.rs"]).includes("chatgpt"));
  assert.ok(!forChanges(["crates/emmet-core/src/bom.rs"]).includes("chatgpt"));
  assert.ok(forChanges(["apps/desktop/src/FirstRun.tsx"]).at(-1) === "first-run", "first run goes last, as its own run");
  for (const [name, area] of Object.entries(AREAS)) assert.ok(area.paths instanceof RegExp, name);
});

test("a spec's own change runs it, the extra ones too; the harness runs every area", () => {
  assert.ok(forChanges(["tests/e2e/specs/screens.e2e.mjs"]).includes("screens"));
  assert.ok(forChanges(["tests/e2e/baselines/macos/screens/start-light.png"]).includes("screens"));
  assert.ok(!forChanges(["tests/e2e/specs/bench.e2e.mjs"]).includes("bench"), "the bench is just bench's");
  assert.deepEqual(forChanges(["tests/e2e/lib/app.mjs"]), every());
});
