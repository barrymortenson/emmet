# SPDX-License-Identifier: Apache-2.0
"""Window box: a planter under a window, bricks with side studs hung from the sill plate, flowers on their studs.

m.use("window_box", x, y, z, length=4, color="Reddish Brown", flowers=("Dark Pink", "Bright Light Orange"),
leaves="Bright Green", title="Window box") lays, in a wall one stud thick along x facing front (-y), a plate 2 x
`length` from (x, y - 1) at z: its back row takes the wall's cells x to x + length - 1 for that plate course, its front
row stands a stud out. Under the front row hang Bricks 1 x 2 with Two Studs on One Side (11211), studs out, from z - 3;
on their side studs go round plates with tabs as blooms, each one of `flowers`, with a round 1 x 1 plate in `leaves`
between each pair. A tile covers the front row. length is 2, 4, or 6. The window stands on the plate's back row: returns
the z it stands at (z + 1).
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43268, set 11387. Not derived
from any model file."""

PARTS = ["11211", "33291", "6141", "3022", "3020", "3795", "3069b", "2431", "6636"]

PLATES = {2: "3022", 4: "3020", 6: "3795"}  # Plate 2 x n
TILES = {2: "3069b", 4: "2431", 6: "6636"}  # Tile 1 x n
FACE = Rot(x=90)  # a round plate on a side stud, its top to -y


def build(
    m,
    x,
    y,
    z,
    length=4,
    color="Reddish Brown",
    flowers=("Dark Pink", "Bright Light Orange"),
    leaves="Bright Green",
    title="Window box",
):
    """The sill plate, the box under it, and the blooms; returns the z the window stands at."""
    if length not in PLATES:
        raise ValueError(f"window_box takes a length of 2, 4, or 6, not {length}")
    with m.step(f"{title}: plate and box"):
        m.add(PLATES[length], color, at=(x, y - 1, z), rot=1)
        for k in range(0, length, 2):
            m.add("11211", color, at=(x + k, y - 1, z - 3), rot=3)
    with m.step(f"{title}: flowers"):
        for k in range(length):
            if k % 2 == 0:
                bloom = flowers[(k // 2) % len(flowers)]
                m.add("33291", bloom, at=(x + k - 0.1, y - 1.4, z - 2.75), rot=FACE)
            else:
                m.add("6141", leaves, at=(x + k, y - 1.4, z - 2.5), rot=FACE)
    with m.step(f"{title}: sill"):
        m.add(TILES[length], color, at=(x, y - 1, z + 1), rot=1)
    return z + 1


def example(m):
    """A window over its box in a short wall."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Tan", at=(0, 0, 0), length=6)
    z = m.use("window_box", x=1, y=0, z=3)
    with m.step("Plates beside the box"):
        for at in (0, 5):
            m.add("3024", "Tan", at=(at, 0, 3))
    with m.step("Window"):
        f = m.add("60592", "White", at=(2, 0, z), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
    for r in range(2):
        with m.step(f"Wall, row {r + 2}"):
            m.add("3004", "Tan", at=(0, 0, z + 3 * r), rot=1)
            m.add("3004", "Tan", at=(4, 0, z + 3 * r), rot=1)
    with m.step("Wall, row 4"):
        m.row(Brick, "Tan", at=(0, 0, z + 6), length=6)
