# SPDX-License-Identifier: Apache-2.0
"""Loft half floor: a sleeping loft over part of a room, a floor of two plate layers spanning between the end walls with
a lattice rail along its open edge.

m.use("loft_half_floor", x, y, z, length, depth, edge="-y", color="Reddish Brown", rail="Reddish Brown", title="Loft")
lays a floor over the rectangle whose front-left corner is (x, y), `length` studs along x and `depth` along y, at height
z, where its two end columns (x and x + length - 1) rest on the tops of the room's end walls. The first layer is plates
in strips along x, the second strips across them along y, so the seams cross and the floor is one piece. A rail of
lattice fences 1 x 4 x 1 (and round bricks where a fence won't fit) stands on the edge toward `edge` ("-y" or "+y"),
between the walls. Carry the end walls on up on the loft's end columns. Set it 5 to 6 bricks over the room's floor, over
a third to a half of the plan, so the room below shows from the open side. It brings its own steps, "<title>, first
layer", "<title>, second layer", and "<title>, rail", and returns the z of the loft's top, z + 2.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 43268. Not derived
from any model file."""

PARTS = ["3034", "3460", "3021", "3633", "3062b"]


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    edge="-y",
    color="Reddish Brown",
    rail="Reddish Brown",
    title="Loft",
):
    """The loft; returns the z of its top."""
    if edge not in ("-y", "+y"):
        raise ValueError(f'edge must be "-y" or "+y", not {edge!r}')
    with m.step(f"{title}, first layer"):
        for j in range(0, depth, 2):
            m.row(
                Plate, color, at=(x, y + j, z), length=length, width=min(2, depth - j)
            )
    with m.step(f"{title}, second layer"):
        for i in range(0, length, 2):
            m.row(
                Plate,
                color,
                at=(x + i, y, z + 1),
                length=depth,
                axis="y",
                width=min(2, length - i),
            )
    ry = y if edge == "-y" else y + depth - 1
    inner = length - 2
    with m.step(f"{title}, rail"):
        for k in range(inner // 4):
            m.add(
                "3633",
                rail,
                at=(x + 1 + 4 * k, ry, z + 2),
                rot=1,
                id=f"{title.lower()}/rail/{k + 1}",
            )
        for k in range(inner % 4):
            m.add(
                "3062b",
                rail,
                at=(x + 1 + inner // 4 * 4 + k, ry, z + 2),
                id=f"{title.lower()}/post/{k + 1}",
            )
    return z + 2


def example(m):
    """A room's two end walls with a loft over its back half."""
    from emmet_dsl import Brick

    with m.step("Floor"):
        m.add("3036", "Tan", at=(0, 0, 0), rot=1)
    for r in range(5):
        with m.step(f"Walls, row {r + 1}"):
            m.row(Brick, "White", at=(0, 0, 1 + 3 * r), length=6, axis="y")
            m.row(Brick, "White", at=(7, 0, 1 + 3 * r), length=6, axis="y")
    top = m.use("loft_half_floor", x=0, y=3, z=16, length=8, depth=3)
    with m.step("Walls, beside the loft"):
        for wx in (0, 7):
            m.add(Plate(1, 3), "White", at=(wx, 0, 16))
            m.add(Plate(1, 3), "White", at=(wx, 0, 17))
    with m.step("Walls, over the loft"):
        m.row(Brick, "White", at=(0, 0, top), length=6, axis="y")
        m.row(Brick, "White", at=(7, 0, top), length=6, axis="y")
