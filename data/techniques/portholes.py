# SPDX-License-Identifier: Apache-2.0
"""Portholes: a row of round portholes along a hull side, Technic bricks with a pin hole laid in one of its courses.

m.use("portholes", x, y, z, length, spacing=3, color="Dark Red", rim=None, title="Portholes") lays one course of a hull
side 1 stud thick and `length` studs long along x from (x, y), from z, with a Technic Brick 1 x 2 with a hole (3700)
every `spacing` studs (from the second stud, and 1 stud of side after the last), its hole through the side (along y),
which reads as a round porthole; plain bricks fill between. Each hole sits between the brick's two studs, so the
portholes land half a stud off the stud grid, centered over the joint below. rim gives the Technic bricks their own
color (black or tan, say), like a porthole's frame. Lay the courses below and above as usual. Returns the z of the
course's top.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3700",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
]


def build(m, x, y, z, length, spacing=3, color="Dark Red", rim=None, title="Portholes"):
    """The course with its Technic bricks; returns the z of the course's top."""
    if spacing < 3 or length < 4:
        raise ValueError(
            f"portholes needs a spacing of 3 or more and a length of 4 or more, not {spacing}, {length}"
        )
    holes = list(range(1, length - 2, spacing))
    with m.step(f"{title}: course"):
        u = 0
        for h in holes + [length]:
            if h > u:
                m.row(Brick, color, at=(x + u, y, z), length=h - u)
            if h < length:
                m.add("3700", rim or color, at=(x + h, y, z), rot=1)
            u = h + 2
    return z + 3


def example(m):
    """A hull side four courses tall with a row of portholes."""
    for r in range(2):
        with m.step(f"Side, row {r + 1}"):
            m.row(
                Brick,
                "Dark Red",
                at=(0, 0, 3 * r),
                length=10,
                first=2 if r % 2 else None,
            )
    m.use("portholes", x=0, y=0, z=6, length=10, rim="Black")
    with m.step("Side, row 4"):
        m.row(Brick, "Dark Red", at=(0, 0, 9), length=10, first=2)
