# SPDX-License-Identifier: Apache-2.0
"""Cab seat: a driver's seat with a steering wheel in front of it, on a cab's floor.

m.use("cab_seat", x, y, z, color="Black", title="Driver") puts a steering wheel holder 2 x 2 (30640) at (x, y, z), its
column leaning back (+x), with a steering wheel (30663) on it, and a seat 2 x 2 (4079) behind it at x + 2, facing
the front (-x). Returns nothing.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["30640", "30663", "4079"]


# The steering wheel on the holder's column, which leans 45° to the front: the holder's pin (the LDCad connector
# data) and the wheel's hole, as an LDraw offset from the holder's origin and the wheel's turn.
WHEEL_AT = (0, -19.799, 11.515)
WHEEL_TURN = (-1, 0, 0, 0, 0.70710678, -0.70710678, 0, -0.70710678, -0.70710678)


def build(m, x, y, z, color="Black", title="Driver"):
    """The steering wheel and the seat."""
    with m.step(title):
        m.add("30640", "Light Bluish Gray", at=(x, y, z), rot=0)
        # The holder's LDraw origin is 1 stud in from its corner each way and 3 plates up.
        ox, oy, oz = 20 * (y + 1), -8 * (z + 3), -20 * (x + 1)
        dx, dy, dz = WHEEL_AT
        m.add_raw("30663", "Black", (ox + dx, oy + dy, oz + dz, *WHEEL_TURN))
        m.add("4079", color, at=(x + 2, y, z), rot=0)


def example(m):
    """A seat and steering wheel on a cab floor."""
    with m.step("Floor"):
        m.add("3031", "Light Bluish Gray", at=(0, 0, 0))
    m.use("cab_seat", x=0, y=1, z=1)
