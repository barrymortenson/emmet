# SPDX-License-Identifier: Apache-2.0
"""SNOT side skirt: a racing-scale car's flush lower sides, long tiles standing on the side studs of the sill course,
with a grille tile as the intake at the back end.

m.use("snot_side_skirt", x, y, z, length=8, width=8, color="Black", skirt="Red", intake="Black", title="Side
skirts") lays the sill course of a car `width` studs wide (y to y + width) on the studs at z, `length` long from x
back: on each side bricks 1 x 4 with four studs on the side (30414), facing out, and 1 x 2 bricks with two (11211) to
finish an odd stretch; on the studs, tiles standing on edge (1 x 4, 1 x 2) from the front, and a 1 x 2 grille tile
as the intake at the back end, so the side is smooth below the doors. The middle of the course is left for the floor.
Lengths are even, from 4. Returns the z of the course's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 77256. Not derived from any
model file."""

PARTS = ["30414", "11211", "2431", "3069b", "2412b"]

# A tile on edge, long along x: its face to -y (the left side) or to +y (the right side).
LEFT = Rot(y=-90, z=90)
RIGHT = Rot(y=-90, z=-90)
TILES = {4: "2431", 2: "3069b"}  # Tile 1 x n


def build(
    m,
    x,
    y,
    z,
    length=8,
    width=8,
    color="Black",
    skirt="Red",
    intake="Black",
    title="Side skirts",
):
    """The side-stud bricks, then the skirt tiles and intakes; returns the z of the course's top."""
    if length < 4 or length % 2:
        raise ValueError(
            f"snot_side_skirt needs an even length of 4 or more, not {length}"
        )
    runs = [4] * (length // 4) + ([2] if length % 4 else [])
    with m.step(f"{title}: side-stud bricks"):
        a = x
        for n in runs:
            m.add("30414" if n == 4 else "11211", color, at=(a, y, z), rot=3)
            m.add(
                "30414" if n == 4 else "11211", color, at=(a, y + width - 1, z), rot=1
            )
            a += n
    # Tiles from the front: the last two studs take the intake.
    tiles = [4] * ((length - 2) // 4) + ([2] if (length - 2) % 4 else [])
    for side, turn, at_y in (("left", LEFT, y - 0.4), ("right", RIGHT, y + width)):
        with m.step(f"{title}: {side} skirt"):
            a = x
            for n in tiles:
                m.add(TILES[n], skirt, at=(a, at_y, z + 0.5), rot=turn)
                a += n
            m.add("2412b", intake, at=(a, at_y, z + 0.5), rot=turn)
    return z + 3


def example(m):
    """The skirts along an 8-wide car's floor, with the floor between them and the doors' sills on top."""
    with m.step("Floor"):
        m.add("41539", "Dark Bluish Gray", at=(0, 0, 0))
    top = m.use("snot_side_skirt", x=0, y=0, z=1)
    with m.step("Sills"):
        for side in (0, 7):
            m.add("3460", "Red", at=(0, side, top), rot=1)
