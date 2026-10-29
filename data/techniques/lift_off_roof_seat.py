# SPDX-License-Identifier: Apache-2.0
"""Lift-off roof seat: wall tops tiled smooth except a stud at each corner, and a frame the roof is built on, which
sits on those four studs and lifts off to show the room.

m.use("lift_off_roof_seat", x, y, z, length, depth, color="Light Bluish Gray", frame="Dark Bluish Gray",
title="Roof seat") tiles the tops of the walls of a length x depth rectangle from (x, y) at z, leaving a 1 x 1 plate at
each corner, and returns a submodel: a 1-wide frame of plates over the same rectangle that grips the four corner studs
only, in two layers whose corners belong to different sides, so the frame is one piece. Build the roof on the frame (in the submodel's function) and place it with m.place(seat, at=(x, y, z + 1)).
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 21335, set 60440. Not derived
from any model file."""

PARTS = [
    "3024",
    "3069b",
    "2431",
    "3710",
    "6636",
    "3666",
    "3460",
    "4162",
    "3023b",
    "3623",
    "3008",
    "3009",
    "3010",
    "3004",
]


def _ring(m, kind, color, x, y, z, length, depth, corners, sides_own=False):
    """A 1-wide ring of `kind` around a length x depth rectangle, the corners left out unless `corners`, which the
    front and back runs own (or the sides, with sides_own)."""
    x1, y1 = x + length - 1, y + depth - 1
    if sides_own:
        m.row(kind, color, at=(x + 1, y, z), length=length - 2)
        m.row(kind, color, at=(x + 1, y1, z), length=length - 2)
        m.row(kind, color, at=(x, y, z), length=depth, axis="y")
        m.row(kind, color, at=(x1, y, z), length=depth, axis="y")
        return
    lo = 0 if corners else 1
    m.row(kind, color, at=(x + lo, y, z), length=length - 2 * lo)
    m.row(kind, color, at=(x + lo, y1, z), length=length - 2 * lo)
    m.row(kind, color, at=(x, y + 1, z), length=depth - 2, axis="y")
    m.row(kind, color, at=(x1, y + 1, z), length=depth - 2, axis="y")


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    color="Light Bluish Gray",
    frame="Dark Bluish Gray",
    title="Roof seat",
):
    """The tiled wall tops; returns the frame as a submodel to build the roof on and place."""
    with m.step(f"{title}: tiles"):
        _ring(m, Tile, color, x, y, z, length, depth, corners=False)
        for cx in (x, x + length - 1):
            for cy in (y, y + depth - 1):
                m.add("3024", color, at=(cx, cy, z))

    def seat(s):
        # Two rings whose corners belong to different sides, so the second ties the first's runs together.
        with s.step("Frame"):
            _ring(s, Plate, frame, 0, 0, 0, length, depth, corners=True)
        with s.step("Frame, second layer"):
            _ring(s, Plate, frame, 0, 0, 1, length, depth, corners=True, sides_own=True)

    return m.submodel("Roof seat", seat)


def example(m):
    """Four walls with their tops tiled, and the roof's frame set on them."""
    m.wall(0, 0, 0, length=8, depth=6, rows=2, color="Tan")
    seat = m.use("lift_off_roof_seat", x=0, y=0, z=6, length=8, depth=6)
    with m.step("Roof on"):
        m.place(seat, at=(0, 0, 7))
