# SPDX-License-Identifier: Apache-2.0
"""Wall-top cornice: a band of light stone round the tops of four walls, corner plates tying the corners under smooth
tiles, with a stud left open midway along each wall for the roof to grip.

m.use("wall_top_cornice", x, y, z, length, depth, color="Tan", lugs=True, title="Cornice") caps the walls (1 stud
thick) of a length x depth rectangle whose front-left corner is (x, y), at the z of the wall tops: an L-shaped 2 x 2
corner plate (2420) on each corner, which crosses the corner joint below, and 1-wide plates between them, then an
L-shaped 2 x 2 corner tile (14719) over each corner plate and tiles over the runs. With lugs, the stud in the middle
of each wall stays open, where a lift-off roof frame locates; without, the top is smooth all round. Returns the z of
the top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335. Not derived from any
model file."""

PARTS = [
    "2420",
    "14719",
    "3623",
    "3710",
    "3666",
    "3460",
    "3069b",
    "2431",
    "6636",
    "3070b",
    "63864",
]


def build(m, x, y, z, length, depth, color="Tan", lugs=True, title="Cornice"):
    """The corner plates and runs, then the tiles; returns the z of the top."""
    if length < 5 or depth < 5:
        raise ValueError(
            f"wall_top_cornice needs a rectangle of 5 x 5 or more, not {length} x {depth}"
        )
    x1, y1 = x + length - 1, y + depth - 1
    # Each corner's L: where its 2 x 2 goes and the turn that puts its square corner on the wall's corner.
    corners = [((x, y), 3), ((x1 - 1, y), 0), ((x1 - 1, y1 - 1), 1), ((x, y1 - 1), 2)]
    # The runs between the L's arms: start, length, axis.
    runs = [
        ((x + 2, y), length - 4, "x"),
        ((x + 2, y1), length - 4, "x"),
        ((x, y + 2), depth - 4, "y"),
        ((x1, y + 2), depth - 4, "y"),
    ]
    with m.step(f"{title}: corner plates"):
        for at, rot in corners:
            m.add("2420", color, at=(*at, z), rot=rot)
    with m.step(f"{title}: plates"):
        for at, n, axis in runs:
            m.row(Plate, color, at=(*at, z), length=n, axis=axis)
    with m.step(f"{title}: corner tiles"):
        for at, rot in corners:
            m.add("14719", color, at=(*at, z + 1), rot=rot)
    with m.step(f"{title}: tiles"):
        for (sx, sy), n, axis in runs:
            # The lug: the run's middle stud, left open.
            gap = n // 2 if lugs else n
            for start, k in ((0, gap), (gap + 1 if lugs else n, n - gap - 1)):
                if k > 0:
                    at = (sx + start, sy) if axis == "x" else (sx, sy + start)
                    m.row(Tile, color, at=(*at, z + 1), length=k, axis=axis)
    return z + 2


def example(m):
    """Four low plastered walls with a stone cornice."""
    m.wall(0, 0, 0, length=10, depth=7, rows=2, color="White")
    m.use("wall_top_cornice", x=0, y=0, z=6, length=10, depth=7)
