# SPDX-License-Identifier: Apache-2.0
"""Cactus column: a 2 x 2 green column of game blocks with studs on every side, the studs read as spines.

m.use("cactus_column", x, y, z, blocks=2, seed=1, colors=("Green", "Dark Green"), title="Cactus") stacks `blocks`
blocks (1 to 4) on the 2 x 2 studs from (x, y), on the studs at height z (sand, as a rule). Each block is 2 x 2 studs
and 5 plates, a cube on the block grid: four 1 x 1 bricks with studs on four sides (4733), so each face shows two
spines, then two 2 x 2 plates that tie them, the top one a 2 x 2 jumper on the last block, so the column ends in one
stud. Each brick takes one of `colors` by seed, two greens mixed like a game texture. Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["4733", "3022", "87580"]


def build(m, x, y, z, blocks=2, seed=1, colors=("Green", "Dark Green"), title="Cactus"):
    """The blocks, bottom up; returns the z of the top."""
    if not 1 <= blocks <= 4:
        raise ValueError(f"cactus_column takes 1 to 4 blocks, not {blocks}")
    if not colors:
        raise ValueError("cactus_column needs at least one color")
    rng = random.Random(seed)
    for b in range(blocks):
        zb = z + 5 * b
        with m.step(f"{title}, block {b + 1}"):
            for k in range(4):
                m.add("4733", rng.choice(colors), at=(x + k % 2, y + k // 2, zb))
            m.add("3022", colors[0], at=(x, y, zb + 3))
            m.add("87580" if b == blocks - 1 else "3022", colors[0], at=(x, y, zb + 4))
    return z + 5 * blocks


def example(m):
    """A cactus three blocks tall on a patch of sand."""
    with m.step("Sand"):
        m.add("3958", "Tan", at=(0, 0, 0))
    m.use("cactus_column", x=2, y=2, z=1, blocks=3)
