# SPDX-License-Identifier: Apache-2.0
"""Bermed wall reveal: a house front that shows only where the hill is cut away, the hill sloping down past both ends
of the wall and its turf overhanging the top, so you never see where the wall stops.

m.use("bermed_wall_reveal", x, y, z, length, rows=3, color="Tan", beam="Reddish Brown", hill="Green", wall=True,
title="Hill front") builds a hill around a wall 1 stud thick and `length` studs long along x from (x, y, z), facing
front (-y), `rows` courses tall. With wall, it lays the wall too, in running bond; without, the wall is already there
(a door or a window in it, say). Each side of the wall, a cheek of the hill 2 studs deep steps down and out a course
at a time: 2 x 2 bricks under a 45-degree 2 x 2 slope, so it falls away at 45 degrees from the wall's top corner to
the ground. Over the top, a beam of plates runs the wall's length and onto the cheeks, and turf plates 2 studs deep
over it overhang the wall's face by a stud. Returns the z of the turf's top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10354. Not derived from any
model file."""

PARTS = [
    "3003",
    "3039",
    "3004",
    "3010",
    "3009",
    "3622",
    "3005",
    "3710",
    "3666",
    "3460",
    "3020",
    "3795",
    "3034",
]

SLOPE = "3039"  # Slope Brick 45 2 x 2
DOWN = {"left": 2, "right": 0}  # the slope's low side toward -x or +x


def build(
    m,
    x,
    y,
    z,
    length,
    rows=3,
    color="Tan",
    beam="Reddish Brown",
    hill="Green",
    wall=True,
    title="Hill front",
):
    """The wall's courses with the cheeks beside them, then the beam and the turf; returns the z of the top."""
    if length < 2 or rows < 1:
        raise ValueError(
            f"bermed_wall_reveal needs a length of 2 or more and 1 row or more, not {length}, {rows}"
        )
    for r in range(rows):
        zr = z + 3 * r
        reach = 2 * (
            rows - r
        )  # how far the cheek reaches out from the wall's end in this course
        with m.step(f"{title}, row {r + 1}"):
            if wall:
                m.row(
                    Brick,
                    color,
                    at=(x, y, zr),
                    length=length,
                    first=1 if r % 2 else None,
                )
            for side, sign in (("left", -1), ("right", 1)):
                edge = x if side == "left" else x + length
                for k in range(0, reach, 2):
                    # The cheek's 2 x 2 blocks, outward from the wall's end; the outermost is the slope.
                    bx = edge - k - 2 if side == "left" else edge + k
                    if k == reach - 2:
                        m.add(SLOPE, hill, at=(bx, y, zr), rot=DOWN[side])
                    else:
                        m.add("3003", hill, at=(bx, y, zr))
    top = z + 3 * rows
    with m.step(f"{title}: beam"):
        m.row(Plate, beam, at=(x - 1, y, top), length=length + 2, first=3)
    with m.step(f"{title}: turf"):
        m.row(
            Plate, hill, at=(x - 1, y - 1, top + 1), length=length + 2, width=2, first=2
        )
    return top + 2


def example(m):
    """A tan front 6 studs long, three courses tall, cut into a hill, on a green plate."""
    with m.step("Ground"):
        m.fill(Plate, "Green", at=(-6, -2, 0), size=(18, 4))
    m.use("bermed_wall_reveal", x=0, y=0, z=1, length=6)
