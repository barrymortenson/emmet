# SPDX-License-Identifier: Apache-2.0
"""Shuttered window: a window with its glass and a louvered shutter each side, the shutters on side studs.

m.use("shuttered_window", x, y, z, color="Tan", frame="White", shutter="Dark Green", title="Window") puts a window 1 x 2
x 2 with its glass (60592, 60601) in a wall along x at (x, y, z), facing front (-y), with a column each side that
carries a shutter: a brick 1 x 1 x 1 2/3 with studs on one side (32952) and a 1 x 1 plate, and a grille tile standing
on the side studs. The wall is 4 studs of it: x - 1 to x + 3. Returns the z of the window's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 21335, set 43217. Not derived
from any model file."""

PARTS = ["60592", "60601", "32952", "3024", "2412b"]

UP = Rot(x=90)  # a 1 x 2 tile on its back, its face to -y, its long side up


def build(m, x, y, z, color="Tan", frame="White", shutter="Dark Green", title="Window"):
    """The window, its glass, and the two shutters; returns the z of its top."""
    with m.step(f"{title}: frame and glass"):
        f = m.add("60592", frame, at=(x, y, z), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
    for side, cx in (("left", x - 1), ("right", x + 2)):
        with m.step(f"{title}: {side} shutter"):
            m.add("32952", color, at=(cx, y, z), rot=3)
            m.add("3024", color, at=(cx, y, z + 5))
            m.add("2412b", shutter, at=(cx, y - 0.4, z), rot=UP)
    return z + 6


def example(m):
    """The window in a wall, on a course of bricks."""
    from emmet_dsl import Brick

    with m.step("Wall"):
        m.row(Brick, "Tan", at=(-1, 0, 0), length=4)
    m.use("shuttered_window", x=0, y=0, z=3)
    with m.step("Over the window"):
        m.row(Brick, "Tan", at=(-1, 0, 9), length=4)
