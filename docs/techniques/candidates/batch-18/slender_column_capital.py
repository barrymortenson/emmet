# SPDX-License-Identifier: Apache-2.0
"""Slender column capital: rows of thin round columns in twin pairs, each pair joined at the top by a plate and a tile
that read as the capital and the springing of the vault, for the inside of a midi-scale hall or nave.

m.use("slender_column_capital", x, y, z, pairs=4, bay=2, height=3, color="Tan", capital="Tan", tiled=True,
title="Columns") stands `pairs` twin columns on the studs at height z, the first pair on the studs at (x, y) and (x, y +
1), the rest every `bay` studs along x. Each column is a submodel built once: a 1 x 1 round plate as the base, `height`
1 x 1 round bricks, and a 1 x 1 round plate as the neck; a pair is joined by a 1 x 2 plate across their tops and, with
tiled, a 1 x 2 tile over it. Without tiled, the plate's two studs are left for the vault. Leave the floor's studs bare
where the columns stand. Returns the z of the capitals' top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file."""

PARTS = ["6141", "3062b", "3023b", "3069b"]


def _column(m, height, color, title):
    """One column in its own frame, on the stud at the origin."""
    with m.step(f"{title}, column"):
        m.add("6141", color, at=(0, 0, 0))
        m.stack("3062b", color, at=(0, 0, 1), count=height)
        m.add("6141", color, at=(0, 0, 1 + 3 * height))


def build(
    m,
    x,
    y,
    z,
    pairs=4,
    bay=2,
    height=3,
    color="Tan",
    capital="Tan",
    tiled=True,
    title="Columns",
):
    """The pairs of columns and their capitals; returns the z of the capitals' top."""
    if pairs < 1 or bay < 1 or not 1 <= height <= 10:
        raise ValueError(
            f"slender_column_capital takes 1 or more pairs, a bay of 1 or more, and a height of 1 to 10, not "
            f"{pairs}, {bay}, {height}"
        )
    column = m.submodel("column", _column, height=height, color=color, title=title)
    for i0 in range(0, pairs, 2):
        with m.step(f"{title}, columns {i0 + 1}-{min(pairs, i0 + 2)}"):
            for i in range(i0, min(pairs, i0 + 2)):
                for c in range(2):
                    m.place(column, at=(x + i * bay, y + c, z))
    top = z + 2 + 3 * height
    for i0 in range(0, pairs, 4):
        with m.step(f"{title}, capitals {i0 + 1}-{min(pairs, i0 + 4)}"):
            for i in range(i0, min(pairs, i0 + 4)):
                m.add("3023b", capital, at=(x + i * bay, y, top))
                if tiled:
                    m.add("3069b", capital, at=(x + i * bay, y, top + 1))
    return z + 3 + 3 * height + (1 if tiled else 0)


def example(m):
    """A row of five column pairs down a nave floor."""
    with m.step("Floor"):
        m.add("3832", "Dark Tan", at=(0, 0, 0), rot=1)
    m.use("slender_column_capital", x=1, y=0, z=1, pairs=5)
