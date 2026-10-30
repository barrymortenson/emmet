# SPDX-License-Identifier: Apache-2.0
"""Panel garage door: a pair of tall paneled leaves, tiles on plates, each hung on strap hinges in clips in its jamb so
they swing open.

m.use("panel_garage_door", x, y, z, color="Dark Red", recess="Black", hinge="Black", wall="Reddish Brown", title="Garage
door") builds, in a wall one stud thick along x facing front (-y), an opening 8 studs wide from (x, y) and 20 plates
tall from z, with a jamb each side (at x - 1 and x + 8) of six bricks and two plates, the second and fifth bricks Bricks
1 x 1 with Clip Vertical (60475b), clips out front, and a lintel of bricks over it. Each leaf is a sub-assembly built
flat: a Plate 4 x 8 framed with tiles in `color` round two Tiles 1 x 6 in the recess color laid lengthwise, with two
Plates 1 x 2 with Handle (48336) on its outer edge as strap hinges. The leaves stand up in front of the opening, their
hinge bars in the jambs' clips. It takes the wall's cells x - 1 to x + 8 from z to z + 23. Returns the z of the lintel's
top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["3035", "2431", "6636", "3069b", "3070b", "48336", "60475b", "3005", "3024"]

UP = Rot(
    x=90
)  # a leaf, built lying on its back, stood up with its face to -y and its long side up
CLIP = 0.7  # how far a clip brick's clip stands out from the wall's face
# A stood-up leaf's hinge bars, half a stud out from its plate's edge and 1 3/4 plates in front of its back, sit in the
# clips half a stud in front of the wall: its footprint goes 0.8 stud out from its hinge edge and 0.8 in front.
OUT = 0.8


def _leaf(m, side, color, recess, hinge):
    """A leaf lying on its back, its hinges on the left edge (side "left") or the right."""
    edge, turn, hx = (0, 2, -OUT) if side == "left" else (3, 0, 3)
    other = 3 - edge
    with m.step("Plate and hinges"):
        m.add("3035", color, at=(0, 0, 0), id="plate")
        m.add("48336", hinge, at=(hx, 1, 1), rot=turn, id="hinge/1")
        m.add("48336", hinge, at=(hx, 4, 1), rot=turn, id="hinge/2")
        m.add("3070b", color, at=(edge, 3, 1))
        m.add("3070b", color, at=(edge, 6, 1))
    with m.step("Panels"):
        m.add("2431", color, at=(0, 0, 1), rot=1)
        m.add("2431", color, at=(0, 7, 1), rot=1)
        for c in (1, 2):
            m.add("6636", recess, at=(c, 1, 1))
        m.add("6636", color, at=(other, 1, 1))


def build(
    m,
    x,
    y,
    z,
    color="Dark Red",
    recess="Black",
    hinge="Black",
    wall="Reddish Brown",
    title="Garage door",
):
    """The jambs a row at a time, the leaves on their clips, and the lintel; returns the z of the lintel's top."""
    leaves = [
        m.submodel(
            f"Garage leaf, {s}", _leaf, side=s, color=color, recess=recess, hinge=hinge
        )
        for s in ("left", "right")
    ]
    for r in range(6):
        with m.step(f"{title}, row {r + 1}"):
            for cx in (x - 1, x + 8):
                if r in (1, 4):
                    m.add("60475b", wall, at=(cx, y - CLIP, z + 3 * r), rot=3)
                else:
                    m.add("3005", wall, at=(cx, y, z + 3 * r))
    with m.step(f"{title}, jamb tops"):
        for cx in (x - 1, x + 8):
            m.add("3024", wall, at=(cx, y, z + 18))
            m.add("3024", wall, at=(cx, y, z + 19))
    with m.step(f"{title}: leaves"):
        m.place(leaves[0], at=(x - OUT, y - OUT, z), rot=UP)
        m.place(leaves[1], at=(x + 4, y - OUT, z), rot=UP)
    with m.step(f"{title}: lintel"):
        m.row(Brick, wall, at=(x - 1, y, z + 20), length=10)
    return z + 23


def example(m):
    """A garage door in a short wall on a floor plate."""
    with m.step("Floor"):
        m.add("3958", "Dark Bluish Gray", at=(-1, -3, 0))
        m.add("3958", "Dark Bluish Gray", at=(5, -3, 0))
    m.use("panel_garage_door", x=0, y=1, z=1)
