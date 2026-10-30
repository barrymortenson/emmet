# SPDX-License-Identifier: Apache-2.0
"""Block tree: a blocky tree, a square trunk under a cube-ish canopy of mixed greens with see-through leaf blocks and a
ragged outline, built apart and placed as a submodel.

m.use("block_tree", x, y, z, trunk=4, seed=1, trunk_color="Reddish Brown", leaves=("Green", "Bright Green",
"Dark Green"), see_through=0.2, title="Tree") stands a tree on the studs at height z, its 2 x 2 trunk on the studs from
(x, y): `trunk` 2 x 2 bricks (3 to 5), then a 6 x 6 canopy centered on it, so keep 2 studs clear all around. The
canopy is a 6 x 6 plate, a course of nine 2 x 2 leaf blocks (a corner may be missing), a second course in which
edge blocks are left out or pushed a stud out past the edge, and a dark green cross on top. Each block takes one
of `leaves`, or, about `see_through` of the time, is two trans-green 1 x 2 bricks; all of it by seed, so no two
trees match. Every edge sits on the 2-stud grid but the pushed-out blocks, which break the outline the way a game
tree's leaves do. It's a submodel, "<title>, trunk" and "<title>, canopy", placed in a step titled "<title>"; it
returns the placed copy's Handle, whose .max[2] is the top of the tree.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3003", "3958", "3065", "2456"]


def _leaf(m, rng, leaves, see_through, at, along="x"):
    """One 2 x 2 leaf block at `at`: a brick in a green, or two trans-green 1 x 2 bricks running `along` x or y (the
    way a pushed-out block sticks out, so each brick keeps a stud on the course below)."""
    if rng.random() < see_through:
        for k in range(2):
            if along == "x":
                m.add("3065", "Trans Green", at=(at[0], at[1] + k, at[2]), rot=1)
            else:
                m.add("3065", "Trans Green", at=(at[0] + k, at[1], at[2]))
    else:
        m.add("3003", rng.choice(leaves), at=at)


def _tree(m, trunk, seed, trunk_color, leaves, see_through, title):
    """The tree in its own frame, the canopy's corner at the origin and the trunk at (2, 2)."""
    rng = random.Random(seed)
    with m.step(f"{title}, trunk"):
        m.stack("3003", trunk_color, at=(2, 2, 0), count=trunk)
    z = 3 * trunk
    cells = [(i, j) for j in range(3) for i in range(3)]
    corners = {(0, 0), (2, 0), (0, 2), (2, 2)}
    gone = rng.choice(sorted(corners))
    with m.step(f"{title}, canopy, plate"):
        m.add("3958", rng.choice(leaves), at=(0, 0, z))
    first = [c for c in cells if c != gone or rng.random() < 0.4]
    second = []
    for i, j in first:
        roll = rng.random()
        edge = (i, j) != (1, 1)
        if edge and roll < 0.3:
            continue  # left out
        push = (
            edge and roll < 0.55 and (i - 1) * (j - 1) == 0
        )  # an edge's middle block, a stud out
        along = "y" if push and i == 1 else "x"
        second.append((2 * i + (i - 1) * push, 2 * j + (j - 1) * push, along))
    course1 = [(2 * i, 2 * j, "x") for i, j in first]
    for course, blocks, zc in ((1, course1, z + 1), (2, second, z + 4)):
        for s in range(0, len(blocks), 4):
            with m.step(f"{title}, canopy, course {course}"):
                for bx, by, along in blocks[s : s + 4]:
                    _leaf(m, rng, leaves, see_through, (bx, by, zc), along)
    with m.step(f"{title}, canopy, top"):
        m.add("2456", "Dark Green", at=(0, 2, z + 7), rot=1)
        m.add("3003", "Dark Green", at=(2, 2, z + 10))


def build(
    m,
    x,
    y,
    z,
    trunk=4,
    seed=1,
    trunk_color="Reddish Brown",
    leaves=("Green", "Bright Green", "Dark Green"),
    see_through=0.2,
    title="Tree",
):
    """The tree; returns the placed copy's Handle."""
    if not 3 <= trunk <= 5:
        raise ValueError(f"block_tree takes a trunk of 3 to 5 bricks, not {trunk}")
    tree = m.submodel(
        "block_tree",
        _tree,
        trunk=trunk,
        seed=seed,
        trunk_color=trunk_color,
        leaves=tuple(leaves),
        see_through=see_through,
        title=title,
    )
    lo = tree.box[0]
    with m.step(title):
        # The trunk sits at (2, 2) in the tree's frame; the box starts wherever the outermost leaf block does.
        return m.place(tree, at=(x - 2 + lo[0], y - 2 + lo[1], z + lo[2]))


def example(m):
    """A tree on a patch of grass."""
    with m.step("Grass"):
        m.add("41539", "Bright Green", at=(0, 0, 0))
    m.use("block_tree", x=3, y=3, z=1, trunk=4, seed=5)
