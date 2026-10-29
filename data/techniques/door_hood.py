# SPDX-License-Identifier: Apache-2.0
"""Door hood: a canopy over a door, a plate that stands out two studs with a tiled edge, on brackets.

m.use("door_hood", x, y, z, width=4, color="White", title="Door hood") lays, over a door `width` studs wide whose
front-left corner is (x, y), at the door's top z, a plate 2 deep and `width` + 2 long half on the wall, a second one on
its front row that stands two studs out, and a tile along its front edge. Returns the z of its top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350, set 43217. Not derived
from any model file."""

PARTS = [
    "3795",
    "3832",
    "3034",
    "3020",
    "6636",
    "3460",
    "4162",
    "2431",
    "60596",
    "60623",
    "3005",
]


def build(m, x, y, z, width=4, color="White", title="Door hood"):
    """The canopy; returns the z of its top."""
    n = width + 2
    with m.step(title):
        # Two layers that overlap a row: the first half on the wall, the second two studs out, held by the first.
        m.row(Plate, color, at=(x - 1, y - 1, z), length=n, width=2)
        m.row(Plate, color, at=(x - 1, y - 2, z + 1), length=n, width=2)
        m.row(Tile, color, at=(x - 1, y - 2, z + 2), length=n)
    return z + 3


def example(m):
    """A door in a wall with its hood over it."""

    with m.step("Door"):
        frame = m.add("60596", "White", at=(1, 0, 0), rot=3)
        (x0, y0, z0), (x1, y1, _) = frame.min, frame.max
        m.add(
            "60623",
            "Reddish Brown",
            at=((x0 + x1 - 3.55) / 2, (y0 + y1 - 0.3) / 2, z0 + 0.875),
            rot=3,
        )
    for r in range(6):
        with m.step(f"Wall, row {r + 1}"):
            m.add("3005", "Tan", at=(0, 0, 3 * r))
            m.add("3005", "Tan", at=(5, 0, 3 * r))
    m.use("door_hood", x=1, y=0, z=18)
