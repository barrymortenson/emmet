# SPDX-License-Identifier: Apache-2.0
"""Fin merlon crenels: battlements whose merlons are thin rounded fins, not square blocks, with grille tiles in the
crenels.

m.use("fin_merlon_crenels", x, y, z, length, rise=1, color="White", tie=True, title="Fin battlements") crowns a wall 2
studs thick that runs `length` studs along x from (x, y), its top at z. A course of 2-wide plates ties the wall's two
leaves (tie=False leaves it out); on its front row (y) a course of bricks is the breastwork, and the back row is the
walk, tiled. On the breastwork, fins stand 2 studs apart: each a stud thick and 2 wide, `rise` bricks 1 x 2 topped by
two Brick 1 x 1 x 1 Round Quarter Dome (5848) back to back, so its top is a half dome, round to the front and both
sides. The crenels' floors between fins are grille tiles 1 x 2 (2412b). length must be 4n + 2 (a fin at each end).
Returns the z of the walk.
"""

from emmet_dsl import Brick, Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = [
    "5848",
    "3004",
    "2412b",
    "3005",
    "3622",
    "3010",
    "3009",
    "3008",
    "3069b",
    "63864",
    "2431",
    "6636",
    "4162",
    "3710",
    "3666",
    "3460",
    "3795",
    "3034",
    "3832",
]

LEFT, RIGHT = 2, 3  # a quarter dome round toward -x and -y, or toward +x and -y


def build(m, x, y, z, length, rise=1, color="White", tie=True, title="Fin battlements"):
    """The tie course, the breastwork and walk, the fins, and the crenels; returns the z of the walk."""
    if length < 6 or (length - 2) % 4 or not 1 <= rise <= 3:
        raise ValueError(
            f"fin_merlon_crenels needs a length of 4n + 2 (n 1 or more) and a rise of 1 to 3, not {length}, {rise}"
        )
    if tie:
        with m.step(f"{title}: tie course"):
            m.row(Plate, color, at=(x, y, z), length=length, width=2)
        z += 1
    with m.step(f"{title}: breastwork and walk"):
        m.row(Brick, color, at=(x, y, z), length=length, first=1)
        m.row(Tile, color, at=(x, y + 1, z), length=length)
    fins = range(0, length, 4)
    zf = z + 3
    for r in range(rise):
        with m.step(f"{title}: fins, course {r + 1}"):
            for u in fins:
                m.add("3004", color, at=(x + u, y, zf + 3 * r), rot=1)
    with m.step(f"{title}: fin tops"):
        for u in fins:
            m.add("5848", color, at=(x + u, y, zf + 3 * rise), rot=LEFT)
            m.add("5848", color, at=(x + u + 1, y, zf + 3 * rise), rot=RIGHT)
    with m.step(f"{title}: crenels"):
        for u in fins[:-1]:
            m.add("2412b", color, at=(x + u + 2, y, zf), rot=1)
    return z + 1


def example(m):
    """A short stretch of white wall, 2 studs thick, crowned with fins."""
    for r in range(2):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "White",
                at=(0, 0, 3 * r),
                length=10,
                width=2,
                first=2 if r else None,
            )
    m.use("fin_merlon_crenels", x=0, y=0, z=6, length=10)
