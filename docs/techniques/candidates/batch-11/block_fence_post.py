# SPDX-License-Identifier: Apache-2.0
"""Block fence post: thin fence posts with stub rails sticking out four ways, each centered on its block, the way a
game draws a fence.

m.use("block_fence_post", x, y, z, posts=3, axis="x", rail=True, post="Tan", tabs="Reddish Brown",
seat="Bright Green", title="Fence") sets `posts` posts in a row along `axis`, one a 2 x 2 block, the first on the
block whose front-left corner is (x, y), on the studs at height z. Each stands on a 2 x 2 jumper in the seat color
(match it to the ground), so it is centered on its block: two 1 x 1 bricks in the post color, each topped by a 1 x 1
round plate with tabs, whose tabs read as rail stubs four ways. With rail, plates in the post color link the post
tops in pairs (1 x 4) and, for an odd count, the last three (1 x 6). Returns the z of the post tops (under the rail).
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["87580", "3005", "33291", "3710", "3666"]

TABS = 0.1  # the round plate with tabs reaches a tenth of a stud past its stud each way


def build(
    m,
    x,
    y,
    z,
    posts=3,
    axis="x",
    rail=True,
    post="Tan",
    tabs="Reddish Brown",
    seat="Bright Green",
    title="Fence",
):
    """The posts, then the rail; returns the z of the post tops."""
    if posts < 1 or axis not in ("x", "y") or (rail and posts < 2):
        raise ValueError(
            f"block_fence_post needs 1 post or more (2 with a rail) and an axis of 'x' or 'y', not {posts}, {axis!r}"
        )
    dx, dy = (2, 0) if axis == "x" else (0, 2)
    for k in range(posts):
        bx, by = x + dx * k, y + dy * k
        with m.step(f"{title}, post {k + 1}"):
            m.add("87580", seat, at=(bx, by, z))
            for level in range(2):
                zl = z + 1 + 4 * level
                m.add("3005", post, at=(bx + 0.5, by + 0.5, zl))
                m.add("33291", tabs, at=(bx + 0.5 - TABS, by + 0.5 - TABS, zl + 3))
    top = z + 9
    if rail:
        with m.step(f"{title}, rail"):
            k = 0
            while k < posts:
                span = 3 if posts - k == 3 else 2
                part = "3666" if span == 3 else "3710"
                bx, by = x + dx * k + 0.5, y + dy * k + 0.5
                # A 1 x n plate's long side runs along y at rot 0.
                m.add(part, post, at=(bx, by, top), rot=1 if axis == "x" else 0)
                k += span
    return top


def example(m):
    """A fence of four posts along a strip of grass."""
    with m.step("Grass"):
        m.add("3034", "Bright Green", at=(0, 0, 0), rot=1)
    m.use("block_fence_post", x=0, y=0, z=1, posts=4)
