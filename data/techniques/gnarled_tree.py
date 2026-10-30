# SPDX-License-Identifier: Apache-2.0
"""Gnarled tree: an old, twisted tree on a flared root base, its trunk of round bricks stepping a stud aside at each
course, and two crossed limbs spreading low, curving down underneath where they leave the trunk, with leaves at their
ends; built apart and placed as a submodel.

m.use("gnarled_tree", x, y, z, height=3, seed=1, bark="Reddish Brown", leaves=("Dark Green", "Green"), title="Old tree")
stands a tree on the studs at height z (a baseplate is z = 0.5) over the 4 x 4 from (x, y): four Brick 2 x 2 Corner
Round (3063b) with their round out and a round brick 2 x 2 in the hole they leave, the flared roots, then `height` round
bricks 2 x 2, each but the first set a stud aside from the one below (which way, seed picks), so the trunk twists as it
rises. On it, two plates 2 x 8 crossed are the limbs, each with an inverted curved slope (24201) under it on both sides
of the trunk, so the limbs' undersides curve; on each limb's end, a Plant Leaves 6 x 5 (2417), and on the crossing a
round plate and four Plant Leaves 4 x 3 (2423) reaching out in a pinwheel. The leaves spread about 7 studs from the
trunk, so keep that clear. height: 2 to 5. It's a submodel; returns the placed copy's Handle, whose .max[2] is the top.
"""

import random

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10354. Not derived from any
model file."""

PARTS = ["3063b", "3941", "3034", "24201", "2417", "2423", "4032b"]

BODIES = {
    "2417": [-48.0, 0.0, -68.0, 48.0, 8.0, 48.0],
    "2423": [-26.0, 0.0, -68.0, 26.0, 8.0, 10.0],
}
# The roots: a corner-round brick in each quarter of the 4 x 4, its round to the outside.
ROOTS = [((0, 0), 2), ((2, 0), 3), ((2, 2), 0), ((0, 2), 1)]
# The crown's pinwheel: a leaf on each stud of the round plate, each reaching out a different way (+x at rot=0).
PINWHEEL = [((1, 0), 0), ((1, 1), 1), ((0, 1), 2), ((0, 0), 3)]


def _on_stud(m, part, color, sx, sy, z, rot=0):
    """A plant part on the stud whose corner is (sx, sy), its bottom at z, turned by rot."""
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), BODIES[part]))
    m.add(part, color, at=(sx + 0.5 + lo[0], sy + 0.5 + lo[1], z), rot=rot)


def _tree(m, height, seed, bark, leaves, title):
    """The tree in its own frame, its roots' corner at the origin."""
    rng = random.Random(seed)
    with m.step(f"{title}, roots"):
        # A corner-round brick bulges back right at rot=0, a quarter on with each turn; the four leave a round hole in
        # the middle, which a round brick fills.
        for (rx, ry), rot in ROOTS:
            m.add("3063b", bark, at=(rx, ry, 0), rot=rot)
        m.add("3941", bark, at=(1, 1, 0))
    tx, ty, z = 1, 1, 3
    with m.step(f"{title}, trunk"):
        for k in range(height):
            if k:
                # A stud aside, but never off the course below: it keeps two of its studs.
                dx, dy = rng.choice([(1, 0), (-1, 0), (0, 1), (0, -1)])
                tx, ty = max(0, min(2, tx + dx)), max(0, min(2, ty + dy))
            m.add("3941", bark, at=(tx, ty, z))
            z += 3
    with m.step(f"{title}, limbs"):
        m.add("3034", bark, at=(tx - 3, ty, z), rot=1)
        # An inverted curved slope is thick at -x and sweeps up thin toward +x at rot=0 (a quarter on with each turn):
        # each one hangs under the lower limb beside the trunk, thick against it, so the limb's underside curves into
        # it. They go on with the limb, while the way up under it is open.
        for (ux, uy), rot in (((tx - 2, ty), 2), ((tx + 2, ty), 0)):
            for k in range(2):
                m.add("24201", bark, at=(ux, uy + k, z - 2), rot=rot)
        m.add("3034", bark, at=(tx, ty - 3, z + 1))
    for name, ends in (
        ("limb ends", [((tx - 3, ty), 2), ((tx + 4, ty + 1), 0)]),
        ("limb ends, crossing", [((tx + 1, ty - 3), 3), ((tx, ty + 4), 1)]),
    ):
        with m.step(f"{title}, leaves, {name}"):
            for (sx, sy), rot in ends:
                _on_stud(
                    m,
                    "2417",
                    leaves[0],
                    sx,
                    sy,
                    z + (1 if "crossing" not in name else 2),
                    rot,
                )
    with m.step(f"{title}, crown"):
        m.add("4032b", bark, at=(tx, ty, z + 2))
        for (sx, sy), rot in PINWHEEL:
            _on_stud(m, "2423", leaves[-1], tx + sx, ty + sy, z + 3, rot)


def build(
    m,
    x,
    y,
    z,
    height=3,
    seed=1,
    bark="Reddish Brown",
    leaves=("Dark Green", "Green"),
    title="Old tree",
):
    """The tree; returns the placed copy's Handle."""
    if not 2 <= height <= 5:
        raise ValueError(f"gnarled_tree takes a height of 2 to 5 bricks, not {height}")
    tree = m.submodel(
        "gnarled_tree",
        _tree,
        height=height,
        seed=seed,
        bark=bark,
        leaves=tuple(leaves),
        title=title,
    )
    lo = tree.box[0]
    with m.step(title):
        return m.place(tree, at=(x + lo[0], y + lo[1], z + lo[2]))


def example(m):
    """An old tree three bricks tall on a baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("gnarled_tree", x=6, y=6, z=0.5, height=3, seed=2)
