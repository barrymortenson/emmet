# SPDX-License-Identifier: Apache-2.0
"""String course: a thin light band across a facade at a floor line, standing a plate proud of the wall, that splits
one story from the next.

m.use("string_course", x, y, z, length, color="White", wall="Medium Nougat", proud=True, title="String course")
runs a band along x from (x, y, z), facing front (-y), on top of a wall course. With proud, a course of bricks with
studs on one side (30414, 11211, 87087) in the wall color carries tiles in the band color on its side studs, the tile
joints crossing the brick joints, and a course of plates in the band color caps it, so the band is three and a half
plates tall and stands out a plate. Without it, the band is just the plate course, flush. Upper-floor sills set on it
line up. Returns the z of the top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302 and set 10350. Not
derived from any model file."""

PARTS = [
    "30414",
    "11211",
    "87087",
    "6636",
    "2431",
    "63864",
    "3069b",
    "3070b",
    "3623",
    "3710",
    "3666",
    "3460",
]

# Bricks with studs on one side, by length.
BRICKS = {4: "30414", 2: "11211", 1: "87087"}
TILES = {6: "6636", 4: "2431", 3: "63864", 2: "3069b", 1: "3070b"}
FACE = Rot(y=-90, z=90)  # a tile on its back, its face to -y, long along x


def _split(n, sizes, first=None):
    out = [first] if first and n > first else []
    n -= sum(out)
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
    color="White",
    wall="Medium Nougat",
    proud=True,
    title="String course",
):
    """The band; returns the z of its top."""
    if length < 2:
        raise ValueError(f"string_course needs a length of 2 or more, not {length}")
    if proud:
        with m.step(f"{title}: side-stud bricks"):
            at = x
            for n in _split(length, BRICKS):
                m.add(BRICKS[n], wall, at=(at, y, z), rot=3)
                at += n
        with m.step(f"{title}: tiles"):
            at = x
            for n in _split(length, TILES, first=2):
                m.add(TILES[n], color, at=(at, y - 0.4, z + 0.5), rot=FACE)
                at += n
        z += 3
    with m.step(f"{title}: cap"):
        m.row(
            Plate, color, at=(x, y, z), length=length, first=3 if length > 4 else None
        )
    return z + 1


def example(m):
    """Two courses of wall, a proud string course, and a course over it."""
    from emmet_dsl import Brick

    for r in range(2):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick, "Medium Nougat", at=(0, 0, 3 * r), length=10, first=2 if r else 4
            )
    top = m.use("string_course", x=0, y=0, z=6, length=10)
    with m.step("Upper wall"):
        m.row(Brick, "Medium Nougat", at=(0, 0, top), length=10, first=2)
