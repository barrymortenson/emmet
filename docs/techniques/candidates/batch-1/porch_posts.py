# SPDX-License-Identifier: Apache-2.0
"""Porch posts: posts of a round brick under a tall 1 x 1 brick, a beam across them, and a low balustrade between.

m.use("porch_posts", x, y, z, length=6, color="White", title="Porch") stands a post at each end of a run `length` studs
along x from (x, y, z), each a round 1 x 1 brick under a brick 1 x 1 x 3, lays a plate across their tops as the beam,
and between them a balustrade of round plates stacked two high under a rail of tiles. Returns the z of the beam's top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 43217. Not derived
from any model file."""

PARTS = [
    "3062b",
    "14716",
    "6141",
    "3666",
    "3460",
    "3710",
    "2431",
    "3069b",
    "6636",
    "3070b",
]


def build(m, x, y, z, length=6, color="White", title="Porch"):
    """The posts, the beam, and the balustrade; returns the z of the beam's top."""
    if length < 3:
        raise ValueError(f"porch_posts needs a length of 3 or more, not {length}")
    with m.step(f"{title}: posts"):
        for px in (x, x + length - 1):
            m.add("3062b", color, at=(px, y, z))
            m.add("14716", color, at=(px, y, z + 3))
    with m.step(f"{title}: balustrade"):
        for a in range(1, length - 1):
            m.add("6141", color, at=(x + a, y, z))
            m.add("6141", color, at=(x + a, y, z + 1))
    with m.step(f"{title}: rail and beam"):
        m.row(Tile, color, at=(x + 1, y, z + 2), length=length - 2)
        m.row(Plate, color, at=(x, y, z + 12), length=length)
    return z + 13


def example(m):
    """A porch front on its floor."""
    with m.step("Floor"):
        m.add("3795", "Reddish Brown", at=(0, 0, 0), rot=1)
    m.use("porch_posts", x=0, y=0, z=1)
