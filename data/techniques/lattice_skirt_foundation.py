# SPDX-License-Identifier: Apache-2.0
"""Lattice skirt foundation: a house or porch raised a brick above the garden on beams, the gap along the front
closed with lattice panels like a real porch skirt.

m.use("lattice_skirt_foundation", x, y, z, length, depth=4, color="White", lattice="White", floor="Tan",
title="Foundation") builds a raised floor `length` x `depth` studs from (x, y, z), its front (-y) along x: a 1 x 1
brick post at each front corner with 1 x 4 lattice panels (3633) between them (and a short brick to make up the
length), a brick beam along each side and the back, and a cross beam under the middle when the floor is long. The
floor is plates over it all, a brick and a plate above z. Steps or stairs climb to it, and the walls stand on it.
Returns the z of the floor's top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43268 and set 11387. Not
derived from any model file."""

PARTS = [
    "3633",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "3034",
    "3832",
    "3795",
    "3020",
    "3710",
    "3666",
]


def build(
    m,
    x,
    y,
    z,
    length,
    depth=4,
    color="White",
    lattice="White",
    floor="Tan",
    title="Foundation",
):
    """The skirt, the beams, and the floor; returns the z of the floor's top."""
    if length < 6 or depth < 3:
        raise ValueError(
            f"lattice_skirt_foundation needs a length of 6 or more and a depth of 3 or more, not {length}, {depth}"
        )
    panels, extra = divmod(length - 2, 4)
    with m.step(f"{title}: skirt"):
        m.add("3005", color, at=(x, y, z))
        at = x + 1
        for _ in range(panels):
            m.add("3633", lattice, at=(at, y, z), rot=1)
            at += 4
        if extra:
            m.row(Brick, color, at=(at, y, z), length=extra)
        m.add("3005", color, at=(x + length - 1, y, z))
    with m.step(f"{title}: beams"):
        for bx in (x, x + length - 1):
            m.row(Brick, color, at=(bx, y + 1, z), length=depth - 2, axis="y")
        m.row(Brick, color, at=(x, y + depth - 1, z), length=length)
        if length >= 10:
            m.row(
                Brick, color, at=(x + length // 2, y + 1, z), length=depth - 2, axis="y"
            )
    with m.step(f"{title}: floor"):
        m.fill(Plate, floor, at=(x, y, z + 3), size=(length, depth))
    return z + 4


def example(m):
    """A porch floor 10 studs long on a lattice skirt, on a patch of lawn."""
    with m.step("Lawn"):
        m.fill(Plate, "Green", at=(0, -2, 0), size=(10, 6))
    m.use("lattice_skirt_foundation", x=0, y=0, z=1, length=10, depth=4)
