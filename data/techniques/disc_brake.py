# SPDX-License-Identifier: Apache-2.0
"""Disc brake: a display motorbike's front brakes, a brake disc on the front axle inside each fork leg and a caliper
on the leg at the disc's rim.

m.use("disc_brake", fork=fork, disc_color="Flat Silver", caliper_color="Red", both=True, title="Brakes") takes what
`suspension_fork` returned. On each side (the right, -y, only when both is False) a brake disc 6 x 6 (65416) goes on
the front axle half a stud inside the leg, and a caliper, a Technic beam 3 x 0.5 (6632), on the leg's inside face:
its middle hole on a Technic pin 3/4 (32002) in the lower leg's hole 2, 3 studs above the axle, its length across the
fork at the disc's rim, so it reads as gripping the disc. The discs go on after the axle here; in a set's
instructions they go on as it goes in. It brings its own step and returns nothing.
"""

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes. Not derived
from any model file."""

PARTS = ["65416", "6632", "32002"]


def build(
    m, fork, disc_color="Flat Silver", caliper_color="Red", both=True, title="Brakes"
):
    """The discs and calipers, on one side or both."""
    x, y, z = fork["front_axle"]
    run, rise = fork["rake"]
    with m.step(title):
        for lower in fork["lowers"][: 2 if both else 1]:
            mount = lower.hole(2)
            s = -1 if mount.at[1] < y else 1  # which side
            inward = "+y" if s < 0 else "-y"
            leg = abs(mount.at[1] - y)  # the leg's middle, from the center plane
            # The caliper on the leg's inside face, across the fork's line (a quarter turn from it).
            at = (mount.at[0], y + s * (leg - 0.75), mount.at[2])
            m.beam("6632", caliper_color, at=at, run=-rise, rise=run, holes=(2, 3))
            m.insert("32002", "Black", mount, out=inward)
            m.add("65416", disc_color, at=(x, y + s * (leg - 1.5), z), rot=1, hole=1)


def example(m):
    """Front brakes on a fork."""
    frame = m.use("motorbike_frame", x=0, y=6, z=27)
    fork = m.use("suspension_fork", frame=frame)
    m.use("disc_brake", fork=fork)
