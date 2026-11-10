# SPDX-License-Identifier: Apache-2.0
"""Kickstand: a display motorbike's or e-bike's side stand, a beam pinned to the left swingarm arm and swung down and
back so its foot is on the table beside the rear wheel, and the bike stands on its wheels and the stand, with no base.

m.use("kickstand", swingarm=swingarm, hole=4, color="Black", title="Kickstand") takes what `swingarm_shock` returned.
A friction pin (2780) through the left arm's hole `hole` (by default the fourth from the pivot, clear of the pivot's and
the shock's bushes) holds a beam in the plane outside the arm; the beam is the shortest that reaches the table (5 to 11
holes), swung down and back so its foot's round end is on the table, level with the tires' bottoms. The friction pin
holds it at that angle, as on a set's stand. It brings its own step and returns the leg's Handle.
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of motorbikes and Barry's notes that a
kickstand, not a base, stands the bike, and that it goes on the rear left swingarm arm. Not derived from any model
file."""

PARTS = ["2780", "32316", "32524", "40490", "32525"]

BEAMS = {5: "32316", 7: "32524", 9: "40490", 11: "32525"}


def build(m, swingarm, hole=4, color="Black", title="Kickstand"):
    """The pin and the leg; returns the leg's Handle."""
    left = swingarm["arms"][1]
    mount = left.hole(hole)
    x, y_arm, z = mount.at
    table = swingarm["rear_axle"][2] - 2.5 * swingarm["radius"]
    # Studs from the pin down to the foot's end hole, its round end on the table
    height = (z - table) / 2.5 - 0.5
    n = next((k for k in sorted(BEAMS) if k - 1 >= height + 0.5), None)
    if n is None:
        raise ValueError(
            f"kickstand: the table is {height:.1f} studs below the pin; no beam reaches it"
        )
    # Back, toward the rear wheel, outside its tire
    run = -math.sqrt((n - 1) ** 2 - height * height)
    with m.step(title):
        leg = m.beam(
            BEAMS[n], color, at=(x, y_arm + 1, z), run=run, rise=-height, holes=(1, n)
        )
        m.insert("2780", "Black", leg.hole(1), out="-y")  # through the leg into the arm
    return leg


def example(m):
    """A frame on its wheels, standing on its kickstand."""
    frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
    swingarm = m.use("swingarm_shock", frame=frame)
    m.use("suspension_fork", frame=frame, swingarm=swingarm)
    m.use("kickstand", swingarm=swingarm)
