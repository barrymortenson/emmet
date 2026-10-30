# SPDX-License-Identifier: Apache-2.0
"""Dome window hood: a padded half-dome canopy over a window, two curved-top round-corner bricks on quarter-circle
plates.

m.use("dome_window_hood", x, y, z, color="Dark Red", trim="White", title="Dome hood") puts, over a window 2 studs wide
whose front-left corner is (x, y) and whose top is at z, in a wall one stud thick along x facing front (-y), two Plates
2 x 2 Quarter Circle (7904) in the trim color from (x - 1, y - 1), and on them two Bricks 2 x 2 x 1 2/3 Corner Round
with Curved Top (5906), their round corners to the front and out to the sides, so together they make a half dome 4 studs
across and 2 deep that stands a stud out of the wall. Its back row takes the wall's cells x - 1 to x + 2 from z to z +
6, two bricks' height. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["7904", "5906"]

# Quarter turns that put each part's round corner to the front-left and to the front-right.
DOME = (3, 0)
PLATE = (2, 3)


def build(m, x, y, z, color="Dark Red", trim="White", title="Dome hood"):
    """The trim plates, then the dome; returns the z of its top."""
    with m.step(f"{title}: trim"):
        m.add("7904", trim, at=(x - 1, y - 1, z), rot=PLATE[0])
        m.add("7904", trim, at=(x + 1, y - 1, z), rot=PLATE[1])
    with m.step(f"{title}: dome"):
        m.add("5906", color, at=(x - 1, y - 1, z + 1), rot=DOME[0])
        m.add("5906", color, at=(x + 1, y - 1, z + 1), rot=DOME[1])
    return z + 6


def example(m):
    """A window in a short wall with the hood over it."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Tan", at=(-2, 0, 0), length=6)
    with m.step("Window"):
        f = m.add("60592", "White", at=(0, 0, 3), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
    for r in range(2):
        with m.step(f"Wall, row {r + 2}"):
            m.row(Brick, "Tan", at=(-2, 0, 3 + 3 * r), length=2, first=1 if r else None)
            m.row(Brick, "Tan", at=(2, 0, 3 + 3 * r), length=2, first=1 if r else None)
    z = m.use("dome_window_hood", x=0, y=0, z=9)
    for r in range(2):
        with m.step(f"Wall beside the hood, row {r + 1}"):
            for cx in (-2, 3):
                m.add("3005", "Tan", at=(cx, 0, 9 + 3 * r))
    with m.step("Wall over the hood"):
        m.row(Brick, "Tan", at=(-2, 0, z), length=6)
