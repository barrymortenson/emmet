# SPDX-License-Identifier: Apache-2.0
"""Downpipe: a rainwater pipe of round bricks running down beside a wall, tied to the wall top.

m.use("downpipe", x, y, z, rows, color="Dark Bluish Gray", title="Downpipe") stands a 1 x 1 round plate at (x, y, z)
and `rows` round 1 x 1 bricks on it, then a 1 x 2 plate across its top that reaches over the wall beside it (at
x + 1), which ties the pipe on. Put it against a wall corner, the wall `rows` bricks and a plate tall. Returns the z of
its top.
"""

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350. Not derived
from any model file."""

PARTS = ["6141", "3062b", "3023b", "3004", "3005", "3010", "3622"]


def build(m, x, y, z, rows, color="Dark Bluish Gray", title="Downpipe"):
    """The pipe and its tie; returns the z of the top."""
    with m.step(title):
        m.add("6141", color, at=(x, y, z))
        for r in range(rows):
            m.add("3062b", color, at=(x, y, z + 1 + 3 * r))
        m.add("3023b", color, at=(x, y, z + 1 + 3 * rows), rot=1)
    return z + 2 + 3 * rows


def example(m):
    """A downpipe at the end of a short wall."""
    from emmet_dsl import Brick, Plate

    with m.step("Wall"):
        m.row(Plate, "White", at=(1, 0, 0), length=4)
        for r in range(4):
            m.row(
                Brick,
                "White",
                at=(1, 0, 1 + 3 * r),
                length=4,
                first=None if r % 2 == 0 else 1,
            )
    m.use("downpipe", x=0, y=0, z=0, rows=4)
