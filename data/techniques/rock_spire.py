# SPDX-License-Identifier: Apache-2.0
"""Rock spire: pointed rocks from tall 75° slopes, each with a small slope on its tip turned another way so the point
looks broken, stood side by side at different heights for a ragged ridge.

m.use("rock_spire", x, y, z, risers=(0, 1, 0), color="Dark Bluish Gray", cap="Light Bluish Gray", title="Spires")
stands one spire per entry of `risers` side by side along x from (x, y), on the studs at height z, each 1 stud wide
and 2 deep (y to y + 2), its steep face to the front (-y). Each entry is how many 1 x 2 bricks raise that spire (0 to
3), so neighbors stop at different heights. On each spire's single top stud goes a 1 x 1 cheese slope facing left,
right, or back, never the way the spire faces, so the tip reads as a broken point. Returns the z of the highest tip.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 21335 (sea stack rock spire). Not derived from any model file."""

PARTS = ["4460b", "54200", "3004"]

# The cheese slope faces right, left, then back, in turn along the ridge.
CAP_TURNS = (0, 2, 1)
CHEESE = 2  # a 1 x 1 cheese slope is 2 plates tall (1.95)


def build(
    m,
    x,
    y,
    z,
    risers=(0, 1, 0),
    color="Dark Bluish Gray",
    cap="Light Bluish Gray",
    title="Spires",
):
    """The spires; returns the z of the highest tip."""
    if not risers or any(r not in (0, 1, 2, 3) for r in risers):
        raise ValueError(
            f"rock_spire needs one or more risers, each 0 to 3 bricks, not {tuple(risers)}"
        )
    if len(risers) > 8:
        raise ValueError(f"rock_spire takes at most 8 spires, not {len(risers)}")
    top = z
    for k, r in enumerate(risers):
        xs = x + k
        with m.step(f"{title}, spire {k + 1}"):
            for i in range(r):
                m.add(Brick(1, 2), color, at=(xs, y, z + 3 * i))
            zs = z + 3 * r
            m.add("4460b", color, at=(xs, y, zs), rot=3)
            m.add("54200", cap, at=(xs, y + 1, zs + 9), rot=CAP_TURNS[k % 3])
            top = max(top, zs + 9 + CHEESE)
    return top


def example(m):
    """A ridge of four spires on a gray plate."""
    with m.step("Base"):
        m.add("3020", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("rock_spire", x=0, y=0, z=1, risers=(1, 0, 2, 0))
