# SPDX-License-Identifier: Apache-2.0
"""Rolled sill: the smooth, rolled lower edge of a classic car's side, a row of curved slopes on a plate stood on side
studs, thick at the top and rolling in to nothing at the bottom.

m.use("rolled_sill", x, y, z, length=8, color="Dark Green", title="Sill") lays a course of bricks 1 x 4 with studs on
the side (30414) along a body's left side at z, `length` studs along x from x, their studs out to -y, and on those studs
a plate 2 x `length` on its side, studs out, with a curved slope 2 x 1 (11477) on every stud column, its thick end up
and its curve rolling down and in. The sill stands 1.2 studs out from y and covers z + 0.5 to z + 5.5; build the rest of
the side on the course. length: 4, 8, or 12. Returns the z of the course's top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11381. Not derived from any
model file."""

PARTS = ["30414", "3020", "3034", "2445", "11477"]

SIDE = Rot(y=-90, z=90)  # a plate on its back, its studs to -y, long along x
# A curved slope on its back, its curve to -y and its thin end down.
ROLL = Rot(y=90, z=270)


def build(m, x, y, z, length=8, color="Dark Green", title="Sill"):
    """The course with side studs, then the plate and slopes on it; returns the z of the course's top."""
    if length not in (4, 8, 12):
        raise ValueError(f"rolled_sill takes a length of 4, 8, or 12, not {length}")
    with m.step(f"{title}: course"):
        for a in range(0, length, 4):
            m.add("30414", color, at=(x + a, y, z), rot=3)
    # The side studs are 1.75 plates up the course; the plate's lower row of anti-studs goes on them.
    low = z + 0.5
    with m.step(f"{title}: plate"):
        m.add(Plate(2, length), color, at=(x, y - 0.4, low), rot=SIDE)
    for half in range(0, length, 4):
        with m.step(f"{title}: curves, {half + 1} to {half + 4}"):
            for a in range(half, half + 4):
                m.add("11477", color, at=(x + a, y - 1.2, low), rot=ROLL)
    return z + 3


def example(m):
    """A sill along the side of a car's floor, with the side's next course on it."""
    with m.step("Floor"):
        m.add("3035", "Black", at=(0, 0, 0), rot=1)
    top = m.use("rolled_sill", x=0, y=0, z=1)
    with m.step("Side"):
        m.add("3008", "Dark Green", at=(0, 0, top), rot=1)
