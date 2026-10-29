# SPDX-License-Identifier: Apache-2.0
"""Antenna mast: a thin mast from a bar standing in a hollow stud, with side antennas on bar holders clipped to it,
for the comms array on a spaceship, a station roof, or a base.

m.use("antenna_mast", x, y, z, arms=2, dish=True, color="Light Bluish Gray", bar_color="Light Bluish Gray",
title="Antenna") stands a round brick 1 x 1 with a hollow stud (3062b) on the stud at (x, y, z), threads a 2 x 2 dish
(4740) over its stud when dish is set, and puts a bar 4L (30374) in the hollow stud, rising 8 plates above the brick.
On the bar go `arms` bar holders with a clip (11090), one above another from the top down, 2.5 plates apart, their
tubes pointing right and left in turn, each holding a bar 3L (87994) across as an antenna that reaches 4 studs out
from the mast. arms is 0 to 3. It brings its own steps and returns the z of the bar's tip (z + 11).
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3062b", "4740", "30374", "11090", "87994"]


def build(
    m,
    x,
    y,
    z,
    arms=2,
    dish=True,
    color="Light Bluish Gray",
    bar_color="Light Bluish Gray",
    title="Antenna",
):
    """The base, the mast bar, and the arms; returns the z of the bar's tip."""
    if not 0 <= arms <= 3:
        raise ValueError(f"antenna_mast takes 0 to 3 arms, not {arms}")
    cx, cy = x + 0.5, y + 0.5  # the mast's axis
    with m.step(f"{title}: base and mast"):
        m.add("3062b", color, at=(x, y, z))
        if dish:
            m.add("4740", color, at=(x - 0.5, y - 0.5, z + 3))
        # A bar 4L is 10 plates long: down to a plate above the brick's underside, up 8 plates past its top.
        m.add("30374", bar_color, at=(cx - 0.2, cy - 0.2, z + 1))
    for i in range(arms):
        zc = z + 10 - 2.5 * i  # the clip's middle
        side = 1 if i % 2 == 0 else -1
        # The holder's clip runs along its own Z, turned upright; its tube (its +Y) points along x, `side` way.
        turn = frame.rotation(0, -90 * side, 0)
        r = frame.to_ldraw_rotation(turn)
        p = frame.to_ldraw_point((cx, cy, zc))
        with m.step(f"{title}: arm {i + 1}"):
            m.add_raw("11090", color, [*p, *r[0], *r[1], *r[2]])
            # The antenna: 10 LDU into the tube, whose mouth is 1.5 studs out, and 3 studs long.
            start = cx + side * 1 if side > 0 else cx - 4
            m.add("87994", bar_color, at=(start, cy - 0.2, zc - 0.5), rot=Rot(y=90))
    return z + 11


def example(m):
    """A mast with a dish and two antennas, on a deck."""
    with m.step("Deck"):
        m.add("3022", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("antenna_mast", x=0, y=0, z=1)
