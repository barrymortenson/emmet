# SPDX-License-Identifier: Apache-2.0
"""Sideways chimney column: a tall, flush chimney built lying down as a sub-assembly, then stood up against a wall's
side studs, so it rises past the eaves in one smooth piece.

m.use("sideways_chimney_column", x, y, z, length=8, color="Dark Red", cap="Dark Bluish Gray", pots="Reddish Brown",
title="Chimney") stands, against the wall face at x (the column goes outside it, toward +x), a column 2 studs wide
from y and `length` studs tall from z (2.5 plates a stud: 8 studs is 20 plates). It's built lying flat: a layer of
plates, a layer of bricks, a layer of plates, and at its top end a brick 1 x 2 x 1 2/3 with studs on one side
(22885), all tied by a layer of plates across them and faced with tiles; stood up, the first layer's underside grips
the wall's side studs and the brick's studs point up for the cap (a plate 2 x 2 and two cones as pots). The wall
needs side studs facing +x in those two stud columns at heights z + 1.25 + 2.5 k: a brick 1 x 2 x 1 2/3 with studs
on one side (22885) in every 5-plate band of the wall does it (example). Returns the z of the pots' top.
"""

from emmet_dsl import Brick, Plate, Rot, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217. Not derived from any
model file."""

PARTS = [
    "22885",
    "3001",
    "3002",
    "3003",
    "3004",
    "3022",
    "3020",
    "3021",
    "3795",
    "3832",
    "3034",
    "87079",
    "3068b",
    "4589",
]

STAND = Rot(
    x=90, z=90
)  # the lying column's x runs along the wall (y), its length up (z), its layers out (x)


def _column(m, length, color):
    """The column lying flat, 2 wide (x), `length` long (y), its underside the face that goes against the wall."""
    body = length - 1
    # Each layer's joint falls somewhere else, so the layers tie each other.
    with m.step("Plates"):
        m.row(Plate, color, at=(0, 0, 0), length=body, axis="y", width=2)
    with m.step("Bricks"):
        m.row(Brick, color, at=(0, 0, 1), length=body, axis="y", width=2, first=2)
    with m.step("Plates, second layer"):
        m.row(Plate, color, at=(0, 0, 4), length=body, axis="y", width=2, first=3)
    with m.step("Top end"):
        # rot 1 turns its side studs toward +y: the column's top once it stands.
        m.add("22885", color, at=(0, body, 0), rot=1)
    with m.step("Plates, across"):
        m.row(
            Plate,
            color,
            at=(0, 0, 5),
            length=length,
            axis="y",
            width=2,
            first=1 if length % 2 else None,
        )
    with m.step("Tiles"):
        m.row(Tile, color, at=(0, 0, 6), length=length, axis="y", width=2)


def build(
    m,
    x,
    y,
    z,
    length=8,
    color="Dark Red",
    cap="Dark Bluish Gray",
    pots="Reddish Brown",
    title="Chimney",
):
    """The column, stood up against the wall, and its cap; returns the z of the pots' top."""
    sub = m.submodel(f"{title}: column", _column, length=length, color=color)
    with m.step(f"{title}: column"):
        placed = m.place(sub, at=(x, y, z), rot=STAND)
    top = z + 2.5 * length
    with m.step(f"{title}: cap and pots"):
        m.add("3022", cap, at=(x, y, top))
        m.add("4589", pots, at=(x, y, top + 1))
        m.add("4589", pots, at=(x + 1, y + 1, top + 1))
    return max(top + 4, placed.max[2])


def example(m):
    """An end wall built in bands 5 plates tall, with the column against it, rising well above."""
    with m.step("Base"):
        m.add("3958", "Light Bluish Gray", at=(-2, -1, 0))
    for k in range(3):
        with m.step(f"Wall, band {k + 1}"):
            m.add("32952", "White", at=(0, 0, 1 + 5 * k))
            m.add("22885", "White", at=(0, 1, 1 + 5 * k))
            m.add("32952", "White", at=(0, 3, 1 + 5 * k))
    m.use("sideways_chimney_column", x=1, y=1, z=1)
