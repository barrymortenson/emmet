# SPDX-License-Identifier: Apache-2.0
"""Chamfered door portal: a sci-fi doorway with cut corners, an opening 4 studs wide in a wall 2 studs thick.

m.use("chamfered_door_portal", x, y, z, height=5, color="Dark Bluish Gray", trim="White", light="Red",
title="Portal") builds a wall section 8 studs long along x from (x, y) and 2 studs thick, standing on the studs at z,
with an opening 4 studs wide in the middle and `height` bricks tall (3 to 8; 5 fits a figure). Its four corners are
cut at 45 degrees: a Slope Brick 45 2 x 2 (3039) at each foot, its high half in the jamb and its slope in the opening,
and an inverted one (3660b) at each head, so the opening reads as an octagon. The jambs are 2 x 2 columns whose
courses cross, one jamb carries a door control (a 1 x 1 tile in `light` on a side stud, facing front), a 2 x 2 tile
is the sill, and the lintel is a 1 x 8 plate in `trim` on the front, one in `color` behind, and a 2 x 8 plate over
both that ties the jambs. Returns the z of the top of the lintel.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75419. Not derived from any
model file."""

PARTS = [
    "3039",
    "3660b",
    "3003",
    "3004",
    "3005",
    "87087",
    "3070b",
    "3068b",
    "3460",
    "3034",
]

FACE = Rot(y=-90, z=90)  # a tile on its back, its face to -y


def build(
    m,
    x,
    y,
    z,
    height=5,
    color="Dark Bluish Gray",
    trim="White",
    light="Red",
    title="Portal",
):
    """The jambs, the cut corners, the control, and the lintel; returns the z of the lintel's top."""
    if not 3 <= height <= 8:
        raise ValueError(
            f"chamfered_door_portal takes a height of 3 to 8 bricks, not {height}"
        )
    for r in range(height):
        zr = z + 3 * r
        with m.step(f"{title}, row {r + 1}"):
            if r == 0 or r == height - 1:
                # A corner: the slope's high half sits in the jamb, its face in the opening.
                part = "3039" if r == 0 else "3660b"
                m.add("3004", color, at=(x, y, zr))
                m.add(part, color, at=(x + 1, y, zr), rot=0)
                m.add(part, color, at=(x + 5, y, zr), rot=2)
                m.add("3004", color, at=(x + 7, y, zr))
            elif r % 2:
                m.add("3003", color, at=(x, y, zr))
                m.add("3003", color, at=(x + 6, y, zr))
            elif r == 2:
                # The control course: two bricks across on the left; on the right a side-stud brick faces front.
                for jx in (x,):
                    m.add("3004", color, at=(jx, y, zr), rot=1)
                    m.add("3004", color, at=(jx, y + 1, zr), rot=1)
                m.add("87087", color, at=(x + 6, y, zr), rot=3)
                m.add("3005", color, at=(x + 7, y, zr))
                m.add("3004", color, at=(x + 6, y + 1, zr), rot=1)
            else:
                for jx in (x, x + 6):
                    m.add("3004", color, at=(jx, y, zr), rot=1)
                    m.add("3004", color, at=(jx, y + 1, zr), rot=1)
        if r == 0:
            with m.step(f"{title}, sill"):
                m.add("3068b", color, at=(x + 3, y, z))
        if r == 2:
            with m.step(f"{title}, door control"):
                m.add("3070b", light, at=(x + 6, y - 0.4, zr + 0.5), rot=FACE)
    zl = z + 3 * height
    with m.step(f"{title}, lintel"):
        m.add("3460", trim, at=(x, y, zl), rot=1)
        m.add("3460", color, at=(x, y + 1, zl), rot=1)
    with m.step(f"{title}, lintel tie"):
        m.add("3034", color, at=(x, y, zl + 1), rot=1)
    return zl + 2


def example(m):
    """The portal on a strip of floor, between two short stretches of wall."""
    from emmet_dsl import Brick, Plate

    with m.step("Floor"):
        m.fill(Plate, "Black", at=(-2, -2, 0), size=(12, 6))
    with m.step("Walls each side"):
        for wx in (-2, 8):
            m.row(Brick, "Dark Bluish Gray", at=(wx, 0, 1), length=2, width=2)
    m.use("chamfered_door_portal", x=0, y=0, z=1)
