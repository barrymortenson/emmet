# SPDX-License-Identifier: Apache-2.0
"""Corbelled balcony: a small balcony on a wall's face, its slab carried by three long pointed corbels hanging under
it, with a railing of round balusters along its front.

m.use("corbelled_balcony", x, y, z, length=5, color="White", rail="White", title="Balcony") builds a balcony `length`
studs long (odd, 3 or more) from x, on a wall 1 stud thick whose face is at y (the wall runs along x and faces front,
-y), at the z of a wall course's top: a slab of plates 3 studs deep, from y - 2 over the wall's top, tied by a second
layer over its back two rows, and under its front, three 75-degree inverted slopes 2 x 1 x 3 (2449) at the ends and the
middle, hanging from the slab's studs against the wall's face, each 2 studs deep at the top and tapering to 1 at the
bottom, 3 bricks down. Round 1 x 1 bricks (3062b) stand on every stud of the slab's front row under a handrail of
plates. Lay the wall's next course on the tie's back row. Returns the z of the tie's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = [
    "2449",
    "3062b",
    "3021",
    "3020",
    "3795",
    "3034",
    "3623",
    "3710",
    "3666",
    "3460",
]


def build(m, x, y, z, length=5, color="White", rail="White", title="Balcony"):
    """The slab, the corbels under it, and the railing; returns the z of the slab's top."""
    if length < 3 or length % 2 == 0:
        raise ValueError(
            f"corbelled_balcony takes an odd length of 3 or more, not {length}"
        )
    with m.step(f"{title}: slab"):
        m.row(Plate, color, at=(x, y, z), length=length, first=2)
        # A second layer over the wall's top and a stud out, which the slab's front then hangs under.
        m.row(Plate, color, at=(x, y - 1, z + 1), length=length, width=2, first=2)
    with m.step(f"{title}: slab front"):
        m.row(Plate, color, at=(x, y - 2, z), length=length, width=2)
    with m.step(f"{title}: corbels"):
        for c in (x, x + length // 2, x + length - 1):
            m.add("2449", color, at=(c, y - 2, z - 9), rot=3)
    with m.step(f"{title}: balusters"):
        for c in range(length):
            m.add("3062b", rail, at=(x + c, y - 2, z + 1))
    with m.step(f"{title}: handrail"):
        m.row(Plate, rail, at=(x, y - 2, z + 4), length=length)
    return z + 2


def example(m):
    """A balcony on a wall four courses up, with a course of wall over its back."""
    from emmet_dsl import Brick

    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=7)
    for r in range(4):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Light Bluish Gray",
                at=(0, 0, 1 + 3 * r),
                length=7,
                first=2 if r % 2 else 3,
            )
    top = m.use("corbelled_balcony", x=0, y=0, z=13, length=7)
    with m.step("Wall, row 5"):
        m.row(Brick, "Light Bluish Gray", at=(0, 0, top), length=7, first=3)
