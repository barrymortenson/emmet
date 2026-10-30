# SPDX-License-Identifier: Apache-2.0
"""Lit window panel: a sci-fi wall of tall pale-blue light panels between dark pillars, a gray rail along the top.

m.use("lit_window_panel", x, y, z, bays=3, frame="White", pillar="Black", glow="Trans-Light Blue",
rail="Light Bluish Gray", title="Light wall") builds a wall along x from (x, y), facing front (-y), on the studs at
height z: `bays` windows 1 x 2 x 3 (60593) in `frame` with their glass (60602) in `glow`, each between pillars of
three 1 x 2 bricks that stand a stud out in front of the glass, so the wall reads as lit panels set back between
ribs. It is 3 * bays + 1 studs long and 2 deep (y - 1 to y + 1), so stand it on a floor 2 studs deep. Over it, a
2-wide plate course ties the pillars and windows, and tiles finish it: `rail` along the front, `pillar` behind.
Returns the z of the top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75419. Not derived from any
model file."""

PARTS = [
    "60593",
    "60602",
    "3004",
    "3832",
    "3460",
    "3666",
    "3710",
    "3023b",
    "3020",
    "3795",
    "2431",
    "6636",
    "4162",
    "3069b",
    "63864",
]


def build(
    m,
    x,
    y,
    z,
    bays=3,
    frame="White",
    pillar="Black",
    glow="Trans-Light Blue",
    rail="Light Bluish Gray",
    title="Light wall",
):
    """The pillars, the windows, and the rail; returns the z of the top."""
    if not 1 <= bays <= 5:
        raise ValueError(f"lit_window_panel takes 1 to 5 bays, not {bays}")
    length = 3 * bays + 1
    for b in range(bays + 1):
        with m.step(f"{title}, pillar {b + 1}"):
            m.stack("3004", pillar, at=(x + 3 * b, y - 1, z), count=3)
        if b == bays:
            break
        with m.step(f"{title}, window {b + 1}"):
            f = m.add("60593", frame, at=(x + 3 * b + 1, y, z), rot=1)
            (x0, y0, z0), (x1, y1, _) = f.min, f.max
            m.add(
                "60602",
                glow,
                at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
                rot=1,
            )
    zt = z + 9
    with m.step(f"{title}, rail plates"):
        m.row(Plate, pillar, at=(x, y - 1, zt), length=length, width=2)
    with m.step(f"{title}, rail"):
        m.row(Tile, rail, at=(x, y - 1, zt + 1), length=length)
        m.row(Tile, pillar, at=(x, y, zt + 1), length=length, first=2)
    return zt + 2


def example(m):
    """Three lit bays on a strip of black floor."""
    with m.step("Floor"):
        m.row(Plate, "Black", at=(0, -1, 0), length=10, width=2)
    m.use("lit_window_panel", x=0, y=0, z=1)
