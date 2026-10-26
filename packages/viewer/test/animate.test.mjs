// SPDX-License-Identifier: AGPL-3.0-or-later
// The build animation's order and timing (plan section 13): runs against the built package (dist/).
import assert from "node:assert/strict";
import { test } from "node:test";
import { CAP_MS, DROP, PART_MS, buildOrder, drop, stagger } from "@emmet/viewer";

// A part at (x, y, z) in LDU; LDraw's −Y is up, so y = -8 is one plate above y = 0.
const p = (id, x, y, z = 0, step = 1) => ({ id, part: "3001", color: 4, matrix: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, x, y, z, 1], step, submodel: [] });

test("parts go in by step, then bottom-up, then nearest the one before", () => {
  const parts = [
    p("roof", 0, -48, 0, 2),
    p("wall-far", 200, -24),
    p("wall-near", 20, -24),
    p("base-far", 200, 0),
    p("base-a", 0, 0),
    p("base-b", 40, 0),
  ];
  assert.deepEqual(
    buildOrder(parts).map((x) => x.id),
    ["base-a", "base-b", "base-far", "wall-far", "wall-near", "roof"],
  );
});

test("a batch staggers 30 ms apart, and a big one fits in 6 s", () => {
  assert.equal(stagger(1), 0);
  assert.equal(stagger(10), 30);
  const n = 2000;
  assert.ok((n - 1) * stagger(n) + PART_MS <= CAP_MS + 1e-9);
});

test("a part waits hidden, drops three plates with ease-out, then sits", () => {
  assert.deepEqual(drop(-1), { visible: false, lift: DROP, done: false });
  assert.equal(drop(0).lift, DROP);
  const half = drop(PART_MS / 2);
  assert.ok(half.visible && half.lift > 0 && half.lift < DROP / 2, "ease-out: past halfway at half time");
  assert.deepEqual(drop(PART_MS), { visible: true, lift: 0, done: true });
});
