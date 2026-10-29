# SPDX-License-Identifier: Apache-2.0
"""Front bumper: a bumper that stands out across a vehicle's front, a plate under the front edge with a tile on top and
round indicator lights at its ends.

m.use("front_bumper", x, y, z, width=4, color="Light Bluish Gray", light="Trans-Orange", title="Bumper") lays a plate
2 deep across a vehicle's front, `width` studs along y from (x - 1, y, z), so its front row stands a stud out (-x),
a tile along that row, and a round 1 x 1 plate at each of its ends as indicators. Returns the z of the plate's top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = [
    "3022",
    "3020",
    "3795",
    "3069b",
    "2431",
    "6636",
    "6141",
    "3070b",
    "3710",
    "3666",
]

PLATES = {2: "3022", 4: "3020", 6: "3795"}  # Plate 2 x n
TILES = {2: "3069b", 4: "2431", 6: "6636"}  # Tile 1 x n


def build(
    m, x, y, z, width=4, color="Light Bluish Gray", light="Trans-Orange", title="Bumper"
):
    """The plate, its tile, and the lights; returns the z of the plate's top."""
    if width not in PLATES:
        raise ValueError(f"front_bumper takes a width of 2, 4, or 6, not {width}")
    with m.step(title):
        m.add(PLATES[width], color, at=(x - 1, y, z), rot=0)
        if width > 2:
            m.add(TILES[width - 2], color, at=(x - 1, y + 1, z + 1), rot=0)
        for side in (y, y + width - 1):
            m.add("6141", light, at=(x - 1, side, z + 1))
    return z + 1


def example(m):
    """A bumper on the front of a 4-wide chassis."""
    with m.step("Chassis"):
        m.add("3031", "Red", at=(0, 0, 1))
    m.use("front_bumper", x=0, y=0, z=0)
