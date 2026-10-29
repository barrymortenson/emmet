# SPDX-License-Identifier: Apache-2.0
"""Dirt path: a path two studs wide winding across the grass, smooth tiles on the ground's studs, its bends rounded
off with 1 x 1 corner-round tiles, and a few pebbles beside it.

m.use("dirt_path", x, y, z, length, wander=2, seed=1, color="Tan", pebbles="Light Bluish Gray", title="Path") lays a
path on the studs at height z (a green baseplate or plates), `length` studs along x from x (even), in blocks of 2 x 2
studs, each block a stud forward or back of the one before, staying within y to y + wander + 2. A block is a 2 x 2
tile, except where a bend leaves a corner of it standing out: there that column is a 1 x 1 corner-round tile (25269),
its round to the outside, and a 1 x 1 tile, and the other column a 1 x 2 tile, so every bend reads as a curve. A few
1 x 1 round tiles beside the path are pebbles. seed picks the bends and pebbles. Returns the z of the path's top.
"""

import random

from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["3068b", "3069b", "25269", "3070b", "98138"]

# The turn that puts a 1 x 1 corner-round tile's rounded corner at a stud's back-right, back-left, front-left,
# front-right.
TURN = {(1, 1): 0, (0, 1): 1, (0, 0): 2, (1, 0): 3}


def build(
    m,
    x,
    y,
    z,
    length,
    wander=2,
    seed=1,
    color="Tan",
    pebbles="Light Bluish Gray",
    title="Path",
):
    """The path and its pebbles; returns the z of the path's top."""
    if length < 2 or length % 2 or length > 32 or not 0 <= wander <= 6:
        raise ValueError(
            f"dirt_path takes an even length of 2 to 32 and a wander of 0 to 6, not {length}, {wander}"
        )
    rng = random.Random(seed)
    n = length // 2
    off = [rng.randint(0, wander)]
    for _ in range(n - 1):
        off.append(
            min(wander, max(0, off[-1] + rng.choice((-1, 1) if wander else (0,))))
        )
    # Each block's rounded corners, as (right?, back?) pairs: a bend rounds the outer corner on each side of it.
    corners = [set() for _ in range(n)]
    for i in range(1, n):
        if off[i] > off[i - 1]:
            corners[i].add((0, 1))
            corners[i - 1].add((1, 0))
        elif off[i] < off[i - 1]:
            corners[i].add((0, 0))
            corners[i - 1].add((1, 1))
    actions = []

    def add(part, at, rot=0, c=color):
        actions.append((1, lambda: m.add(part, c, at=at, rot=rot)))

    for i in range(n):
        bx, by = x + 2 * i, y + off[i]
        if not corners[i]:
            add("3068b", (bx, by, z))
            continue
        for right in (0, 1):
            col = [back for r, back in corners[i] if r == right]
            if not col:
                add("3069b", (bx + right, by, z))
            for back in col:
                add("25269", (bx + right, by + back, z), TURN[(right, back)])
                add("3070b", (bx + right, by + 1 - back, z))
    in_steps(m, f"{title}, path", actions)
    # Pebbles: a stud in front of or behind a block, clear of the path.
    actions = []
    path = {(2 * i + a, off[i] + b) for i in range(n) for a in (0, 1) for b in (0, 1)}
    spots = [
        (i, j)
        for i in range(length)
        for j in (off[i // 2] - 1, off[i // 2] + 2)
        if 0 <= j < wander + 2 and (i, j) not in path
    ]
    for i, j in rng.sample(spots, min(len(spots), max(1, length // 3))):
        add("98138", (x + i, y + j, z), c=pebbles)
    in_steps(m, f"{title}, pebbles", actions)
    return z + 1


def example(m):
    """A path 8 studs long across a patch of grass."""
    with m.step("Grass"):
        m.add("3035", "Green", at=(0, 0, 0), rot=1)
    m.use("dirt_path", x=0, y=0, z=1, length=8, wander=2, seed=8)
