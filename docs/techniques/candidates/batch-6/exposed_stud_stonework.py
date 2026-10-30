# SPDX-License-Identifier: Apache-2.0
"""Exposed stud stonework: a big stone or concrete face with texture but no brick courses, made of panels on side
studs, some left as bare studs, some tiled smooth, some standing a plate proud.

m.use("exposed_stud_stonework", x, y, z, length, rows=2, color="Dark Tan", seed=1, title="Stonework") builds a wall
along x from (x, y, z), facing front (-y), `rows` bands tall, each band 5 plates (2 studs) of bricks with studs on one
side (22885, and 32952 for an odd stud). Each band's face is split at random (seed) into panels 2 to 4 studs wide,
and each panel is one of: a 2 x n plate standing on edge with its studs out (rough), a 2 x n tile (smooth), or a
plate with a tile on it (a plate proud, so its edges cast a shadow line). Panel joints move band to band. length is
2 or more. Returns the z of the top.
"""

import random

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75936. Not derived from any
model file."""

PARTS = ["22885", "32952", "3022", "3021", "3020", "3068b", "26603", "87079"]

PLATES = {2: "3022", 3: "3021", 4: "3020"}  # Plate 2 x n
TILES = {2: "3068b", 3: "26603", 4: "87079"}  # Tile 2 x n
# A 2 x n plate or tile on its back, its face to -y, long along x, 2 studs tall.
FACE = Rot(y=-90, z=90)
OUT = 0.4  # a plate's or tile's thickness, in studs


def build(m, x, y, z, length, rows=2, color="Dark Tan", seed=1, title="Stonework"):
    """The bands of side-stud bricks and their panels; returns the z of the top."""
    if length < 2 or rows < 1:
        raise ValueError(
            f"exposed_stud_stonework needs a length of 2 or more and 1 row or more, not {length}, {rows}"
        )
    rng = random.Random(seed)
    for r in range(rows):
        zr = z + 5 * r
        with m.step(f"{title}, band {r + 1}: bricks"):
            at = x
            for _ in range(length // 2):
                m.add("22885", color, at=(at, y, zr), rot=3)
                at += 2
            if length % 2:
                m.add("32952", color, at=(at, y, zr), rot=3)
        panels, left = [], length
        while left:
            n = rng.choice(
                [k for k in (2, 3, 4) if k <= left and left - k != 1] or [left]
            )
            panels.append(n)
            left -= n
        with m.step(f"{title}, band {r + 1}: panels"):
            at = x
            for n in panels:
                kind = rng.choice(("rough", "smooth", "proud"))
                if kind == "smooth":
                    m.add(TILES[n], color, at=(at, y - OUT, zr), rot=FACE)
                else:
                    m.add(PLATES[n], color, at=(at, y - OUT, zr), rot=FACE)
                    if kind == "proud":
                        m.add(TILES[n], color, at=(at, y - 2 * OUT, zr), rot=FACE)
                at += n
    return z + 5 * rows


def example(m):
    """A stone face 10 studs long and three bands tall, on a plate."""
    from emmet_dsl import Plate

    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, -1, 0), length=10, width=2, first=4)
    m.use("exposed_stud_stonework", x=0, y=0, z=1, length=10, rows=3)
