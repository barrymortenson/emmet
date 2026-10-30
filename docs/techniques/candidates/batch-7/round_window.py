# SPDX-License-Identifier: Apache-2.0
"""Round window: a bull's-eye window, a round plate with a round hole and a round glass tile in it, on side studs.

m.use("round_window", x, y, z, color="Tan", frame="White", glass="Trans-Clear", title="Round window") sets, in a wall
one stud thick along x facing front (-y), a block of four Bricks 1 x 2 x 1 2/3 with Studs on One Side (22885), two wide
and two high, between a plate 1 x 4 under them and one over them, and on their side studs a Plate 4 x 4 Round with a 2 x
2 Round Hole (11833) as the frame, with a Tile 2 x 2 Round in the glass color in its hole. The window takes the wall's
cells x to x + 3 from z to z + 12, four bricks' height; lay the wall beside it as usual. Returns the z of its top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217, set 11387. Not derived
from any model file."""

PARTS = ["22885", "11833", "14769", "3710"]

FACE = Rot(x=90)  # a plate or tile on side studs, its top to -y


def build(
    m, x, y, z, color="Tan", frame="White", glass="Trans-Clear", title="Round window"
):
    """The block of side-stud bricks, then the frame and glass on it; returns the z of the top."""
    with m.step(f"{title}: block"):
        m.add("3710", color, at=(x, y, z), rot=1)
        for row in range(2):
            for k in range(2):
                m.add("22885", color, at=(x + 2 * k, y, z + 1 + 5 * row), rot=3)
        m.add("3710", color, at=(x, y, z + 11), rot=1)
    with m.step(f"{title}: frame and glass"):
        m.add("11833", frame, at=(x, y - 0.4, z + 1), rot=FACE)
        m.add("14769", glass, at=(x + 1, y - 0.4, z + 3.5), rot=FACE)
    return z + 12


def example(m):
    """A round window high in a short wall."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Tan", at=(0, 0, 0), length=8)
    z = m.use("round_window", x=2, y=0, z=3)
    for r in range(4):
        with m.step(f"Wall beside the window, row {r + 2}"):
            for cx in (0, 6):
                m.add(Brick(1, 2), "Tan", at=(cx, 0, 3 + 3 * r), rot=1)
    with m.step("Wall, row 6"):
        m.row(Brick, "Tan", at=(0, 0, z), length=8)
