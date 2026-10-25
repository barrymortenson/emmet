// SPDX-License-Identifier: AGPL-3.0-or-later
// The diff by Emmet id that every viewer update applies (plan section 13). Runs against the built package (dist/).
import assert from "node:assert/strict";
import { test } from "node:test";
import { bucketKey, diffPlacements, dirtyBuckets } from "@emmet/viewer";

const at = (x) => [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, x, 0, 0, 1];
const p = (id, part, color, x = 0) => ({ id, part, color, matrix: at(x), step: 1, submodel: [] });

test("a diff sorts every id into added, removed, moved, recolored, or unchanged", () => {
  const before = [p("a", "3001", 4), p("b", "3001", 4), p("c", "3004", 1), p("d", "3005", 2)];
  const after = [p("a", "3001", 4), p("b", "3001", 4, 20), p("c", "3004", 15), p("e", "3001", 4)];
  const d = diffPlacements(before, after);
  assert.deepEqual(d.added.map((x) => x.id), ["e"]);
  assert.deepEqual(d.removed.map((x) => x.id), ["d"]);
  assert.deepEqual(d.moved.map((x) => x.id), ["b"]);
  assert.deepEqual(d.recolored.map((x) => x.id), ["c"]);
  assert.equal(d.unchanged, 1);
});

test("a new part under the same id changes bucket, like a new color", () => {
  const d = diffPlacements([p("a", "3001", 4)], [p("a", "3002", 4)]);
  assert.deepEqual(d.recolored.map((x) => x.id), ["a"]);
});

test("only the buckets a diff touches are dirty, the old one included for a recolor", () => {
  const before = [p("a", "3001", 4), p("b", "3004", 1), p("c", "3005", 2)];
  const after = [p("a", "3001", 4), p("b", "3004", 15), p("c", "3005", 2)];
  const d = diffPlacements(before, after);
  const dirty = dirtyBuckets(d, new Map(before.map((x) => [x.id, x])));
  assert.deepEqual([...dirty].sort(), ["3004|1", "3004|15"]);
});

test("the same list twice is all unchanged and dirties nothing", () => {
  const list = [p("a", "3001", 4), p("b", "3004", 1, 40)];
  const d = diffPlacements(list, structuredClone(list));
  assert.equal(d.unchanged, 2);
  assert.equal(dirtyBuckets(d, new Map()).size, 0);
});

test("bucket keys ignore the part id's case, as LDraw does", () => {
  assert.equal(bucketKey({ part: "U9074", color: 4 }), bucketKey({ part: "u9074", color: 4 }));
});
