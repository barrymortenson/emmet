# SPDX-License-Identifier: Apache-2.0
"""Woodpile: a stack of cut logs seen end on, round 1 x 1 tiles in wood colors pressed onto a face of side studs, each
row shorter than the one below so the pile tapers.

m.use("woodpile", x, y, z, length=4, rows=3, seed=1, wood="Reddish Brown", ends=("Tan", "Medium Nougat", "Dark Tan"),
title="Woodpile") stacks, on the studs at height z, `rows` courses of bricks with studs on their front (-y), the
bottom one `length` studs along x from x, each next one a stud shorter and stepped in from the left every other
row, so the pile narrows to the top; the courses are 1 x 2 bricks with two side studs (11211) and, for an odd stud, a
1 x 1 with one (87087), the odd one at alternate ends so the joints don't line up. Each side stud carries a round
1 x 1 tile, its face to the front, in one of `ends` at random: the logs' cut ends. The backing bricks are `wood`, the
color of the bark showing between. length: 2 to 8; rows: 1 to length. seed picks the ends' colors. Returns the z of
the top.
"""

import random

from emmet_dsl import Rot
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["11211", "87087", "98138"]

FACE = Rot(x=90)  # a round tile on its back, its face to -y
OUT = 0.4  # a tile's thickness, in studs


def build(
    m,
    x,
    y,
    z,
    length=4,
    rows=3,
    seed=1,
    wood="Reddish Brown",
    ends=("Tan", "Medium Nougat", "Dark Tan"),
    title="Woodpile",
):
    """The pile; returns the z of its top."""
    if not 2 <= length <= 8 or not 1 <= rows <= length:
        raise ValueError(
            f"woodpile takes a length of 2 to 8 and 1 to length rows, not {length}, {rows}"
        )
    if not ends:
        raise ValueError("woodpile needs at least one color for the ends")
    rng = random.Random(seed)
    for r in range(rows):
        n = length - r
        # The rows step in a stud every other row, so the pile narrows evenly.
        x0 = x + r // 2
        zr = z + 3 * r
        pieces, k = [], 0
        if n % 2 and r % 2:
            pieces.append((k, "87087"))
            k += 1
        while k + 2 <= n:
            pieces.append((k, "11211"))
            k += 2
        if k < n:
            pieces.append((k, "87087"))
        actions = []
        for k, part in pieces:
            at = (x0 + k, y, zr)
            actions.append((1, lambda p=part, at=at: m.add(p, wood, at=at, rot=3)))
        for k in range(n):
            c = rng.choice(ends)
            at = (x0 + k, y - OUT, zr + 0.5)
            actions.append((1, lambda c=c, at=at: m.add("98138", c, at=at, rot=FACE)))
        in_steps(m, f"{title}, row {r + 1}", actions)
    return z + 3 * rows


def example(m):
    """A pile of logs 5 studs long, three rows tall, on a patch of ground."""
    with m.step("Ground"):
        m.add("3795", "Dark Tan", at=(0, 0, 0), rot=1)
    m.use("woodpile", x=0, y=1, z=1, length=5, rows=3)
