# SPDX-License-Identifier: Apache-2.0
"""Rock facet: a rock face broken into angled planes by cheese slopes pressed onto side studs, so they point up, down,
left, or right instead of all up.

m.use("rock_facet", x, y, z, length, rows=2, seed=1, color="Dark Bluish Gray", facet="Light Bluish Gray",
title="Rock face") builds a face one stud thick, `length` studs along x from (x, y), `rows` bricks tall from z, its
front at y. In each course a few bricks have studs on their front side (-y): a 1 x 1 with one (87087), carrying a 1 x 1
cheese slope that faces up, down, left, or right, or a 1 x 2 with two (11211), carrying a 1 x 2 cheese slope that
faces up or down. Plain bricks fill between. The facets stand 0.78 studs proud of the face, so the face reads as
broken rock. seed picks where the facets go and which way they point. Returns the z of the top.
"""

import random

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 21335 (sideways slope on a side stud). Not derived from any model file."""

PARTS = ["87087", "11211", "54200", "85984", "3005", "3004", "3622"]

PROUD = 0.78  # a cheese slope on its back stands out 1.95 plates, in studs
# A 1 x 1 cheese slope on a front stud, its face to the right, down, up, or left.
SMALL = (Rot(x=90), Rot(x=90, y=90), Rot(x=90, y=-90), Rot(x=90, y=180))
# A 1 x 2 cheese slope on two front studs, lying along x, its face down or up.
LONG = (Rot(x=90, y=90), Rot(x=90, y=-90))


def _course(rng, length):
    """[(start, kind, n)] for one course: kind 1 is a single side stud, 2 a double, 0 n studs of plain bricks."""
    out, k = [], 0
    while k < length:
        roll = rng.random()
        if roll < 0.25 and k + 2 <= length:
            out.append((k, 2, 2))
            k += 2
        elif roll < 0.55:
            out.append((k, 1, 1))
            k += 1
        else:
            n = min(length - k, rng.choice((1, 2, 3)))
            out.append((k, 0, n))
            k += n
    return out


def build(
    m,
    x,
    y,
    z,
    length,
    rows=2,
    seed=1,
    color="Dark Bluish Gray",
    facet="Light Bluish Gray",
    title="Rock face",
):
    """The face and its facets; returns the z of the top."""
    if not 2 <= length <= 16 or not 1 <= rows <= 8:
        raise ValueError(
            f"rock_facet takes a length of 2 to 16 and 1 to 8 rows, not {length}, {rows}"
        )
    rng = random.Random(seed)
    for r in range(rows):
        zr = z + 3 * r
        pieces = _course(rng, length)
        with m.step(f"{title}, row {r + 1}: bricks"):
            for k, kind, n in pieces:
                if kind == 0:
                    m.row(Brick, color, at=(x + k, y, zr), length=n, sizes=(1, 2, 3))
                elif kind == 1:
                    m.add("87087", color, at=(x + k, y, zr), rot=3)
                else:
                    m.add("11211", color, at=(x + k, y, zr), rot=3)
        with m.step(f"{title}, row {r + 1}: facets"):
            for k, kind, n in pieces:
                at = (x + k, y - PROUD, zr + 0.5)
                if kind == 1:
                    m.add("54200", facet, at=at, rot=rng.choice(SMALL))
                elif kind == 2:
                    m.add("85984", facet, at=at, rot=rng.choice(LONG))
    return z + 3 * rows


def example(m):
    """A rock face 8 studs long and 3 bricks tall."""
    with m.step("Base"):
        m.add("3460", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("rock_facet", x=0, y=0, z=1, length=8, rows=3)
