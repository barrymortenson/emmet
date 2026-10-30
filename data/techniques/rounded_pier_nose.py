# SPDX-License-Identifier: Apache-2.0
"""Rounded pier nose: a bridge pier 4 studs wide whose ends are smooth half-rounds, quarter-round bricks round a round
core, so the water parts round its nose.

m.use("rounded_pier_nose", x, y, z, length=8, courses=3, both=True, stone="Tan", core="Light Bluish Gray", title="Pier")
builds, on the studs at height z, a pier `length` studs along x (6 or more) and 4 across from (x, y), `courses` bricks
tall. Each course's upstream end (-x) is two 2 x 2 quarter-round bricks (3063b) whose curves make a half circle 4 studs
across, with a 2 x 2 round brick in `core` filling the round hole they leave; with both, the downstream end is the same.
Between them the walls are 1 x n bricks along the sides, every other course starting a stud on so the joints miss, and
the middle is filled in `core`. The top is capped smooth: quarter-round tiles on the noses, a round tile on each core,
and tiles along the sides and middle. Returns the z of the cap's top.
"""

from emmet_dsl import Brick, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21067. Not derived from any
model file."""

PARTS = [
    "3063b",
    "3941",
    "27925",
    "14769",
    "3010",
    "3009",
    "3008",
    "3001",
    "3003",
    "2431",
    "6636",
    "3068b",
]

# A quarter-round's turn by where the middle of its curve is in its 2 x 2 cell.
TURN = {(1, 1): 2, (1, 0): 1, (0, 1): 3, (0, 0): 0}


def build(
    m,
    x,
    y,
    z,
    length=8,
    courses=3,
    both=True,
    stone="Tan",
    core="Light Bluish Gray",
    title="Pier",
):
    """The courses of noses, walls, and core, then the cap; returns the z of the cap's top."""
    if not 6 <= length <= 24 or not 1 <= courses <= 12:
        raise ValueError(
            f"rounded_pier_nose takes a length of 6 to 24 and 1 to 12 courses, not {length}, {courses}"
        )
    # Each rounded end: its cells' x, and 1 when the middle of its curve is on its +x side (the upstream end).
    ends = [(x, 1)] + ([(x + length - 2, 0)] if both else [])
    lo = x + 2  # where the walls start and end
    hi = x + length - 2 if both else x + length
    mid_lo, mid_hi = x + 3, (x + length - 3 if both else x + length)
    for c in range(courses):
        zc = z + 3 * c
        with m.step(f"{title}, course {c + 1}: noses"):
            for ex, side in ends:
                m.add("3063b", stone, at=(ex, y, zc), rot=TURN[(side, 1)])
                m.add("3063b", stone, at=(ex, y + 2, zc), rot=TURN[(side, 0)])
                m.add("3941", core, at=(ex + 1 if side else ex - 1, y + 1, zc))
        with m.step(f"{title}, course {c + 1}: walls"):
            first = 2 if c % 2 and hi - lo > 2 else None
            m.row(Brick, stone, at=(lo, y, zc), length=hi - lo, first=first)
            m.row(Brick, stone, at=(lo, y + 3, zc), length=hi - lo, first=first)
            if mid_hi > mid_lo:
                m.row(
                    Brick, core, at=(mid_lo, y + 1, zc), length=mid_hi - mid_lo, width=2
                )
    zt = z + 3 * courses
    with m.step(f"{title}, cap: noses"):
        for ex, side in ends:
            m.add("27925", stone, at=(ex, y, zt), rot=TURN[(side, 1)])
            m.add("27925", stone, at=(ex, y + 2, zt), rot=TURN[(side, 0)])
            m.add("14769", stone, at=(ex + 1 if side else ex - 1, y + 1, zt))
    with m.step(f"{title}, cap: tiles"):
        m.row(Tile, stone, at=(lo, y, zt), length=hi - lo)
        m.row(Tile, stone, at=(lo, y + 3, zt), length=hi - lo)
        if mid_hi > mid_lo:
            m.row(Tile, stone, at=(mid_lo, y + 1, zt), length=mid_hi - mid_lo, width=2)
    return zt + 1


def example(m):
    """A pier 8 studs long, three bricks tall, standing in water."""
    from emmet_dsl import Plate

    with m.step("Riverbed"):
        m.fill(Plate, "Dark Blue", at=(0, 0, 0), size=(12, 6))
    m.use("rounded_pier_nose", x=2, y=1, z=1)
