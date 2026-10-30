# SPDX-License-Identifier: Apache-2.0
"""Stepping stones: a meandering trail of flat stones across a lawn, each a tile of a different outline, turned its own
way, with grass between them.

m.use("stepping_stones", x, y, z, length=12, wander=2, gap=1, seed=1, colors=("Dark Bluish Gray", "Light Bluish Gray"),
title="Stepping stones") sets stones on the studs at height z (a baseplate is z = 0.5, or the top of a lawn), along x
from x for `length` studs, each somewhere from y to y + wander + 2: a round 2 x 2 tile (14769), a 2 x 2 tile with a
rounded end (5520), a 3 x 2 tile with an angled end (22385), a plain 2 x 2 or 2 x 3 tile, each turned one of the ways it
can face, no two the same in a row, and `gap` studs of grass between neighbors so each reads as a stone of its own. A
round 1 x 1 tile beside a stone, here and there, is a pebble. Each stone is one of `colors`, mostly the first. seed
picks the shapes, turns, and wander. Returns the z of the stones' top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10315. Not derived from any
model file."""

PARTS = ["14769", "5520", "22385", "3068b", "26603", "98138"]

# (part, size along x, size along y, the turns it can take) for a stone at rot 0; the size swaps on an odd turn.
STONES = [
    ("14769", 2, 2, (0,)),
    ("5520", 2, 2, (0, 1, 2, 3)),
    ("22385", 3, 2, (0, 1, 2, 3)),
    ("3068b", 2, 2, (0,)),
    ("26603", 2, 3, (0, 1)),
]


def build(
    m,
    x,
    y,
    z,
    length=12,
    wander=2,
    gap=1,
    seed=1,
    colors=("Dark Bluish Gray", "Light Bluish Gray"),
    title="Stepping stones",
):
    """The stones and pebbles; returns the z of the stones' top."""
    if not 4 <= length <= 32 or not 0 <= wander <= 6 or gap not in (1, 2):
        raise ValueError(
            f"stepping_stones takes a length of 4 to 32, a wander of 0 to 6, and a gap of 1 or 2, not "
            f"{length}, {wander}, {gap}"
        )
    rng = random.Random(seed)
    stones, at, last, dy = [], x, None, rng.randint(0, wander)
    while True:
        # Every stone and turn that fits in what's left of the length, never the shape just laid.
        fits = [
            (part, (d, w) if rot % 2 else (w, d), rot)
            for part, w, d, turns in STONES
            for rot in turns
            if part != last and at + (d if rot % 2 else w) <= x + length
        ]
        if not fits:
            break
        part, (w, d), rot = rng.choice(fits)
        dy = max(0, min(wander + 2 - d, dy + rng.choice((-1, 0, 1))))
        color = (
            colors[0]
            if rng.random() < 0.7 or len(colors) == 1
            else rng.choice(colors[1:])
        )
        stones.append((part, color, at, y + dy, rot, w, d))
        last, at = part, at + w + gap
    for k in range(0, len(stones), 3):
        with m.step(f"{title}, {k + 1} to {min(k + 3, len(stones))}"):
            for part, color, sx, sy, rot, w, d in stones[k : k + 3]:
                m.add(part, color, at=(sx, sy, z), rot=rot)
    # Pebbles: a stud in a gap, just in front of or behind the stones either side of it.
    pebbles = []
    for (_, _, ax, ay, _, aw, ad), (_, _, bx, by, _, bw, bd) in zip(stones, stones[1:]):
        if rng.random() < 0.5:
            py = rng.choice((min(ay, by) - 1, max(ay + ad, by + bd)))
            if y - 1 <= py <= y + wander + 2:
                pebbles.append((ax + aw, py))
    if pebbles:
        with m.step(f"{title}, pebbles"):
            for px, py in pebbles:
                m.add("98138", colors[-1], at=(px, py, z))
    return z + 1


def example(m):
    """A trail of stones across a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("stepping_stones", x=1, y=5, z=0.5, length=15, wander=3, seed=4)
