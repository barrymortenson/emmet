# SPDX-License-Identifier: Apache-2.0
"""Exhaust stack: a truck's upright exhaust pipe beside its cab, round bricks on a bracket with a cone on top.

m.use("exhaust_stack", x, y, z, rows=3, color="Light Bluish Gray", title="Exhaust") puts a round 1 x 1 plate at
(x, y, z) and `rows` round 1 x 1 bricks on it, capped with a cone 1 x 1 (4589) as the pipe's mouth, standing beside a
cab's back corner. Returns the z of the top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["6141", "3062b", "4589"]


def build(m, x, y, z, rows=3, color="Light Bluish Gray", title="Exhaust"):
    """The pipe; returns the z of its top."""
    if rows < 1 or rows > 6:
        raise ValueError(f"exhaust_stack takes 1 to 6 rows, not {rows}")
    with m.step(title):
        m.add("6141", color, at=(x, y, z))
        for r in range(rows):
            m.add("3062b", color, at=(x, y, z + 1 + 3 * r))
        m.add("4589", color, at=(x, y, z + 1 + 3 * rows))
    return z + 4 + 3 * rows


def example(m):
    """An exhaust behind a cab's corner, on the chassis."""
    with m.step("Chassis"):
        m.add("3031", "Blue", at=(0, 0, 0))
    m.use("exhaust_stack", x=3, y=0, z=1)
