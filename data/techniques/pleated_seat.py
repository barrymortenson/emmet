# SPDX-License-Identifier: Apache-2.0
"""Pleated seat: a classic car's bucket or bench seat with rolled, pleated upholstery, curved slopes side by side for
the cushion and the same stood up on side studs for the backrest.

m.use("pleated_seat", x, y, z, width=4, color="Tan", title="Seat") sets a seat 4 studs deep on the studs at z, its
front-left corner at (x, y), facing -x, `width` studs wide along y: on a plate, a curved slope 2 x 1 (11477) on each
stud column, curving down to the front, and a tile behind them (the crease); at the back, two plates and bricks 1 x 4
with studs on the side (30414) facing forward, and on their side studs a plate 2 x `width` stood up with a curved slope
on each stud column, the backrest. width: 4 (a bucket seat) or 8 (a bench). It's a submodel, built once however many
seats use it. Returns the placed copy's Handle; its .max[2] is the top of the backrest (z + 8.5).
"""

from emmet_dsl import Plate, Rot, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11381. Not derived from any
model file."""

PARTS = [
    "3031",
    "3035",
    "11477",
    "2431",
    "4162",
    "3710",
    "3460",
    "30414",
    "3020",
    "3034",
]

# A plate or slope stood up, its studs or curve to -x, its length up.
UPRIGHT = Rot(y=270)


def _seat(m, width, color):
    """The seat in its own frame, its front-left corner at the origin, facing -x."""
    with m.step("Seat: cushion"):
        m.add(Plate(4, width), color, at=(0, 0, 0), rot=0)
        for k in range(width):
            m.add("11477", color, at=(0, k, 1), rot=2)  # curves down to the front
        m.add(Tile(1, width), color, at=(2, 0, 1), rot=0)
    with m.step("Seat: back"):
        m.add(Plate(1, width), color, at=(3, 0, 1), rot=0)
        m.add(Plate(1, width), color, at=(3, 0, 2), rot=0)
        for k in range(0, width, 4):
            m.add("30414", color, at=(3, k, 3), rot=2)  # its side studs to the front
    with m.step("Seat: backrest"):
        # The side studs are 1.75 plates up the bricks; the plate's lower row of anti-studs goes on them.
        m.add(Plate(2, width), color, at=(2.6, 0, 3.5), rot=UPRIGHT)
        for k in range(width):
            m.add("11477", color, at=(1.8, k, 3.5), rot=UPRIGHT)


def build(m, x, y, z, width=4, color="Tan", title="Seat"):
    """The seat, built apart and set on; returns the placed copy's Handle."""
    if width not in (4, 8):
        raise ValueError(f"pleated_seat takes a width of 4 or 8, not {width}")
    seat = m.submodel("pleated_seat", _seat, width=width, color=color)
    with m.step(title):
        return m.place(seat, at=(x, y, z))


def example(m):
    """Two bucket seats on a car's floor."""
    with m.step("Floor"):
        m.add("41539", "Dark Green", at=(0, 0, 0))
    m.use("pleated_seat", x=2, y=0, z=1, title="Driver's seat")
    m.use("pleated_seat", x=2, y=4, z=1, title="Passenger's seat")
