# SPDX-License-Identifier: Apache-2.0
"""Rear spoiler: a wing over the back of a car, a tile on a plate held up on two small posts.

m.use("rear_spoiler", x, y, z, width=4, color="Black", title="Spoiler") stands a round 1 x 1 brick at each end of a
car's back edge, `width` studs along y from (x, y, z), and lays a plate 1 x width across their tops, tiled. Returns the
z of the wing's top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3062b", "3710", "3666", "2431", "6636"]

PLATES = {4: "3710", 6: "3666"}  # Plate 1 x n
TILES = {4: "2431", 6: "6636"}  # Tile 1 x n


def build(m, x, y, z, width=4, color="Black", title="Spoiler"):
    """The posts and the wing; returns the z of its top."""
    if width not in PLATES:
        raise ValueError(f"rear_spoiler takes a width of 4 or 6, not {width}")
    with m.step(title):
        for side in (y, y + width - 1):
            m.add("3062b", color, at=(x, side, z))
        m.add(PLATES[width], color, at=(x, y, z + 3), rot=0)
        m.add(TILES[width], color, at=(x, y, z + 4), rot=0)
    return z + 5


def example(m):
    """A spoiler on the back of a car's plate."""
    with m.step("Car"):
        m.add("3031", "Red", at=(0, 0, 0))
    m.use("rear_spoiler", x=3, y=0, z=1)
