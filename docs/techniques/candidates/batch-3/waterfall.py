# SPDX-License-Identifier: Apache-2.0
"""Waterfall: water pouring over a rock lip and down the face, clear tiles hung on side studs course by course, with
white foam where it lands.

m.use("waterfall", x, y, z, rows=4, color="Dark Bluish Gray", water=("Trans-Clear", "Trans-Light Blue"),
foam="White", title="Waterfall") builds a rock face 4 studs wide along x from x and 2 deep (y to y + 2), `rows` bricks
tall from z (3 to 8), its face to the front (-y), on the studs at height z with two studs of ground clear in front of
it. Above the bottom course, the middle two studs of each course are a brick with two studs on its front (11211)
carrying a 1 x 2 tile on its back, in the two `water` colors in turn, so the water falls down the face in sheets with
a thin line between. On top, two clear cheese slopes make the lip it pours over and a clear tile behind them the stream
that feeds it; at the foot, white cheese slopes and round tiles are the splash. Returns the z of the top.
"""

from emmet_dsl import Rot
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["3005", "11211", "3004", "3010", "3069b", "54200", "98138"]

SHEET = Rot(y=-90, z=90)  # a tile on its back, its face to -y, long along x


def build(
    m,
    x,
    y,
    z,
    rows=4,
    color="Dark Bluish Gray",
    water=("Trans-Clear", "Trans-Light Blue"),
    foam="White",
    title="Waterfall",
):
    """The face, the falling water, the lip, and the splash; returns the z of the top."""
    if not 3 <= rows <= 8:
        raise ValueError(f"waterfall takes 3 to 8 rows, not {rows}")
    if not water:
        raise ValueError("waterfall needs at least one water color")

    def add(part, c, at, rot=0):
        return (1, lambda: m.add(part, c, at=at, rot=rot))

    splash = [
        add("54200", foam, (x + 1, y - 1, z), rot=3),
        add("54200", foam, (x + 2, y - 1, z), rot=3),
        add("98138", water[0], (x, y - 1, z)),
        add("3069b", water[-1], (x + 1, y - 2, z), rot=1),
        add("98138", foam, (x + 3, y - 2, z)),
    ]
    for r in range(rows):
        zr = z + 3 * r
        if r % 2 == 0:
            # Cheeks of 1 x 1 bricks, a 1 x 4 behind tying them.
            cheek, back, at = "3005", "3010", (x, y + 1, zr)
        else:
            # Cheeks of 1 x 2 bricks running back, a 1 x 2 behind between them.
            cheek, back, at = "3004", "3004", (x + 1, y + 1, zr)
        course = [
            add(cheek, color, (x, y, zr)),
            add(cheek, color, (x + 3, y, zr)),
            add(back, color, at, rot=1),
        ]
        if r == 0:
            course.append(add("3004", color, (x + 1, y, zr), rot=1))
        else:
            sheet = (x + 1, y - 0.4, zr + 0.5)
            course.append(add("11211", color, (x + 1, y, zr), rot=3))
            course.append(add("3069b", water[r % len(water)], sheet, rot=SHEET))
        in_steps(m, f"{title}, row {r + 1}", course)
        if r == 0:
            # The splash goes in before the water above it closes the way in.
            in_steps(m, f"{title}, splash", splash)
    top = z + 3 * rows
    in_steps(
        m,
        f"{title}, lip",
        [
            add("54200", water[0], (x + 1, y, top), rot=3),
            add("54200", water[0], (x + 2, y, top), rot=3),
            add("3069b", water[-1], (x + 1, y + 1, top), rot=1),
            add("54200", color, (x, y, top), rot=2),
            add("54200", color, (x + 3, y + 1, top), rot=0),
        ],
    )
    return top


def example(m):
    """A waterfall four bricks tall on a plate."""
    with m.step("Ground"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("waterfall", x=0, y=2, z=1, rows=4)
