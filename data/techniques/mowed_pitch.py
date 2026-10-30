# SPDX-License-Identifier: Apache-2.0
"""Mowed pitch: a micro playing field, tiled in stripes of two greens like a mown lawn, inside white touchlines with a
halfway line and a goal at each end.

m.use("mowed_pitch", x, y, z, length=12, width=8, light="Bright Green", dark="Green", line="White", goals=True,
title="Pitch") covers, on the studs at height z, a length x width field from (x, y), its length along x: a 1-stud border
of white tiles all around (the touchlines and the goal lines), a white halfway line across the middle (two studs wide, as
the length is even), and between them stripes 2 studs wide running across the field, light and dark in turn from each
goal line, each stripe two 1-wide runs of tiles. With goals, the middle two studs of each goal line take a white 1 x 2
tile with a handle (2432) instead, the handle standing up as the goal frame. length is 8 or more and width 6 or more,
both even. Returns the z of the field's top.
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10272. Not derived from any
model file."""

PARTS = ["6636", "2431", "63864", "3069b", "3070b", "4162", "2432"]

SIZES = (6, 4, 3, 2, 1)  # tile lengths made in both greens


def build(
    m,
    x,
    y,
    z,
    length=12,
    width=8,
    light="Bright Green",
    dark="Green",
    line="White",
    goals=True,
    title="Pitch",
):
    """The touchlines, the stripes, the halfway line, and the goals; returns the z of the top."""
    if length < 8 or width < 6 or length % 2 or width % 2:
        raise ValueError(
            f"mowed_pitch takes an even length of 8 or more and an even width of 6 or more, not {length}, {width}"
        )
    inner = width - 2
    with m.step(f"{title}: touchlines"):
        m.row(Tile, line, at=(x, y, z), length=length)
        m.row(Tile, line, at=(x, y + width - 1, z), length=length)
    # The goal lines, with a gap for the goal in the middle of each.
    gy = y + width // 2 - 1
    with m.step(f"{title}: goal lines"):
        for gx in (x, x + length - 1):
            if goals:
                m.row(Tile, line, at=(gx, y + 1, z), length=gy - y - 1, axis="y")
                m.add("2432", line, at=(gx, gy, z))
                m.row(
                    Tile,
                    line,
                    at=(gx, gy + 2, z),
                    length=y + width - 1 - gy - 2,
                    axis="y",
                )
            else:
                m.row(Tile, line, at=(gx, y + 1, z), length=inner, axis="y")
    # Stripes from each end toward the middle, mirror images, and the halfway line between.
    half = (length - 3) // 2  # the columns each side of the halfway line
    mid = x + 1 + half
    columns = []
    for i in range(half):
        color = light if (i // 2) % 2 == 0 else dark
        columns += [(x + 1 + i, color), (x + length - 2 - i, color)]
    columns.sort()
    for n in range(0, len(columns), 4):
        with m.step(f"{title}: stripes {n // 4 + 1}"):
            for cx, color in columns[n : n + 4]:
                m.row(
                    Tile, color, at=(cx, y + 1, z), length=inner, axis="y", sizes=SIZES
                )
    with m.step(f"{title}: halfway line"):
        for cx in range(mid, x + length - 1 - half):
            m.row(Tile, line, at=(cx, y + 1, z), length=inner, axis="y", sizes=SIZES)
    return z + 1


def example(m):
    """A pitch 12 x 8 on a plate."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.add(Plate(6, 8), "Dark Bluish Gray", at=(0, 0, 0))
        m.add(Plate(6, 8), "Dark Bluish Gray", at=(6, 0, 0))
    m.use("mowed_pitch", x=0, y=0, z=1, length=12, width=8)
