# SPDX-License-Identifier: Apache-2.0
"""Pine tree: a conical fir of stacked tiers of slopes, each tier a plate with a ring of slopes facing out round a core
and corner slopes, each two studs narrower than the one below and held clear of it by the core, on a round trunk.

m.use("pine_tree", x, y, z, tiers=2, trunk=1, color="Green", trunk_color="Reddish Brown", title="Pine") stands a pine
on the studs at height z over the square from (x, y), 2 x tiers + 2 studs on a side: a trunk of `trunk` round bricks
2 x 2 in the middle, then the tiers, the widest first, each on a square plate its own width. A tier 8 wide is a 4 x 4
core of 2 x 2 bricks with two 2 x 2 slopes facing out on each side; 6 wide, a round 2 x 2 core in the trunk's color
with two 2 x 1 slopes a side; 4 wide, just the corners. Every tier has a 2 x 2 double-convex slope at each corner. The
core stands a brick taller than the slopes and carries the next tier's plate, so each tier's edge hangs clear over the
one below like a fir's boughs. A cone 2 x 2 x 2 on the top tier's studs is the tip. tiers: 1 to 3. Returns the z of the
tip's top.
"""

from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["3941", "3031", "3958", "41539", "3003", "3039", "3040b", "3045", "3942c"]

PLATES = {4: "3031", 6: "3958", 8: "41539"}  # Plate W x W
# The side slope of a tier W wide, and its width.
SIDES = {6: ("3040b", 1), 8: ("3039", 2)}


def _tier(m, w, tx, ty, tz, color, trunk_color):
    """A tier w studs wide on its plate from (tx, ty, tz), as in_steps actions."""
    actions = []

    def add(part, at, rot=0, c=color):
        actions.append((1, lambda: m.add(part, c, at=at, rot=rot)))

    add(PLATES[w], (tx, ty, tz))
    tz += 1
    # The core stands two bricks tall, a brick above the slopes, to carry the next tier's plate clear of them.
    for a in range(0, w - 4, 2):
        for b in range(0, w - 4, 2):
            for h in (0, 3):
                if w == 6:
                    add("3941", (tx + 2, ty + 2, tz + h), c=trunk_color)
                else:
                    add("3003", (tx + 2 + a, ty + 2 + b, tz + h))
    if w in SIDES:
        part, sw = SIDES[w]
        for s in range(0, w - 4, sw):
            add(part, (tx + 2 + s, ty, tz), 3)
            add(part, (tx + w - 2, ty + 2 + s, tz), 0)
            add(part, (tx + 2 + s, ty + w - 2, tz), 1)
            add(part, (tx, ty + 2 + s, tz), 2)
    for (cx, cy), rot in (
        ((0, 0), 2),
        ((w - 2, 0), 3),
        ((w - 2, w - 2), 0),
        ((0, w - 2), 1),
    ):
        add("3045", (tx + cx, ty + cy, tz), rot)
    return actions


def build(
    m,
    x,
    y,
    z,
    tiers=2,
    trunk=1,
    color="Green",
    trunk_color="Reddish Brown",
    title="Pine",
):
    """The tree; returns the z of its tip's top."""
    if tiers not in (1, 2, 3) or trunk not in (1, 2, 3):
        raise ValueError(
            f"pine_tree takes 1 to 3 tiers and a trunk of 1 to 3 bricks, not {tiers}, {trunk}"
        )
    width = 2 * tiers + 2
    mid = width // 2 - 1
    with m.step(f"{title}, trunk"):
        m.stack("3941", trunk_color, at=(x + mid, y + mid, z), count=trunk)
    zt = z + 3 * trunk
    for k, w in enumerate(range(width, 2, -2)):
        o = (width - w) // 2
        tier = _tier(m, w, x + o, y + o, zt, color, trunk_color)
        in_steps(m, f"{title}, tier {k + 1}", tier)
        zt += 7 if w > 4 else 4
    with m.step(f"{title}, tip"):
        m.add("3942c", color, at=(x + mid, y + mid, zt))
    return zt + 6


def example(m):
    """A pine of two tiers on a trunk two bricks tall, on a patch of grass."""
    with m.step("Grass"):
        m.add("3958", "Bright Green", at=(0, 0, 0))
    m.use("pine_tree", x=0, y=0, z=1, tiers=2, trunk=2)
