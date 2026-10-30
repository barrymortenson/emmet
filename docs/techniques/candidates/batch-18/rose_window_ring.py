# SPDX-License-Identifier: Apache-2.0
"""Rose window ring: a big round window in a flat Gothic facade, a ring plate standing on side studs with colored glass
flush in its hole, over a gallery of little columns and under a gable.

m.use("rose_window_ring", x, y, z, stone="Tan", ring="White", glass="Trans Dark Blue", title="Rose window") builds, on
the studs at height z, a facade 6 studs wide along x from (x, y), 1 stud thick, facing front (-y): a course of bricks; a
gallery course whose 1 x 4 brick with studs on its side carries two 1 x 2 grille tiles, their ribs the little columns;
two courses of side-stud bricks with plates between, so their studs are two studs apart; on those studs, a 4 x 4 round
plate with a 2 x 2 hole (11833) stood up as the ring, and through its hole a 2 x 2 dish in `glass` on the studs behind,
flush with the ring's face; a course over it; and a gable of 45 degree slopes as the head. 1 x 1 bricks frame the ring
at each side. Returns the z of the top of the gable.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file."""

PARTS = [
    "3009",
    "30414",
    "3005",
    "2412b",
    "3666",
    "3010",
    "11833",
    "4740",
    "3040b",
    "3004",
    "3044b",
]

FACE = Rot(x=90)  # a plate or dish stood up, its top to the front (-y)
FLAT = Rot(y=-90, z=90)  # a tile on its back, its face to the front, long along x
OUT = 0.4  # a plate or tile on its back stands out this far, in studs


def build(
    m, x, y, z, stone="Tan", ring="White", glass="Trans Dark Blue", title="Rose window"
):
    """The courses, the gallery, the ring and its glass, and the gable; returns the z of the gable's top."""
    with m.step(f"{title}, base course"):
        m.add("3009", stone, at=(x, y, z), rot=1)
    zg = z + 3
    with m.step(f"{title}, gallery"):
        m.add("3005", stone, at=(x, y, zg))
        m.add("30414", stone, at=(x + 1, y, zg), rot=3)
        m.add("3005", stone, at=(x + 5, y, zg))
        for k in range(2):
            m.add("2412b", stone, at=(x + 1 + 2 * k, y - OUT, zg + 0.5), rot=FLAT)
    za = zg + 3
    for i, zc in enumerate((za, za + 5)):
        with m.step(f"{title}, ring course {i + 1}"):
            m.add("3005", stone, at=(x, y, zc))
            m.add("30414", stone, at=(x + 1, y, zc), rot=3)
            m.add("3005", stone, at=(x + 5, y, zc))
            if i == 0:
                m.add("3666", stone, at=(x, y, zc + 3), rot=1)
                m.add("3666", stone, at=(x, y, zc + 4), rot=1)
    zt = za + 8
    with m.step(f"{title}, course over the ring"):
        m.add("3005", stone, at=(x, y, zt))
        m.add("3010", stone, at=(x + 1, y, zt), rot=1)
        m.add("3005", stone, at=(x + 5, y, zt))
    with m.step(f"{title}, ring and glass"):
        # The side studs are 1.75 plates up a brick, a plate's first anti-studs 1.25 plates in: the ring starts half
        # a plate up the first ring course.
        m.add("11833", ring, at=(x + 1, y - OUT, za + 0.5), rot=FACE)
        m.add("4740", glass, at=(x + 2, y - OUT, za + 3), rot=FACE)
    with m.step(f"{title}, gable 1"):
        m.add("3040b", stone, at=(x, y, zt + 3), rot=2)
        m.add("3004", stone, at=(x + 2, y, zt + 3), rot=1)
        m.add("3040b", stone, at=(x + 4, y, zt + 3), rot=0)
    with m.step(f"{title}, gable 2"):
        m.add("3040b", stone, at=(x + 1, y, zt + 6), rot=2)
        m.add("3040b", stone, at=(x + 3, y, zt + 6), rot=0)
        m.add("3044b", stone, at=(x + 2, y, zt + 9))
    return zt + 12


def example(m):
    """A rose window facade on a plate."""
    with m.step("Ground"):
        m.add("3795", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("rose_window_ring", x=0, y=1, z=1)
