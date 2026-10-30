# SPDX-License-Identifier: Apache-2.0
"""Balustrade parapet: a stone balustrade along a roof edge, round balusters on every stud between a base rail and a
handrail, with an arched centerpiece carrying an urn and stone piers at the ends.

m.use("balustrade_parapet", x, y, z, length, color="Tan", baluster="White", title="Balustrade") builds a balustrade
along x from (x, y, z) on a wall top or a roof's edge, facing front (-y): a base rail of 1 x n plates, a 1 x 1 brick
pier at each end, a 1 x 3 arch (4490) in the middle, and a round 1 x 1 brick (3062b) in the baluster color on every
stud between. A handrail of plates ties their tops, tiled smooth; over the arch a 1 x 3 plate raises the centerpiece, an
urn of a round plate and a cone in the baluster color on it, and a round plate caps each pier. length is odd, 7 or
more. Returns the z of the handrail's top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = [
    "3062b",
    "4490",
    "3005",
    "6141",
    "4589",
    "3623",
    "3070b",
    "3069b",
    "3710",
    "3666",
    "3460",
]


def build(m, x, y, z, length, color="Tan", baluster="White", title="Balustrade"):
    """The rails, piers, balusters, and centerpiece; returns the z of the handrail's top."""
    if length < 7 or length % 2 == 0:
        raise ValueError(
            f"balustrade_parapet needs an odd length of 7 or more, not {length}"
        )
    mid = x + (length - 3) // 2  # the arch's first stud
    with m.step(f"{title}: base rail"):
        m.row(
            Plate, color, at=(x, y, z), length=length, first=3 if length > 4 else None
        )
    with m.step(f"{title}: piers and arch"):
        for px in (x, x + length - 1):
            m.add("3005", color, at=(px, y, z + 1))
        m.add("4490", color, at=(mid, y, z + 1), rot=1)
    with m.step(f"{title}: balusters"):
        for c in range(x + 1, x + length - 1):
            if not mid <= c < mid + 3:
                m.add("3062b", baluster, at=(c, y, z + 1))
    top = z + 4
    with m.step(f"{title}: handrail"):
        m.row(Plate, color, at=(x, y, top), length=length, first=2)
    with m.step(f"{title}: handrail tiles"):
        for start, end in ((x + 1, mid), (mid + 3, x + length - 1)):
            m.row(Tile, color, at=(start, y, top + 1), length=end - start)
        for px in (x, x + length - 1):
            m.add("6141", baluster, at=(px, y, top + 1))
    with m.step(f"{title}: centerpiece"):
        m.add("3623", color, at=(mid, y, top + 1), rot=1)
        for c in (mid, mid + 2):
            m.add("3070b", color, at=(c, y, top + 2))
        m.add("6141", baluster, at=(mid + 1, y, top + 2))
        m.add("4589", baluster, at=(mid + 1, y, top + 3))
    return top + 2


def example(m):
    """A balustrade along the top of a short wall."""
    from emmet_dsl import Brick

    for r in range(2):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick, "Medium Nougat", at=(0, 0, 3 * r), length=11, first=2 if r else 3
            )
    m.use("balustrade_parapet", x=0, y=0, z=6, length=11)
