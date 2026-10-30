# SPDX-License-Identifier: Apache-2.0
"""Headlights: a vehicle's lights in its front face, headlight bricks with round clear tiles on their front studs.

m.use("headlights", x, y, z, width=4, color="Light Bluish Gray", lens="Trans-Clear", title="Headlights") lays a course
across a vehicle's front, `width` studs along y from (x, y, z), its face to the front (-x): a headlight brick (4070)
at each end with a round 1 x 1 tile on its front stud, and bricks between. Returns the z of the course's top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["4070", "98138", "3004", "3005", "3622", "3010"]

LENS = Rot(y=-90)  # a tile on its back, its face to -x


def build(
    m,
    x,
    y,
    z,
    width=4,
    color="Light Bluish Gray",
    lens="Trans-Clear",
    title="Headlights",
):
    """The lights and the course between them; returns the z of its top."""
    if width < 2:
        raise ValueError(f"headlights needs a width of 2 or more, not {width}")
    with m.step(title):
        for side in (y, y + width - 1):
            m.add("4070", color, at=(x, side, z), rot=2)
            # The headlight brick's front stud is set back in its face, so the tile seats 0.2 out, not a plate's 0.4.
            m.add("98138", lens, at=(x - 0.2, side, z + 0.5), rot=LENS)
        if width > 2:
            m.row(Brick, color, at=(x, y + 1, z), length=width - 2, axis="y")
    return z + 3


def example(m):
    """The front of a 4-wide car body, on a plate."""
    with m.step("Base"):
        m.add("3031", "Red", at=(0, 0, 0))
    m.use("headlights", x=0, y=0, z=1, color="Red")
