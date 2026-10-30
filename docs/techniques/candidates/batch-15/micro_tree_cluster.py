# SPDX-License-Identifier: Apache-2.0
"""Micro tree cluster: a park of micro trees, each one to three parts, in three forms mixed at random, with the ground
between them tiled.

m.use("micro_tree_cluster", x, y, z, length=6, depth=4, seed=1, leaves=("Dark Green", "Green"), poplar="Green",
trunk="Reddish Brown", ground="Dark Bluish Gray", title="Park") plants, on the studs at height z, a length x depth patch
from (x, y): about every other stud, never two side by side, a tree of one of three forms: a round tree (a round 1 x 1
plate as the trunk under two ice cream scoops, 6254, stacked), a short tree (a round 1 x 1 brick under one scoop), or,
less often, a poplar (a 1 x 1 cone, 4589, in the poplar color, on a round plate). Each other tree's leaves are one of
`leaves`. The studs left bare are covered with 1-wide tiles in the ground color, so only the trees stand up. seed picks
the planting. Returns the cells planted, a list of (x, y).
"""

import random

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = ["6141", "6254", "3062b", "4589", "3069b", "3070b", "63864", "2431", "6636"]

SCOOP = (
    1.11,
    1.13,
    4,
)  # an ice cream scoops part's footprint in studs and height in plates (to the next one's seat)


def _scoops(m, color, x, y, z, n):
    for k in range(n):
        m.add(
            "6254",
            color,
            at=(x + 0.5 - SCOOP[0] / 2, y + 0.5 - SCOOP[1] / 2, z + SCOOP[2] * k),
        )


def build(
    m,
    x,
    y,
    z,
    length=6,
    depth=4,
    seed=1,
    leaves=("Dark Green", "Green"),
    poplar="Green",
    trunk="Reddish Brown",
    ground="Dark Bluish Gray",
    title="Park",
):
    """The ground tiles and the trees; returns the cells planted."""
    if not 1 <= length <= 16 or not 1 <= depth <= 16 or not leaves:
        raise ValueError(
            f"micro_tree_cluster takes a length and depth of 1 to 16 and a leaves color, not {length}, {depth}"
        )
    rng = random.Random(seed)
    trees = []
    for j in range(depth):
        for i in range(length):
            if (i - 1, j) in trees or (i, j - 1) in trees:
                continue
            if rng.random() < 0.55:
                trees.append((i, j))
    for j in range(depth):
        with m.step(f"{title}: ground, row {j + 1}"):
            start = 0
            for i in [i for i in range(length) if (i, j) in trees] + [length]:
                if i > start:
                    m.row(Tile, ground, at=(x + start, y + j, z), length=i - start)
                start = i + 1
    for n in range(0, len(trees), 3):
        with m.step(f"{title}: trees {n // 3 + 1}"):
            for i, j in trees[n : n + 3]:
                tx, ty = x + i, y + j
                form = rng.choices(("round", "short", "poplar"), weights=(2, 2, 1))[0]
                color = rng.choice(leaves)
                if form == "round":
                    m.add("6141", trunk, at=(tx, ty, z))
                    _scoops(m, color, tx, ty, z + 1, 2)
                elif form == "short":
                    m.add("3062b", trunk, at=(tx, ty, z))
                    _scoops(m, color, tx, ty, z + 3, 1)
                else:
                    m.add("6141", trunk, at=(tx, ty, z))
                    m.add("4589", poplar, at=(tx, ty, z + 1))
    return [(x + i, y + j) for i, j in trees]


def example(m):
    """A park 6 x 4 studs on a plate, edged with a sidewalk."""
    from emmet_dsl import Plate

    with m.step("Ground"):
        m.add(Plate(6, 8), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    with m.step("Sidewalk"):
        m.row(Tile, "Light Bluish Gray", at=(0, 0, 1), length=8)
        m.row(Tile, "Light Bluish Gray", at=(0, 5, 1), length=8)
        m.row(Tile, "Light Bluish Gray", at=(0, 1, 1), length=4, axis="y")
        m.row(Tile, "Light Bluish Gray", at=(7, 1, 1), length=4, axis="y")
    m.use("micro_tree_cluster", x=1, y=1, z=1, length=6, depth=4, seed=3)
