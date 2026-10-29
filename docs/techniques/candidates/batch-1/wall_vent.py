# SPDX-License-Identifier: Apache-2.0
"""Wall vent: a grille in a brick wall, on the side studs of a brick with two studs on one side.

m.use("wall_vent", x, y, z, color="Light Bluish Gray", grille="Dark Bluish Gray", title="Vent") puts a brick 1 x 2 with
two studs on one side (11211) at (x, y, z) in a wall running along x, its studs out the front (-y), and a grille tile
on them, standing a plate proud of the wall: a vent, a radiator, a grate. Build the wall around it, leaving its two
cells open in that course. Returns nothing.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 43217. Not derived from
any model file."""

PARTS = ["11211", "2412b"]

FACE = Rot(y=-90, z=90)  # a tile on its back, its face to -y, long along x


def build(
    m, x, y, z, color="Light Bluish Gray", grille="Dark Bluish Gray", title="Vent"
):
    """The brick and its grille."""
    with m.step(title):
        m.add("11211", color, at=(x, y, z), rot=3)
        m.add("2412b", grille, at=(x, y - 0.4, z + 0.5), rot=FACE)


def example(m):
    """A vent in the middle course of a short wall."""
    for r in range(3):
        with m.step(f"Wall, row {r + 1}"):
            if r == 1:
                m.row(Brick, "Light Bluish Gray", at=(0, 0, 3), length=2)
                m.row(Brick, "Light Bluish Gray", at=(4, 0, 3), length=2)
            else:
                m.row(
                    Brick,
                    "Light Bluish Gray",
                    at=(0, 0, 3 * r),
                    length=6,
                    first=None if r == 0 else 1,
                )
        if r == 1:
            m.use("wall_vent", x=2, y=0, z=3)
