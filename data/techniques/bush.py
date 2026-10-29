# SPDX-License-Identifier: Apache-2.0
"""Bush: a leafy shrub with no trunk, round 1 x 1 bricks with bamboo leaves on a round 2 x 2 plate, each turned to throw
its leaves outward, stacked in two greens, crowned with broad leaves.

m.use("bush", x, y, z, tiers=2, colors=("Green", "Lime"), title="Bush") stands a bush on the studs at height z over the
2 x 2 from (x, y): a round plate 2 x 2, on each of its four studs a Plant Brick 1 x 1 Round with 3 Bamboo Leaves (30176)
turned so its leaves point away from the middle, and for each tier more, four more on top of those, turned a quarter
further so the leaves fill the gaps, in the next of `colors`. On the top tier's front-left stud a plant with three large
leaves (6255) crowns it. The leaves spread about 2½ studs round the plate, so keep that much clear. tiers: 1 to 3.
Returns the z of the top.
"""

from emmet_dsl import frame
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["4032a", "30176", "6255"]

# Each plant part's body box in LDU, and how far its origin (the stud it hangs on) is above its bottom, in plates.
BODIES = {
    "30176": ([-10.0, 0.0, -38.011, 48.88, 24.0, 38.011], 3),
    "6255": ([-46.061, -27.45, -33.194, 45.979, 16.0, 54.95], 2),
}
# The four studs of the plate, from its front-left corner, and the turn that points a leaf brick there outward.
QUARTERS = (((0, 0), 2), ((1, 0), 3), ((1, 1), 0), ((0, 1), 1))


def _on_stud(m, part, color, cx, cy, z, rot=0):
    """Puts a plant part on the stud whose center is (cx, cy) and whose base is at z."""
    body, above = BODIES[part]
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), body))
    return m.add(part, color, at=(cx + lo[0], cy + lo[1], z + above + lo[2]), rot=rot)


def build(m, x, y, z, tiers=2, colors=("Green", "Lime"), title="Bush"):
    """The bush; returns the z of its top."""
    if tiers not in (1, 2, 3):
        raise ValueError(f"bush takes 1 to 3 tiers, not {tiers}")
    if not colors:
        raise ValueError("bush needs at least one color")
    with m.step(f"{title}, base"):
        m.add("4032a", colors[0], at=(x, y, z))
    zt = z + 1
    for t in range(tiers):
        color = colors[t % len(colors)]
        actions = []
        for (dx, dy), rot in QUARTERS:
            at = (x + dx + 0.5, y + dy + 0.5, zt, (rot + t) % 4)

            def leaf(at=at, color=color):
                cx, cy, zl, r = at
                _on_stud(m, "30176", color, cx, cy, zl, r)

            actions.append((1, leaf))
        in_steps(m, f"{title}, tier {t + 1}", actions)
        zt += 3
    with m.step(f"{title}, crown"):
        _on_stud(m, "6255", colors[0], x + 0.5, y + 0.5, zt)
    return zt + 2


def example(m):
    """A bush of two tiers on a patch of grass."""
    with m.step("Grass"):
        m.add("3958", "Green", at=(0, 0, 0))
    m.use("bush", x=2, y=2, z=1, tiers=2)
