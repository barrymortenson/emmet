# SPDX-License-Identifier: Apache-2.0
"""Track curve: plastic train track curves (53400, radius 40 studs, 22.5° each), turning left or right.

m.use("track_curve", at=pose, heading=None, count=1, turn="left", color="Dark Bluish Gray", title="Track") lays `count` curves
from a pose, (x, y, z, heading) as track_straight_run takes and returns it: 4 curves make a quarter turn, 8 a half,
16 a full circle 80 studs across the center line. turn is "left" (counterclockwise seen from above) or "right".

It returns the pose where the curve ends. A loop is a run, 8 curves, an equal run back, and 8 more curves the same
way: it closes exactly, and validate checks every joint. It brings its own steps, titled "<title>: curve".
"""

import math

from emmet_dsl.core import in_steps
from emmet_dsl.techniques.track_straight_run import pose, put

PARTS = ["53400"]

RADIUS = 40  # the center line's radius, in studs
ANGLE = 22.5  # one curve's turn, in degrees


def _turn(x, y, cx, cy, deg):
    a = math.radians(deg)
    dx, dy = x - cx, y - cy
    return cx + dx * math.cos(a) - dy * math.sin(a), cy + dx * math.sin(
        a
    ) + dy * math.cos(a)


def build(
    m, at, heading=None, count=1, turn="left", color="Dark Bluish Gray", title="Track"
):
    if turn not in ("left", "right"):
        raise ValueError(f'turn is "left" or "right", not {turn!r}')
    x, y, z, h = pose(at, heading)
    d = 1 if turn == "left" else -1
    actions = []
    for _ in range(count):
        a = math.radians(h)
        cx, cy = x - d * RADIUS * math.sin(a), y + d * RADIUS * math.cos(a)
        mx, my = _turn(x, y, cx, cy, d * ANGLE / 2)
        # A curve's origin is its arc's middle; along its LDraw +X it turns right, so a left turn lays it backward.
        facing = h + d * ANGLE / 2 + (180 if d == 1 else 0)
        actions.append(
            (
                1,
                lambda mx=mx, my=my, facing=facing: put(
                    m, "53400", color, mx, my, z, facing
                ),
            )
        )
        x, y = _turn(x, y, cx, cy, d * ANGLE)
        h += d * ANGLE
    in_steps(m, f"{title}: curve", actions)
    return (x, y, z, h % 360)


def example(m):
    """A quarter turn to the left, then a quarter turn to the right."""
    p = m.use("track_curve", at=(0, 4, 0, 0), count=4, turn="left")
    m.use("track_curve", at=p, count=4, turn="right")
