# SPDX-License-Identifier: Apache-2.0
"""Two-story color split: a wall whose lower courses are a darker tone than the rest, with corner boards in a third
tone at its ends and a vent grille on a side stud, the plain way a diner or a shop is painted.

m.use("two_story_color_split", x, y, z, length, rows=5, low=2, color="Tan", base="Dark Tan", corner="Reddish Brown",
vent=True, title="Wall") lays a wall along x from (x, y, z), facing front (-y), `rows` courses tall: a column of 1 x 1
bricks in the corner color at each end (the corner boards), and between them `low` courses in the base color, then
the rest in the wall color, each course in running bond. With vent, a brick with two studs on one side (11211) near
the middle of the top course but one carries a 1 x 2 grille tile (2412b). Returns the z of the top.
"""

from emmet_dsl import Brick, Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10352. Not derived from any
model file."""

PARTS = ["3005", "3004", "3622", "3010", "3009", "3008", "11211", "2412b"]

FACE = Rot(y=-90, z=90)  # a 1 x 2 tile on its back, its face to -y, long along x


def build(
    m,
    x,
    y,
    z,
    length,
    rows=5,
    low=2,
    color="Tan",
    base="Dark Tan",
    corner="Reddish Brown",
    vent=True,
    title="Wall",
):
    """The courses with their corner boards; returns the z of the top."""
    if length < 6 or not 1 <= low < rows:
        raise ValueError(
            f"two_story_color_split needs a length of 6 or more and 1 <= low < rows, not {length}, {low}, {rows}"
        )
    inner = length - 2
    vent_row, vent_at = max(low, rows - 2), (inner - 2) // 2
    for r in range(rows):
        zr = z + 3 * r
        tone = base if r < low else color
        with m.step(f"{title}, row {r + 1}"):
            for cx in (x, x + length - 1):
                m.add("3005", corner, at=(cx, y, zr))
            first = 1 if r % 2 else None
            if vent and r == vent_row:
                m.row(Brick, tone, at=(x + 1, y, zr), length=vent_at, first=first)
                m.add("11211", tone, at=(x + 1 + vent_at, y, zr), rot=3)
                m.row(
                    Brick, tone, at=(x + 3 + vent_at, y, zr), length=inner - vent_at - 2
                )
                m.add(
                    "2412b",
                    "Dark Bluish Gray",
                    at=(x + 1 + vent_at, y - 0.4, zr + 0.5),
                    rot=FACE,
                )
            else:
                m.row(
                    Brick,
                    tone,
                    at=(x + 1, y, zr),
                    length=inner,
                    first=first,
                    sizes=(6, 4, 3, 2, 1),
                )
    return z + 3 * rows


def example(m):
    """A wall 10 studs long, dark below and light above, on a plate."""
    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=10, first=4)
    m.use("two_story_color_split", x=0, y=0, z=1, length=10)
