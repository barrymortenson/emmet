# SPDX-License-Identifier: Apache-2.0
"""Chimney stack: a brick chimney 2 x 2 with embossed brickwork, a cap that steps out a stud all round, and a pot.

m.use("chimney_stack", x, y, z, rows=4, color="Medium Nougat", cap="Dark Bluish Gray", pot="Dark Orange",
title="Chimney") stacks `rows` courses of two 1 x 2 embossed bricks from (x, y, z), each course turned a quarter from
the one below so they bond, then a 4 x 4 plate a stud out all round, a 2 x 2 plate, a jumper, and a round brick on the
jumper's stud as the pot. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350. Not derived
from any model file."""

PARTS = ["98283", "3031", "3022", "87580", "3062b"]


def build(
    m,
    x,
    y,
    z,
    rows=4,
    color="Medium Nougat",
    cap="Dark Bluish Gray",
    pot="Dark Orange",
    title="Chimney",
):
    """The stack, its cap, and its pot; returns the z of the top."""
    for r in range(rows):
        with m.step(f"{title}, row {r + 1}"):
            for k in range(2):
                if r % 2:
                    m.add("98283", color, at=(x, y + k, z + 3 * r), rot=1)
                else:
                    m.add("98283", color, at=(x + k, y, z + 3 * r))
    top = z + 3 * rows
    with m.step(f"{title}: cap and pot"):
        m.add("3031", cap, at=(x - 1, y - 1, top))
        m.add("3022", cap, at=(x, y, top + 1))
        m.add("87580", cap, at=(x, y, top + 2))
        m.add("3062b", pot, at=(x + 0.5, y + 0.5, top + 3))
    return top + 6


def example(m):
    """A chimney four bricks tall."""
    m.use("chimney_stack", x=1, y=1, z=0)
