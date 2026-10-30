# SPDX-License-Identifier: Apache-2.0
"""Grass block: single raised blocks scattered over a lawn, each a dirt brick under a grass top with one stud.

m.use("grass_block", x, y, z, length=8, depth=8, count=3, tall=0.3, seed=1, dirt="Reddish Brown",
grass="Bright Green", title="Grass blocks") sets `count` blocks on the 2-stud grid of the length x depth lawn from
(x, y), on its studs at height z, the cells picked by seed. A block is 2 x 2 studs and 5 plates: a 2 x 2 brick in the
dirt color, a 2 x 2 plate and a 2 x 2 jumper in the grass color, so its top is smooth but for one stud in the middle,
the way a game block's face reads. About `tall` of them stand two blocks high, a dirt block under the grass one. For a
sand block use Tan for both colors; for stone, Light Bluish Gray. length and depth: even. Returns the blocks as
(x, y, z of the top), in the order placed.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3003", "3022", "87580"]


def build(
    m,
    x,
    y,
    z,
    length=8,
    depth=8,
    count=3,
    tall=0.3,
    seed=1,
    dirt="Reddish Brown",
    grass="Bright Green",
    title="Grass blocks",
):
    """The blocks; returns their (x, y, z of the top)."""
    if length % 2 or depth % 2 or length < 2 or depth < 2:
        raise ValueError(
            f"grass_block takes an even length and depth of 2 or more, not {length}, {depth}"
        )
    cells = [(i, j) for j in range(0, depth, 2) for i in range(0, length, 2)]
    if not 1 <= count <= len(cells):
        raise ValueError(f"grass_block fits 1 to {len(cells)} blocks here, not {count}")
    rng = random.Random(seed)
    out = []
    for n, (i, j) in enumerate(rng.sample(cells, count)):
        bx, by, bz = x + i, y + j, z
        with m.step(f"{title}, block {n + 1}"):
            if rng.random() < tall:
                # A dirt block under it: a brick and two plates.
                m.add("3003", dirt, at=(bx, by, bz))
                m.stack("3022", dirt, at=(bx, by, bz + 3), count=2)
                bz += 5
            m.add("3003", dirt, at=(bx, by, bz))
            m.add("3022", grass, at=(bx, by, bz + 3))
            m.add("87580", grass, at=(bx, by, bz + 4))
        out.append((bx, by, bz + 5))
    return out


def example(m):
    """Grass blocks on an 8 x 8 lawn, and a sand block."""
    with m.step("Lawn"):
        m.add("41539", "Bright Green", at=(0, 0, 0))
    m.use("grass_block", x=0, y=0, z=1, length=8, depth=6, count=3, seed=2)
    m.use(
        "grass_block",
        x=6,
        y=6,
        z=1,
        length=2,
        depth=2,
        count=1,
        tall=0,
        dirt="Tan",
        grass="Tan",
        title="Sand block",
    )
