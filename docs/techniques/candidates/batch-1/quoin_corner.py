# SPDX-License-Identifier: Apache-2.0
"""Quoin corner: stone corner blocks on a wall's outside corner, long and short in turn, standing out from the walls.

m.use("quoin_corner", x, y, z, rows, color="Light Bluish Gray", stone="Tan", length=4, title="Corner") builds the
corner of a front wall (along +x from x) and a left wall (along +y from y), `rows` bricks tall from z, `length` studs
each way, with the corner itself a column of bricks with studs on four sides (4733). Each course a tile hangs on the
corner brick's front stud and another on its left stud, one long (1 x 2, running along its wall) and one short (1 x 1),
swapped course to course the way stone quoins interlock. The tiles stand out a plate from the walls. Returns the z of
the top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 21335 (corner trim on studs-out
corner columns) and set 10350 (stone corner strips); the miner found the key parts joined in few official sets. Not
derived from any model file."""

PARTS = ["4733", "3069b", "3070b", "3004", "3010", "3005", "3622"]

FRONT, LEFT = (
    Rot(y=-90, z=90),
    Rot(y=-90),
)  # a tile on its back, its face to -y (long along x), or to -x
OUT = 0.4  # a tile's thickness, in studs


def build(
    m, x, y, z, rows, color="Light Bluish Gray", stone="Tan", length=4, title="Corner"
):
    """The corner and its walls; returns the z of the top."""
    if rows < 1 or length < 2:
        raise ValueError(
            f"quoin_corner needs rows of 1 or more and a length of 2 or more, not {rows}, {length}"
        )
    for r in range(rows):
        zr = z + 3 * r
        with m.step(f"{title}, row {r + 1}"):
            m.add("4733", color, at=(x, y, zr))
            # The walls each side, laid so their joints don't line up course to course.
            first = 2 if r % 2 else 1
            for axis, at in (("x", (x + 1, y, zr)), ("y", (x, y + 1, zr))):
                m.row(
                    Brick,
                    color,
                    at=at,
                    length=length - 1,
                    axis=axis,
                    first=first if length - 1 > first else None,
                )
            long_front = r % 2 == 0
            front = "3069b" if long_front else "3070b"
            left = "3070b" if long_front else "3069b"
            m.add(front, stone, at=(x, y - OUT, zr + 0.5), rot=FRONT)
            m.add(left, stone, at=(x - OUT, y, zr + 0.5), rot=LEFT)
    return z + 3 * rows


def example(m):
    """A corner four bricks tall."""
    m.use("quoin_corner", x=0, y=0, z=0, rows=4)
