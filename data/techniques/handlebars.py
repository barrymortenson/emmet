# SPDX-License-Identifier: Apache-2.0
"""Handlebars: a display motorbike's bars on top of the fork tubes, so they turn with the fork, each with a grip on
the end and a brake or clutch lever in front of it.

m.use("handlebars", fork=fork, color="Black", grip_color="Black", lever_color="Light Bluish Gray", droop=20,
title="Handlebars") takes what `suspension_fork` returned. On each fork tube's top, above the upper triple clamp, a
Technic cross block 1 x 2 (6536) turns the line to a pin hole across the bike a stud back of the tube, toward the
rider; an axle 4 (3705) goes in from the inside and sticks 3 studs out: a bar. On it, from the block out: a lever, a
Technic beam 3 x 0.5 (6632) on its end axle hole, reaching 2 studs forward and `droop` degrees down; a stud and a
half of bare bar; and a grip, a Technic axle joiner inline (59443), 2 studs long, its inner half on the bar's last
stud and its outer half past the end. It brings its own steps and returns the bar ends, (x, y, z) on the right then
the left.
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of motorbikes. Not derived from any
model file."""

PARTS = ["6536", "3705", "6632", "59443"]


def build(
    m,
    fork,
    color="Black",
    grip_color="Black",
    lever_color="Light Bluish Gray",
    droop=20,
    title="Handlebars",
):
    """The bar mounts, bars, levers, and grips; returns the bar ends."""
    y = fork["front_axle"][1]
    (rx, rz), (fx, fz) = fork["rake"], fork["forward"]
    a = math.radians(droop)
    ends = []
    with m.step(f"{title}: bars and levers"):
        for tube in fork["tubes"]:
            mid = tube.hole(1)
            s = -1 if mid.at[1] < y else 1
            # The tube's top stud: its middle is 4.5 studs up the steering axis from the tube's middle.
            k = mid.length / 2 - 0.5
            top = (mid.at[0] - k * rx, mid.at[1], mid.at[2] - 2.5 * k * rz)
            back = (top[0] - fx, top[1], top[2] - 2.5 * fz)
            block = m.beam(
                "6536", color, at=top, to=back, axis=(rx, 0.0, rz), holes=(2, 1)
            )
            bar = block.hole(1)
            m.insert("3705", color, bar, out="-y" if s < 0 else "+y", out_by=3)
            edge = bar.at[1] + s * bar.length / 2  # the block's outside face
            lever = (bar.at[0], edge + s * 0.25, bar.at[2])
            m.beam(
                "6632",
                lever_color,
                at=lever,
                run=2 * math.cos(a),
                rise=-2 * math.sin(a),
                holes=(1, 3),
            )
            ends.append((bar.at[0], edge + s * 3, bar.at[2]))
    with m.step(f"{title}: grips"):
        for x, end, z in ends:
            m.add("59443", grip_color, at=(x, end, z), rot=1, hole=1)
    return ends


def example(m):
    """Handlebars on a fork."""
    frame = m.use("motorbike_frame", x=0, y=8, z=27)
    fork = m.use("suspension_fork", frame=frame, wheel=False)
    m.use("handlebars", fork=fork)
