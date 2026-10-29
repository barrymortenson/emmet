# SPDX-License-Identifier: Apache-2.0
"""Sidewalk: the street edge of a baseplate, a light curb along the street, dark paving behind it, and a few studs
left bare where a lamp, a bench, or a sign stands.

m.use("sidewalk", x, y, z, length, depth=3, curb="Light Bluish Gray", paving="Dark Bluish Gray", studs=(),
title="Sidewalk") covers `length` studs along x from (x, y, z) (z is the baseplate's top) and `depth` deep: a row of
1-wide tiles at the front (the curb), and rows of 1-wide tiles over the rest, except the cells in `studs`, (x, y)
pairs, which are left bare. Returns the z of its top.
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["4162", "6636", "2431", "3069b", "3070b"]


def build(
    m,
    x,
    y,
    z,
    length,
    depth=3,
    curb="Light Bluish Gray",
    paving="Dark Bluish Gray",
    studs=(),
    title="Sidewalk",
):
    """The curb and the paving; returns the z of the top."""
    bare = {(round(a), round(b)) for a, b in studs}
    with m.step(f"{title}: curb"):
        m.row(Tile, curb, at=(x, y, z), length=length)
    with m.step(f"{title}: paving"):
        for r in range(1, depth):
            start = 0
            for a in [c - x for c in range(x, x + length) if (c, y + r) in bare] + [
                length
            ]:
                if a > start:
                    m.row(Tile, paving, at=(x + start, y + r, z), length=a - start)
                start = a + 1
    return z + 1


def example(m):
    """A sidewalk 8 studs long on a plate, with a stud bare for a lamp post."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.fill(Plate, "Green", at=(0, 0, 0), size=(8, 4))
    m.use("sidewalk", x=0, y=0, z=1, length=8, studs=[(5, 2)])
