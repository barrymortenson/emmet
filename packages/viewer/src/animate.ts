// SPDX-License-Identifier: AGPL-3.0-or-later
// The build animation's order and timing (plan section 13, D-34), apart from three.js so they're tested in Node.

import type { Placement } from "./diff.js";

/** How far a part drops, in LDU: three plates. */
export const DROP = 24;
/** How long one part takes to drop, in ms. */
export const PART_MS = 160;
/** The gap between one part's start and the next's, in ms, when the batch fits the cap. */
export const STAGGER_MS = 30;
/** The longest a batch takes, however many parts it has, in ms. */
export const CAP_MS = 6000;

const PLATE = 8;

/** The order parts go in: by build step, then bottom-up a plate at a time (LDraw's −Y is up, so a larger y is lower),
 * then each one nearest the part before it, so no part appears floating and the model grows outward. */
export function buildOrder(added: readonly Placement[]): Placement[] {
  const pos = (p: Placement) => [p.matrix[12] ?? 0, p.matrix[13] ?? 0, p.matrix[14] ?? 0] as const;
  const layers = new Map<string, Placement[]>();
  for (const p of added) {
    const key = `${p.step}|${-Math.round(pos(p)[1] / PLATE)}`;
    let list = layers.get(key);
    if (!list) layers.set(key, (list = []));
    list.push(p);
  }
  const keys = [...layers.keys()].sort((a, b) => {
    const [sa, la] = a.split("|").map(Number) as [number, number];
    const [sb, lb] = b.split("|").map(Number) as [number, number];
    return sa - sb || la - lb;
  });
  const out: Placement[] = [];
  for (const key of keys) {
    const rest = [...layers.get(key)!].sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
    // Greedy: the next part is the one nearest the last placed (the first layer starts from its first part).
    let last = out.length ? pos(out[out.length - 1]!) : pos(rest[0]!);
    while (rest.length) {
      let best = 0;
      let bestD = Infinity;
      for (let i = 0; i < rest.length; i++) {
        const [x, y, z] = pos(rest[i]!);
        const d = (x - last[0]) ** 2 + (y - last[1]) ** 2 + (z - last[2]) ** 2;
        if (d < bestD) {
          bestD = d;
          best = i;
        }
      }
      const [next] = rest.splice(best, 1);
      out.push(next!);
      last = pos(next!);
    }
  }
  return out;
}

/** The gap between parts' starts for a batch of `n`: STAGGER_MS, or less, so the last part lands within CAP_MS. */
export function stagger(n: number): number {
  if (n <= 1) return 0;
  return Math.min(STAGGER_MS, (CAP_MS - PART_MS) / (n - 1));
}

/** Where a part is `elapsed` ms after its own start: hidden before it, falling with ease-out for PART_MS, then home.
 * `lift` is how far above its place it is, in LDU. */
export function drop(elapsed: number): { visible: boolean; lift: number; done: boolean } {
  if (elapsed < 0) return { visible: false, lift: DROP, done: false };
  if (elapsed >= PART_MS) return { visible: true, lift: 0, done: true };
  const t = elapsed / PART_MS;
  return { visible: true, lift: DROP * (1 - t) ** 3, done: false };
}
