# SPDX-License-Identifier: Apache-2.0
"""Wall-top tie: a course of plates across the tops of four walls, laid over the corner joints so it ties them.

m.use("wall_top_tie", x, y, z, length, depth, color="Light Bluish Gray", tile=True, title="Wall top") lays 1-wide
plates along the tops of the walls of a length x depth rectangle whose front-left corner is (x, y), at the z of the
wall tops. The front and back runs own the corners when the course below gave them to the sides, and the other way
round, so no plate joint sits on a corner joint and every corner is bridged. With tile, tiles cover the plates except
a stud at each corner, which is where a roof or the next floor locates and lifts off. Returns the z of the top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350, set 21335. Not derived from
any model file."""

PARTS = [
    "3666",
    "3460",
    "3710",
    "3623",
    "3023b",
    "3024",
    "4162",
    "6636",
    "2431",
    "3069b",
    "3070b",
    "3008",
    "3009",
    "3010",
    "3004",
    "3005",
    "3622",
]


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    color="Light Bluish Gray",
    tile=True,
    sides_own=None,
    title="Wall top",
):
    """The plates, then the tiles; returns the z of the top. sides_own: whether the left and right runs own the
    corners (by default, the opposite of the course below, which m.wall gives the front and back on even courses)."""
    if length < 3 or depth < 3:
        raise ValueError(
            f"wall_top_tie needs a rectangle of 3 x 3 or more, not {length} x {depth}"
        )
    if sides_own is None:
        # m.wall gives the front and back the corners on even courses, so the plates go the other way round from
        # the course below (the one that ends at z).
        sides_own = round(z / 3) % 2 == 1
    x1, y1 = x + length - 1, y + depth - 1
    if sides_own:
        runs = [
            ((x + 1, y), length - 2, "x"),
            ((x + 1, y1), length - 2, "x"),
            ((x, y), depth, "y"),
            ((x1, y), depth, "y"),
        ]
    else:
        runs = [
            ((x, y), length, "x"),
            ((x, y1), length, "x"),
            ((x, y + 1), depth - 2, "y"),
            ((x1, y + 1), depth - 2, "y"),
        ]
    with m.step(f"{title}: plates"):
        for at, n, axis in runs:
            m.row(Plate, color, at=(*at, z), length=n, axis=axis)
    if not tile:
        return z + 1
    with m.step(f"{title}: tiles"):
        # Every cell of the plates but the four corners.
        for (sx, sy), n, axis in [
            ((x + 1, y), length - 2, "x"),
            ((x + 1, y1), length - 2, "x"),
            ((x, y + 1), depth - 2, "y"),
            ((x1, y + 1), depth - 2, "y"),
        ]:
            m.row(Tile, color, at=(sx, sy, z + 1), length=n, axis=axis)
    return z + 2


def example(m):
    """Four low walls with their tops tied."""
    m.wall(0, 0, 0, length=8, depth=6, rows=2, color="Tan")
    m.use("wall_top_tie", x=0, y=0, z=6, length=8, depth=6, color="Tan")
