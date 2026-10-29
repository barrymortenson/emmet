# SPDX-License-Identifier: Apache-2.0
"""Bartizan: a small round turret corbelled out over the outside corner of two walls, on a corbel that hangs below it,
with a cone roof.

m.use("bartizan", x, y, z, corner="front-left", rows=2, color="Light Bluish Gray", roof="Dark Bluish Gray",
title="Bartizan") builds a turret 4 studs across centered on the outside corner (x, y) of two walls 1 stud thick whose
tops are at z: for corner="front-left", a front wall running +x from x and a left wall running +y from y; the other
corners ("front-right", "back-left", "back-right") turn it to match. A Plate 4 x 4 Round (60474) on the corner takes
three of the walls' studs and overhangs both faces; an inverted double convex slope 2 x 2 (3676) hangs under its outer
quarter as the corbel, its foot tucked into the corner. On the plate, `rows` rings of four Brick 2 x 2 Corner Round
(3063b), a round plate over them to tie the ring, and a cone roof (3942c) with a cone 1 x 1 finial. Returns the z of the
roof's top.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["60474", "3676", "3063b", "3942c", "4589"]

QUARTERS = [
    ((0, 0), 0),
    ((-2, 0), 1),
    ((-2, -2), 2),
    ((0, -2), 3),
]  # each quarter ring's corner from (x, y), its turn
# Which way the walls run from the corner, and the corbel's turn that puts its foot toward them.
CORNERS = {
    "front-left": ((1, 1), 2),
    "front-right": ((-1, 1), 3),
    "back-right": ((-1, -1), 0),
    "back-left": ((1, -1), 1),
}


def build(
    m,
    x,
    y,
    z,
    corner="front-left",
    rows=2,
    color="Light Bluish Gray",
    roof="Dark Bluish Gray",
    title="Bartizan",
):
    """The base plate and its corbel, the rings, and the roof; returns the z of the roof's top."""
    if rows < 1 or corner not in CORNERS:
        raise ValueError(
            f"bartizan needs 1 row or more and a corner of {', '.join(CORNERS)}, not {rows}, {corner!r}"
        )
    (sx, sy), turn = CORNERS[corner]
    with m.step(f"{title}: base"):
        m.add("60474", color, at=(x - 2, y - 2, z))
    with m.step(f"{title}: corbel"):
        # Its top under the base's outer quarter, its foot at the inner corner, beside the walls' corner.
        m.add(
            "3676",
            color,
            at=(x - 2 if sx > 0 else x, y - 2 if sy > 0 else y, z - 3),
            rot=turn,
        )
    zr = z + 1
    for r in range(rows):
        with m.step(f"{title}, ring {r + 1}"):
            for (dx, dy), turn in QUARTERS:
                m.add("3063b", color, at=(x + dx, y + dy, zr), rot=turn)
        zr += 3
    with m.step(f"{title}: roof"):
        m.add("60474", roof, at=(x - 2, y - 2, zr))
        m.add("3942c", roof, at=(x - 1, y - 1, zr + 1))
        m.add("4589", roof, at=(x - 0.5, y - 0.5, zr + 7))
    return zr + 10


def example(m):
    """The front-right corner of two walls, three bricks tall, with a bartizan on it."""
    for r in range(3):
        with m.step(f"Walls, row {r + 1}"):
            # The front and right walls take the corner in turn, course by course.
            own = r % 2 == 0
            m.row(Brick, "Light Bluish Gray", at=(0, 0, 3 * r), length=8 if own else 7)
            m.row(
                Brick,
                "Light Bluish Gray",
                at=(7, 1 if own else 0, 3 * r),
                length=5 if own else 6,
                axis="y",
            )
    m.use("bartizan", x=8, y=0, z=9, corner="front-right")
