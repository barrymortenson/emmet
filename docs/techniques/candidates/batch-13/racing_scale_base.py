# SPDX-License-Identifier: Apache-2.0
"""Racing-scale base: an 8-wide car's low, stiff floor, wheels on short axles through Technic bricks, Technic sills
along the cabin, and two seats side by side.

m.use("racing_scale_base", x, y, z, length=20, wheelbase=12, color="Black", sill="Black", rim="Light Bluish Gray",
title="Base") stands an 8-wide car base (y to y + 8) `length` studs long on the ground at height z, its front to -x:
a 4-wide spine of plates down the middle; at each axle (the front one 3 studs from the front, the rear `wheelbase`
behind it) two Technic bricks 1 x 2 with an axle hole on the spine and an axle 8 through them, with an 18 mm rim and
its low 30 mm tire on each end, so the track is the full 8; 8-wide plates across the cabin between the wheels and on
the front and rear overhangs; Technic bricks 1 x 8 with holes as the cabin's sills (their holes take pins for side
skirts); and two seats side by side a plate above the floor. The floor stands 2 plates off the ground. Returns the z
of the cabin floor's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 77256, set 77264. Not derived
from any model file."""

PARTS = [
    "3030",
    "32064b",
    "3707",
    "55982",
    "58090",
    "3034",
    "41539",
    "3702",
    "3894",
    "3022",
    "4079",
]

TIRE = 38 / 8  # the tire's radius (58090), in plates
HOLE = 14 / 8  # the Technic brick's hole above its bottom
# A wheel's turn: its hub's axle hole (on its own +z face) toward the car's middle, on the left (-y) and right sides.
LEFT = (0, 0, 1, 0, 1, 0, -1, 0, 0)
RIGHT = (0, 0, -1, 0, 1, 0, 1, 0, 0)
SILLS = {
    4: "3701",
    6: "3894",
    8: "3702",
    10: "2730",
    12: "3895",
}  # Technic Brick 1 x n with Holes


def _raw(m, part, color, c, turn):
    """A part placed by its LDraw origin at the DSL point c (studs, studs, plates)."""
    x, y, z = c
    return m.add_raw(part, color, (20 * y, -8 * z, -20 * x, *turn))


def _across(m, color, x0, x1, y, z):
    """Covers x0 to x1, 8 wide, with an 8 x 8 plate where it fits and 2 x 8 plates for the rest."""
    a = x0
    while x1 - a >= 8:
        m.add("41539", color, at=(a, y, z))
        a += 8
    while x1 - a >= 2:
        m.add("3034", color, at=(a, y, z), rot=0)
        a += 2


def build(
    m,
    x,
    y,
    z,
    length=20,
    wheelbase=12,
    color="Black",
    sill="Black",
    rim="Light Bluish Gray",
    title="Base",
):
    """The spine, axles, wheels, floor, sills, and seats; returns the z of the cabin floor's top."""
    front = x + 3
    rear = front + wheelbase
    if wheelbase - 4 not in SILLS or rear + 5 > x + length or length % 2:
        raise ValueError(
            f"racing_scale_base takes an even wheelbase from 8 to 16 and an even length at least wheelbase + 8, "
            f"not {wheelbase} and {length}"
        )
    floor = z + 2  # the spine's bottom: the tires stand on z
    with m.step(f"{title}: spine"):
        a = x
        while a < x + length:
            n = min(10, x + length - a)
            if n > 2:
                m.add(Plate(4, n), color, at=(a, y + 2, floor), rot=1)
            else:
                m.add(Plate(2, 4), color, at=(a, y + 2, floor), rot=0)
            a += n
    for k, ax in enumerate((front, rear)):
        with m.step(f"{title}: {('front', 'rear')[k]} axle"):
            m.add("32064b", color, at=(ax, y + 3, floor + 1), rot=1)
            m.add("32064b", color, at=(ax, y + 4, floor + 1), rot=1)
            m.add(
                "3707",
                "Black",
                at=(ax + 0.7, y + 0.025, floor + 1 + HOLE - 0.75),
                rot=0,
            )
        with m.step(f"{title}: {('front', 'rear')[k]} wheels"):
            for cy, turn in ((y + 0.85, LEFT), (y + 7.15, RIGHT)):
                c = (ax + 1, cy, floor + 1 + HOLE)
                _raw(m, "55982", rim, c, turn)
                _raw(m, "58090", "Black", c, turn)
    with m.step(f"{title}: floor"):
        _across(m, color, x, front - 1, y, floor + 1)
        _across(m, color, front + 3, rear - 1, y, floor + 1)
        _across(m, color, rear + 3, x + length, y, floor + 1)
    with m.step(f"{title}: sills and seats"):
        for side in (y, y + 7):
            m.add(SILLS[wheelbase - 4], sill, at=(front + 3, side, floor + 2), rot=1)
        for k in range(2):
            m.add("3022", color, at=(rear - 4, y + 2 + 2 * k, floor + 2))
            m.add("4079", "Black", at=(rear - 4, y + 2 + 2 * k, floor + 3))
    return floor + 2


def example(m):
    """An 8-wide base 20 long."""
    m.use("racing_scale_base", x=0, y=0, z=0, color="Dark Bluish Gray", sill="Red")
