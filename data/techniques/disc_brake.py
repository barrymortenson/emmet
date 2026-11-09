# SPDX-License-Identifier: Apache-2.0
"""Disc brake: a display motorbike's front brakes, a brake disc on the front axle inside each shock and a caliper arm
from the axle to the disc's rim.

m.use("disc_brake", fork=fork, disc_color="Light Bluish Gray", caliper_color="Red", both=True, title="Brakes") takes
what `suspension_fork` returned. On each side (the right, -y, only when both is False) a brake disc 6 x 6 (65416) goes
on the front axle a stud and a half inside the shock, clear of the hub, and the caliper, a Technic beam 3 x 0.5 (6632),
on the axle between the shock and the disc by its end axle hole, reaching 2 studs up the fork so its far end sits over
the disc's face near the rim. The discs go on after the axle here; in a set's instructions they go on as it goes in. It
brings its own step and returns nothing.
"""

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes. Not derived
from any model file."""

PARTS = ["65416", "6632"]


def build(
    m,
    fork,
    disc_color="Light Bluish Gray",
    caliper_color="Red",
    both=True,
    title="Brakes",
):
    """The discs and calipers, on one side or both."""
    x, y, z = fork["front_axle"]
    run, rise = fork["rake"]
    half = fork["half"]
    with m.step(title):
        for s in (-1, 1)[: 2 if both else 1]:
            # The shock's inside face is half + 0.5 studs from the center: the caliper against it, the disc a stud in.
            m.beam(
                "6632",
                caliper_color,
                at=(x, y + s * (half + 0.25), z),
                run=-2 * run,
                rise=-2 * rise,
                holes=(1, 3),
            )
            m.add("65416", disc_color, at=(x, y + s * (half - 1), z), rot=1, hole=1)


def example(m):
    """Front brakes on a fork."""
    frame = m.use("motorbike_frame", x=0, y=6, z=27)
    fork = m.use("suspension_fork", frame=frame)
    m.use("disc_brake", fork=fork)
