# SPDX-License-Identifier: Apache-2.0
"""Rusticated stone pilaster: a tall pale stone pier of long and short blocks whose joints wander course to course,
with the odd block end standing out and a light band at each floor line.

m.use("rusticated_stone_pilaster", x, y, z, rows, color="Tan", band="White", band_every=4, title="Pier") builds a
2 x 2 pier from x to x + 2 and y - 1 to y + 1, standing a stud proud of a wall whose face is at y (the wall runs
along x and faces front, -y), `rows` courses tall from z. Each course is a 1 x 2 brick and two 1 x 1 bricks, the long
one turned a quarter turn around the pier each course, so the joints on the face wander like cut stone. Every third
course one front 1 x 1 is a brick with a stud on one side (87087) with a 1 x 1 tile on it, a block end standing a
plate proud, left and right in turn. After every `band_every` courses, and on top, a 2 x 2 plate in the band color
crosses the pier: carry the same plate course across the wall beside it for a string course. Returns the z of the top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["3004", "3005", "87087", "3070b", "3022"]

FACE = Rot(y=-90, z=90)  # a 1 x 1 tile on its back, its face to -y
# Course by course around the pier: where the 1 x 2 goes (cell, rot) and the two 1 x 1 cells, cells as (dx, dy) from
# the pier's front-left corner.
TURNS = [
    (((0, 0), 1), [(0, 1), (1, 1)]),  # long along the front
    (((1, 0), 0), [(0, 0), (0, 1)]),  # long up the right side
    (((0, 1), 1), [(0, 0), (1, 0)]),  # long along the back
    (((0, 0), 0), [(1, 0), (1, 1)]),  # long up the left side
]


def build(m, x, y, z, rows, color="Tan", band="White", band_every=4, title="Pier"):
    """The courses, the block ends, and the bands; returns the z of the top."""
    if rows < 1 or band_every < 1:
        raise ValueError(
            f"rusticated_stone_pilaster needs 1 row or more and band_every of 1 or more, not {rows}, {band_every}"
        )
    y0, zr, proud = y - 1, z, 0
    for r in range(rows):
        (cell, rot), ones = TURNS[r % 4]
        with m.step(f"{title}, row {r + 1}"):
            m.add("3004", color, at=(x + cell[0], y0 + cell[1], zr), rot=rot)
            front = [c for c in ones if c[1] == 0]
            stud = front[proud % len(front)] if r % 3 == 2 and front else None
            for c in ones:
                if c == stud:
                    m.add("87087", color, at=(x + c[0], y0, zr), rot=3)
                    m.add("3070b", color, at=(x + c[0], y0 - 0.4, zr + 0.5), rot=FACE)
                    proud += 1
                else:
                    m.add("3005", color, at=(x + c[0], y0 + c[1], zr))
        zr += 3
        if (r + 1) % band_every == 0 or r == rows - 1:
            with m.step(f"{title}, row {r + 1}: band"):
                m.add("3022", band, at=(x, y0, zr))
            zr += 1
    return zr


def example(m):
    """A pier beside a brick wall, with a string course across both at the floor line."""
    from emmet_dsl import Plate

    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, -1, 0), length=8, width=2)
    m.use("rusticated_stone_pilaster", x=0, y=0, z=1, rows=8)
    zw = m.use(
        "masonry_wall", x=2, y=0, z=1, length=6, rows=4, patches=0, title="Wall, lower"
    )
    with m.step("String course"):
        m.row(Plate, "White", at=(2, 0, zw), length=6)
    zw = m.use(
        "masonry_wall",
        x=2,
        y=0,
        z=zw + 1,
        length=6,
        rows=4,
        patches=0,
        title="Wall, upper",
    )
    with m.step("Coping"):
        m.row(Plate, "White", at=(2, 0, zw), length=6)
