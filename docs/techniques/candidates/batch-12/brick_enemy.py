# SPDX-License-Identifier: Apache-2.0
"""Brick enemy: a small walking game enemy, a wide domed head over a short body on two feet, with eyes on side studs.

m.use("brick_enemy", x, y, z, facing=3, head="Reddish Brown", body="Tan", feet="Black", title="Enemy") stands an enemy
on the studs at height z, its feet over (x, y) to (x + 2, y + 2): two 1 x 2 plates as feet, a 2 x 2 brick body, and a
head 4 studs wide and 2 deep that overhangs the body a stud each side: a 2 x 4 plate, a course of a brick 1 x 4 with
studs on one side (30414) in front and a plain 1 x 4 behind, two printed eye tiles (98138p07) on the middle side studs
and plain 1 x 1 tiles on the outer two, a 2 x 4 plate, and two curved slopes 2 x 2 falling away left and right, so the
top is a dome. It's a submodel; facing is the quarter turns from facing +x (3, the default, faces front, -y). Returns
the placed copy's Handle, whose .max[2] is the top of the head.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72040. Not derived from any
model file."""

PARTS = ["3023b", "3003", "3020", "30414", "3010", "98138p07", "3070b", "15068"]

EYE = Rot(x=90)  # a 1 x 1 tile on its back, its face to -y
OUT = 0.4  # a tile's thickness, in studs


def _enemy(m, head, body, feet):
    """The enemy facing -y, its feet over (0, 0) to (2, 2)."""
    with m.step("Feet and body"):
        m.add("3023b", feet, at=(0, 0, 0))
        m.add("3023b", feet, at=(1, 0, 0))
        m.add("3003", body, at=(0, 0, 1))
    with m.step("Head"):
        m.add("3020", head, at=(-1, 0, 4), rot=1)
        m.add("30414", head, at=(-1, 0, 5), rot=3)
        m.add("3010", head, at=(-1, 1, 5), rot=1)
    with m.step("Eyes"):
        # On the middle two of the four side studs; a side stud's middle is 1.75 plates up the brick.
        for ex in (0, 1):
            m.add("98138p07", "White", at=(ex, -OUT, 5.5), rot=EYE)
        # The outer two smoothed over in the head's color.
        for ex in (-1, 2):
            m.add("3070b", head, at=(ex, -OUT, 5.5), rot=EYE)
    with m.step("Dome"):
        m.add("3020", head, at=(-1, 0, 8), rot=1)
        m.add("15068", head, at=(-1, 0, 9), rot=2)
        m.add("15068", head, at=(1, 0, 9), rot=0)


def build(
    m, x, y, z, facing=3, head="Reddish Brown", body="Tan", feet="Black", title="Enemy"
):
    """The enemy; returns the placed copy's Handle."""
    sub = m.submodel(title, _enemy, head=head, body=body, feet=feet)
    # The box's corner after turning the enemy about the middle of its feet, (x + 1, y + 1); its own front is -y.
    turn = (facing - 3) % 4
    (x0, y0, z0), (x1, y1, _) = sub.box
    corners = [(cx - 1, cy - 1) for cx in (x0, x1) for cy in (y0, y1)]
    for _ in range(turn):
        corners = [(-cy, cx) for cx, cy in corners]
    lo = (min(c[0] for c in corners) + 1, min(c[1] for c in corners) + 1)
    with m.step(title):
        return m.place(sub, at=(x + lo[0], y + lo[1], z + z0), rot=turn)


def example(m):
    """Two enemies on a strip of ground, one walking left."""
    from emmet_dsl import Plate

    with m.step("Ground"):
        m.row(Plate, "Bright Green", at=(0, 0, 0), length=10, width=4)
    m.use("brick_enemy", x=2, y=1, z=1)
    m.use("brick_enemy", x=6, y=1, z=1, facing=2, title="Second enemy")
