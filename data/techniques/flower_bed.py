# SPDX-License-Identifier: Apache-2.0
"""Flower bed: a bed of soil plates edged with a smooth border of tiles, planted with clumps of leaves and flowers of
two heights in mixed colors, spaced so their petals never touch.

m.use("flower_bed", x, y, z, length, depth=2, seed=1, edge="Dark Bluish Gray", soil="Reddish Brown",
flowers=("Red", "Dark Pink", "Bright Light Orange"), title="Flower bed") builds, on the studs at height z, a bed
(length + 2) x (depth + 2) studs from (x, y): a layer of plates in the soil color, a 1-wide ring of tiles round its
edge, and in its length x depth middle, plants at random: a leafy 2 x 2 clump (4727) where it fits, flowers (a round
plate with tabs as the bloom, on a green round 1 x 1 plate or brick as the stem), and low leaves (bright green round
1 x 1 plates). A bloom's tabs reach past its stud, so no bloom or clump stands beside another bloom, side by side or
corner to corner; low leaves fill between. Each bloom is one of `flowers`. A few studs stay bare. length: 2 to 12;
depth: 1 to 6. seed picks the planting. Returns the z of the edge's top.
"""

import random

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3034",
    "3832",
    "3795",
    "3020",
    "3460",
    "3666",
    "3710",
    "3023b",
    "3024",
    "4727",
    "3062b",
    "6141",
    "33291",
    "2431",
    "2445",
    "3069b",
    "3070b",
    "4162",
    "4477",
    "60479",
    "63864",
    "6636",
    "91988",
]

TABS = 0.1  # the round plate with tabs reaches a tenth of a stud past its stud each way


def build(
    m,
    x,
    y,
    z,
    length,
    depth=2,
    seed=1,
    edge="Dark Bluish Gray",
    soil="Reddish Brown",
    flowers=("Red", "Dark Pink", "Bright Light Orange"),
    title="Flower bed",
):
    """The bed and its plants; returns the z of the edge's top."""
    if not 2 <= length <= 12 or not 1 <= depth <= 6:
        raise ValueError(
            f"flower_bed takes a length of 2 to 12 and a depth of 1 to 6, not {length}, {depth}"
        )
    if not flowers:
        raise ValueError("flower_bed needs at least one flower color")
    rng = random.Random(seed)
    with m.step(f"{title}, soil"):
        m.fill(Plate, soil, at=(x, y, z), size=(length + 2, depth + 2))
    ring = row_actions(m, Tile, edge, (x, y, z + 1), length + 2)
    ring += row_actions(m, Tile, edge, (x, y + depth + 1, z + 1), length + 2)
    ring += row_actions(m, Tile, edge, (x, y + 1, z + 1), depth, axis="y")
    ring += row_actions(m, Tile, edge, (x + length + 1, y + 1, z + 1), depth, axis="y")
    in_steps(m, f"{title}, edge", ring)
    used, blooms, clumps = set(), set(), set()

    def near(cells):
        return {
            (i + a, j + b) for i, j in cells for a in (-1, 0, 1) for b in (-1, 0, 1)
        }

    actions = []
    for j in range(depth):
        for i in range(length):
            if (i, j) in used:
                continue
            cell = {(i + a, j + b) for a in (0, 1) for b in (0, 1)}
            at = (x + 1 + i, y + 1 + j, z + 1)
            roll = rng.random()
            if (
                roll < 0.25
                and i + 2 <= length
                and j + 2 <= depth
                and not cell & used
                and not near(cell) & blooms
            ):
                used |= cell
                clumps |= cell
                actions.append((1, lambda at=at: m.add("4727", "Green", at=at)))
                continue
            used.add((i, j))
            # A bloom's tabs reach past its stud: nothing else stands at its height beside it.
            if roll < 0.85 and not near({(i, j)}) & (blooms | clumps):
                blooms.add((i, j))
                bloom = rng.choice(flowers)
                stem = rng.choice((1, 3))  # a round plate or a round brick

                def flower(at=at, bloom=bloom, stem=stem):
                    sx, sy, sz = at
                    m.add("6141" if stem == 1 else "3062b", "Green", at=at)
                    m.add("33291", bloom, at=(sx - TABS, sy - TABS, sz + stem))

                actions.append((2, flower))
            elif roll < 0.85:
                actions.append((1, lambda at=at: m.add("6141", "Bright Green", at=at)))
    in_steps(m, f"{title}, plants", actions)
    return z + 2


def example(m):
    """A bed with 6 x 3 studs of planting."""
    m.use("flower_bed", x=0, y=0, z=0, length=6, depth=3)
