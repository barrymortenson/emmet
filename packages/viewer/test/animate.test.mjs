// SPDX-License-Identifier: AGPL-3.0-or-later
// The build animation's order and timing (plan section 13): runs against the built package (dist/).
import assert from "node:assert/strict";
import { test } from "node:test";
import { CAP_MS, DROP, FAST_MS, GLIDE_MS, LEAVE_MS, PART_MS, buildOrder, drop, glide, leave, simplifyStuds, stagger } from "@emmet/viewer";

// A part at (x, y, z) in LDU; LDraw's −Y is up, so y = -8 is one plate above y = 0.
const p = (id, x, y, z = 0, step = 1, copies) => ({ id, part: "3001", color: 4, matrix: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, x, y, z, 1], step, submodel: copies ? ["tower.ldr"] : [], copies });

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

test("a sub-assembly's parts go in at the step that places the copy, in its own steps' order", () => {
  // The tower's cap is its own step 3, but the copy goes on at step 2: before the main model's step 3, and after the
  // tower's base (its step 1) even though the base is lower.
  const parts = [
    p("roof", 0, -96, 0, 3),
    p("tower/1/cap", 0, -72, 0, 3, [{ id: "tower/1", step: 2 }]),
    p("tower/1/base", 0, -24, 0, 1, [{ id: "tower/1", step: 2 }]),
    p("plate", 0, 0, 0, 1),
  ];
  assert.deepEqual(
    buildOrder(parts).map((x) => x.id),
    ["plate", "tower/1/base", "tower/1/cap", "roof"],
  );
});

test("Animated staggers 30 ms apart and fits a big batch in 6 s; Fast fits any batch in 0.6 s; Off doesn't stagger", () => {
  assert.equal(stagger(1), 0);
  assert.equal(stagger(10), 30);
  const n = 2000;
  assert.ok((n - 1) * stagger(n) + PART_MS <= CAP_MS + 1e-9);
  assert.ok((n - 1) * stagger(n, "fast") + PART_MS <= FAST_MS + 1e-9);
  assert.ok((9 * stagger(10, "fast")) + PART_MS <= FAST_MS + 1e-9);
  assert.equal(stagger(n, "off"), 0);
});

test("a part waits hidden, drops three plates and fades in with ease-out, then sits", () => {
  assert.deepEqual(drop(-1), { visible: false, lift: DROP, fade: 0, done: false });
  assert.equal(drop(0).lift, DROP);
  const half = drop(PART_MS / 2);
  assert.ok(half.visible && half.lift > 0 && half.lift < DROP / 2, "ease-out: past halfway at half time");
  assert.ok(half.fade > 0.5 && half.fade < 1);
  assert.deepEqual(drop(PART_MS), { visible: true, lift: 0, fade: 1, done: true });
});

test("a removed part lifts and fades out, and a moved one glides", () => {
  assert.deepEqual(leave(0), { visible: true, lift: 0, fade: 1, done: false });
  const mid = leave(LEAVE_MS / 2);
  assert.ok(mid.lift > 0 && mid.fade < 1 && !mid.done);
  assert.equal(leave(LEAVE_MS).visible, false);
  assert.deepEqual(glide(0), { t: 0, done: false });
  assert.ok(glide(GLIDE_MS / 2).t > 0.5);
  assert.deepEqual(glide(GLIDE_MS), { t: 1, done: true });
});

test("the stud LOD draws top studs 8-sided and leaves out the tubes under a brick", () => {
  const brick = [
    "0 FILE 3001.dat",
    "1 16 10 0 10 1 0 0 0 1 0 0 0 1 stud.dat",
    "1 16 0 4 0 1 0 0 0 -1 0 0 0 1 STUD4.DAT",
    "1 16 0 0 0 1 0 0 0 1 0 0 0 1 s/3001s01.dat",
    "0 FILE stud.dat",
  ].join("\n");
  const low = simplifyStuds(brick);
  assert.ok(low);
  assert.match(low, /^1 16 10 0 10 1 0 0 0 1 0 0 0 1 emmet-lo-stud\.dat$/m);
  assert.doesNotMatch(low, /STUD4/i);
  assert.match(low, /s\/3001s01\.dat/);
  // 8 triangles for the top, 8 quads for the side.
  assert.equal(low.split("\n").filter((l) => l.startsWith("3 16")).length, 8);
  assert.equal(simplifyStuds("0 FILE 3068b.dat\n1 16 0 0 0 1 0 0 0 1 0 0 0 1 s/3068bs01.dat"), null);
});
