# SPDX-License-Identifier: Apache-2.0
"""Big round eye: a large expressive eye, a printed round tile 2 x 2 on the four side studs of a brick 1 x 2 x 1 2/3,
under a curved slope for an eyelid: for a bird, a robot, a cartoon creature.

m.use("big_round_eye", x, y, z, facing="front", color="Bright Green", eye="14769p0p", lid=True, lid_color=None,
title="Eye") builds a block 2 studs wide and 2 deep from (x, y), on the studs at height z, 5 plates tall: in the face
row a brick 1 x 2 x 1 2/3 with four studs on its side pointing out, and behind it a brick 1 x 2 and two plates 1 x 2; on
the side studs a round tile 2 x 2 printed with an eye (14769p0p, a copper iris; 14769p01 is a plain black eye on white),
which covers the brick's face; and with lid, a curved slope 2 x 2 on top, falling toward the face, its edge over the eye
(lid_color, the head's color by default). Lay the head around the block. facing is the face: "front" (-y) and "back" (+y)
blocks are 2 along x, "left" (-x) and "right" (+x) ones 2 along y; (x, y) is the block's corner with the lowest x and y.
Returns the z of the block's top (with the lid, the lid's top).
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 43290 and 43279. Not derived
from any model file."""

PARTS = ["22885", "14769p0p", "14769p01", "15068", "3004", "3023b"]

# By facing: the side stud brick's quarter turn and its cell, the plain brick's quarter turn and cell, the turn that
# stands the tile on its side facing out and where it goes, and the lid's turn (a curved slope falls toward +x at 0).
FACING = {
    "front": (3, (0, 0), 1, (0, 1), Rot(x=90), (0, -0.4), 3),
    "back": (1, (0, 1), 1, (0, 0), Rot(x=-90), (0, 2), 1),
    "left": (2, (0, 0), 0, (1, 0), Rot(y=-90), (-0.4, 0), 2),
    "right": (0, (1, 0), 0, (0, 0), Rot(y=90), (2, 0), 0),
}


def build(
    m,
    x,
    y,
    z,
    facing="front",
    color="Bright Green",
    eye="14769p0p",
    lid=True,
    lid_color=None,
    title="Eye",
):
    """The eye block, the eye, and the lid; returns the z of the top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    stud_turn, (si, sj), plain_turn, (pi, pj), stand, (ex, ey), fall = FACING[facing]
    with m.step(f"{title}: block"):
        m.add("22885", color, at=(x + si, y + sj, z), rot=stud_turn)
        m.add("3004", color, at=(x + pi, y + pj, z), rot=plain_turn)
        m.add("3023b", color, at=(x + pi, y + pj, z + 3), rot=plain_turn)
        m.add("3023b", color, at=(x + pi, y + pj, z + 4), rot=plain_turn)
    with m.step(f"{title}: eye"):
        m.add(eye, "White", at=(x + ex, y + ey, z), rot=stand)
    if not lid:
        return z + 5
    with m.step(f"{title}: lid"):
        m.add("15068", lid_color or color, at=(x, y, z + 5), rot=fall)
    return z + 7


def example(m):
    """A bird's head, 6 studs wide, with an eye each side of the middle."""
    with m.step("Neck plate"):
        m.add("3032", "Dark Bluish Gray", at=(0, 0, 0), rot=1)  # Plate 4 x 6, along x
    with m.step("Head, course 1"):
        m.add("3001", "Bright Green", at=(0, 0, 1), rot=1)  # Brick 2 x 4
        m.add("3003", "Bright Green", at=(4, 0, 1))  # Brick 2 x 2
        m.add("3001", "Bright Green", at=(0, 2, 1), rot=1)
        m.add("3003", "Bright Green", at=(4, 2, 1))
    with m.step("Head, plate"):
        m.add("3032", "Bright Green", at=(0, 0, 4), rot=1)
    m.use("big_round_eye", x=0, y=0, z=5, title="Left eye")
    m.use("big_round_eye", x=4, y=0, z=5, title="Right eye")
    with m.step("Head between and behind the eyes"):
        m.add("3004", "Bright Green", at=(2, 0, 5))  # Brick 1 x 2, along y
        m.add("3004", "Bright Green", at=(3, 0, 5))
        m.add("3001", "Bright Green", at=(1, 2, 5), rot=1)
        m.add("3004", "Bright Green", at=(0, 2, 5))
        m.add("3004", "Bright Green", at=(5, 2, 5))
