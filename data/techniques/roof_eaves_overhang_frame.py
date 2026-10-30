# SPDX-License-Identifier: Apache-2.0
"""Roof eaves overhang frame: a lift-off frame for a roof, a stud bigger than the walls all around so the roof's
eaves and gables overhang them, which sits on four corner studs of the tiled wall tops and lifts straight off.

m.use("roof_eaves_overhang_frame", x, y, z, length, depth, roof=None, color="Light Bluish Gray", frame="White",
title="Roof frame") tiles the tops of the walls of a length x depth rectangle from (x, y) at z, leaving a 1 x 1
plate at each corner, and returns a submodel: a frame of plates 2 wide, (length + 2) x (depth + 2), whose inner
row sits over the walls and whose outer row overhangs them by a stud, in two layers whose corners belong to
different sides, so the frame is one piece. It grips the four corner studs only. roof(s), if given, builds the roof
on the frame in the submodel's own frame, whose top is at z 2 (a gable_roof over the whole (length + 2) x
(depth + 2) from (0, 0, 2), say). Place it with m.place(sub, at=(x - 1, y - 1, z + 1)). lift_off_roof_seat is the
same seat with a 1-wide frame flush with the walls; this one carries the eaves (eaves_fascia can finish them).
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335. Not derived from any
model file."""

PARTS = [
    "3024",
    "3069b",
    "2431",
    "6636",
    "4162",
    "63864",
    "3710",
    "3666",
    "3460",
    "3795",
    "3832",
    "3034",
    "3020",
]


def _ring(m, color, length, depth, z, sides_own):
    """A 2-wide ring of plates around a length x depth rectangle from (0, 0); the front and back runs own the corners,
    or the sides with sides_own."""
    if sides_own:
        m.row(Plate, color, at=(0, 0, z), length=depth, axis="y", width=2)
        m.row(Plate, color, at=(length - 2, 0, z), length=depth, axis="y", width=2)
        m.row(Plate, color, at=(2, 0, z), length=length - 4, width=2)
        m.row(Plate, color, at=(2, depth - 2, z), length=length - 4, width=2)
    else:
        m.row(Plate, color, at=(0, 0, z), length=length, width=2)
        m.row(Plate, color, at=(0, depth - 2, z), length=length, width=2)
        m.row(Plate, color, at=(0, 2, z), length=depth - 4, axis="y", width=2)
        m.row(Plate, color, at=(length - 2, 2, z), length=depth - 4, axis="y", width=2)


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    roof=None,
    color="Light Bluish Gray",
    frame="White",
    title="Roof frame",
):
    """The tiled wall tops; returns the frame, with the roof on it, as a submodel to place."""
    if length < 4 or depth < 4:
        raise ValueError(
            f"roof_eaves_overhang_frame needs walls 4 x 4 or more, not {length} x {depth}"
        )
    x1, y1 = x + length - 1, y + depth - 1
    with m.step(f"{title}: tiles"):
        m.row(Tile, color, at=(x + 1, y, z), length=length - 2)
        m.row(Tile, color, at=(x + 1, y1, z), length=length - 2)
        m.row(Tile, color, at=(x, y + 1, z), length=depth - 2, axis="y")
        m.row(Tile, color, at=(x1, y + 1, z), length=depth - 2, axis="y")
    with m.step(f"{title}: corner studs"):
        for cx in (x, x1):
            for cy in (y, y1):
                m.add("3024", color, at=(cx, cy, z))

    def seat(s):
        with s.step("Frame"):
            _ring(s, frame, length + 2, depth + 2, 0, sides_own=False)
        with s.step("Frame, second layer"):
            _ring(s, frame, length + 2, depth + 2, 1, sides_own=True)
        if roof:
            roof(s)

    return m.submodel(title, seat)


def example(m):
    """Four walls 8 x 6 with their tops tiled, and on the frame a gable roof that overhangs them a stud all around."""
    m.wall(0, 0, 0, length=8, depth=6, rows=2, color="White")
    seat = m.use(
        "roof_eaves_overhang_frame",
        x=0,
        y=0,
        z=6,
        length=8,
        depth=6,
        roof=lambda s: s.use(
            "gable_roof", x=0, y=0, z=2, length=10, depth=8, color="Dark Red"
        ),
    )
    with m.step("Roof on"):
        m.place(seat, at=(-1, -1, 7))
