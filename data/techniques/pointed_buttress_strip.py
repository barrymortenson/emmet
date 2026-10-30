# SPDX-License-Identifier: Apache-2.0
"""Pointed buttress strip: a slim stone pilaster with fine vertical ribs and a rounded head, standing a plate proud of a
tower's face on side studs.

m.use("pointed_buttress_strip", x, y, z, color="Tan", strip="Tan", head="Tan", title="Pilaster") builds, on the studs at
height z, a 1 x 1 column at (x, y), part of a wall whose front face is at y: three units of a brick with a stud on its
front side and two 1 x 1 plates, 15 plates in all, so its side studs are two studs apart. On them a 1 x 5 plate stands
up, studs to the front, and carries two 1 x 2 grille tiles, ribs upright, and a 1 x 1 tile with a rounded end on top as
the pointed head. The strip stands 0.8 studs out from the wall. Lay the wall either side of the column in courses that
meet its top. Returns the z of the column's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file."""

PARTS = ["87087", "3024", "78329", "2412b", "24246"]

UP = Rot(x=90)  # a plate or tile stood up, its face to the front (-y), its long side up
OUT = 0.4  # a plate or tile on its back stands out this far, in studs


def build(m, x, y, z, color="Tan", strip="Tan", head="Tan", title="Pilaster"):
    """The column, the strip, its ribs, and its head; returns the z of the column's top."""
    for u in range(3):
        with m.step(f"{title}, column {u + 1}"):
            m.add("87087", color, at=(x, y, z + 5 * u), rot=3)
            m.stack("3024", color, at=(x, y, z + 5 * u + 3), count=2)
    with m.step(f"{title}, strip"):
        # Side studs are 1.75 plates up a brick, a plate's first anti-studs 1.25 plates in.
        m.add("78329", strip, at=(x, y - OUT, z + 0.5), rot=UP)
        m.add("2412b", strip, at=(x, y - 2 * OUT, z + 0.5), rot=UP)
        m.add("2412b", strip, at=(x, y - 2 * OUT, z + 5.5), rot=UP)
        m.add("24246", head, at=(x, y - 2 * OUT, z + 10.5), rot=UP)
    return z + 15


def example(m):
    """A tower face 4 studs wide with a pilaster at each corner."""
    from emmet_dsl import Brick, Plate, Tile

    with m.step("Ground"):
        m.add("3020", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("pointed_buttress_strip", x=0, y=0, z=1, title="Left pilaster")
    m.use("pointed_buttress_strip", x=3, y=0, z=1, title="Right pilaster")
    for r in range(5):
        with m.step(f"Wall, row {r + 1}"):
            m.add(Brick(1, 2), "Tan", at=(1, 0, 1 + 3 * r), rot=1)
    with m.step("Wall top"):
        m.row(Plate, "Tan", at=(0, 0, 16), length=4)
        m.row(Tile, "Dark Tan", at=(0, 0, 17), length=4)
