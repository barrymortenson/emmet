# SPDX-License-Identifier: Apache-2.0
"""Trailer hitch: a tow ball under the back of a vehicle and a socket on a trailer's drawbar, so the trailer swings.

m.use("trailer_hitch", x, y, z, color="Black", title="Hitch") puts a plate 2 x 2 with a tow ball (3731) whose 2 x 2
plate is at (x, y, z), under a vehicle's back, its ball out behind (+x), and a plate 2 x 2 with a tow ball socket
(3730) facing it, on the ball, which is the start of the trailer's drawbar: build the trailer on the socket plate's
studs. Returns the (x, y, z) of the socket plate's 2 x 2 footprint.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3731", "3730"]


# The socket plate faces the ball plate, its middle 70 LDU (3.5 studs) behind the ball plate's: the ball is 40 LDU
# out from its plate's middle and the socket 30 from its own (the connector data's positions).
BEHIND = 3.5


def build(m, x, y, z, color="Black", title="Hitch"):
    """The ball and the socket; returns where the socket plate's 2 x 2 is."""
    # Each plate's LDraw origin is the middle of its top face; +x in the model is LDraw -z.
    ox, oy, oz = 20 * (y + 1), -8 * (z + 1), -20 * (x + 1)
    with m.step(title):
        m.add_raw("3731", color, (ox, oy, oz, 1, 0, 0, 0, 1, 0, 0, 0, 1))
        m.add_raw(
            "3730", color, (ox, oy, oz - 20 * BEHIND, -1, 0, 0, 0, 1, 0, 0, 0, -1)
        )
    return (x + BEHIND, y, z)


def example(m):
    """A hitch under the back of a car's plate, which stands on a brick; the socket hangs on the ball."""
    with m.step("Car"):
        m.add("3003", "Red", at=(-2, 0, 0))
        m.add("3020", "Red", at=(-2, 0, 3), rot=1)
    m.use("trailer_hitch", x=0, y=0, z=2)
