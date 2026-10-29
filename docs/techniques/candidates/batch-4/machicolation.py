# SPDX-License-Identifier: Apache-2.0
"""Machicolation: a wall's top course of corbels, inverted slopes that carry a slab a stud out over the wall's face,
with gaps between them to drop things through.

m.use("machicolation", x, y, z, length, depth=1, color="Light Bluish Gray", spacing=2, title="Machicolation") lays the
top course of a wall `depth` studs thick (1 or 2) that runs `length` studs along x from (x, y), on the course whose top
is at z: an inverted slope 2 x 1 (3665b) every `spacing` studs on the front row, its slope facing out (-y) and its top a
stud past the face, a corbel at each end, and bricks between and behind. Then a course of plates, `depth` + 1 studs wide
from y - 1, ties the corbels and the wall and makes the overhanging slab; the openings between the corbels show under
it. length must be a whole number of spacing, plus 1. Returns (y, z) of the slab's front edge and top, where a parapet
(battlements with tie=False, when depth is 1) goes.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3665b",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "91988",
    "4282",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
]

CORBEL = "3665b"  # Slope Brick 45 2 x 1 Inverted


def build(
    m,
    x,
    y,
    z,
    length,
    depth=1,
    color="Light Bluish Gray",
    spacing=2,
    title="Machicolation",
):
    """The corbel course and the slab; returns (y, z) of the slab's front edge and top."""
    if depth not in (1, 2) or spacing not in (2, 3, 4):
        raise ValueError(
            f"machicolation takes a depth of 1 or 2 and a spacing of 2 to 4, not {depth}, {spacing}"
        )
    if length < spacing + 1 or (length - 1) % spacing:
        raise ValueError(
            f"machicolation needs a length of n x {spacing} + 1 (n 1 or more), not {length}"
        )
    corbels = range(0, length, spacing)
    with m.step(f"{title}: corbels"):
        for u in corbels:
            m.add(CORBEL, color, at=(x + u, y - 1, z), rot=3)
    with m.step(f"{title}: bricks between"):
        for u in corbels[:-1]:
            m.row(Brick, color, at=(x + u + 1, y, z), length=spacing - 1)
        if depth == 2:
            m.row(Brick, color, at=(x, y + 1, z), length=length)
    with m.step(f"{title}: slab"):
        # Two-wide plates over the corbels' tops and the wall's front row, started a stud in so their joints cross
        # the bricks'; a deep wall's back row gets its own run.
        m.row(Plate, color, at=(x, y - 1, z + 3), length=length, width=2, first=2)
        if depth == 2:
            m.row(Plate, color, at=(x, y + 1, z + 3), length=length)
    return y - 1, z + 4


def example(m):
    """A wall 1 stud thick with machicolations and a parapet on the slab they carry."""
    for r in range(2):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Light Bluish Gray",
                at=(0, 0, 3 * r),
                length=7,
                first=2 if r else None,
            )
    y, z = m.use("machicolation", x=0, y=0, z=6, length=7)
    with m.step("Parapet"):
        m.row(Brick, "Light Bluish Gray", at=(0, y, z), length=7)
        for u in range(0, 7, 2):
            m.add("3005", "Light Bluish Gray", at=(u, y, z + 3))
