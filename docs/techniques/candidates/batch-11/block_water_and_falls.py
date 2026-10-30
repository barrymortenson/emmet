# SPDX-License-Identifier: Apache-2.0
"""Block water and falls: a square pool set flush in the ground and a column of see-through blocks falling into it
down a stone face, water the way a block world draws it.

m.use("block_water_and_falls", x, y, z, width=6, rows=4, pool=3, seed=1, stone=("White", "Light Bluish Gray"),
water="Trans Light Blue", title="Falls") builds, on the studs at height z, a pool `width` x `pool` studs from (x, y)
and behind it a stone face `width` studs wide, 2 deep and `rows` bricks tall. The face is 2 x 2 to 2 x 4 bricks in
the `stone` shades mixed by seed, each course's joints missing the ones below, with a 2 x 2 jumper on each block of
its top. Down its middle, in front of it, falls a column of 1 x 2 bricks in the water color, one a course, with a
2 x 2 tile of water on the top behind it as the source. The pool is tiles in the water color on the same studs,
square-edged, around the foot of the fall; lay ground plates around it and its surface is flush. width: even, 4 or
more. Returns the z of the face's top.
"""

import random

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3003", "3002", "3001", "3004", "87580", "3068b", "3069b", "2431"]

STONE = {2: "3003", 3: "3002", 4: "3001"}  # 2 x n bricks


def _course(rng, length, below):
    """The (start, size) of one course: 2 to 4 long by seed, no joint over a joint below where it can."""
    out, at = [], 0
    while at < length:
        sizes = [n for n in (2, 3, 4) if at + n <= length and length - at - n != 1] or [
            length - at
        ]
        good = [n for n in sizes if at + n == length or at + n not in below] or sizes
        n = rng.choice(good)
        out.append((at, n))
        at += n
    return out


def build(
    m,
    x,
    y,
    z,
    width=6,
    rows=4,
    pool=3,
    seed=1,
    stone=("White", "Light Bluish Gray"),
    water="Trans Light Blue",
    title="Falls",
):
    """The pool, the face and the fall, a course at a time, then the top; returns the z of the top."""
    if width % 2 or width < 4 or rows < 1 or pool < 1:
        raise ValueError(
            "block_water_and_falls takes an even width of 4 or more, 1 row or more, and a pool of 1 or more, "
            f"not {width}, {rows}, {pool}"
        )
    rng = random.Random(seed)
    mid = x + 2 * (
        width // 4
    )  # the fall's left edge, on the middle block (right of middle when there's none)
    face = y + pool  # the stone face's front
    # 2 x 2 tiles over the pool but its last row, a row of 1 x n when that leaves an odd one, then the last row,
    # either side of the fall's foot: a step a row.
    for j in range(0, pool - 2, 2):
        with m.step(f"{title}, pool"):
            m.grid("3068b", water, at=(x, y + j, z), nx=width // 2)
    if (pool - 1) % 2:
        with m.step(f"{title}, pool"):
            m.row(Tile, water, at=(x, face - 2, z), length=width, sizes=(4, 2, 1))
    with m.step(f"{title}, pool"):
        for start, run in ((x, mid - x), (mid + 2, x + width - mid - 2)):
            if run:
                m.row(Tile, water, at=(start, face - 1, z), length=run, sizes=(4, 2, 1))
    below = set()
    for r in range(rows):
        zr = z + 3 * r
        pieces = _course(rng, width, below)
        with m.step(f"{title}, row {r + 1}"):
            for c, n in pieces:
                m.add(STONE[n], rng.choice(stone), at=(x + c, face, zr), rot=1)
            m.add("3004", water, at=(mid, face - 1, zr), rot=1)
        below = {c + n for c, n in pieces}
    top = z + 3 * rows
    with m.step(f"{title}, top"):
        for c in range(0, width, 2):
            if x + c == mid:
                m.add("3068b", water, at=(mid, face, top))
            else:
                m.add("87580", rng.choice(stone), at=(x + c, face, top))
        m.add("3069b", water, at=(mid, face - 1, top), rot=1)
    return top


def example(m):
    """Falls four bricks tall into a pool, on a grass plate."""
    with m.step("Ground"):
        m.add("3036", "Bright Green", at=(0, 0, 0), rot=1)
    m.use("block_water_and_falls", x=0, y=0, z=1, width=8, rows=4, pool=4)
