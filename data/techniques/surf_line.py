# SPDX-License-Identifier: Apache-2.0
"""Surf line: a beach on water, its edge ragged, with broken white surf at the waterline, some touching the sand and
some a stud out.

m.use("surf_line", x, y, z, length, depth=3, seed=1, sand="Tan", surf=("White", "Trans-Clear"), title="Beach")
lays a beach of 1-wide plates on water-colored studs at height z (a blue baseplate or plates), `length` studs along x
from x, its land to the back: rows from y + 2 back to y + depth are full, and the two front rows stop short by 0 to 2
studs, a stud more or less from column to column, so the edge wanders. In front of most columns goes a bit of surf:
a 1 x 1 tile, a 1 x 1 round tile, or a 1 x 1 cheese slope facing the water, against the sand or a stud out, in the
`surf` colors, so the foam reads broken rather than a solid band. Keep two studs of water clear in front of y. seed
picks the edge and the surf. Returns the z of the sand's top.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 21335 (surf line on a baseplate). Not derived from any model file."""

PARTS = [
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3024",
    "3070b",
    "98138",
    "54200",
    "4477",
    "60479",
]

SURF = ("3070b", "98138", "54200")  # Tile 1 x 1, Tile 1 x 1 Round, a cheese slope


def build(
    m,
    x,
    y,
    z,
    length,
    depth=3,
    seed=1,
    sand="Tan",
    surf=("White", "Trans-Clear"),
    title="Beach",
):
    """The beach and its surf; returns the z of the sand's top."""
    if not 2 <= length <= 24 or not 2 <= depth <= 6:
        raise ValueError(
            f"surf_line takes a length of 2 to 24 and a depth of 2 to 6, not {length}, {depth}"
        )
    if not surf:
        raise ValueError("surf_line needs at least one surf color")
    rng = random.Random(seed)
    # How far each column's sand stops short of y: 0 to 2, changing by at most 1 from one column to the next.
    edge = [rng.randrange(3)]
    for _ in range(length - 1):
        edge.append(min(2, max(0, edge[-1] + rng.choice((-1, 0, 1)))))
    edge = [min(e, depth - 1) for e in edge]
    actions = []
    for r in range(depth - 1, -1, -1):
        k = 0
        while k < length:
            if edge[k] > r:
                k += 1
                continue
            n = 1
            while k + n < length and edge[k + n] <= r:
                n += 1
            actions += row_actions(m, Plate, sand, (x + k, y + r, z), n)
            k += n
    in_steps(m, f"{title}, sand", actions)
    foam = []
    for k, e in enumerate(edge):
        if rng.random() < 0.25:
            continue
        out = 1 + (rng.random() < 0.3)
        part = rng.choice(SURF)
        at = (x + k, y + e - out, z)
        color = rng.choice(surf)
        if part == "54200":
            color = surf[0]  # a cheese slope in the first, opaque color
        foam.append(
            (1, lambda part=part, color=color, at=at: m.add(part, color, at=at, rot=3))
        )
    in_steps(m, f"{title}, surf", foam)
    return z + 1


def example(m):
    """A beach 8 studs long on blue plates."""
    with m.step("Water"):
        m.add("3036", "Blue", at=(0, 0, 0), rot=1)
    m.use("surf_line", x=0, y=2, z=1, length=8, depth=4)
