# SPDX-License-Identifier: Apache-2.0
"""Rounded car end: the smooth rounded corners at the end of a loco or passenger car, quarter-round bricks stacked
at each corner and a cap of round-corner plates.

m.use("rounded_car_end", x, y, z, rows=3, color="White", cap="Light Bluish Gray", title="Car end") builds the end
of a 6-wide car body (y to y + 6) whose end face is at x, the body running from x toward -x, on the studs at z: at
each of the two corners a quarter-round brick 2 x 2 (85080) with its round side out, `rows` high, and 1 x 2 bricks
between them across the end; on top a round-corner plate 2 x 2 (79491) on each corner and a 2 x 2 plate between, so
the roof that goes on them has a round end too. Returns the z of the cap's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 60337. Not derived from any
model file."""

PARTS = ["85080", "3004", "79491", "3022"]

# Quarter turns that put the brick's round side out: at rot=0 it's at the +x, +y corner, so +x, -y takes 3. (The
# round-corner plate on top is turned one more: at rot=0 its round corner is at +x, -y.)
RIGHT_CORNER = 3
LEFT_CORNER = 0
CAP_RIGHT = 0
CAP_LEFT = 1


def build(m, x, y, z, rows=3, color="White", cap="Light Bluish Gray", title="Car end"):
    """The corners and the bricks between them, then the cap; returns the z of the cap's top."""
    for r in range(rows):
        with m.step(f"{title}, row {r + 1}"):
            m.add("85080", color, at=(x - 2, y, z + 3 * r), rot=RIGHT_CORNER)
            m.add("85080", color, at=(x - 2, y + 4, z + 3 * r), rot=LEFT_CORNER)
            m.add("3004", color, at=(x - 1, y + 2, z + 3 * r), rot=0)
    top = z + 3 * rows
    with m.step(f"{title}: cap"):
        m.add("79491", cap, at=(x - 2, y, top), rot=CAP_RIGHT)
        m.add("79491", cap, at=(x - 2, y + 4, top), rot=CAP_LEFT)
        m.add("3022", cap, at=(x - 2, y + 2, top))
    return top + 1


def example(m):
    """The end of a 6-wide car: a floor, its side walls, and the rounded end."""
    with m.step("Floor"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    for r in range(3):
        with m.step(f"Side walls, row {r + 1}"):
            for side in (0, 5):
                m.add("3009", "White", at=(0, side, 1 + 3 * r), rot=1)
    m.use("rounded_car_end", x=8, y=0, z=1)
