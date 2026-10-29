# SPDX-License-Identifier: Apache-2.0
"""Thick wall: a castle wall 2 studs thick, header courses of 2-wide bricks across it between courses of two 1-wide
leaves, so the two faces are tied together, with darker stones scattered through it.

m.use("thick_wall", x, y, z, length, rows=4, color="Light Bluish Gray", stone="Dark Bluish Gray", stones=0.2, seed=1,
title="Thick wall") lays `rows` courses of a wall 2 studs thick and `length` studs long along x from (x, y), from z.
Even courses are headers: 2 x n bricks across the whole thickness. Odd courses are two leaves of 1 x n bricks, the front
one starting with a 1 x 2 and the back one with a 1 x 3, so no joint of either sits on a joint of the header course
below or above, nor on the other leaf's. A share `stones` of the bricks, picked by seed, are in the stone color, the
patchy look of an old wall. Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["3001", "3002", "3003", "3004", "3622", "3010", "3005"]

HEADERS = {
    4: "3001",
    3: "3002",
    2: "3003",
    1: "3004",
}  # Brick 2 x n (a 1 x 2 turned across, for 1)
STRETCHERS = {4: "3010", 3: "3622", 2: "3004", 1: "3005"}  # Brick 1 x n


def _sizes(n, first, have):
    """A run of n studs as piece lengths: `first`, then 4s, then what's left, as the longest pieces there are."""
    out = []
    if first and first <= n:
        out.append(first)
        n -= first
    for k in sorted(have, reverse=True):
        while n >= k:
            out.append(k)
            n -= k
    return out


def build(
    m,
    x,
    y,
    z,
    length,
    rows=4,
    color="Light Bluish Gray",
    stone="Dark Bluish Gray",
    stones=0.2,
    seed=1,
    title="Thick wall",
):
    """The courses, headers and leaves in turn; returns the z of the top."""
    if length < 4 or rows < 1 or not 0 <= stones <= 1:
        raise ValueError(
            f"thick_wall needs a length of 4 or more, 1 row or more and stones from 0 to 1, not {length}, {rows}, "
            f"{stones}"
        )
    rng = random.Random(seed)

    def pick():
        return stone if rng.random() < stones else color

    for r in range(rows):
        zr = z + 3 * r
        if r % 2 == 0:
            with m.step(f"{title}, row {r + 1}: headers"):
                u = x
                for k in _sizes(length, None, HEADERS):
                    m.add(HEADERS[k], pick(), at=(u, y, zr), rot=0 if k == 1 else 1)
                    u += k
        else:
            for leaf, row, first in (("back", y + 1, 3), ("front", y, 2)):
                with m.step(f"{title}, row {r + 1}: {leaf} leaf"):
                    u = x
                    for k in _sizes(length, first, STRETCHERS):
                        m.add(STRETCHERS[k], pick(), at=(u, row, zr), rot=1)
                        u += k
    return z + 3 * rows


def example(m):
    """A stretch of thick wall 8 studs long and four courses tall."""
    m.use("thick_wall", x=0, y=0, z=0, length=8)
