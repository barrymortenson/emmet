# SPDX-License-Identifier: Apache-2.0
"""Hover stand: a figure floating above its base on a short glowing column of transparent round bricks, the way a
hovering robot or a spirit is displayed.

m.use("hover_stand", x, y, z, height=2, size=6, color="Light Bluish Gray", glow="Trans Light Blue", title="Stand")
builds, on the studs at height z (or on the table), a round plate `size` studs across (6 or 8) from (x, y), a ring of
round tiles 1 x 1 in the glow color on its edge studs (lights round the rim, at the four sides), and in its middle a
column of round bricks 2 x 2 in the glow color, `height` bricks tall (1 to 4), capped with a round plate 2 x 2 in the
base color whose four studs take the figure's underside (a plate 2 x 2, or a round plate on the bottom of its core).
Returns (cx, cy, top): the column's front-left corner and the z of its top, where the figure goes.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43279. Not derived from any
model file."""

PARTS = ["11213", "74611", "3941", "4032a", "98138"]

PLATES = {6: "11213", 8: "74611"}  # Plate N x N Round


def build(
    m,
    x,
    y,
    z,
    height=2,
    size=6,
    color="Light Bluish Gray",
    glow="Trans Light Blue",
    title="Stand",
):
    """The base, its rim lights, and the glowing column; returns (cx, cy, top)."""
    if size not in PLATES:
        raise ValueError(f"hover_stand takes a size of 6 or 8, not {size}")
    if not 1 <= height <= 4:
        raise ValueError(f"hover_stand takes a height of 1 to 4 bricks, not {height}")
    c = size // 2 - 1  # the column's corner, from the base's
    with m.step(f"{title}: base"):
        m.add(PLATES[size], color, at=(x, y, z))
        # A light on the middle stud of each side of the round plate's edge.
        for i, j in ((c, 0), (c + 1, size - 1), (0, c + 1), (size - 1, c)):
            m.add("98138", glow, at=(x + i, y + j, z + 1))
    with m.step(f"{title}: column"):
        m.stack("3941", glow, at=(x + c, y + c, z + 1), count=height)
        m.add("4032a", color, at=(x + c, y + c, z + 1 + 3 * height))
    return (x + c, y + c, z + 2 + 3 * height)


def example(m):
    """A little dome-headed robot hovering over a round base."""
    cx, cy, top = m.use("hover_stand", x=0, y=0, z=0, height=3)
    with m.step("Robot, body"):
        m.add("60474", "White", at=(cx - 1, cy - 1, top))  # Round plate 4 x 4
        m.add(
            "3063b", "White", at=(cx + 1, cy + 1, top + 1), rot=0
        )  # Corner round bricks 2 x 2, a ring
        m.add("3063b", "White", at=(cx - 1, cy + 1, top + 1), rot=1)
        m.add("3063b", "White", at=(cx - 1, cy - 1, top + 1), rot=2)
        m.add("3063b", "White", at=(cx + 1, cy - 1, top + 1), rot=3)
    with m.step("Robot, head"):
        m.add("60474", "White", at=(cx - 1, cy - 1, top + 4))
        for (i, j), rot in {(1, 1): 0, (0, 1): 1, (0, 0): 2, (1, 0): 3}.items():
            m.add(
                "5852", "White", at=(cx - 1 + 2 * i, cy - 1 + 2 * j, top + 5), rot=rot
            )  # Quarter circle 2 x 2
