# SPDX-License-Identifier: Apache-2.0
"""Signboard beam: a deep sign beam over a shopfront, a smooth sign panel on side studs between dark ends, a cornice, a
half-dome pediment, and a post at each end.

m.use("signboard_beam", x, y, z, length=10, color="Dark Red", sign="White", cornice="Tan", pediment="White",
title="Signboard") lays, along a wall one stud thick along x facing front (-y), over a shopfront `length` studs wide (6,
8, or 10) from (x, y), a course of Bricks 1 x 2 x 1 2/3 with Studs on One Side (22885) in `color`, studs out; on their
side studs two Tiles 1 x (length - 2) in the sign color, one over the other, as the board (leave it plain or use printed
tiles), framed by a Tile 1 x 2 on end in `color` at each end. Over the course a Plate 2 x length in the cornice color
stands a stud out, with tiles along its front row; on its middle, two Bricks 1 x 1 Round Quarter Dome (5848) make a
small half-dome pediment, and a Brick 1 x 1 with a round plate on it stands at each end as a post. It takes the wall's
cells x to x + length - 1 from z to z + 6, two bricks' height. Returns the z of the cornice plate's top.
"""

from emmet_dsl import Plate, Rot, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = [
    "22885",
    "3069b",
    "2431",
    "6636",
    "4162",
    "3795",
    "3034",
    "3832",
    "5848",
    "3005",
    "6141",
]

BOARDS = {6: "2431", 8: "6636", 10: "4162"}  # Tile 1 x (length - 2)
PANEL = Rot(y=-90, z=90)  # a tile on side studs, its face to -y, long along x
UP = Rot(x=90)  # a tile on side studs, its face to -y, its long side up
DOME = (
    2,
    3,
)  # quarter turns that put a quarter dome's round corner front-left and front-right


def build(
    m,
    x,
    y,
    z,
    length=10,
    color="Dark Red",
    sign="White",
    cornice="Tan",
    pediment="White",
    title="Signboard",
):
    """The side-stud course and its board, the cornice, the pediment, and the posts; returns the cornice's top z."""
    if length not in BOARDS:
        raise ValueError(f"signboard_beam takes a length of 6, 8, or 10, not {length}")
    with m.step(f"{title}: course"):
        for k in range(0, length, 2):
            m.add("22885", color, at=(x + k, y, z), rot=3)
    with m.step(f"{title}: board"):
        for px in (x, x + length - 1):
            m.add("3069b", color, at=(px, y - 0.4, z), rot=UP)
        for row in range(2):
            m.add(BOARDS[length], sign, at=(x + 1, y - 0.4, z + 2.5 * row), rot=PANEL)
    mid = x + length // 2
    with m.step(f"{title}: cornice"):
        m.row(Plate, cornice, at=(x, y - 1, z + 5), length=length, width=2)
        m.row(Tile, cornice, at=(x + 1, y - 1, z + 6), length=mid - 2 - x)
        m.row(Tile, cornice, at=(mid + 1, y - 1, z + 6), length=x + length - 2 - mid)
    with m.step(f"{title}: pediment and posts"):
        m.add("5848", pediment, at=(mid - 1, y - 1, z + 6), rot=DOME[0])
        m.add("5848", pediment, at=(mid, y - 1, z + 6), rot=DOME[1])
        for px in (x, x + length - 1):
            m.add("3005", color, at=(px, y - 1, z + 6))
            m.add("6141", color, at=(px, y - 1, z + 9))
    return z + 6


def example(m):
    """The signboard over a shopfront of three windows."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Sand Green", at=(-1, 0, 0), length=10)
    with m.step("Windows"):
        for k in range(3):
            f = m.add("60592", "White", at=(1 + 2 * k, 0, 3), rot=1)
            (x0, y0, z0), (x1, y1, _) = f.min, f.max
            m.add(
                "60601",
                "Trans-Clear",
                at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
                rot=1,
            )
    for r in range(2):
        with m.step(f"Wall beside the windows, row {r + 2}"):
            m.row(Brick, "Sand Green", at=(-1, 0, 3 + 3 * r), length=2)
            m.add("3005", "Sand Green", at=(7, 0, 3 + 3 * r))
            m.add("3005", "Sand Green", at=(8, 0, 3 + 3 * r))
    m.use("signboard_beam", x=-1, y=0, z=9, length=10)
