# SPDX-License-Identifier: Apache-2.0
"""Jettied floor: an upper story whose floor stands a stud out past the walls below, front and back.

m.use("jettied_floor", x, y, z, length, depth, color="Reddish Brown", title="Upper floor") lays the floor of a story on
walls whose rectangle is length x depth from (x, y), at the z of their top: plates `length` long and `depth` + 2 deep
from y - 1, so the floor overhangs the walls by a stud at the front and the back, and the upper story's walls build on
its edge. Returns the z of the floor's top.
"""

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350. Not derived from
any model file."""

PARTS = [
    "3034",
    "3832",
    "2445",
    "3795",
    "3020",
    "3021",
    "3022",
    "3023b",
    "3710",
    "3666",
    "3460",
]


def build(m, x, y, z, length, depth, color="Reddish Brown", title="Upper floor"):
    """The floor; returns the z of its top."""
    from emmet_dsl import Plate

    with m.step(title):
        m.fill(Plate, color, at=(x, y - 1, z), size=(length, depth + 2))
    return z + 1


def example(m):
    """A small house: a brick story, then a jettied floor with the upper story's walls on its edge."""
    m.wall(0, 0, 0, length=6, depth=4, rows=2, color="Sand Green")
    z = m.use("jettied_floor", x=0, y=0, z=6, length=6, depth=4)
    m.wall(0, -1, z, length=6, depth=6, rows=2, color="White")
