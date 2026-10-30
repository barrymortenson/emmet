# SPDX-License-Identifier: Apache-2.0
"""Bay window: a square bay that pushes three studs out of a wall, glazed on its front and both sides, under a lean-to
roof.

m.use("bay_window", x, y, z, width=6, plinth=0, color="Dark Red", trim="Reddish Brown", roof="Dark Green", wall="Medium
Nougat", title="Bay window") builds, out of a wall one stud thick along x facing front (-y), a bay `width` studs wide (6
or 8) from (x, y) and 3 studs deep: a floor plate at z, four studs deep, its back row in the wall; on it a Brick 1 x 1 x
3 at each front corner and at each end of the wall line, a Window 1 x 2 x 3 with its glass (60593, 60602) on each side
between them, and (width - 2) / 2 more across the front; a cap plate the same size on top at z + 10, and on it a lean-to
of Slope 45 2 x 2 with a Brick 1 x N behind them. The bay takes the wall's cells x to x + width - 1 from z to z + 14;
between its two end posts the wall is open into the bay, and the floor and cap plates' back rows are plate courses in
the wall line. plinth > 0 lays that many rows of `wall` bricks under the bay's three outer sides, down from z, so it
stands on the ground. Returns the z of the roof's top.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 43217. Not derived
from any model file."""

PARTS = ["60593", "60602", "14716", "3032", "3035", "3039", "3009", "3008", "3004"]

PLATES = {6: "3032", 8: "3035"}  # Plate 4 x n
BACKS = {6: "3009", 8: "3008"}  # Brick 1 x n


def _window(m, at, along_x, color):
    """A Window 1 x 2 x 3 with its glass, along x (on the front) or along y (on a side)."""
    f = m.add("60593", color, at=at, rot=1 if along_x else 0)
    (x0, y0, z0), (x1, y1, _) = f.min, f.max
    if along_x:
        g = ((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875)
    else:
        g = ((x0 + x1 - 0.2) / 2, (y0 + y1 - 1.65) / 2, z0 + 0.875)
    m.add("60602", "Trans-Clear", at=g, rot=1 if along_x else 0)


def build(
    m,
    x,
    y,
    z,
    width=6,
    plinth=0,
    color="Dark Red",
    trim="Reddish Brown",
    roof="Dark Green",
    wall="Medium Nougat",
    title="Bay window",
):
    """The plinth, the floor, the posts and windows, the cap, and the roof; returns the z of the roof's top."""
    if width not in PLATES:
        raise ValueError(f"bay_window takes a width of 6 or 8, not {width}")
    front, right = y - 3, x + width - 1
    for r in range(plinth):
        zr = z - 3 * (plinth - r)
        with m.step(f"{title}: plinth, row {r + 1}"):
            m.row(
                Brick, wall, at=(x, front, zr), length=width, first=2 if r % 2 else None
            )
            for cx in (x, right):
                m.add(Brick(1, 2), wall, at=(cx, front + 1, zr))
    with m.step(f"{title}: floor and posts"):
        m.add(PLATES[width], trim, at=(x, front, z), rot=1)
        for cx in (x, right):
            m.add("14716", color, at=(cx, front, z + 1))
            m.add("14716", color, at=(cx, y, z + 1))
    with m.step(f"{title}: side windows"):
        for cx in (x, right):
            _window(m, (cx, front + 1, z + 1), False, color)
    with m.step(f"{title}: front windows"):
        for k in range((width - 2) // 2):
            _window(m, (x + 1 + 2 * k, front, z + 1), True, color)
    with m.step(f"{title}: cap"):
        m.add(PLATES[width], trim, at=(x, front, z + 10), rot=1)
    with m.step(f"{title}: roof"):
        for k in range(0, width, 2):
            m.add("3039", roof, at=(x + k, front, z + 11), rot=3)
        m.add(BACKS[width], roof, at=(x, y - 1, z + 11), rot=1)
    return z + 14


def example(m):
    """A bay on a one-row plinth, in a wall with a course to tie it in."""
    with m.step("Wall, row 1"):
        m.row(Brick, "Tan", at=(-2, 0, 0), length=10)
    z = m.use("bay_window", x=0, y=0, z=3, plinth=1, wall="Tan")
    for r in range(5):
        with m.step(f"Wall beside the bay, course {r + 1}"):
            for cx in (-2, 6):
                if r in (0, 4):  # plate courses level with the bay's floor and cap
                    m.add("3023b", "Tan", at=(cx, 0, 3 + 10 * (r // 4)), rot=1)
                else:
                    m.add(Brick(1, 2), "Tan", at=(cx, 0, 1 + 3 * r), rot=1)
    with m.step("Wall over the bay"):
        m.row(Brick, "Tan", at=(-2, 0, 14), length=10)
