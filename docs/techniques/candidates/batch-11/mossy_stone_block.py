# SPDX-License-Identifier: Apache-2.0
"""Mossy stone block: a gray game block with green showing in a thin band around its side, like moss in the seams.

m.use("mossy_stone_block", x, y, z, blocks=1, moss=2, seed=1, stone="Light Bluish Gray", moss_color="Green",
title="Mossy block") stacks `blocks` blocks on the 2 x 2 studs from (x, y), on the studs at height z. Each is 2 x 2
studs and 5 plates, a cube on the block grid: a 2 x 2 brick in the stone color, then four 1 x 1 round plates, `moss`
of them (0 to 4, which ones by seed) in the moss color and the rest in stone, then a 2 x 2 plate, or on the top block
a 2 x 2 jumper, so it ends in one stud like any block. The round plates sit in from the block's edge, so the band
reads as a shadowed seam with moss in it. Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3003", "6141", "3022", "87580"]


def build(
    m,
    x,
    y,
    z,
    blocks=1,
    moss=2,
    seed=1,
    stone="Light Bluish Gray",
    moss_color="Green",
    title="Mossy block",
):
    """The blocks, bottom up; returns the z of the top."""
    if blocks < 1 or not 0 <= moss <= 4:
        raise ValueError(
            f"mossy_stone_block needs 1 block or more and a moss of 0 to 4, not {blocks}, {moss}"
        )
    rng = random.Random(seed)
    for b in range(blocks):
        zb = z + 5 * b
        green = set(rng.sample(range(4), moss))
        with m.step(f"{title}, block {b + 1}"):
            m.add("3003", stone, at=(x, y, zb))
            for k in range(4):
                color = moss_color if k in green else stone
                m.add("6141", color, at=(x + k % 2, y + k // 2, zb + 3))
            m.add("87580" if b == blocks - 1 else "3022", stone, at=(x, y, zb + 4))
    return z + 5 * blocks


def example(m):
    """Three mossy blocks in a row, the middle one two high, on a stone plate."""
    with m.step("Ground"):
        m.add("3795", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("mossy_stone_block", x=0, y=0, z=1, moss=2, seed=1)
    m.use("mossy_stone_block", x=2, y=0, z=1, blocks=2, moss=3, seed=2)
    m.use("mossy_stone_block", x=4, y=0, z=1, moss=1, seed=3)
