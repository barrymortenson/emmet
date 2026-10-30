# SPDX-License-Identifier: Apache-2.0
"""Micro traffic: a 2-lane road strip of dark tiles with tiny cars and buses standing in it, each two or three parts,
for the streets of a micro city.

m.use("micro_traffic", x, y, z, length=16, axis="x", cars=3, buses=1, seed=1, road="Dark Bluish Gray", colors=("Red",
"Blue", "White", "Yellow"), bus="Bright Light Orange", title="Traffic") lays, on the studs at height z, a road 2 studs
wide and `length` long from (x, y) along `axis` ("x" or "y"), a lane per stud, and stands `cars` cars and `buses` buses
in the lanes at random, a stud or more apart. A car is a 1 x 2 jumper plate (3794b) in one of `colors` with a black 1 x
1 tile on its center stud as the cabin, so it has a hood and a trunk; a bus is a 1 x 3 plate in the bus color, a 1 x 3
plate in transparent clear for its windows, and a 1 x 3 tile roof. Under each the road has a plate in its own color
instead of tiles, flush with them, and the vehicle stands on its studs. seed picks the places and colors. Returns the z
of the road's top.
"""

import random

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066 and set 21067. Not
derived from any model file."""

PARTS = [
    "3794b",
    "3070b",
    "3023b",
    "3623",
    "63864",
    "3069b",
    "2431",
    "4162",
    "6636",
    "3666",
    "3460",
]

LONG = {"car": 2, "bus": 3}
SEAT = {
    "car": "3023b",
    "bus": "3623",
}  # the road plate under each, flush with the tiles


def build(
    m,
    x,
    y,
    z,
    length=16,
    axis="x",
    cars=3,
    buses=1,
    seed=1,
    road="Dark Bluish Gray",
    colors=("Red", "Blue", "White", "Yellow"),
    bus="Bright Light Orange",
    title="Traffic",
):
    """The vehicles and the road around them; returns the z of the road's top."""
    if axis not in ("x", "y") or length < 4 or cars < 0 or buses < 0 or not colors:
        raise ValueError(
            "micro_traffic takes an axis of 'x' or 'y', a length of 4 or more, counts of 0 or more, and a color, "
            f"not {axis!r}, {length}, {cars}, {buses}"
        )
    rng = random.Random(seed)

    def at(u, v, h=0):
        """A point u studs along the road and v across it."""
        return (x + u, y + v, z + h) if axis == "x" else (x + v, y + u, z + h)

    turn = 1 if axis == "x" else 0  # a 1 x n part's long side along the road
    # Place the vehicles a lane at a time: each takes its length and a stud of gap after it.
    taken = {0: set(), 1: set()}
    vehicles = []
    for kind in ["bus"] * buses + ["car"] * cars:
        n = LONG[kind]
        spots = [
            (lane, u)
            for lane in (0, 1)
            for u in range(length - n + 1)
            if not {u - 1, *range(u, u + n), u + n} & taken[lane]
        ]
        if not spots:
            raise ValueError(
                f"micro_traffic: {cars} cars and {buses} buses don't fit on a road {length} long"
            )
        lane, u = rng.choice(spots)
        taken[lane] |= set(range(u, u + n))
        vehicles.append((kind, lane, u))
    vehicles.sort(key=lambda v: (v[1], v[2]))
    for lane in (0, 1):
        with m.step(f"{title}: lane {lane + 1}"):
            start = 0
            for kind, ln, u in [v for v in vehicles if v[1] == lane] + [
                ("end", lane, length)
            ]:
                if u > start:
                    m.row(Tile, road, at=at(start, lane), length=u - start, axis=axis)
                if kind in LONG:
                    # The vehicle's seat: a plate flush with the tiles, its studs left bare.
                    m.add(SEAT[kind], road, at=at(u, lane), rot=turn)
                start = u + LONG.get(kind, 0)
    for n in range(0, len(vehicles), 3):
        with m.step(f"{title}: vehicles {n // 3 + 1}"):
            for kind, lane, u in vehicles[n : n + 3]:
                if kind == "car":
                    m.add("3794b", rng.choice(colors), at=at(u, lane, 1), rot=turn)
                    m.add("3070b", "Black", at=at(u + 0.5, lane, 2))
                else:
                    m.add("3623", bus, at=at(u, lane, 1), rot=turn)
                    m.add("3623", "Trans Clear", at=at(u, lane, 2), rot=turn)
                    m.add("63864", bus, at=at(u, lane, 3), rot=turn)
    return z + 1


def example(m):
    """A street 12 studs long between two sidewalks."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.add(Plate(4, 12), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    with m.step("Sidewalks"):
        m.row(Tile, "Light Bluish Gray", at=(0, 0, 1), length=12)
        m.row(Tile, "Light Bluish Gray", at=(0, 3, 1), length=12)
    m.use("micro_traffic", x=0, y=1, z=1, length=12, cars=3, buses=1, seed=2)
