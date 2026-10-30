# SPDX-License-Identifier: Apache-2.0
"""Doorstep and mat: a lived-in entrance, a landing with a mat in front of the door and a round-nosed step down to the
ground, a potted plant at one side.

m.use("doorstep_and_mat", x, y, z, width=4, color="Light Bluish Gray", mat="Red", pot="Dark Orange", bloom="Bright Light
Orange", title="Doorstep") builds, in front of a door `width` studs wide (2, 4, or 6) whose front-left corner is (x, y)
and whose floor is at z, a brick high above the ground at z - 3: a landing of a Brick 2 x (width + 2) from (x - 1, y -
2), a Tile 2 x width as the mat in front of the door with a Tile 1 x 2 to its right, a pot (a round 1 x 1 brick with a
bloom on it) and a Tile 1 x 1 to its left; and in front of the landing a step of Slope Brick Curved 2 x 1 (11477) on a
plate, their rounded noses out, two studs deep. It stands on the ground in front of the wall and takes none of the
wall's cells. Returns the z of the landing's top (z).
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 21335, set 43268.
Not derived from any model file."""

PARTS = [
    "11477",
    "87079",
    "3068b",
    "6934",
    "3069b",
    "3070b",
    "3062b",
    "33291",
    "3001",
    "2456",
    "3007",
    "3020",
    "3795",
    "3832",
]

LANDINGS = {4: "3001", 6: "2456", 8: "3007"}  # Brick 2 x n
PLATES = {4: "3020", 6: "3795", 8: "3832"}  # Plate 2 x n
MATS = {2: "3068b", 4: "87079", 6: "6934"}  # Tile 2 x n


def build(
    m,
    x,
    y,
    z,
    width=4,
    color="Light Bluish Gray",
    mat="Red",
    pot="Dark Orange",
    bloom="Bright Light Orange",
    title="Doorstep",
):
    """The landing, the step, the mat, and the pot; returns the z of the landing's top."""
    n = width + 2
    if width not in MATS:
        raise ValueError(f"doorstep_and_mat takes a door 2, 4, or 6 wide, not {width}")
    g = z - 3
    with m.step(f"{title}: landing and step"):
        m.add(LANDINGS[n], color, at=(x - 1, y - 2, g), rot=1)
        m.add(PLATES[n], color, at=(x - 1, y - 4, g), rot=1)
        for k in range(n):
            m.add("11477", color, at=(x - 1 + k, y - 4, g + 1), rot=3)
    with m.step(f"{title}: mat and pot"):
        m.add(MATS[width], mat, at=(x, y - 2, z), rot=1)
        m.add("3069b", color, at=(x + width, y - 2, z))
        m.add("3062b", pot, at=(x - 1, y - 2, z))
        m.add("3070b", color, at=(x - 1, y - 1, z))
        m.add("33291", bloom, at=(x - 1.1, y - 2.1, z + 3))
    return z


def example(m):
    """A door on a raised floor, with its step and mat."""
    from emmet_dsl import Brick

    with m.step("Floor"):
        m.row(Brick, "Dark Bluish Gray", at=(-1, 0, 0), length=6)
    with m.step("Door"):
        frame = m.add("60596", "White", at=(0, 0, 3), rot=3)
        (x0, y0, z0), (x1, y1, _) = frame.min, frame.max
        m.add(
            "60623",
            "Dark Green",
            at=((x0 + x1 - 3.55) / 2, (y0 + y1 - 0.3) / 2, z0 + 0.875),
            rot=3,
        )
    m.use("doorstep_and_mat", x=0, y=0, z=3)
