# SPDX-License-Identifier: Apache-2.0
"""Battlements: a parapet of merlons and crenels along the outer edge of a thick wall's top, with a tiled walk behind.

m.use("battlements", x, y, z, length, color="Light Bluish Gray", merlon=2, crenel=1, tie=True, title="Battlements")
crowns a wall 2 studs thick that runs `length` studs along x from (x, y), its top at z. A course of 2-wide plates ties
the wall's two leaves and bridges their joints (tie=False leaves it out, for a top that is already tied, such as
machicolation's); on its front row (y) a course of bricks makes the breastwork, and on that merlons, 1 x `merlon` bricks
with a tile cap, stand `crenel` studs apart, a merlon at each end, the crenels' floors tiled. The back row is the wall
walk, tiled. length must be a whole number of merlon + crenel, plus a merlon. Returns the z of the walk, so figures and
the next technique know where to stand.
"""

from emmet_dsl import Brick, Plate, Tile

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3004",
    "3005",
    "3622",
    "3069b",
    "3070b",
    "63864",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "91988",
    "4282",
    "2431",
    "6636",
    "4162",
]

MERLONS = {1: "3005", 2: "3004", 3: "3622"}  # Brick 1 x n
CAPS = {1: "3070b", 2: "3069b", 3: "63864"}  # Tile 1 x n


def build(
    m,
    x,
    y,
    z,
    length,
    color="Light Bluish Gray",
    merlon=2,
    crenel=1,
    tie=True,
    title="Battlements",
):
    """The tie course, the breastwork, the merlons and their caps, and the walk; returns the z of the walk."""
    if merlon not in MERLONS or crenel not in CAPS:
        raise ValueError(
            f"battlements takes merlons and crenels 1 to 3 studs wide, not {merlon}, {crenel}"
        )
    bay = merlon + crenel
    if length < merlon + bay or (length - merlon) % bay:
        raise ValueError(
            f"battlements needs a length of n x {bay} + {merlon} (n 1 or more), not {length}"
        )
    if tie:
        with m.step(f"{title}: tie course"):
            m.row(Plate, color, at=(x, y, z), length=length, width=2)
        z += 1
    with m.step(f"{title}: breastwork and walk"):
        # Started a stud in, so no joint sits over a joint of the plates below.
        m.row(Brick, color, at=(x, y, z), length=length, first=1)
        m.row(Tile, color, at=(x, y + 1, z), length=length)
    starts = range(0, length, bay)
    zm = z + 3
    with m.step(f"{title}: merlons"):
        for u in starts:
            m.add(MERLONS[merlon], color, at=(x + u, y, zm), rot=1)
    with m.step(f"{title}: caps"):
        for u in starts:
            m.add(CAPS[merlon], color, at=(x + u, y, zm + 3), rot=1)
            if u + merlon < length:
                m.add(CAPS[crenel], color, at=(x + u + merlon, y, zm), rot=1)
    return z + 1


def example(m):
    """A short stretch of wall, 2 studs thick, crowned with battlements."""
    for r in range(2):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Light Bluish Gray",
                at=(0, 0, 3 * r),
                length=8,
                width=2,
                first=2 if r else None,
            )
    m.use("battlements", x=0, y=0, z=6, length=8)
