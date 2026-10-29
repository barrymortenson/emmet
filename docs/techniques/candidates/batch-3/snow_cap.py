# SPDX-License-Identifier: Apache-2.0
"""Snow cap: a mountain peak of stacked tiers of slopes facing out every way, gray rock below and white snow above,
with a ragged snowline where the two mix.

m.use("snow_cap", x, y, z, tiers=3, snowline=1, seed=1, rock="Dark Bluish Gray", snow="White", title="Peak") builds
a peak on the studs at height z over the square from (x, y), 2 x tiers + 2 studs on a side, each tier a brick tall and
two studs narrower than the one under it, sitting on its studs: a tier 8 wide is a 4 x 4 core of 2 x 2 bricks with two
2 x 2 slopes facing out on each side, 6 wide a 2 x 2 core with two 2 x 1 slopes a side, 4 wide just the corners, and
each has a 2 x 2 double-convex slope at its corners. Tiers from `snowline` up (counting the bottom tier as 0) are
snow; the tier just under the snowline is a mix, about half its slopes snow at random, so the snowline wanders. A
1 x 1 cheese slope of snow tops the peak. seed picks the mix. Returns the z of the top.
"""

import random

from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["3003", "3039", "3040b", "3045", "54200"]

# The side slope of a tier W wide, and its width.
SIDES = {6: ("3040b", 1), 8: ("3039", 2)}


def _tier(m, w, tx, ty, tz, pick, rock):
    """A tier w studs wide from (tx, ty, tz), as in_steps actions; pick() gives each slope's color."""
    actions = []

    def add(part, at, rot=0, c=None):
        c = c or pick()
        actions.append((1, lambda: m.add(part, c, at=at, rot=rot)))

    for a in range(0, w - 4, 2):
        for b in range(0, w - 4, 2):
            add("3003", (tx + 2 + a, ty + 2 + b, tz), c=rock)
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
    tiers=3,
    snowline=1,
    seed=1,
    rock="Dark Bluish Gray",
    snow="White",
    title="Peak",
):
    """The peak; returns the z of its top."""
    if tiers not in (1, 2, 3) or not 0 <= snowline <= tiers:
        raise ValueError(
            f"snow_cap takes 1 to 3 tiers and a snowline from 0 to the tiers, not {tiers}, {snowline}"
        )
    rng = random.Random(seed)
    width = 2 * tiers + 2
    zt = z
    for k, w in enumerate(range(width, 2, -2)):
        o = (width - w) // 2
        if k >= snowline:
            mix = 1.0
        elif k == snowline - 1:
            mix = 0.5
        else:
            mix = 0.0
        tier = _tier(
            m,
            w,
            x + o,
            y + o,
            zt,
            lambda mix=mix: snow if rng.random() < mix else rock,
            rock,
        )
        in_steps(m, f"{title}, tier {k + 1}", tier)
        zt += 3
    with m.step(f"{title}, summit"):
        m.add("54200", snow, at=(x + width // 2 - 1, y + width // 2 - 1, zt), rot=3)
    return zt + 2


def example(m):
    """A peak of three tiers, gray at the foot, snow on top, and a ragged snowline between."""
    with m.step("Ground"):
        m.add("41539", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("snow_cap", x=0, y=0, z=1, tiers=3, snowline=2)
