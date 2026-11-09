# SPDX-License-Identifier: Apache-2.0
"""Kickstand: a display motorbike's or e-bike's side stand, a beam swung down from the frame's left side so its foot is
on the table beside the wheels, and the bike stands on its own, with no base.

m.use("kickstand", frame=frame, swingarm=swingarm, hole=None, color="Black", title="Kickstand") takes what
`motorbike_frame` and `swingarm_shock` returned. On the left side (+y), a Technic pin 3L (6558) through the bottom
spar's hole `hole` (by default the one a stud ahead of `chain_drive`'s crank) carries a beam in the frame's outer plane,
outside a fat tire; the beam is the shortest that reaches the table, swung down and back so its foot's round end is on
the table, level with the tires' bottoms (the swingarm's drop and the wheel's radius below the pivot). A friction pin
holds it at that angle, as on a set's stand. It brings its own step and returns the beam's Handle.
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of motorbikes and Barry's note that a
kickstand, not a base, stands the bike. Not derived from any model file."""

PARTS = ["6558", "32523", "32316", "32524", "40490", "32525", "41239", "32278"]

BEAMS = {
    3: "32523",
    5: "32316",
    7: "32524",
    9: "40490",
    11: "32525",
    13: "41239",
    15: "32278",
}
END = 9 / 20  # a beam's round end past its end hole's middle, in studs


def build(m, frame, swingarm, hole=None, color="Black", title="Kickstand"):
    """The pin and the leg; returns the leg's Handle."""
    y = frame["y"]
    if hole is None:
        hole = max(
            2, frame["lengths"][2] - 1
        )  # a stud ahead of the crank (`chain_drive`'s)
    mount = frame["bottom"].hole(hole)
    x, _, z = mount.at  # the left bottom spar's hole is there too
    plane = (
        y + frame["width"] / 2 - 0.5
    )  # the left top spar's plane, outside the frame's middle planes
    floor = frame["pivot"][2] - 2.5 * (swingarm["drop"] + swingarm["radius"])
    height = (z - floor) / 2.5 - END  # studs from the pin down to the foot's end hole
    reach = next((n - 1 for n in sorted(BEAMS) if n - 1 >= height + 0.5), None)
    if reach is None:
        raise ValueError(
            f"kickstand: the table is {height:.1f} studs below the pin; no beam reaches it"
        )
    run = -math.sqrt(
        reach * reach - height * height
    )  # back, toward the rear wheel, outside its tire
    with m.step(title):
        leg = m.beam(
            BEAMS[reach + 1],
            color,
            at=(x, plane, z),
            run=run,
            rise=-height,
            holes=(1, reach + 1),
        )
        m.insert(
            "6558", "Black", leg.hole(1), out="-y", out_by=2
        )  # through the leg and the left bottom spar
    return leg


def example(m):
    """A frame on its wheels, standing on its kickstand."""
    frame = m.use("motorbike_frame", x=10, y=6, z=27)
    swingarm = m.use("swingarm_shock", frame=frame)
    m.use("suspension_fork", frame=frame, drop=swingarm["drop"])
    m.use("kickstand", frame=frame, swingarm=swingarm)
