# SPDX-License-Identifier: Apache-2.0
"""Brick campfire: two crossed logs ringed by stones, with a brick of studs on four sides at the crossing carrying
flames on every side and a glowing cone on top.

m.use("brick_campfire", x, y, z, logs="Reddish Brown", stones="Dark Bluish Gray", core="Orange",
flames=("Trans-Orange", "Trans-Yellow"), tip="Trans-Red", title="Campfire") builds, on the studs at height z over (x,
y) to (x + 3, y + 3), two 1 x 3 plates crossed as logs (one along x on the middle row, one along y on top of it on the
middle column), a 1 x 1 round plate as a stone in each corner, and at the crossing a brick 1 x 1 with studs on four
sides (4733) whose side studs each carry a 1 x 1 cheese slope (54200) with its point up, in the flame colors in turn,
under a cone 1 x 1 (4589) as the tip. The flames reach 0.8 studs past the core each way. Returns the z of the tip's
top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72040. Not derived from any
model file."""

PARTS = ["3623", "6141", "4733", "54200", "4589"]

# A flame on each side of the core: its corner from the core's corner, and the turn that stands a cheese slope on
# that side stud, its bottom to the core and its point up.
FLAMES = (
    ((0, -0.8), Rot(y=-90, z=90)),
    ((1, 0), Rot(y=-90, z=180)),
    ((0, 1), Rot(y=-90, z=-90)),
    ((-0.8, 0), Rot(y=-90)),
)
SIDE_STUD = 0.5  # a flame's bottom above the core's bottom, so its anti-stud meets the side stud, in plates


def build(
    m,
    x,
    y,
    z,
    logs="Reddish Brown",
    stones="Dark Bluish Gray",
    core="Orange",
    flames=("Trans-Orange", "Trans-Yellow"),
    tip="Trans-Red",
    title="Campfire",
):
    """The logs, the stones, and the fire; returns the z of the tip's top."""
    with m.step(f"{title}: logs and stones"):
        m.add("3623", logs, at=(x, y + 1, z), rot=1)
        m.add("3623", logs, at=(x + 1, y, z + 1))
        for dx in (0, 2):
            for dy in (0, 2):
                m.add("6141", stones, at=(x + dx, y + dy, z))
    cx, cy = x + 1, y + 1
    with m.step(f"{title}: fire"):
        m.add("4733", core, at=(cx, cy, z + 2))
        for k, ((dx, dy), rot) in enumerate(FLAMES):
            m.add(
                "54200",
                flames[k % len(flames)],
                at=(cx + dx, cy + dy, z + 2 + SIDE_STUD),
                rot=rot,
            )
        m.add("4589", tip, at=(cx, cy, z + 5))
    return z + 8


def example(m):
    """A campfire on a patch of grass."""
    from emmet_dsl import Plate

    with m.step("Grass"):
        m.fill(Plate, "Green", at=(0, 0, 0), size=(5, 5))
    m.use("brick_campfire", x=1, y=1, z=1)
