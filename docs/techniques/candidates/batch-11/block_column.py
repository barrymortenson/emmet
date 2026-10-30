# SPDX-License-Identifier: Apache-2.0
"""Block column: a stack of game blocks on the 2 x 2 grid, each a cube 5 plates tall, textured by mixing shades.

m.use("block_column", x, y, z, blocks=3, colors=("Light Bluish Gray", "Dark Bluish Gray"), height=5, seed=1,
cap=True, title="Block column") stacks `blocks` blocks on the 2 x 2 studs from (x, y), on the studs at height z. A
block is the blocky style's unit: 2 x 2 studs and `height` plates (5, a brick and two plates, is a true cube; 4 or 6
also read as one). Its brick layer is two 1 x 2 bricks, turned a quarter turn block to block, then 2 x 2 plates that
tie them, one to three to make up the height. Each piece takes one of `colors` by seed, so the column shows two or
three shades mixed like a game texture. With cap, the top plate is a 2 x 2 jumper, so the column ends in one smooth
block with a single stud. Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3004", "3022", "87580"]


def build(
    m,
    x,
    y,
    z,
    blocks=3,
    colors=("Light Bluish Gray", "Dark Bluish Gray"),
    height=5,
    seed=1,
    cap=True,
    title="Block column",
):
    """The blocks, bottom up; returns the z of the top."""
    if blocks < 1 or height not in (4, 5, 6):
        raise ValueError(
            f"block_column needs 1 block or more and a height of 4, 5, or 6 plates, not {blocks}, {height}"
        )
    if not colors:
        raise ValueError("block_column needs at least one color")
    rng = random.Random(seed)
    for b in range(blocks):
        zb = z + height * b
        top = cap and b == blocks - 1
        with m.step(f"{title}, block {b + 1}"):
            # Two 1 x 2 bricks, along x on even blocks and along y on odd ones, so each block's bricks cross the
            # last one's.
            for k in range(2):
                if b % 2 == 0:
                    m.add("3004", rng.choice(colors), at=(x, y + k, zb), rot=1)
                else:
                    m.add("3004", rng.choice(colors), at=(x + k, y, zb))
            for p in range(3, height):
                plate = "87580" if top and p == height - 1 else "3022"
                m.add(plate, rng.choice(colors), at=(x, y, zb + p))
    return z + height * blocks


def example(m):
    """A stone column of three blocks beside a two-block wood one, on a plate."""
    with m.step("Ground"):
        m.add("3020", "Bright Green", at=(0, 0, 0), rot=1)
    m.use("block_column", x=0, y=0, z=1, blocks=3)
    m.use(
        "block_column",
        x=2,
        y=0,
        z=1,
        blocks=2,
        colors=("Tan", "Medium Nougat", "Reddish Brown"),
        seed=3,
        title="Wood column",
    )
