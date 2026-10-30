# SPDX-License-Identifier: Apache-2.0
"""Dome from quarters: a smooth half ball from four quarter pieces round a center, each a quarter turn from the last: a
nose tip, the top of a head, a robot's dome.

m.use("dome_from_quarters", x, y, z, size=4, color="White", title="Dome") puts four quarter pieces on the studs at
height z, round the point (x + size/2, y + size/2), so the dome covers x to x + size and y to y + size: size 2 is four
quarter domes 1 x 1 (3 plates tall), 4 four quarter circles 2 x 2 with a curved top (2 plates, a low dome), 6 four
quarter circles 3 x 3 x 1 (3 plates), and 8 four corner round bricks 4 x 4 with a dome top (8 plates, a tall dome). Set
it on a round plate or the top of a rounded_column of the same size. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21368 and 43279. Not derived
from any model file."""

PARTS = ["5848", "5852", "76797", "49612"]

QUARTERS = {
    2: ("5848", 3),
    4: ("5852", 2),
    6: ("76797", 3),
    8: ("49612", 8),
}  # the piece and its height in plates
# A quarter piece's round corner points to +x +y at rot 0, and a quarter turn further each turn: the turn for each
# quarter of the dome, by which half of x and y it's in.
TURNS = {(1, 1): 0, (0, 1): 1, (0, 0): 2, (1, 0): 3}


def build(m, x, y, z, size=4, color="White", title="Dome"):
    """The four quarters; returns the z of the top."""
    if size not in QUARTERS:
        raise ValueError(
            f"dome_from_quarters takes a size of 2, 4, 6, or 8, not {size}"
        )
    part, tall = QUARTERS[size]
    h = size // 2
    with m.step(title):
        for (i, j), rot in TURNS.items():
            m.add(part, color, at=(x + i * h, y + j * h, z), rot=rot)
    return z + tall


def example(m):
    """A robot head: a round plate on a neck under a dome 6 studs across, and beside it a lamp, a small dome on a
    post."""
    from emmet_dsl import Plate

    with m.step("Base and neck"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(10, 6))
        m.add("3941", "Light Bluish Gray", at=(2, 2, 1))  # Round brick 2 x 2
    with m.step("Head plate"):
        m.add("11213", "White", at=(0, 0, 4))  # Round plate 6 x 6
    m.use("dome_from_quarters", x=0, y=0, z=5, size=6, title="Head dome")
    with m.step("Lamp post"):
        m.stack("3941", "Light Bluish Gray", at=(7, 2, 1), count=2)
    m.use("dome_from_quarters", x=7, y=2, z=7, size=2, color="Red", title="Lamp dome")
