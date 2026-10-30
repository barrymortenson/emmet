# SPDX-License-Identifier: Apache-2.0
"""Blade sign: a tall shop sign standing out from the wall edge-on to the street, hung on two bar handles in clips.

m.use("blade_sign", x, y, z, color="Tan", board="White", trim="Lavender", hinge="Black", title="Blade sign") builds one
column of a wall one stud thick along x facing front (-y), at cell x from (x, y, z), five bricks tall: Bricks 1 x 1 with
Clip Vertical (60475b) as its first and fifth, clips out front, and plain 1 x 1 bricks between. The sign is a
sub-assembly built flat: a Plate 2 x 6 with a Tile 1 x 4 in the board color framed by round tiles in the trim color at
top and bottom, and two Plates 1 x 2 with Handle (48336) along its back edge. It stands on edge, square to the wall, its
face to +x and two studs out into the street, its handles in the clips. It takes the wall's cell x from z to z + 15.
Returns the z of the column's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["3795", "48336", "2431", "98138", "3069b", "60475b", "3005"]

EDGE_ON = Rot(
    x=90, z=90
)  # the sign, built lying on its back, stood on edge: its face to +x, its long side up
CLIP = 0.7  # how far a clip brick's clip stands out from the wall's face
# The stood sign's footprint corner from the clip column's front-left corner: its handles' bars, half a stud beyond the
# plate's back edge and 1 3/4 plates off its back, sit in the clips half a stud in front of the wall.
SIGN_AT = (-0.2, -3)


def _sign(m, board, trim, hinge):
    with m.step("Plate and handles"):
        m.add("3795", trim, at=(0, 0, 0), id="plate")
        m.add("48336", hinge, at=(1, 0, 1), id="handle/1")
        m.add("48336", hinge, at=(1, 4, 1), id="handle/2")
        m.add("3069b", trim, at=(1, 2, 1))
    with m.step("Board"):
        m.add("98138", trim, at=(0, 0, 1))
        m.add("2431", board, at=(0, 1, 1))
        m.add("98138", trim, at=(0, 5, 1))


def build(
    m,
    x,
    y,
    z,
    color="Tan",
    board="White",
    trim="Lavender",
    hinge="Black",
    title="Blade sign",
):
    """The clip column a brick at a time, then the sign on its clips; returns the z of the column's top."""
    sign = m.submodel("Blade sign", _sign, board=board, trim=trim, hinge=hinge)
    for r in range(5):
        with m.step(f"{title}, row {r + 1}"):
            if r in (0, 4):
                m.add("60475b", color, at=(x, y - CLIP, z + 3 * r), rot=3)
            else:
                m.add("3005", color, at=(x, y, z + 3 * r))
    with m.step(f"{title}: sign"):
        m.place(sign, at=(x + SIGN_AT[0], y + SIGN_AT[1], z), rot=EDGE_ON)
    return z + 15


def example(m):
    """A sign at the end of a short shop wall."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Tan", at=(0, 0, 0), length=6)
    z = m.use("blade_sign", x=5, y=0, z=3)
    for r in range(5):
        with m.step(f"Wall beside the sign, row {r + 2}"):
            m.row(
                Brick, "Tan", at=(0, 0, 3 + 3 * r), length=5, first=2 if r % 2 else None
            )
    with m.step("Wall, row 7"):
        m.row(Brick, "Tan", at=(0, 0, z), length=6)
