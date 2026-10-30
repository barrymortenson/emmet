# SPDX-License-Identifier: Apache-2.0
"""Dentil band: a row of round studs in two colors across a facade, under a cornice or over a wide doorway, like the
teeth of a classical cornice.

m.use("dentil_band", x, y, z, length, rows=1, wall="Medium Nougat", dots=("White", "Black"), title="Dentil band")
lays `rows` courses along x from (x, y, z), facing front (-y), of bricks with studs on one side (30414, 11211, 87087)
in the wall color, every other course starting with a shorter one so the joints cross, and puts a 1 x 1 round tile
(98138) on every side stud, the colors in `dots` taking turns along the row and shifting by one each course. The tiles
stand out a plate. Returns the z of the top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["30414", "11211", "87087", "98138"]

# Bricks with studs on one side, by length.
BRICKS = {4: "30414", 2: "11211", 1: "87087"}
FACE = Rot(y=-90, z=90)  # a tile on its back, its face to -y


def build(
    m,
    x,
    y,
    z,
    length,
    rows=1,
    wall="Medium Nougat",
    dots=("White", "Black"),
    title="Dentil band",
):
    """The side-stud courses and their round tiles; returns the z of the top."""
    if length < 2 or rows < 1 or not dots:
        raise ValueError(
            f"dentil_band needs a length of 2 or more, 1 row or more, and a color, not {length}, {rows}, {dots}"
        )
    for r in range(rows):
        zr = z + 3 * r
        sizes, left = (
            ([2] if r % 2 and length > 2 else []),
            length - (2 if r % 2 and length > 2 else 0),
        )
        for n in (4, 2, 1):
            sizes += [n] * (left // n)
            left %= n
        with m.step(f"{title}, row {r + 1}: bricks"):
            at = x
            for n in sizes:
                m.add(BRICKS[n], wall, at=(at, y, zr), rot=3)
                at += n
        with m.step(f"{title}, row {r + 1}: studs"):
            for c in range(length):
                m.add(
                    "98138",
                    dots[(c + r) % len(dots)],
                    at=(x + c, y - 0.4, zr + 0.5),
                    rot=FACE,
                )
    return z + 3 * rows


def example(m):
    """A wall with a double dentil band under a tiled top."""
    from emmet_dsl import Brick, Tile

    for r in range(2):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Medium Nougat",
                at=(0, 0, 3 * r),
                length=8,
                first=2 if r else None,
            )
    top = m.use("dentil_band", x=0, y=0, z=6, length=8, rows=2)
    with m.step("Coping"):
        m.row(Tile, "Tan", at=(0, 0, top), length=8, first=2)
