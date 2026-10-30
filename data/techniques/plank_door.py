# SPDX-License-Identifier: Apache-2.0
"""Plank door: a board door of tiles on a plate, hung on its two strap hinges in clips in the jamb, so it swings.

m.use("plank_door", x, y, z, color="Dark Green", hinge="Black", wall="Tan", title="Plank door") builds, in a wall one
stud thick along x facing front (-y), a doorway 4 studs wide from (x, y), 5 bricks tall from z, with a jamb of bricks
each side (at x - 1 and x + 4) and a Brick 1 x 6 over it as the lintel; the left jamb's first and fifth bricks are
Bricks 1 x 1 with Clip Vertical (60475b), their clips out front. The door is a sub-assembly built flat: a Plate 4 x 6
with Tiles 1 x 6 down it as the planks, two Plates 1 x 2 with Handle (48336) on its hinge edge as the strap hinges, and
a round knob; it stands up in front of the doorway with its hinge bars in the jamb's clips, so it can swing open. It
takes the wall's cells x - 1 to x + 4 from z to z + 18. Returns the z of the lintel's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335. Not derived from any
model file."""

PARTS = ["3032", "6636", "3069b", "3070b", "6141", "48336", "60475b", "3005", "3009"]

UP = Rot(
    x=90
)  # the door, built lying on its back, stood up with its face to -y and its long side up
CLIP = 0.7  # how far a clip brick's clip stands out from the wall's face
# Where the stood-up door's footprint corner goes from the doorway's front-left corner: its hinge bars, half a stud left
# of the plate's edge and 1 3/4 plates in front of its back, sit in the clips, half a stud in front of the wall.
DOOR_AT = (-0.8, -0.8)


def _door(m, color, hinge):
    with m.step("Plate and hinges"):
        m.add("3032", color, at=(0, 0, 0), id="plate")
        m.add("48336", hinge, at=(-0.8, 0, 1), rot=2, id="hinge/1")
        m.add("48336", hinge, at=(-0.8, 4, 1), rot=2, id="hinge/2")
    with m.step("Planks"):
        m.add("3069b", color, at=(0, 2, 1))
        for c in (1, 2):
            m.add("6636", color, at=(c, 0, 1))
        m.add("3069b", color, at=(3, 0, 1))
        m.add("6141", hinge, at=(3, 2, 1), id="knob")
        m.add("3070b", color, at=(3, 3, 1))
        m.add("3069b", color, at=(3, 4, 1))


def build(
    m, x, y, z, color="Dark Green", hinge="Black", wall="Tan", title="Plank door"
):
    """The jambs a row at a time, the door on its clips, and the lintel; returns the z of the lintel's top."""
    door = m.submodel("Plank door", _door, color=color, hinge=hinge)
    for r in range(5):
        with m.step(f"{title}, row {r + 1}"):
            if r in (0, 4):
                m.add("60475b", wall, at=(x - 1, y - CLIP, z + 3 * r), rot=3)
            else:
                m.add("3005", wall, at=(x - 1, y, z + 3 * r))
            m.add("3005", wall, at=(x + 4, y, z + 3 * r))
    with m.step(f"{title}: door"):
        m.place(door, at=(x + DOOR_AT[0], y + DOOR_AT[1], z), rot=UP)
    with m.step(f"{title}: lintel"):
        m.add("3009", wall, at=(x - 1, y, z + 15), rot=1)
    return z + 18


def example(m):
    """A plank door in a short wall on a floor plate."""
    with m.step("Floor"):
        m.add("3958", "Dark Bluish Gray", at=(-1, -2, 0))
    m.use("plank_door", x=0, y=2, z=1)
