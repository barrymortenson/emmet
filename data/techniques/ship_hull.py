# SPDX-License-Identifier: Apache-2.0
"""Ship hull: a brick-built hull 6 studs wide whose sides close in to a pointed bow on wedge bricks, decked over with
plates that tie the courses together.

m.use("ship_hull", x, y, z, length=6, rows=2, color="Dark Red", deck="Tan", title="Hull") builds a hull from (x, y, z),
its stern at x and its bow toward +x, 6 studs wide (y to y + 6). Each course is a side of 1 x n bricks along each edge
for `length` studs, a transom across the stern, and the bow: Wedge 4 x 2 Right and Left (41767, 41768) on the two edges
with a 2 x 4 brick between, which close the hull in from 6 studs to 4, then Wedge 3 x 2 Right and Left (6564, 6565),
which close it to a 2-stud stem; the bow adds 7 studs to the length. On top, a deck of plates: two strips along the
sides of the hull and a middle strip that runs on out over the bow, tying it to the hull, and Wing 2 x 4 plates (41769a,
41770a) that match the first wedges. Returns the z of the deck's top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "41767",
    "41768",
    "6564",
    "6565",
    "3001",
    "41769a",
    "41770a",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "91988",
    "4282",
    "3023b",
]


def build(m, x, y, z, length=6, rows=2, color="Dark Red", deck="Tan", title="Hull"):
    """The courses, stern to bow, and the deck; returns the z of the deck's top."""
    if length < 2 or rows < 1:
        raise ValueError(
            f"ship_hull needs a length of 2 or more and 1 row or more, not {length}, {rows}"
        )
    b = x + length  # where the bow starts
    for r in range(rows):
        zr = z + 3 * r
        with m.step(f"{title}, row {r + 1}: sides and stern"):
            # The transom and the sides take the stern's corners in turn, course by course.
            own = r % 2 == 0
            for row in (y, y + 5):
                m.row(Brick, color, at=(x + own, row, zr), length=length - own)
            m.row(Brick, color, at=(x, y + 1 - own, zr), length=4 + 2 * own, axis="y")
        with m.step(f"{title}, row {r + 1}: bow"):
            m.add("41767", color, at=(b, y, zr))
            m.add("3001", color, at=(b, y + 2, zr), rot=1)
            m.add("41768", color, at=(b, y + 4, zr))
            m.add("6564", color, at=(b + 4, y + 1, zr))
            m.add("6565", color, at=(b + 4, y + 3, zr))
    zd = z + 3 * rows
    with m.step(f"{title}: deck"):
        m.row(Plate, deck, at=(x, y, zd), length=length, width=2)
        m.row(Plate, deck, at=(x, y + 4, zd), length=length, width=2)
        m.row(Plate, deck, at=(x, y + 2, zd), length=length + 7, width=2)
        m.add("41769a", deck, at=(b, y, zd))
        m.add("41770a", deck, at=(b, y + 4, zd))
    return zd + 1


def example(m):
    """A small hull, two courses deep."""
    m.use("ship_hull", x=0, y=0, z=0, length=6)
