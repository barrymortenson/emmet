# SPDX-License-Identifier: Apache-2.0
"""Tree: a classic brick tree, a trunk of round bricks with layers of leaves, built apart and placed as a submodel.

m.use("tree", x, y, z, size="medium", trunk_color="Reddish Brown", leaves_color="Green", title="Tree") stands a tree
on the studs at height z, its trunk on the stud at (x, y): a 1 × 1 trunk covering x to x + 1 and y to y + 1, or, for
size="large", a 2 × 2 trunk covering x to x + 2 and y to y + 2. The leaves spread about 3.5 studs around the trunk,
so keep that much clear of walls. size: "small" (a round brick 1 x 1, two layers of Plant Leaves 4 x 3, and a plant
with three large leaves on top: about 10½ plates tall), "medium" (two round bricks and four layers: about 15½), or
"large" (two round bricks 2 x 2, jumper plates that bring a single stud to the middle, a round brick 1 x 1, and six
layers: about 22½). Each layer of leaves (2423) sits on the open stud of the one under it, turned a quarter turn further, so
they spiral around the trunk. It's a submodel (one file in the .mpd for each size and colors, however many trees use it) with its own steps,
"<title>, trunk" and "<title>, leaves", placed in a step titled "<title>"; it returns the placed copy's Handle,
whose .max[2] is the top of the tree.
"""

from emmet_dsl import frame

PARTS = ["3062b", "3941", "3794b", "2423", "6255"]

# Each plant part's body box in LDU, and how far its origin is above the bottom of the part, in plates: the origin is
# the stud (or anti-stud) the part hangs on, so the part goes on a stud by its origin.
BODIES = {
    "2423": ([-26.0, 0.0, -68.0, 26.0, 8.0, 10.0], 1),
    "6255": ([-46.061, -27.45, -33.194, 45.979, 16.0, 54.95], 2),
}

# (round bricks 2 x 2, round bricks 1 x 1, layers of leaves) by size.
SIZES = {"small": (0, 1, 2), "medium": (0, 2, 4), "large": (2, 1, 6)}


def _on_stud(m, part, color, cx, cy, z, rot=0):
    """Puts a plant part on the stud whose center is (cx, cy) and whose base is at z."""
    body, above = BODIES[part]
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), body))
    return m.add(part, color, at=(cx + lo[0], cy + lo[1], z + above + lo[2]), rot=rot)


def _tree(m, size, trunk_color, leaves_color, title):
    """The tree in its own frame, the trunk's corner at the origin."""
    thick, thin, layers = SIZES[size]
    c = 1.0 if thick else 0.5  # the trunk's middle
    z = 0
    with m.step(f"{title}, trunk"):
        if thick:
            m.stack("3941", trunk_color, at=(0, 0, z), count=thick)
            z += 3 * thick
            # Two jumpers along y over the 2 x 2's studs, and one across them: its stud is in the middle.
            m.add("3794b", trunk_color, at=(0, 0, z), rot=0)
            m.add("3794b", trunk_color, at=(1, 0, z), rot=0)
            m.add("3794b", trunk_color, at=(0, 0.5, z + 1), rot=1)
            z += 2
        m.stack("3062b", trunk_color, at=(c - 0.5, c - 0.5, z), count=thin)
        z += 3 * thin
    with m.step(f"{title}, leaves"):
        for k in range(layers):
            _on_stud(m, "2423", leaves_color, c, c, z, rot=k % 4)
            z += 1
        _on_stud(m, "6255", leaves_color, c, c, z)


def build(
    m,
    x,
    y,
    z,
    size="medium",
    trunk_color="Reddish Brown",
    leaves_color="Green",
    title="Tree",
):
    """The tree; returns the placed copy's Handle."""
    if size not in SIZES:
        raise ValueError(f"tree: size must be one of {', '.join(SIZES)}, not {size!r}")
    tree = m.submodel(
        "tree",
        _tree,
        size=size,
        trunk_color=trunk_color,
        leaves_color=leaves_color,
        title=title,
    )
    lo = tree.box[0]
    with m.step(title):
        return m.place(tree, at=(x + lo[0], y + lo[1], z + lo[2]))


def example(m):
    """A medium tree on a patch of grass."""
    with m.step("Grass"):
        m.add("3958", "Bright Green", at=(0, 0, 0))
    m.use("tree", x=2, y=2, z=1, size="medium")
