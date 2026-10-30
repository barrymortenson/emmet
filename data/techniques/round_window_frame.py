# SPDX-License-Identifier: Apache-2.0
"""Round window frame: a round window in a big ring of four quarter-ring plates on side studs, a beam over it.

m.use("round_window_frame", x, y, z, color="Tan", frame="Reddish Brown", sash="Pearl Gold", glass="Trans-Clear",
title="Round window") sets, in a wall one stud thick along x facing front (-y), a block of Bricks 1 x 2 x 1 2/3 with
Studs on One Side (22885) six studs wide and three high from (x, y, z), studs out, and on their side studs four Plates 3
x 3 with a Round Corner and a Round Cutout (68568), each turned to its corner, as a ring six studs across with a round
hole four studs across in its middle. In the hole goes the window: a Plate 4 x 4 Round with a 2 x 2 Round Hole (11833)
in the sash color, and a Tile 2 x 2 Round in the glass color in its hole. Over the block a Plate 2 x 6 in the frame
color, its front row a stud out, is the beam. It takes the wall's cells x to x + 5 from z to z + 16. Returns the z of
the beam's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10354. Not derived from any
model file."""

PARTS = ["22885", "68568", "11833", "14769", "3795"]

# A quarter-ring plate on side studs, its face to -y, turned in the wall's plane so its round corner is at the top
# left, top right, bottom right, or bottom left.
RING = {
    "tl": Rot(x=90),
    "tr": Rot(x=90, y=90),
    "br": Rot(x=90, y=180),
    "bl": Rot(x=90, y=270),
}
FACE = Rot(x=90)  # a round plate or tile on side studs, its top to -y


def build(
    m,
    x,
    y,
    z,
    color="Tan",
    frame="Reddish Brown",
    sash="Pearl Gold",
    glass="Trans-Clear",
    title="Round window",
):
    """The side-stud block a row at a time, the ring and glass, and the beam; returns the z of the beam's top."""
    for row in range(3):
        with m.step(f"{title}: block, row {row + 1}"):
            for k in range(3):
                m.add("22885", color, at=(x + 2 * k, y, z + 5 * row), rot=3)
    with m.step(f"{title}: ring"):
        for corner, (dx, dz) in {
            "bl": (0, 0),
            "br": (3, 0),
            "tl": (0, 7.5),
            "tr": (3, 7.5),
        }.items():
            m.add("68568", frame, at=(x + dx, y - 0.4, z + dz), rot=RING[corner])
    with m.step(f"{title}: window"):
        m.add("11833", sash, at=(x + 1, y - 0.4, z + 2.5), rot=FACE)
        m.add("14769", glass, at=(x + 2, y - 0.4, z + 5), rot=FACE)
    with m.step(f"{title}: beam"):
        m.add("3795", frame, at=(x, y - 1, z + 15), rot=1)
    return z + 16


def example(m):
    """The window in a short wall on a course of bricks."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Tan", at=(-2, 0, 0), length=10)
    z = m.use("round_window_frame", x=0, y=0, z=3)
    for r in range(5):
        with m.step(f"Wall beside the window, row {r + 2}"):
            for cx in (-2, 6):
                m.add(Brick(1, 2), "Tan", at=(cx, 0, 3 + 3 * r), rot=1)
    with m.step("Wall beside the beam"):
        for cx in (-2, 6):
            m.add("3023b", "Tan", at=(cx, 0, 18), rot=1)
    with m.step("Wall over the beam"):
        m.row(Brick, "Tan", at=(-2, 0, z), length=10)
