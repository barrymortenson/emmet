# SPDX-License-Identifier: Apache-2.0
"""Masonry wall: a straight wall of bricks with embossed brickwork, in bond, with a few old bricks patched in.

m.use("masonry_wall", x, y, z, length, rows, color="Medium Nougat", patch="Dark Tan", patches=2, seed=1,
title="Brick wall") lays `rows` courses of 1 x 4 and 1 x 2 embossed bricks (15533, 98283) along x from (x, y, z), each
course starting half a brick over from the one below, and swaps `patches` bricks (picked by seed) for another color,
the way an old wall shows repairs. Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350. Not derived from
any model file."""

PARTS = ["15533", "98283"]

PIECES = {4: "15533", 2: "98283"}


def build(
    m,
    x,
    y,
    z,
    length,
    rows,
    color="Medium Nougat",
    patch="Dark Tan",
    patches=2,
    seed=1,
    title="Brick wall",
):
    """The wall; returns the z of its top."""
    if length < 2 or length % 2 or rows < 1:
        raise ValueError(
            f"masonry_wall needs an even length of 2 or more and 1 row or more, not {length}, {rows}"
        )
    rng = random.Random(seed)
    courses = []
    for r in range(rows):
        sizes, left = (
            ([2] if r % 2 and length > 2 else []),
            length - (2 if r % 2 and length > 2 else 0),
        )
        while left >= 4:
            sizes.append(4)
            left -= 4
        if left:
            sizes.append(left)
        courses.append(sizes)
    every = [(r, k) for r, sizes in enumerate(courses) for k in range(len(sizes))]
    old = set(rng.sample(every, min(patches, len(every))))
    for r, sizes in enumerate(courses):
        with m.step(f"{title}, row {r + 1}"):
            at = x
            for k, n in enumerate(sizes):
                m.add(
                    PIECES[n],
                    patch if (r, k) in old else color,
                    at=(at, y, z + 3 * r),
                    rot=1,
                )
                at += n
    return z + 3 * rows


def example(m):
    """A wall 8 studs long and 4 bricks tall."""
    m.use("masonry_wall", x=0, y=0, z=0, length=8, rows=4)
