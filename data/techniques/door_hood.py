# SPDX-License-Identifier: Apache-2.0
"""Door hood: a canopy over a door, a plate that stands out two studs with a tiled edge, on brackets.

m.use("door_hood", x, y, z, width=4, color="White", title="Door hood") lays, over a door `width` studs wide whose
front-left corner is (x, y), at the door's top z, a plate 2 deep and `width` + 2 long half on the wall, a second one on
its front row that stands two studs out, and a tile along its front edge. z is the door's top (a door_frame's
.door_top), so a figure walks under the hood (docs/spec/building-rules.md, rule 1); the wall stops there, or leaves the
hood's cells free in that course. Returns the z of its top, with .underside (z).
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import fit

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
    """The canopy; returns the z of its top, with .underside."""
    n = width + 2
    with m.step(title):
        # Two layers that overlap a row: the first half on the wall, the second two studs out, held by the first.
        m.row(Plate, color, at=(x - 1, y - 1, z), length=n, width=2)
        m.row(Plate, color, at=(x - 1, y - 2, z + 1), length=n, width=2)
        m.row(Tile, color, at=(x - 1, y - 2, z + 2), length=n)
    return fit(z + 3, underside=z)


def example(m):
    """A door in a wall with its hood over it."""
    wall = m.use("door_frame", x=0, y=0, z=0, length=6, rows=6, offset=1)
    m.use("door_hood", x=1, y=0, z=wall.door_top)
