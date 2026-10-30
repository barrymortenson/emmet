# SPDX-License-Identifier: Apache-2.0
"""Side vent grille: a supercar's or superbike's side radiator vent, a panel built flat with a wing plate for its raked
outline and a field of grille tiles inside a border in the body color, then stood on side studs.

m.use("side_vent_grille", x, y, z, length=8, color="Red", grille="Dark Bluish Gray", title="Side vent") lays a course of
bricks 1 x 4 with studs on the side (30414) along a body's left side at z, `length` studs along x from x, their studs
out to -y, and stands on their studs a panel 4 studs tall built as a submodel: a wing 2 x 4 (41769a) at the top of its
front end for the raked outline and plates under and behind it; tiles 1 x n in the body color along its bottom row and
along its top row from a stud into the wing, which ties the wing to the plates; grille tiles 1 x 2 (2412b) in `grille`
standing across the two rows between, the whole length, slats that tie the panel's halves. The panel covers z + 0.5 to z
+ 10.5 and stands 0.8 studs out from y. length: 8, 12, or 16. Returns the z of the course's top (z + 3).
"""

from emmet_dsl import Plate, Rot, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 42202. Not derived from any
model file."""

PARTS = [
    "30414",
    "41769a",
    "3020",
    "3035",
    "3036",
    "2412b",
    "3666",
    "3460",
    "4162",
    "63864",
]

STAND = Rot(x=90)  # a panel built flat, stood up: its top to -y, its rows up


def _panel(m, length, color, grille):
    """The panel lying flat in its own frame: its front (raked) end at x = 0, its 4 rows along y."""
    with m.step("Side vent: outline"):
        m.add("3020", color, at=(0, 0, 0), rot=1)
        # Its slant on the top edge, narrow end forward.
        m.add("41769a", color, at=(0, 2, 0), rot=2)
        m.fill(Plate, color, at=(4, 0, 0), size=(length - 4, 4))
    with m.step("Side vent: border"):
        m.row(Tile, color, at=(0, 0, 1), length=length)
        m.row(Tile, color, at=(3, 3, 1), length=length - 3)
    # Grille tiles across the two middle rows: slats that tie the panel's lower and upper halves.
    slats = range(length)
    for k in range(0, len(slats), 7):
        with m.step(f"Side vent: grille, {k // 7 + 1}"):
            for a in slats[k : k + 7]:
                m.add("2412b", grille, at=(a, 1, 1), rot=0)


def build(
    m, x, y, z, length=8, color="Red", grille="Dark Bluish Gray", title="Side vent"
):
    """The course with side studs, then the panel on them; returns the z of the course's top."""
    if length not in (8, 12, 16):
        raise ValueError(
            f"side_vent_grille takes a length of 8, 12, or 16, not {length}"
        )
    with m.step(f"{title}: course"):
        for a in range(0, length, 4):
            m.add("30414", color, at=(x + a, y, z), rot=3)  # its side studs to -y
    panel = m.submodel("side_vent", _panel, length=length, color=color, grille=grille)
    # The panel's bottom row of anti-studs goes on the side studs, 1.75 plates up the course.
    with m.step(title):
        m.place(panel, at=(x, y - 0.8, z + 0.5), rot=STAND)
    return z + 3


def example(m):
    """A vent on the side of a body."""
    with m.step("Floor"):
        m.add("3035", "Black", at=(0, 0, 0), rot=1)
    m.use("side_vent_grille", x=0, y=0, z=1)
