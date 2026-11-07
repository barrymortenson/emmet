# SPDX-License-Identifier: Apache-2.0
"""Swingarm with its shock: a display motorbike's rear swingarm on the frame's pivot axle, a shock absorber in the
middle from the swingarm up to the frame, and the rear wheel on its axle between the arms.

m.use("swingarm_shock", frame=frame, color="Black", shock_color="Yellow", axle_color="Light Bluish Gray", wheel=True,
title="Swingarm") takes the frame `motorbike_frame` returned. Two 11-hole beams (32525), one outside each side of the
frame, hang on the pivot axle and slope down to the rear. A shock absorber 6.5L (73129, its holes 5.5 studs apart)
in the center plane joins an axle 10 (3737) through the arms' hole 4 to an axle 4 (3705) through the frame's bottom
spars' hole 4: the swingarm's angle is the one that makes that 5.5 studs (about 24.5 degrees down), worked out from the
frame, so both of the shock's ends are on axles. An axle 10 through the arms' end holes carries the rear wheel (a
motorcycle wheel, 88517c01, 12.6 studs across) when wheel is True. It brings its own steps and returns a dict of
"rear_axle" (the rear axle's middle, in the center plane), "arm" (the right arm's Handle), and "angle" (degrees below
level).
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes. Not derived
from any model file."""

PARTS = ["32525", "73129", "3737", "3705", "88517c01"]

SHOCK = 5.5  # studs between the shock's holes
ARM_HOLE = 4  # the arm's hole the shock's lower axle goes through (3 studs from the pivot: clear of the wheel)
SPAR_HOLE = 4  # the bottom spar's hole its upper axle goes through
REACH = 10  # studs from the pivot to the rear axle (an 11-hole beam's end)


def _angle(pivot, anchor):
    """The arm's angle (radians, from +x toward +z) that puts its hole ARM_HOLE SHOCK studs from `anchor`, sloping down
    to the rear."""
    dx, dz = anchor[0] - pivot[0], (anchor[2] - pivot[2]) / 2.5
    r, toward = math.hypot(dx, dz), math.atan2(dz, dx)
    a = ARM_HOLE - 1
    c = (a * a + r * r - SHOCK * SHOCK) / (2 * a * r)
    if abs(c) > 1:
        raise ValueError(
            "swingarm_shock: the shock can't reach the frame's bottom spar from the swingarm"
        )
    turn = math.acos(c)
    down_back = [
        t for t in (toward + turn, toward - turn) if math.cos(t) < 0 and math.sin(t) < 0
    ]
    if not down_back:
        raise ValueError(
            "swingarm_shock: the shock would hold the swingarm up, not down to the rear"
        )
    return min(down_back, key=lambda t: abs(math.degrees(t) % 360 - 200))


def build(
    m,
    frame,
    color="Black",
    shock_color="Yellow",
    axle_color="Light Bluish Gray",
    wheel=True,
    title="Swingarm",
):
    """The arms, the shock, and the rear axle (with the wheel); returns where the rear axle is."""
    x, y, z = frame["pivot"]
    half = frame["width"] / 2 + 0.5  # the arms' planes, outside the frame's sides
    t = _angle(frame["pivot"], frame["bottom"].hole(SPAR_HOLE).at)
    run, rise = REACH * math.cos(t), REACH * math.sin(t)
    with m.step(f"{title}: arms"):
        arm = m.beam("32525", color, at=(x, y - half, z), run=run, rise=rise)
        m.beam("32525", color, at=(x, y + half, z), run=run, rise=rise)
    with m.step(f"{title}: shock"):
        low, high = arm.hole(ARM_HOLE).at, frame["bottom"].hole(SPAR_HOLE).at
        m.beam(
            "73129",
            shock_color,
            at=(low[0], y, low[2]),
            run=high[0] - low[0],
            rise=(high[2] - low[2]) / 2.5,
            holes=(1, 2),
        )
        m.insert("3737", axle_color, arm.hole(ARM_HOLE), out="+y", out_by=2 * half)
        m.insert(
            "3705", axle_color, frame["bottom"].hole(SPAR_HOLE), out="+y", out_by=3
        )
    end = arm.hole(REACH + 1).at
    with m.step(f"{title}: rear wheel" if wheel else f"{title}: rear axle"):
        if wheel:
            m.add("88517c01", "Black", at=(end[0], y, end[2]), rot=1, hole=3)
        m.insert("3737", axle_color, arm.hole(REACH + 1), out="+y", out_by=2 * half)
    return {
        "rear_axle": (end[0], y, end[2]),
        "arm": arm,
        "angle": -math.degrees(math.asin(math.sin(t))),
    }


def example(m):
    """The swingarm on a frame, its rear wheel on the table."""
    frame = m.use("motorbike_frame", x=10, y=6, z=27)
    m.use("swingarm_shock", frame=frame)
