# SPDX-License-Identifier: Apache-2.0
"""Vintage radiator: the tall brass radiator of an early car or fire engine, a gold bracket faced with grille tiles, a
gold cap on top, and a brass lamp each side.

m.use("vintage_radiator", x, y, z, brass="Pearl Gold", core="Dark Bluish Gray", lamp_body="Black",
title="Radiator") stands a radiator 2 studs wide (y to y + 2) on the studs at z, its face to the front (-x) at x: a
bracket 1 x 2 - 2 x 2 up (99207) in brass, its 2 x 2 face covered by two 1 x 2 grille tiles as the core; behind the
face a 1 x 2 brick and plate to its height, a brass 1 x 2 plate over them and a round 1 x 1 plate as the filler cap;
and beside it, at y - 1 and y + 2, a brick 1 x 1 with a stud on the side (87087) facing the front with a round brass
tile on its stud, the lamps. The face is 5 plates tall. Returns the z of the cap's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302, set 43277. Not
derived from any model file."""

PARTS = ["99207", "2412b", "3004", "3023b", "6141", "87087", "98138"]

FACE = Rot(y=-90)  # a tile on its back, its face to -x, long along y


def build(
    m,
    x,
    y,
    z,
    brass="Pearl Gold",
    core="Dark Bluish Gray",
    lamp_body="Black",
    title="Radiator",
):
    """The shell and core, the cap, and the lamps; returns the z of the cap's top."""
    with m.step(f"{title}: shell and lamps"):
        m.add("99207", brass, at=(x - 0.2, y, z), rot=2)
        for side in (y - 1, y + 2):
            m.add("87087", lamp_body, at=(x, side, z), rot=2)
    with m.step(f"{title}: core and lamp faces"):
        for k in range(2):
            m.add("2412b", core, at=(x - 0.6, y, z + 2.5 * k), rot=FACE)
        for side in (y - 1, y + 2):
            m.add("98138", brass, at=(x - 0.4, side, z + 0.5), rot=FACE)
    with m.step(f"{title}: cap"):
        m.add("3004", brass, at=(x, y, z + 1), rot=0)
        m.add("3023b", brass, at=(x, y, z + 4), rot=0)
        m.add("3023b", brass, at=(x, y, z + 5), rot=0)
        m.add("6141", brass, at=(x, y, z + 6))
    return z + 7


def example(m):
    """The front of a vintage car: the radiator and its lamps on a 6-wide plate, two courses of hood behind."""
    with m.step("Front plate"):
        m.add("3032", "Black", at=(0, 0, 0), rot=0)
    m.use("vintage_radiator", x=0, y=2, z=1)
    for r in range(2):
        with m.step(f"Hood, row {r + 1}"):
            m.add("3001", "Dark Red", at=(1, 1, 1 + 3 * r), rot=0)
