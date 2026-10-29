# SPDX-License-Identifier: Apache-2.0
"""Clapboard siding: lap siding on a brick wall, a strip of plate along each course, faced with cheese slopes that
read as overlapping boards, hung on side studs.

m.use("clapboard_siding", x, y, z, length, rows=4, color="Light Bluish Gray", siding="White", title="Siding") builds a
wall `length` studs long along x from (x, y, z), `rows` bricks tall, each course with a brick with a stud on one side
(87087) facing front (-y) at each end and every 4 studs between, and hangs a plate along each course on those studs,
its face out, with 1 x 2 cheese slopes along it that read as boards. Lengths are even, 2 to 12. Returns the z of the
wall's top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 43217. Not derived from any
model file."""

PARTS = [
    "87087",
    "3023b",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "85984",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "3005",
]

PLATES = {
    2: "3023b",
    4: "3710",
    6: "3666",
    8: "3460",
    10: "4477",
    12: "60479",
}  # Plate 1 x n
FACE = Rot(y=-90, z=90)  # on its back, its face to -y, long along x
BOARD = Rot(y=90, z=270)
DEEP = 0.78  # a cheese slope on its back stands out 1.95 plates, in studs


def build(
    m,
    x,
    y,
    z,
    length,
    rows=4,
    color="Light Bluish Gray",
    siding="White",
    title="Siding",
):
    """The wall and its siding; returns the z of the top."""
    if length not in PLATES:
        raise ValueError(
            f"clapboard_siding takes an even length from 2 to 12, not {length}"
        )
    anchors = sorted({0, length - 1, *range(4, length - 1, 4)})
    for r in range(rows):
        zr = z + 3 * r
        with m.step(f"{title}, row {r + 1}: wall"):
            start = 0
            for a in anchors + [length]:
                if a > start:
                    m.row(Brick, color, at=(x + start, y, zr), length=a - start)
                if a < length:
                    m.add("87087", color, at=(x + a, y, zr), rot=3)
                start = a + 1
        with m.step(f"{title}, row {r + 1}: siding"):
            m.add(PLATES[length], siding, at=(x, y - 0.4, zr + 0.5), rot=FACE)
            for k in range(0, length, 2):
                m.add("85984", siding, at=(x + k, y - 0.4 - DEEP, zr + 0.5), rot=BOARD)
    return z + 3 * rows


def example(m):
    """A wall 6 studs long and 4 bricks tall, sided."""
    m.use("clapboard_siding", x=0, y=0, z=0, length=6)
