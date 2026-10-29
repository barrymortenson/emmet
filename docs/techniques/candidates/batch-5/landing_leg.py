# SPDX-License-Identifier: Apache-2.0
"""Landing leg: a splayed leg under a ship's hull, a Technic beam pinned to a Technic brick under the hull and to a
foot that stays level, so the ship stands on its feet.

m.use("landing_leg", x, y, z, side="left", splay=25, length=5, color="Light Bluish Gray", leg_color="Dark Bluish
Gray", pin_color="Black", title="Leg") hangs a Technic brick 1 x 2 with a hole (3700) along x under the hull plate
whose underside is at z, at (x, y), its hole running along y. A Technic beam `length` holes long (32316, 32524, or
40490 for 5, 7, or 9) is pinned (2780) in front of it, on y - 1, and reaches down and out to the `side` ("left", -x,
or "right", +x), `splay` degrees from straight down (0 to 45). At its foot another Technic brick 1 x 2 with a hole,
kept level, is pinned to the beam's last hole, with a 2 x 2 round plate (4032a) under the brick and the beam's end as
its pad. The pads are the model's lowest parts, so it stands on them; give every leg of a ship the same splay and
length. It brings its own steps and returns the z of the pad's underside.
"""

import math

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3700", "2780", "32316", "32524", "40490", "4032a"]

BEAMS = {5: "32316", 7: "32524", 9: "40490"}


def _put(m, part, color, point, local, r_dsl):
    """Places a part turned by r_dsl (DSL axes) so the point `local` of its own frame (LDU) lands on `point` (DSL)."""
    r = frame.to_ldraw_rotation(r_dsl)
    p = frame.to_ldraw_point(point)
    pos = [p[i] - sum(r[i][k] * local[k] for k in range(3)) for i in range(3)]
    return m.add_raw(part, color, [*pos, *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    side="left",
    splay=25,
    length=5,
    color="Light Bluish Gray",
    leg_color="Dark Bluish Gray",
    pin_color="Black",
    title="Leg",
):
    """The mount, the leg, and the foot; returns the z of the pad's underside."""
    if side not in ("left", "right"):
        raise ValueError(f"landing_leg takes side='left' or 'right', not {side!r}")
    if length not in BEAMS:
        raise ValueError(f"landing_leg takes beams 5, 7, or 9 long, not {length}")
    if not 0 <= splay <= 45:
        raise ValueError(f"landing_leg takes a splay of 0 to 45 degrees, not {splay}")
    out = -1 if side == "left" else 1
    a = math.radians(splay)
    knee = (
        x + 1,
        y,
        z - 3 + 1.75,
    )  # the mount's hole, and the middle of the pin through it and the beam
    reach = length - 1
    ankle = (
        knee[0] + out * reach * math.sin(a),
        y,
        knee[2] - 2.5 * reach * math.cos(a),
    )
    # The beam's holes run along LDraw Y and its length along LDraw Z: a quarter turn about x, then one about y,
    # stands it up with its holes along y; less the splay about y tips its foot out.
    tip = frame.rotation(90, 90 - out * splay, 0)
    with m.step(f"{title}: mount"):
        m.add("3700", color, at=(x, y, z - 3), rot=1)
        _put(m, "2780", pin_color, knee, (0, 0, 0), frame.rotation())
    with m.step(f"{title}: beam"):
        _put(
            m,
            BEAMS[length],
            leg_color,
            (knee[0], y - 0.5, knee[2]),
            (0, 0, 10 * reach),
            tip,
        )
    # The foot goes together apart: its pin first, into the beam's last hole, then the brick and its pad on it.
    with m.step(f"{title}: foot"):
        _put(m, "2780", pin_color, ankle, (0, 0, 0), frame.rotation())
        m.add("3700", color, at=(ankle[0] - 1, y, ankle[2] - 1.75), rot=1)
        m.add("4032a", leg_color, at=(ankle[0] - 1, y - 1, ankle[2] - 2.75))
    return ankle[2] - 2.75


def example(m):
    """A small hull standing on two legs."""
    with m.step("Hull"):
        m.add("3020", "White", at=(0, 0, 10), rot=1)
        m.add("3020", "White", at=(0, 2, 10), rot=1)
        m.add("3020", "White", at=(0, 1, 11), rot=1)
    m.use("landing_leg", x=0, y=1, z=10, side="left", title="Left leg")
    m.use("landing_leg", x=2, y=1, z=10, side="right", title="Right leg")
