# SPDX-License-Identifier: Apache-2.0
"""Window sill: a sill that stands out under a window, a plate a stud deeper than the wall with a tile on its front.

m.use("window_sill", x, y, z, width=2, color="White", title="Sill") lays a plate 2 studs deep and `width` + 2 long
under a window `width` studs wide whose front-left corner is (x, y), at the window's bottom z: its back row is in the
wall, its front row stands a stud out, and a tile covers that row. The window goes on the plate at z + 1. Returns the
z the window stands at.
"""

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 43217, set 21335. Not derived from
any model file."""

PARTS = [
    "3024",
    "3020",
    "3795",
    "3832",
    "2431",
    "6636",
    "3460",
    "60592",
    "3005",
    "3004",
    "3010",
    "3622",
]

PLATES = {4: "3020", 6: "3795", 8: "3832"}  # Plate 2 x n
TILES = {4: "2431", 6: "6636"}  # Tile 1 x n


def build(m, x, y, z, width=2, color="White", title="Sill"):
    """The sill; returns the z the window stands at."""
    n = width + 2
    if n not in PLATES:
        raise ValueError(f"window_sill takes a window 2, 4, or 6 wide, not {width}")
    with m.step(title):
        m.add(PLATES[n], color, at=(x - 1, y - 1, z), rot=1)
        if n in TILES:
            m.add(TILES[n], color, at=(x - 1, y - 1, z + 1), rot=1)
        else:
            m.add(TILES[4], color, at=(x - 1, y - 1, z + 1), rot=1)
            m.add(TILES[4], color, at=(x + 3, y - 1, z + 1), rot=1)
    return z + 1


def example(m):
    """A window on its sill in a short wall."""
    from emmet_dsl import Brick

    with m.step("Wall"):
        m.row(Brick, "Tan", at=(0, 0, 0), length=6)
    z = m.use("window_sill", x=2, y=0, z=3)
    with m.step("Plates beside the sill"):
        # The sill's plate is part of a course: plates each side keep the bricks beside the window level with it.
        for at in (0, 5):
            m.add("3024", "Tan", at=(at, 0, 3))
    with m.step("Window and wall"):
        m.add("60592", "White", at=(2, 0, z))
        for r in range(2):
            for at in ((0, 0), (4, 0)):
                m.row(Brick, "Tan", at=(*at, z + 3 * r), length=2)
