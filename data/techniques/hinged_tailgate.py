# SPDX-License-Identifier: Apache-2.0
"""Hinged tailgate: a truck bed's tailgate on hinge bricks, so it folds down.

m.use("hinged_tailgate", x, y, z, width=4, color="Light Bluish Gray", title="Tailgate") puts a hinge brick 1 x 2 base
(3937) at each end of a bed's back edge, `width` studs along y from (x, y, z), with a hinge 2 x 2 top (6134) closed on
each, and a 1 x width plate and tiles across the tops as the tailgate's top. Built closed; it folds down on the
hinges. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["3937", "6134", "3020", "3795", "2431", "6636"]

PLATES = {4: "3020", 6: "3795"}  # Plate 2 x n
TILES = {4: "2431", 6: "6636"}  # Tile 1 x n


def build(m, x, y, z, width=4, color="Light Bluish Gray", title="Tailgate"):
    """The hinges and the tailgate's top; returns the z of its top."""
    if width not in PLATES:
        raise ValueError(f"hinged_tailgate takes a width of 4 or 6, not {width}")
    with m.step(f"{title}: hinges"):
        for side in (y, y + width - 2):
            # The base and its top share an origin: the top closed on it, its fingers on the base's.
            ox, oy, oz = 20 * (side + 1), -8 * (z + 3), -20 * (x + 0.5)
            m.add_raw("3937", color, (ox, oy, oz, 1, 0, 0, 0, 1, 0, 0, 0, 1))
            m.add_raw("6134", color, (ox, oy, oz, 1, 0, 0, 0, 1, 0, 0, 0, 1))
    with m.step(f"{title}: top"):
        m.add(PLATES[width], color, at=(x - 1, y, z + 3), rot=0)
        m.add(TILES[width], color, at=(x - 1, y, z + 4), rot=0)
        m.add(TILES[width], color, at=(x, y, z + 4), rot=0)
    return z + 5


def example(m):
    """A tailgate on the back of a bed's floor."""
    with m.step("Bed floor"):
        m.add("3031", "Blue", at=(0, 0, 0))
    m.use("hinged_tailgate", x=0, y=0, z=1)
