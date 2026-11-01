// SPDX-License-Identifier: AGPL-3.0-or-later
// The build animation's order and timing (plan section 13, D-34), apart from three.js so they're tested in Node.

import { type Placement, topStep } from "./diff.js";

/** How far a part drops, in LDU: three plates. */
export const DROP = 24;
/** How long one part takes to drop and fade in, in ms. */
export const PART_MS = 160;
/** How long a removed part takes to lift and fade out, in ms. */
export const LEAVE_MS = 150;
/** How long a moved part takes to glide to its new place, in ms. */
export const GLIDE_MS = 200;
/** The gap between one part's start and the next's, in ms, when the batch fits the cap. */
export const STAGGER_MS = 30;
/** The longest a batch takes, however many parts it has, in ms: Animated, and Fast. */
export const CAP_MS = 6000;
export const FAST_MS = 600;

/** Settings' build animation: Animated (one part after another), Fast (the batch in 0.6 s), or Off. */
export type Pace = "animated" | "fast" | "off";

const PLATE = 8;

/** The order parts go in: by build step, then bottom-up a plate at a time (LDraw's −Y is up, so a larger y is lower),
 * then each one nearest the part before it, so no part appears floating and the model grows outward. A sub-assembly's
 * parts go in at the step that places the copy, in its own steps' order. */
export function buildOrder(added: readonly Placement[]): Placement[] {
  const pos = (p: Placement) => [p.matrix[12] ?? 0, p.matrix[13] ?? 0, p.matrix[14] ?? 0] as const;
  const layers = new Map<string, Placement[]>();
  for (const p of added) {
    const inner = p.copies?.length ? p.step : 0;
    const key = `${topStep(p)}|${inner}|${-Math.round(pos(p)[1] / PLATE)}`;
    let list = layers.get(key);
    if (!list) layers.set(key, (list = []));
    list.push(p);
  }
  const keys = [...layers.keys()].sort((a, b) => {
    const [sa, ia, la] = a.split("|").map(Number) as [number, number, number];
    const [sb, ib, lb] = b.split("|").map(Number) as [number, number, number];
    return sa - sb || ia - ib || la - lb;
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

/** The gap between parts' starts for a batch of `n`: STAGGER_MS, or less, so the last part lands within CAP_MS
 * (Animated) or FAST_MS (Fast). */
export function stagger(n: number, pace: Pace = "animated"): number {
  if (n <= 1 || pace === "off") return 0;
  const cap = pace === "fast" ? FAST_MS : CAP_MS;
  return Math.min(STAGGER_MS, Math.max(0, cap - PART_MS) / (n - 1));
}

const easeOut = (t: number) => 1 - (1 - t) ** 3;

/** Where an added part is `elapsed` ms after its own start: hidden before it, then falling with ease-out and fading
 * in for PART_MS, then home. `lift` is how far above its place it is, in LDU; `fade` how far it has faded in, 0 to 1. */
export function drop(elapsed: number): { visible: boolean; lift: number; fade: number; done: boolean } {
  if (elapsed < 0) return { visible: false, lift: DROP, fade: 0, done: false };
  if (elapsed >= PART_MS) return { visible: true, lift: 0, fade: 1, done: true };
  const t = elapsed / PART_MS;
  return { visible: true, lift: DROP * (1 - easeOut(t)), fade: easeOut(t), done: false };
}

/** A removed part `elapsed` ms after the batch starts: lifting and fading out over LEAVE_MS, then gone. */
export function leave(elapsed: number): { visible: boolean; lift: number; fade: number; done: boolean } {
  if (elapsed >= LEAVE_MS) return { visible: false, lift: DROP, fade: 0, done: true };
  const t = easeOut(Math.max(0, elapsed) / LEAVE_MS);
  return { visible: true, lift: DROP * t, fade: 1 - t, done: false };
}

/** How far along its glide a moved part is, 0 to 1, `elapsed` ms after the batch starts. */
export function glide(elapsed: number): { t: number; done: boolean } {
  if (elapsed >= GLIDE_MS) return { t: 1, done: true };
  return { t: easeOut(Math.max(0, elapsed) / GLIDE_MS), done: false };
}
