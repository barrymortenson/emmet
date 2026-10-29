# SPDX-License-Identifier: Apache-2.0
"""Track straight run: plastic train track straights end to end (53401, 16 studs each).

m.use("track_straight_run", at=(x, y, z), heading=0, count=1, color="Dark Bluish Gray", title="Track") lays `count`
straights from a pose: (x, y) the track's center line where the run starts, in studs; z the track's bottom, in plates
(0 on the table, 0.5 on a baseplate); heading the way it runs, in degrees counterclockwise from +x seen from above
(0 runs along +x, 90 along +y; 0 when left out). `at` may also be a whole pose, (x, y, z, heading), as every track technique
returns; then heading is ignored.

It returns the pose where the run ends, which the next piece's `at` takes as it is: another run, a track_curve, or a
track_switch. Track is 8 studs wide (4 each side of the center line) and 3 plates tall, and stands on the table or a
flat surface; its ends interlock, and validate checks each joint. It brings its own steps, at most the step target
each, titled "<title>: straight".
"""

import math

from emmet_dsl import frame
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3B), written from general building knowledge; not derived from any model
file."""

PARTS = ["53401"]

STRAIGHT = 16  # a straight's length, in studs


def pose(at, heading=None):
    """(x, y, z, heading) from at=(x, y, z) and a heading, or from a whole pose."""
    if len(at) == 4:
        return tuple(float(v) for v in at)
    return (*(float(v) for v in at), float(heading or 0))


def ahead(x, y, heading, d):
    """The point d studs ahead of (x, y) along heading."""
    a = math.radians(heading)
    return x + d * math.cos(a), y + d * math.sin(a)


def put(m, part, color, x, y, z, heading, id=None):
    """A track part by its origin (its center line, 1 plate above its bottom) and the way its LDraw +X runs."""
    r = frame.to_ldraw_rotation(frame.rotation(z=heading - 90))
    pos = frame.to_ldraw_point((x, y, z + 1))
    return m.add_raw(part, color, [*pos, *r[0], *r[1], *r[2]], id=id)


def build(m, at, heading=None, count=1, color="Dark Bluish Gray", title="Track"):
    x, y, z, h = pose(at, heading)
    actions = []
    for _ in range(count):
        cx, cy = ahead(x, y, h, STRAIGHT / 2)
        actions.append((1, lambda cx=cx, cy=cy: put(m, "53401", color, cx, cy, z, h)))
        x, y = ahead(x, y, h, STRAIGHT)
    in_steps(m, f"{title}: straight", actions)
    return (x, y, z, h)


def example(m):
    """Three straights on the table."""
    m.use("track_straight_run", at=(0, 4, 0), heading=0, count=3)
