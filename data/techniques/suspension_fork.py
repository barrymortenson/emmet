# SPDX-License-Identifier: Apache-2.0
"""Suspension fork: a display motorbike's front fork, two legs raked forward from the steering head, each an upper
beam, a shock absorber, and a lower beam on one line, with the front wheel on an axle through the lower ends.

m.use("suspension_fork", frame=frame, color="Black", shock_color="Yellow", axle_color="Light Bluish Gray", wheel=True,
title="Fork") takes the frame `motorbike_frame` returned. Each leg, outside the frame's side on the head axle, runs
down and forward on a 5-12-13 rake (about 22.6 degrees from upright): a 5-hole beam (32316) whose hole 2 is on the
head axle (hole 1 a stud above, for `handlebars`); a shock absorber 6.5L (73129) outside it from the beam's hole 3 down
5.5 studs, pinned at both ends (friction pins, 2780); and below it a 5-hole beam on the same line, from the shock's
lower end down to the front axle (an axle 10, 3737) at its hole 5. The lower beam's hole 2, 3 studs above the axle, is
free on the inside for a brake caliper (`disc_brake`). With wheel True the front wheel (88517c01) goes on the axle,
level with `swingarm_shock`'s rear wheel to within a plate. Both legs go on in each step, the right (-y) first. It
brings its own steps and returns a dict of "front_axle" (the axle's middle, in the center plane), "uppers" and
"lowers" (the legs' beams' Handles, the right leg's first), and "rake" (the run and rise a stud down the fork).
"""

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes. Not derived
from any model file."""

PARTS = ["32316", "73129", "2780", "3737", "88517c01"]

RAKE = (5 / 13, -12 / 13)  # a stud down the fork: run, rise (studs)
SHOCK = 5.5


def _along(point, k):
    """The point k studs down the fork from `point` (DSL: x and y studs, z plates)."""
    return (point[0] + k * RAKE[0], point[1], point[2] + 2.5 * k * RAKE[1])


def build(
    m,
    frame,
    color="Black",
    shock_color="Yellow",
    axle_color="Light Bluish Gray",
    wheel=True,
    title="Fork",
):
    """The legs, their shocks, the lower legs, and the front axle (with the wheel); returns where the axle is."""
    head, y = frame["head"], frame["y"]
    half = frame["width"] / 2 + 0.5  # the legs' planes, outside the frame's sides
    sides = [(-1, "-y"), (1, "+y")]  # (which side, the way out of the frame)
    uppers, lowers = [], []
    with m.step(f"{title}: legs"):
        for s, _ in sides:
            top = _along((head[0], y + s * half, head[2]), -1)
            uppers.append(
                m.beam("32316", color, at=top, run=4 * RAKE[0], rise=4 * RAKE[1])
            )
    with m.step(f"{title}: shocks"):
        for (s, out), upper in zip(sides, uppers):
            a = upper.hole(3).at
            at = (a[0], y + s * (half + 1), a[2])
            m.beam(
                "73129",
                shock_color,
                at=at,
                run=SHOCK * RAKE[0],
                rise=SHOCK * RAKE[1],
                holes=(2, 1),
            )
            m.insert("2780", "Black", upper.hole(3), out=out)
    with m.step(f"{title}: lower legs"):
        for (s, out), upper in zip(sides, uppers):
            at = _along(upper.hole(3).at, SHOCK)
            lower = m.beam("32316", color, at=at, run=4 * RAKE[0], rise=4 * RAKE[1])
            m.insert("2780", "Black", lower.hole(1), out=out)
            lowers.append(lower)
    end = lowers[0].hole(5).at
    with m.step(f"{title}: front wheel" if wheel else f"{title}: front axle"):
        if wheel:
            m.add("88517c01", "Black", at=(end[0], y, end[2]), rot=1, hole=3)
        m.insert("3737", axle_color, lowers[0].hole(5), out="+y", out_by=2 * half)
    return {
        "front_axle": (end[0], y, end[2]),
        "uppers": uppers,
        "lowers": lowers,
        "rake": RAKE,
    }


def example(m):
    """The fork on a frame, its front wheel on the table."""
    frame = m.use("motorbike_frame", x=0, y=6, z=27)
    m.use("suspension_fork", frame=frame)
