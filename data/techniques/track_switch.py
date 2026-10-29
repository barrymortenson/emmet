# SPDX-License-Identifier: Apache-2.0
"""Track switch: a 9V switch point (75541-f1 right, 75542-f1 left), which joins plastic track, with a straight way through
and a branch.

m.use("track_switch", at=pose, heading=None, side="right", color="Dark Bluish Gray", title="Track") lays one switch from a pose,
(x, y, z, heading) as track_straight_run takes and returns it. Its straight way is 32 studs long, as two straights;
its branch leaves to the `side` ("right" or "left" of the way it runs) at 22.5°.

It returns {"straight": pose, "branch": pose}, where each way ends. One track_curve turning back (left after a right
switch) from the branch end runs parallel to the straight way, 16 studs to its side: a siding. It brings its own
step, "<title>: switch". (The 2026-09 LDraw library has no plastic switch; the 9V one joins plastic track.)
"""

import math

from emmet_dsl.techniques.track_straight_run import ahead, pose, put

PROVENANCE = """Emmet's own (session 3B), written from general building knowledge; not derived from any model
file."""

PARTS = ["75541-f1", "75542-f1"]

LENGTH = 32  # the straight way, in studs; the origin is in its middle
# The branch's end from the origin, measured from the part: 16.69 studs ahead and 12.96 to the side.
BRANCH_AHEAD = 16.69267
BRANCH_SIDE = 12.95518
BRANCH_ANGLE = 22.5


def build(m, at, heading=None, side="right", color="Dark Bluish Gray", title="Track"):
    if side not in ("left", "right"):
        raise ValueError(f'side is "left" or "right", not {side!r}')
    x, y, z, h = pose(at, heading)
    ox, oy = ahead(x, y, h, LENGTH / 2)
    with m.step(f"{title}: switch"):
        put(m, "75541-f1" if side == "right" else "75542-f1", color, ox, oy, z, h)
    sx, sy = ahead(ox, oy, h, LENGTH / 2)
    s = -1 if side == "right" else 1
    a = math.radians(h)
    bx = ox + BRANCH_AHEAD * math.cos(a) - s * BRANCH_SIDE * math.sin(a)
    by = oy + BRANCH_AHEAD * math.sin(a) + s * BRANCH_SIDE * math.cos(a)
    return {
        "straight": (sx, sy, z, h),
        "branch": (bx, by, z, (h + s * BRANCH_ANGLE) % 360),
    }


def example(m):
    """A straight into a right switch, a straight on, and a siding off the branch."""
    p = m.use("track_straight_run", at=(0, 20, 0), heading=0, count=1)
    ways = m.use("track_switch", at=p, side="right")
    m.use("track_straight_run", at=ways["straight"], count=1)
    siding = m.use("track_curve", at=ways["branch"], count=1, turn="left")
    m.use("track_straight_run", at=siding, count=1)
