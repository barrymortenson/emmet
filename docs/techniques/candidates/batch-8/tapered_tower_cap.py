# SPDX-License-Identifier: Apache-2.0
"""Tapered tower cap: a small round tower for a micro or mini castle, a shaft of round bricks 2 x 2 on a plinth
that slopes out at its foot, capped with a dome and a cone finial.

m.use("tapered_tower_cap", x, y, z, rows=3, color="Light Bluish Gray", trim="White", cap="Sand Green",
title="Turret") builds, on the studs at height z, a turret whose plinth covers x to x + 4 and y to y + 4: a plate
4 x 4 with a 1 x 1 cheese slope facing out on each of its eight edge studs between the corners (the taper), a shaft
of `rows` round bricks 2 x 2 in the middle, a round plate 2 x 2 in `trim` as a collar, a dome 2 x 2 (30367c), and a
cone 1 x 1 on the dome's stud. Returns the z of the finial's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = ["3031", "54200", "3941", "4032b", "30367c", "4589"]

# The quarter turn that points a slope down toward each side.
DOWN = {"+x": 0, "+y": 1, "-x": 2, "-y": 3}


def build(
    m,
    x,
    y,
    z,
    rows=3,
    color="Light Bluish Gray",
    trim="White",
    cap="Sand Green",
    title="Turret",
):
    """The plinth, the shaft, and the cap; returns the z of the finial's top."""
    with m.step(f"{title}: plinth"):
        m.add("3031", color, at=(x, y, z))
    with m.step(f"{title}: taper"):
        for k in (1, 2):
            m.add("54200", trim, at=(x + k, y, z + 1), rot=DOWN["-y"])
            m.add("54200", trim, at=(x + 3, y + k, z + 1), rot=DOWN["+x"])
            m.add("54200", trim, at=(x + k, y + 3, z + 1), rot=DOWN["+y"])
            m.add("54200", trim, at=(x, y + k, z + 1), rot=DOWN["-x"])
    with m.step(f"{title}: shaft"):
        m.stack("3941", color, at=(x + 1, y + 1, z + 1), count=rows)
    top = z + 1 + 3 * rows
    with m.step(f"{title}: cap"):
        m.add("4032b", trim, at=(x + 1, y + 1, top))
        m.add("30367c", cap, at=(x + 1, y + 1, top + 1))
        m.add("4589", cap, at=(x + 1.5, y + 1.5, top + 4))
    return top + 7


def example(m):
    """Two turrets on a strip of wall."""
    with m.step("Base"):
        m.add("3029", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("tapered_tower_cap", x=0, y=1, z=1)
    with m.step("Wall"):
        m.add("3010", "Light Bluish Gray", at=(4, 2, 1), rot=1)
        m.add("3010", "Light Bluish Gray", at=(4, 2, 4), rot=1)
    m.use("tapered_tower_cap", x=8, y=1, z=1, rows=2, title="Second turret")
