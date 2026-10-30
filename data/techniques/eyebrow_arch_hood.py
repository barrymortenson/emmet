# SPDX-License-Identifier: Apache-2.0
"""Eyebrow arch hood: a half ring of trim over a window, two quarter-ring plates on side studs standing a plate proud of
the wall.

m.use("eyebrow_arch_hood", x, y, z, color="Dark Red", hood="White", title="Eyebrow hood") lays, over a window 2 studs
wide whose front-left corner is (x, y) and whose top is at z, in a wall one stud thick along x facing front (-y), a
course of three Bricks 1 x 2 x 1 2/3 with Studs on One Side (22885) from x - 2 to x + 4, studs out, and on their side
studs two Plates 3 x 3 with a Round Corner and a Round Cutout (68568), turned to face out, that meet over the window's
middle as a half ring: 6 studs across and 3 high, its round hole over the window. The course takes the wall's cells x -
2 to x + 3 from z to z + 5; the ring stands out in front of the wall up to z + 7.5, over the next course. Returns the z
of the course's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["22885", "68568"]

LEFT = Rot(
    x=90
)  # a quarter-ring plate on side studs, its face to -y, its round corner up and to the left
RIGHT = Rot(
    x=90, y=90
)  # the same, turned in the wall's plane: its round corner up and to the right


def build(m, x, y, z, color="Dark Red", hood="White", title="Eyebrow hood"):
    """The side-stud course and the two halves of the ring; returns the z of the course's top."""
    with m.step(f"{title}: course"):
        for k in range(3):
            m.add("22885", color, at=(x - 2 + 2 * k, y, z), rot=3)
    with m.step(f"{title}: ring"):
        m.add("68568", hood, at=(x - 2, y - 0.4, z), rot=LEFT)
        m.add("68568", hood, at=(x + 1, y - 0.4, z), rot=RIGHT)
    return z + 5


def example(m):
    """A window in a short wall with its eyebrow over it."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Dark Red", at=(-3, 0, 0), length=8)
    with m.step("Window"):
        f = m.add("60592", "White", at=(0, 0, 3), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
    for r in range(2):
        with m.step(f"Wall, row {r + 2}"):
            m.row(
                Brick,
                "Dark Red",
                at=(-3, 0, 3 + 3 * r),
                length=3,
                first=1 if r else None,
            )
            m.row(
                Brick,
                "Dark Red",
                at=(2, 0, 3 + 3 * r),
                length=3,
                first=2 if r else None,
            )
    z = m.use("eyebrow_arch_hood", x=0, y=0, z=9)
    with m.step("Wall beside the hood"):
        for cx in (-3, 4):
            m.add("3005", "Dark Red", at=(cx, 0, 9))
            m.add("3024", "Dark Red", at=(cx, 0, 12))
            m.add("3024", "Dark Red", at=(cx, 0, 13))
    with m.step("Wall over the hood"):
        m.row(Brick, "Dark Red", at=(-3, 0, z), length=8)
