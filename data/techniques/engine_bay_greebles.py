# SPDX-License-Identifier: Apache-2.0
"""Engine bay greebles: a busy, believable engine bay for a large car, a seeded mix of small units on a 2-stud grid:
round cylinders with dish caps, coils topped with lights, grilled boxes, and clusters of round tiles.

m.use("engine_bay_greebles", x, y, z, length=6, width=6, seed=1, color="Dark Bluish Gray", metal="Light Bluish Gray",
title="Engine bay") fills a `length` x `width` area on the studs at z, from (x, y), with a unit on each 2 x 2 cell,
picked by `seed`: a cylinder (a round brick 2 x 2, 3941, with a dish 2 x 2, 4740, in `metal` on it), a coil (a plate 2 x
2 with one stud, 87580, a round brick 1 x 1 and a round plate 1 x 1 in Trans Orange), a box (a brick 2 x 2 with two
grille tiles 1 x 2, 2412b), or a cluster (a round plate 2 x 2 with four round tiles 1 x 1 in `metal`, Trans Clear, and
Trans Red). No two cells side by side get the same unit. length and width are even, 2 to 12. Returns the z of the
tallest unit's top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10300. Not derived from any
model file."""

PARTS = ["3941", "4740", "87580", "3062b", "6141", "3003", "2412b", "4032a", "98138"]

UNITS = ("cylinder", "coil", "box", "cluster")
HEIGHT = {"cylinder": 4, "coil": 5, "box": 4, "cluster": 2}  # in plates


def _unit(m, kind, x, y, z, color, metal):
    """One unit on the 2 x 2 cell whose corner is (x, y), at z."""
    if kind == "cylinder":
        m.add("3941", color, at=(x, y, z))
        m.add("4740", metal, at=(x, y, z + 3))
    elif kind == "coil":
        m.add("87580", color, at=(x, y, z))
        m.add("3062b", "Black", at=(x + 0.5, y + 0.5, z + 1))
        m.add("6141", "Trans Orange", at=(x + 0.5, y + 0.5, z + 4))
    elif kind == "box":
        m.add("3003", "Black", at=(x, y, z))
        for k in range(2):
            m.add("2412b", color, at=(x + k, y, z + 3), rot=0)
    else:
        m.add("4032a", color, at=(x, y, z))
        for (dx, dy), c in zip(
            ((0, 0), (1, 0), (0, 1), (1, 1)),
            (metal, "Trans Clear", "Trans Red", metal),
            strict=True,
        ):
            m.add("98138", c, at=(x + dx, y + dy, z + 1))


def build(
    m,
    x,
    y,
    z,
    length=6,
    width=6,
    seed=1,
    color="Dark Bluish Gray",
    metal="Light Bluish Gray",
    title="Engine bay",
):
    """The units, two cells a step, the far rows first; returns the z of the tallest one's top."""
    if length % 2 or width % 2 or not (2 <= length <= 12 and 2 <= width <= 12):
        raise ValueError(
            f"engine_bay_greebles takes an even length and width from 2 to 12, not {length}, {width}"
        )
    rng = random.Random(seed)
    picks = {}
    for i in range(0, length, 2):
        for j in range(0, width, 2):
            near = {picks.get((i - 2, j)), picks.get((i, j - 2))}
            picks[(i, j)] = rng.choice([u for u in UNITS if u not in near])
    cells = sorted(picks, key=lambda c: (-c[1], c[0]))
    for k in range(0, len(cells), 2):
        with m.step(f"{title}, {k // 2 + 1}"):
            for i, j in cells[k : k + 2]:
                _unit(m, picks[(i, j)], x + i, y + j, z, color, metal)
    return z + max(HEIGHT[u] for u in picks.values())


def example(m):
    """An engine bay 6 x 8 on a chassis plate."""
    with m.step("Chassis"):
        m.add("3036", "Black", at=(0, 0, 0))
    m.use("engine_bay_greebles", x=0, y=0, z=1, length=6, width=8, seed=3)
