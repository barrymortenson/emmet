# SPDX-License-Identifier: Apache-2.0
"""Polka-dot cube: a white cube 4 studs a side with a big round spot standing proud on every side face and on top,
like a game's spotted cap.

m.use("polka_dot_cube", x, y, z, color="White", spots="Red", title="Spotted cube") builds, on the studs at height z
over (x, y) to (x + 4, y + 4), a cube 4 studs tall (10 plates): two 4 x 4 plates, a ring 5 plates tall whose middle on
each side is a brick 1 x 2 x 1 2/3 with studs on one side (22885) facing out and whose corners are a 1 x 1 brick under
two 1 x 1 plates, two more 4 x 4 plates, then on top a round tile 2 x 2 (14769) in the middle and four 1 x 3 tiles
round it. A round tile 2 x 2 in `spots` stands on each side's studs, a plate proud of the face. Returns the z of the
top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72040 and set 72036. Not
derived from any model file."""

PARTS = ["3031", "22885", "3005", "3024", "14769", "63864"]

# A side by name: where its brick with side studs goes from the cube's corner and its turn, then where its spot goes
# and the turn that lays a tile on its back against that side.
SIDES = {
    "front": ((1, 0), 3, (1, -0.4), Rot(x=90)),
    "back": ((1, 3), 1, (1, 4), Rot(x=-90)),
    "left": ((0, 1), 2, (-0.4, 1), Rot(y=-90)),
    "right": ((3, 1), 0, (4, 1), Rot(y=90)),
}
CORNERS = ((0, 0), (3, 0), (0, 3), (3, 3))
# The four 1 x 3 tiles round the top's middle, turning like a pinwheel: corner, quarter turns.
RING = (((0, 0), 1), ((3, 0), 0), ((1, 3), 1), ((0, 1), 0))


def build(m, x, y, z, color="White", spots="Red", title="Spotted cube"):
    """The cube and its spots; returns the z of the top."""
    with m.step(f"{title}: base"):
        m.add("3031", color, at=(x, y, z))
        m.add("3031", color, at=(x, y, z + 1))
    with m.step(f"{title}: sides"):
        for (dx, dy), turn, _, _ in SIDES.values():
            m.add("22885", color, at=(x + dx, y + dy, z + 2), rot=turn)
    with m.step(f"{title}: corners"):
        for dx, dy in CORNERS:
            m.add("3005", color, at=(x + dx, y + dy, z + 2))
    with m.step(f"{title}: corner plates"):
        for dx, dy in CORNERS:
            m.stack("3024", color, at=(x + dx, y + dy, z + 5), count=2)
    with m.step(f"{title}: top"):
        m.add("3031", color, at=(x, y, z + 7))
        m.add("3031", color, at=(x, y, z + 8))
        m.add("14769", spots, at=(x + 1, y + 1, z + 9))
        for (dx, dy), turn in RING:
            m.add("63864", color, at=(x + dx, y + dy, z + 9), rot=turn)
    with m.step(f"{title}: spots"):
        for _, _, (dx, dy), rot in SIDES.values():
            m.add("14769", spots, at=(x + dx, y + dy, z + 2), rot=rot)
    return z + 10


def example(m):
    """A spotted cube on a patch of grass."""
    from emmet_dsl import Plate

    with m.step("Grass"):
        m.fill(Plate, "Green", at=(0, 0, 0), size=(6, 6))
    m.use("polka_dot_cube", x=1, y=1, z=1)
