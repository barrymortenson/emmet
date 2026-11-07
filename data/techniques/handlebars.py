# SPDX-License-Identifier: Apache-2.0
"""Handlebars: a display motorbike's bars, out from the top of each fork leg, with a grip on the end and a brake or
clutch lever in front of it.

m.use("handlebars", fork=fork, color="Black", grip_color="Black", lever_color="Light Bluish Gray", droop=20,
title="Handlebars") takes what `suspension_fork` returned. On each leg's top hole (hole 1, a stud above the head
axle) an axle 4 (3705) goes in from the inside face and sticks 3 studs out: a clip-on bar. On it, from the leg out: a
lever, a Technic beam 3 x 0.5 (6632) on its end axle hole, reaching 2 studs forward and `droop` degrees down; a stud
and a half of bare bar; and a grip, a Technic axle joiner inline (59443), 2 studs long, its inner half on the bar's
last stud and its outer half past the end. It brings its own
steps and returns the bar ends, (x, y, z) on the right then the left.
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of motorbikes. Not derived from any
model file."""

PARTS = ["3705", "6632", "59443"]


def build(
    m,
    fork,
    color="Black",
    grip_color="Black",
    lever_color="Light Bluish Gray",
    droop=20,
    title="Handlebars",
):
    """The bars, levers, and grips; returns the bar ends."""
    y = fork["front_axle"][1]
    a = math.radians(droop)
    ends = []
    with m.step(f"{title}: bars and levers"):
        for upper in fork["uppers"]:
            top = upper.hole(1)
            s = -1 if top.at[1] < y else 1
            outward = "-y" if s < 0 else "+y"
            m.insert("3705", color, top, out=outward, out_by=3)
            edge = top.at[1] + s * top.length / 2  # the leg's outside face
            m.beam(
                "6632",
                lever_color,
                at=(top.at[0], edge + s * 0.25, top.at[2]),
                run=2 * math.cos(a),
                rise=-2 * math.sin(a),
                holes=(1, 3),
            )
            ends.append((top.at[0], edge + s * 3, top.at[2]))
    with m.step(f"{title}: grips"):
        for x, end, z in ends:
            s = -1 if end < y else 1
            m.add("59443", grip_color, at=(x, end, z), rot=1, hole=1)
    return ends


def example(m):
    """Handlebars on a fork."""
    frame = m.use("motorbike_frame", x=0, y=8, z=27)
    fork = m.use("suspension_fork", frame=frame, wheel=False)
    m.use("handlebars", fork=fork)
