# SPDX-License-Identifier: Apache-2.0
"""Tail lights: red lights in a vehicle's back face, on bricks with a stud on one side, with tiles flush between them.

m.use("tail_lights", x, y, z, width=4, color="Light Bluish Gray", lens="Trans-Red", title="Tail lights") lays a course
across a vehicle's back, `width` studs along y from (x, y, z), its face to the back (+x): a brick with a stud on one
side (87087) at each end with a red 1 x 1 tile on it, and between them bricks with a stud on one side carrying a
plain tile, so the whole back is smooth. Returns the z of the course's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["87087", "3070b"]

LENS = Rot(y=90)  # a tile on its back, its face to +x


def build(
    m,
    x,
    y,
    z,
    width=4,
    color="Light Bluish Gray",
    lens="Trans-Red",
    title="Tail lights",
):
    """The lights and the smooth face between them; returns the z of its top."""
    if width < 2:
        raise ValueError(f"tail_lights needs a width of 2 or more, not {width}")
    with m.step(title):
        for k in range(width):
            m.add("87087", color, at=(x, y + k, z), rot=0)
            m.add(
                "3070b",
                lens if k in (0, width - 1) else color,
                at=(x + 1, y + k, z + 0.5),
                rot=LENS,
            )
    return z + 3


def example(m):
    """The back of a 4-wide car body, on a plate."""
    with m.step("Base"):
        m.add("3031", "White", at=(0, 0, 0))
    m.use("tail_lights", x=3, y=0, z=1, color="White")
