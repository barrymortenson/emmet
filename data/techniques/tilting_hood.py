# SPDX-License-Identifier: Apache-2.0
"""Tilting hood: a hood on hinge bricks, so the whole nose of a vehicle tips up to show the engine.

m.use("tilting_hood", x, y, z, width=4, color="Red", title="Hood") puts a hinge brick 1 x 2 base (3937) at each side
of a vehicle's nose, `width` studs along y from (x, y, z), their hinges at the back (+x side of the 1-stud base), with a
hinge 2 x 2 top (6134) closed on each, and a plate across the tops as the hood's base, which the hood builds on. It's
built closed; in the real model it tips up on the hinges. Returns the z of the hood plate's top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["3937", "6134", "3020", "3795", "3022"]

PLATES = {4: "3020", 6: "3795"}  # Plate 2 x n


def build(m, x, y, z, width=4, color="Red", title="Hood"):
    """The hinges and the hood's plate; returns the z of its top."""
    if width not in PLATES:
        raise ValueError(f"tilting_hood takes a width of 4 or 6, not {width}")
    with m.step(f"{title}: hinges"):
        for side in (y, y + width - 2):
            # The base and its top share an origin: the top closed on it, its fingers on the base's.
            ox, oy, oz = 20 * (side + 1), -8 * (z + 3), -20 * (x + 0.5)
            m.add_raw(
                "3937", "Light Bluish Gray", (ox, oy, oz, 1, 0, 0, 0, 1, 0, 0, 0, 1)
            )
            m.add_raw("6134", color, (ox, oy, oz, 1, 0, 0, 0, 1, 0, 0, 0, 1))
    with m.step(f"{title}: plate"):
        m.add(PLATES[width], color, at=(x - 1, y, z + 3), rot=0)
    return z + 4


def example(m):
    """A hood on a chassis plate."""
    with m.step("Chassis"):
        m.add("3031", "Light Bluish Gray", at=(-1, 0, 0))
    m.use("tilting_hood", x=1, y=0, z=1)
