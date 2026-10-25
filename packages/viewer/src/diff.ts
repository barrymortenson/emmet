// SPDX-License-Identifier: AGPL-3.0-or-later
// The diff between two placement lists, by Emmet id (plan section 13). Pure, so it's tested without WebGL.

/** One placed part, as build/placements.json lists it. The matrix is column-major, in LDraw units (−Y up). */
export interface Placement {
  id: string;
  part: string;
  color: number;
  matrix: number[];
  step: number;
  submodel: string[];
  src?: string;
}

export interface Diff {
  added: Placement[];
  removed: Placement[];
  /** Same part and color, a new matrix. */
  moved: Placement[];
  /** A new color or a new part under the same id: it changes bucket. */
  recolored: Placement[];
  unchanged: number;
}

/** The (part, color) bucket a placement draws in. */
export function bucketKey(p: Pick<Placement, "part" | "color">): string {
  return `${p.part.toLowerCase()}|${p.color}`;
}

function sameMatrix(a: number[], b: number[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
  return true;
}

/** What changes from `before` to `after`. A repeated id keeps its last placement, as a map would. */
export function diffPlacements(before: Iterable<Placement>, after: Iterable<Placement>): Diff {
  const old = new Map<string, Placement>();
  for (const p of before) old.set(p.id, p);
  const next = new Map<string, Placement>();
  for (const p of after) next.set(p.id, p);
  const diff: Diff = { added: [], removed: [], moved: [], recolored: [], unchanged: 0 };
  for (const [id, p] of next) {
    const was = old.get(id);
    if (!was) diff.added.push(p);
    else if (bucketKey(was) !== bucketKey(p)) diff.recolored.push(p);
    else if (!sameMatrix(was.matrix, p.matrix)) diff.moved.push(p);
    else diff.unchanged++;
  }
  for (const [id, p] of old) if (!next.has(id)) diff.removed.push(p);
  return diff;
}

/** The buckets a diff touches: each one has to be rewritten, and every other bucket stays as it is. */
export function dirtyBuckets(diff: Diff, before: ReadonlyMap<string, Placement>): Set<string> {
  const keys = new Set<string>();
  for (const p of [...diff.added, ...diff.removed, ...diff.moved]) keys.add(bucketKey(p));
  for (const p of diff.recolored) {
    keys.add(bucketKey(p));
    const was = before.get(p.id);
    if (was) keys.add(bucketKey(was));
  }
  return keys;
}
