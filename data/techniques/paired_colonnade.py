# SPDX-License-Identifier: Apache-2.0
"""Paired colonnade: a classical museum front at micro size, pairs of columns from upright binoculars between end piers,
under a plate cornice with sloped shoulders.

m.use("paired_colonnade", x, y, z, pairs=4, color="White", wall="Dark Bluish Gray", title="Colonnade") builds, on the
studs at height z from (x, y), a front `pairs` + 2 studs long along x and 2 studs deep, facing front (-y): a back wall
of 1 x 1 and longer bricks a brick high, with a plate on it, in the wall color; in the front row, an end pier at each
end (a 1 x 1 brick and a 1 x 1 plate), and between them, on each stud, a pair of binoculars (30162) stood on end so
their two barrels are a pair of columns. A plate 2 studs deep caps it all as the cornice, a cheese slope at each end
facing out makes the pediment's shoulders, and tiles cover the middle. Returns the z of the cornice's top.
"""

from emmet_dsl import Brick, Plate, Rot, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = [
    "30162",
    "3005",
    "3024",
    "3010",
    "3004",
    "3623",
    "3666",
    "3460",
    "3020",
    "3795",
    "54200",
    "3069b",
    "2431",
]

UPRIGHT = Rot(
    y=90, z=90
)  # binoculars on end, their barrels upright, the pair side by side along x
PAIR = (1.2, 0.6)  # the upright binoculars' footprint, in studs


def build(
    m, x, y, z, pairs=4, color="White", wall="Dark Bluish Gray", title="Colonnade"
):
    """The back wall, the piers and columns, the cornice and shoulders; returns the z of the top."""
    if not 1 <= pairs <= 10:
        raise ValueError(f"paired_colonnade takes 1 to 10 pairs, not {pairs}")
    length = pairs + 2
    with m.step(f"{title}: back wall"):
        m.row(Brick, wall, at=(x, y + 1, z), length=length)
        m.row(Plate, wall, at=(x, y + 1, z + 3), length=length)
    with m.step(f"{title}: piers"):
        for px in (x, x + length - 1):
            m.add(Brick(1, 1), color, at=(px, y, z))
            m.add(Plate(1, 1), color, at=(px, y, z + 3))
    with m.step(f"{title}: columns"):
        for i in range(pairs):
            cx, cy = x + 1.5 + i, y + 0.5
            m.add(
                "30162", color, at=(cx - PAIR[0] / 2, cy - PAIR[1] / 2, z), rot=UPRIGHT
            )
    zc = z + 4
    with m.step(f"{title}: cornice"):
        m.fill(Plate, color, at=(x, y, zc), size=(length, 2))
    with m.step(f"{title}: shoulders"):
        for j in range(2):
            m.add("54200", color, at=(x, y + j, zc + 1), rot=2)
            m.add("54200", color, at=(x + length - 1, y + j, zc + 1), rot=0)
            m.row(Tile, color, at=(x + 1, y + j, zc + 1), length=pairs)
    return zc + 1


def example(m):
    """A museum front of four column pairs on a tiled plaza."""
    with m.step("Plaza"):
        m.add(Plate(4, 8), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    with m.step("Steps"):
        m.row(Tile, "Light Bluish Gray", at=(0, 0, 1), length=8)
    m.use("paired_colonnade", x=1, y=1, z=1, pairs=4)
