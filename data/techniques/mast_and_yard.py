# SPDX-License-Identifier: Apache-2.0
"""Mast and yard: a mast of round bricks with a yard across it held in two clips, and sails hung from the yard.

m.use("mast_and_yard", x, y, z, height=6, yard_at=4, color="Reddish Brown", sail="White", title="Mast") stands a mast of
Brick 2 x 2 Round (3941) on the deck at (x, y, z), its front-left corner there, `height` bricks tall. Over the
`yard_at`th brick a layer of two plates 1 x 1 with a clip (4081b), their clips toward the stern (-x), and a plate 1 x 2
beside them carries the mast on up; the clips hold a bar 6L (63965a) across the ship (along y) as the yard. Two flags 2
x 2 (2335), turned so their clips run along their top edge, hang from the yard one each side of the mast as the sails.
The mast is topped with a cone. Returns the z of the mast's top (under the cone), where a crow's nest goes.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["3941", "4081b", "3023b", "63965a", "2335", "3942c"]

ACROSS = Rot(x=90)  # the bar on its side, across the ship
# A flag with its clips along its top edge, the cloth hanging below and facing fore and aft.
HANGING = Rot(y=-90, z=90)
AXIS_UP = 0.75  # a clip plate's axis above its base, in plates
# A hanging flag, in plates; its clips' axis is 0.5 plates below its top.
FLAG_TALL = 6.75


def build(
    m, x, y, z, height=6, yard_at=4, color="Reddish Brown", sail="White", title="Mast"
):
    """The mast, the yard layer, the yard, the sails, and the cone; returns the z of the mast's top."""
    if height < 2 or not 1 <= yard_at < height:
        raise ValueError(
            f"mast_and_yard needs a height of 2 or more and yard_at from 1 to height - 1, not {height}, {yard_at}"
        )
    with m.step(f"{title}: lower mast"):
        m.stack("3941", color, at=(x, y, z), count=yard_at)
    zl = z + 3 * yard_at
    with m.step(f"{title}: yard clips"):
        for dy in (0, 1):
            m.add("4081b", color, at=(x - 0.9, y + dy, zl - 0.25), rot=2)
        m.add("3023b", color, at=(x + 1, y, zl))
    with m.step(f"{title}: upper mast"):
        m.stack("3941", color, at=(x, y, zl + 1), count=height - yard_at)
    top = zl + 1 + 3 * (height - yard_at)
    # The clips' axis, a stud out from their plates' centers.
    axis_x, axis_z = x - 0.5, zl + AXIS_UP
    with m.step(f"{title}: yard"):
        m.add("63965a", color, at=(axis_x - 0.3, y - 2, axis_z - 0.75), rot=ACROSS)
    with m.step(f"{title}: sails"):
        for sy in (y - 2, y + 2):
            m.add(
                "2335",
                sail,
                at=(axis_x - 0.33, sy, axis_z + 0.5 - FLAG_TALL),
                rot=HANGING,
            )
    with m.step(f"{title}: top"):
        m.add("3942c", color, at=(x, y, top))
    return top


def example(m):
    """A mast on a short deck."""
    with m.step("Deck"):
        m.add("3031", "Tan", at=(-1, -1, 0))  # Plate 4 x 4
    m.use("mast_and_yard", x=0, y=0, z=1)
