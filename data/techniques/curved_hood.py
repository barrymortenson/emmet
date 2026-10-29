# SPDX-License-Identifier: Apache-2.0
"""Curved hood: a vehicle's hood that rounds down to the front, curved slopes on a plate.

m.use("curved_hood", x, y, z, width=4, length=2, color="Red", title="Hood") covers a hood `width` studs along y and
`length` studs along x (back from x, the front) at z with curved slopes 2 x 2 (15068) whose low side faces the front,
over a plate; a width of 4 or 6 and a length of 2 or 4. Returns the z of the hood's top at its back.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["15068", "3022", "3020", "3021", "3795", "3031"]


def build(m, x, y, z, width=4, length=2, color="Red", title="Hood"):
    """The plate and the curved slopes; returns the z of the top."""
    if width not in (4, 6) or length not in (2, 4):
        raise ValueError(
            f"curved_hood takes a width of 4 or 6 and a length of 2 or 4, not {width} x {length}"
        )
    with m.step(f"{title}: plate"):
        m.fill(Plate, color, at=(x, y, z), size=(length, width))
    with m.step(f"{title}: slopes"):
        for a in range(0, length, 2):
            for b in range(0, width, 2):
                m.add("15068", color, at=(x + a, y + b, z + 1), rot=2)
    return z + 3


def example(m):
    """A hood 4 wide and 2 long."""
    m.use("curved_hood", x=0, y=0, z=0)
