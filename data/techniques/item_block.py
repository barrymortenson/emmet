# SPDX-License-Identifier: Apache-2.0
"""Item block: a game's floating prize block, a 2 x 2 cube faced on all four sides, held up in the air on a stand of
clear panels.

m.use("item_block", x, y, z, color="Yellow", mark="White", lift=1, face_part=None, title="Item block") stands, on the
studs at height z, two clear panels 1 x 2 x 2 (4864b) back to back over (x, y) to (x + 2, y + 2), `lift` pairs high,
and on them a cube 2.8 studs a side: a 2 x 2 plate, a core of two bricks 1 x 2 x 1 2/3 with studs on three sides
(67329) back to back, so every side has four studs, and a 2 x 2 jumper on each side and on top, each with a 1 x 1
round tile in `mark` on its middle stud. face_part swaps the side jumpers and marks for a 2 x 2 tile: pass a printed
one if the library has the print you want (search "tile 2 x 2 with ... pattern"). The cube reaches 0.4 studs past (x,
y) each way. Returns the z of the cube's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72036. Not derived from any
model file."""

PARTS = ["4864b", "3022", "67329", "87580", "98138"]

# A side face by name: where its jumper's corner goes from the cube's (x, y) and the turn that lays it on its back
# against that side, studs out.
SIDES = {
    "front": ((0, -0.4), Rot(x=90)),
    "back": ((0, 2), Rot(x=-90)),
    "left": ((-0.4, 0), Rot(y=-90)),
    "right": ((2, 0), Rot(y=90)),
}
MARK = 0.4  # a jumper's thickness: a mark on a side jumper's stud starts that far past the jumper, in studs
STAND = 6  # a panel's height, in plates


def build(
    m, x, y, z, color="Yellow", mark="White", lift=1, face_part=None, title="Item block"
):
    """The stand, the cube, and its faces; returns the z of the cube's top."""
    if lift < 1:
        raise ValueError(f"item_block needs a lift of 1 or more, not {lift}")
    for k in range(lift):
        with m.step(f"{title}: stand" + (f" {k + 1}" if lift > 1 else "")):
            # The panels' walls meet in the middle, back to back.
            m.add("4864b", "Trans-Clear", at=(x, y, z + STAND * k), rot=2)
            m.add("4864b", "Trans-Clear", at=(x + 1, y, z + STAND * k), rot=0)
    zb = z + STAND * lift
    with m.step(f"{title}: core"):
        m.add("3022", color, at=(x, y, zb))
        m.add("67329", color, at=(x, y, zb + 1), rot=2)
        m.add("67329", color, at=(x + 1, y, zb + 1), rot=0)
    for name, ((dx, dy), rot) in SIDES.items():
        with m.step(f"{title}: {name} face"):
            if face_part:
                m.add(face_part, color, at=(x + dx, y + dy, zb + 1), rot=rot)
                continue
            m.add("87580", color, at=(x + dx, y + dy, zb + 1), rot=rot)
            # The mark on the jumper's middle stud.
            mx = x + (dx - MARK if dx < 0 else dx + MARK if dx > 0 else 0.5)
            my = y + (dy - MARK if dy < 0 else dy + MARK if dy > 0 else 0.5)
            m.add("98138", mark, at=(mx, my, zb + 2.25), rot=rot)
    with m.step(f"{title}: top"):
        m.add("87580", color, at=(x, y, zb + 6))
        m.add("98138", mark, at=(x + 0.5, y + 0.5, zb + 7))
    return zb + 7


def example(m):
    """Two item blocks floating over a patch of track."""
    from emmet_dsl import Plate

    with m.step("Track"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=9, width=8)
    m.use("item_block", x=1, y=1, z=1)
    m.use("item_block", x=6, y=5, z=1, color="Bright Pink", mark="White", lift=2)
