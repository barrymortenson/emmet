# SPDX-License-Identifier: Apache-2.0
"""Statuette group: a small bronze statue group for a micro plaza, statuettes side by side on a dark plinth with a
plaque on its front.

m.use("statuette_group", x, y, z, count=3, color="Pearl Gold", plinth="Dark Bluish Gray", plaque="Metallic Gold",
title="Statue") builds, on the studs at height z from (x, y), a step of plates (count + 2) x 3 studs in the plinth
color, and on its middle row a course of bricks with studs on one side facing front (11211 and 87087), with a plaque
tile on the middle side studs (a 1 x 2, or a 1 x 1 for an odd count) and 1 x 1 tiles in the plinth color on the others,
so the front is smooth. On the bricks' studs stand `count` statuettes (90398, 1 to 4) facing front. Returns the z of the
plinth's top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10272. Not derived from any
model file."""

PARTS = [
    "90398",
    "11211",
    "87087",
    "3069b",
    "3070b",
    "3021",
    "3022",
    "3623",
    "3710",
    "3666",
]

FACE = Rot(y=-90, z=90)  # a tile on its back, its face to -y, long along x
FIGURE = (0.98, 0.8)  # a statuette's footprint facing front, in studs
OUT = 0.4


def build(
    m,
    x,
    y,
    z,
    count=3,
    color="Pearl Gold",
    plinth="Dark Bluish Gray",
    plaque="Metallic Gold",
    title="Statue",
):
    """The step, the plinth with its plaque, and the statuettes; returns the z of the plinth's top."""
    if not 1 <= count <= 4:
        raise ValueError(f"statuette_group takes 1 to 4 statuettes, not {count}")
    with m.step(f"{title}: step"):
        m.fill(Plate, plinth, at=(x, y, z), size=(count + 2, 3))
    px, py, zp = x + 1, y + 1, z + 1
    # The plinth course: 1 x 2 bricks with two side studs, a 1 x 1 one for an odd count.
    bricks, at = [], px
    for n in [2] * (count // 2) + [1] * (count % 2):
        bricks.append((at, n))
        at += n
    with m.step(f"{title}: plinth"):
        for bx, n in bricks:
            m.add("11211" if n == 2 else "87087", plinth, at=(bx, py, zp), rot=3)
    # The plaque: a 1 x 2 tile on the middle two side studs (bridging two bricks for 4), a 1 x 1 for an odd count.
    mid = count // 2 - 1 if count % 2 == 0 else count // 2
    with m.step(f"{title}: plaque"):
        m.add(
            "3069b" if count % 2 == 0 else "3070b",
            plaque,
            at=(px + mid, py - OUT, zp + 0.5),
            rot=FACE,
        )
        for i in range(count):
            if not mid <= i < mid + (2 if count % 2 == 0 else 1):
                m.add("3070b", plinth, at=(px + i, py - OUT, zp + 0.5), rot=FACE)
    with m.step(f"{title}: statuettes"):
        for i in range(count):
            cx, cy = px + i + 0.5, py + 0.5
            m.add(
                "90398",
                color,
                at=(cx - FIGURE[0] / 2, cy - FIGURE[1] / 2, zp + 3),
                rot=3,
            )
    return zp + 3


def example(m):
    """A group of three on a paved square."""
    from emmet_dsl import Tile

    with m.step("Square"):
        m.add(Plate(6, 6), "Light Bluish Gray", at=(0, 0, 0))
    with m.step("Paving"):
        m.row(Tile, "Light Bluish Gray", at=(0, 0, 1), length=6)
        m.row(Tile, "Light Bluish Gray", at=(0, 4, 1), length=6)
        m.row(Tile, "Light Bluish Gray", at=(0, 5, 1), length=6)
    m.use("statuette_group", x=0, y=1, z=1, count=3)
