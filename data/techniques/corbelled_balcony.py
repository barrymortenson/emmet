# SPDX-License-Identifier: Apache-2.0
"""Corbelled balcony: a small balcony on a wall's face, its slab carried by three long pointed corbels hanging under
it, with a railing of round balusters along its front.

m.use("corbelled_balcony", x, y, z, length=5, color="White", rail="White", title="Balcony") builds a balcony `length`
studs long (odd, 3 or more) from x, on a wall 1 stud thick whose face is at y (the wall runs along x and faces front,
-y), at the z of a wall course's top: a slab of plates 3 studs deep, from y - 2 over the wall's top, tied by a second
layer over its back two rows, and under its front, three 75-degree inverted slopes 2 x 1 x 3 (2449) at the ends and the
middle, hanging from the slab's studs against the wall's face, each 2 studs deep at the top and tapering to 1 at the
bottom, 3 bricks down. Round 1 x 1 bricks (3062b) stand on every stud of the slab's front row under a handrail of
plates. It's a submodel, built apart (its corbels go in from below) and set on the wall in one step; lay the wall's next
course on the tie's back row. A figure walks under the corbels, not the slab: over a door, z goes 9 plates or more over
the door's top (a door_frame's .door_top + 9), and with no door, 5 bricks and 9 plates or more over the ground
(docs/spec/building-rules.md, rule 1). The floor behind the balusters is a stud deep, room for a figure to stand but not
the 3 studs a door's way out needs (rule 2): set it under a window, not a door. Returns the z of the tie's top, with
.underside (the z of the corbels' bottom, z - 9).
"""

from emmet_dsl import Plate
from emmet_dsl.core import fit

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


def _balcony(m, length, color, rail, title):
    """The balcony in its own frame: the wall's row at y = 0, the slab's bottom at z = 0."""
    with m.step(f"{title}: slab"):
        m.row(Plate, color, at=(0, 0, 0), length=length, first=2)
        # A second layer over the wall's top and a stud out, which the slab's front then hangs under.
        m.row(Plate, color, at=(0, -1, 1), length=length, width=2, first=2)
    with m.step(f"{title}: slab front"):
        m.row(Plate, color, at=(0, -2, 0), length=length, width=2)
    with m.step(f"{title}: corbels"):
        for c in (0, length // 2, length - 1):
            m.add("2449", color, at=(c, -2, -9), rot=3)
    with m.step(f"{title}: balusters"):
        for c in range(length):
            m.add("3062b", rail, at=(c, -2, 1))
    with m.step(f"{title}: handrail"):
        m.row(Plate, rail, at=(0, -2, 4), length=length)


def build(m, x, y, z, length=5, color="White", rail="White", title="Balcony"):
    """The slab, the corbels under it, and the railing; returns the z of the tie's top, with .underside."""
    if length < 3 or length % 2 == 0:
        raise ValueError(
            f"corbelled_balcony takes an odd length of 3 or more, not {length}"
        )
    balcony = m.submodel(
        "corbelled_balcony",
        _balcony,
        length=length,
        color=color,
        rail=rail,
        title=title,
    )
    lo = balcony.box[0]
    with m.step(title):
        m.place(balcony, at=(x + lo[0], y + lo[1], z + lo[2]))
    return fit(z + 2, underside=z - 9)


def example(m):
    """A balcony on a wall eight courses up, its corbels 5 bricks over the ground, with a course of wall over its
    back."""
    from emmet_dsl import Brick

    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=7)
    for r in range(8):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Light Bluish Gray",
                at=(0, 0, 1 + 3 * r),
                length=7,
                first=2 if r % 2 else 3,
            )
    top = m.use("corbelled_balcony", x=0, y=0, z=25, length=7)
    with m.step("Wall, row 9"):
        m.row(Brick, "Light Bluish Gray", at=(0, 0, top), length=7, first=3)
