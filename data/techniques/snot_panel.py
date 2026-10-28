# SPDX-License-Identifier: Apache-2.0
"""SNOT panel: a stretch of wall faced with tiles on their sides, held by bricks with a stud on one side.

m.use("snot_panel", x, y, z, length, height=1, facing="front", color="Light Bluish Gray", face_color="White",
title="Panel") builds a wall one stud thick and `length` studs long from (x, y), starting at z (on the ground or a
wall below), and covers its outer face with smooth tiles standing on edge: `height` rows of tiles, each 2 studs tall
(5 plates). Each row is two courses of plates and a course of bricks, in which a brick 1 x 1 with a stud on one side
(87087) sits behind each end of each tile, its stud pointing out; the tiles (2 x 4, 2 x 2, and a 1 x 2 for an odd
length) hang on those studs, their tops flush with the bricks'. facing is the side the tiles face, which also sets the
wall's direction: "front" (−y) and "back" (+y) walls run along x, "left" (−x) and "right" (+x) ones along y. Build on
it at the z it returns; the tiles stand out 0.4 studs (one plate) from the wall. It brings its own steps, a course at
a time, titled "<title>, row 1: plates" and so on, then the row's tiles, and returns the z of the panel's top.
"""

from emmet_dsl import Brick, Plate, Rot

PARTS = [
    "87087",
    "87079",
    "3068b",
    "3069b",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "2465",
]

TILES = {4: "87079", 2: "3068b", 1: "3069b"}
# By facing: the way the wall runs, the quarter turn that points a SNOT brick's side stud out, and the turns that
# stand a tile on edge, its top out: one whose long side runs along the wall (2 x 4, and 2 x 2), and a 1 x 2 whose
# long side stands up.
FACING = {
    "front": ("x", 3, Rot(y=-90, z=90), Rot(x=90)),
    "back": ("x", 1, Rot(y=90, z=90), Rot(x=-90)),
    "left": ("y", 2, Rot(y=-90), Rot(x=-90, z=90)),
    "right": ("y", 0, Rot(y=90), Rot(x=90, z=90)),
}
OUT = 0.4  # a tile's thickness on its side, in studs


def _lengths(n, sizes):
    out = []
    for size in sorted(sizes, reverse=True):
        while n >= size:
            out.append(size)
            n -= size
    return out


def build(
    m,
    x,
    y,
    z,
    length,
    height=1,
    facing="front",
    color="Light Bluish Gray",
    face_color="White",
    title="Panel",
):
    """The wall and its tiles; returns the z of the panel's top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if length < 1 or height < 1:
        raise ValueError(
            f"snot_panel needs a length and height of 1 or more, not {length}, {height}"
        )
    axis, turn, long_rot, upright_rot = FACING[facing]

    def cell(a):
        return (x + a, y) if axis == "x" else (x, y + a)

    def tile_at(a, zr):
        """Where a tile starting `a` studs along the wall goes: in front of the wall's outer face."""
        cx, cy = cell(a)
        if facing == "front":
            cy -= OUT
        elif facing == "back":
            cy += 1
        elif facing == "left":
            cx -= OUT
        else:
            cx += 1
        return (cx, cy, zr)

    tiles, a = [], 0
    for n in _lengths(length, TILES):
        tiles.append((a, n))
        a += n
    # A SNOT brick under each end of each tile (one under a 1-wide tile).
    snot = sorted({c for a, n in tiles for c in (a, a + n - 1)})

    for r in range(height):
        zr = z + 5 * r
        for p in range(2):
            with m.step(f"{title}, row {r + 1}: plates" + (" (2 of 2)" if p else "")):
                # The second course starts with a shorter plate, so the joints don't line up.
                first = (
                    max(1, Plate.longest(length, 1) // 2) if p and length > 2 else None
                )
                m.row(
                    Plate,
                    color,
                    at=(*cell(0), zr + p),
                    length=length,
                    axis=axis,
                    first=first,
                )
        with m.step(f"{title}, row {r + 1}: bricks"):
            start = 0
            for c in snot + [length]:
                if c > start:
                    m.row(
                        Brick,
                        color,
                        at=(*cell(start), zr + 2),
                        length=c - start,
                        axis=axis,
                    )
                if c < length:
                    m.add("87087", color, at=(*cell(c), zr + 2), rot=turn)
                start = c + 1
        with m.step(f"{title}, row {r + 1}: tiles"):
            for a, n in tiles:
                rot = upright_rot if n == 1 else long_rot
                m.add(TILES[n], face_color, at=tile_at(a, zr), rot=rot)
    return z + 5 * height


def example(m):
    """A panel 6 studs long and two rows of tiles tall, standing on the ground."""
    m.use("snot_panel", x=0, y=0, z=0, length=6, height=2, color="Dark Bluish Gray")
