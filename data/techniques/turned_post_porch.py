# SPDX-License-Identifier: Apache-2.0
"""Turned post porch: porch posts turned from round bricks and round plates, under a fascia that curves around the front
corners and returns to the wall.

m.use("turned_post_porch", x, y, z, length=6, depth=3, tiers=4, color="Medium Nougat", fascia="Reddish Brown",
rail="White", title="Porch") stands, on a porch floor whose studs are at z, a post at each front corner of the length x
depth rectangle whose front-left corner is (x, y) (the house wall runs along its back, at y + depth): each post is
`tiers` round 1 x 1 bricks with a round 1 x 1 plate on each, so it reads as turned. Over them goes the fascia: a Plate 2
x 2 Quarter Circle (7904) at each front corner, so the edge curves round, plates along the front between them, and
plates back along each side to the wall; a second layer of plates, set in from the round corners, bridges their joints.
Between the posts at the front, a low rail of round plates under tiles. length 5 to 12, depth 3 to 8. Returns the z of
the fascia's top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 43217. Not derived
from any model file."""

PARTS = [
    "3062b",
    "6141",
    "7904",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3069b",
    "2431",
    "6636",
]

CORNER = {
    "left": 2,
    "right": 3,
}  # the quarter turn that puts a corner plate's round corner front-left or front-right


def build(
    m,
    x,
    y,
    z,
    length=6,
    depth=3,
    tiers=4,
    color="Medium Nougat",
    fascia="Reddish Brown",
    rail="White",
    title="Porch",
):
    """The posts a tier at a time, the rail, and the fascia; returns the z of the fascia's top."""
    if not 5 <= length <= 12 or not 3 <= depth <= 8:
        raise ValueError(
            f"turned_post_porch takes a length of 5 to 12 and a depth of 3 to 8, not {length} x {depth}"
        )
    right = x + length - 1
    for t in range(tiers):
        with m.step(f"{title}: posts, tier {t + 1}"):
            for px in (x, right):
                m.add("3062b", color, at=(px, y, z + 4 * t))
                m.add("6141", color, at=(px, y, z + 4 * t + 3))
            if t == 0:
                for a in range(1, length - 1):
                    m.add("6141", rail, at=(x + a, y, z))
    with m.step(f"{title}: rail"):
        m.row(Tile, rail, at=(x + 1, y, z + 1), length=length - 2)
    top = z + 4 * tiers
    with m.step(f"{title}: fascia, corners"):
        m.add("7904", fascia, at=(x, y, top), rot=CORNER["left"])
        m.add("7904", fascia, at=(right - 1, y, top), rot=CORNER["right"])
        m.row(Plate, fascia, at=(x + 2, y, top), length=length - 4)
        for px in (x, right):
            m.row(Plate, fascia, at=(px, y + 2, top), length=depth - 2, axis="y")
    with m.step(f"{title}: fascia, top"):
        # A second layer, set in from the round corners, bridges every joint of the first.
        m.row(Plate, fascia, at=(x + 1, y, top + 1), length=length - 2)
        for px in (x, right):
            m.row(Plate, fascia, at=(px, y + 1, top + 1), length=depth - 1, axis="y")
    return top + 2


def example(m):
    """A porch on its floor against a stub of the house wall."""
    from emmet_dsl import Brick

    with m.step("Floor"):
        m.add("3795", "Reddish Brown", at=(0, 0, 0), rot=1)
        m.add("3666", "Reddish Brown", at=(0, 2, 0), rot=1)
    for r in range(6):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick, "White", at=(0, 3, 3 * r), length=6, first=2 if r % 2 else None
            )
    m.use("turned_post_porch", x=0, y=0, z=1)
