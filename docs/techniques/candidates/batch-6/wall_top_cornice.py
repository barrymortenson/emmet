# SPDX-License-Identifier: Apache-2.0
"""Wall-top cornice: a band of light stone round the tops of four walls, corner plates tying the corners under smooth
tiles, with a stud standing up midway along each wall for the roof to grip.

m.use("wall_top_cornice", x, y, z, length, depth, color="Tan", lugs=True, title="Cornice") caps the walls (1 stud
thick) of a length x depth rectangle whose front-left corner is (x, y), at the z of the wall tops: an L-shaped 2 x 2
corner plate (2420) on each corner, which crosses the corner joint below, and 1-wide plates between them, then an
L-shaped 2 x 2 corner tile (14719) over each corner plate and tiles over the runs. With lugs, a 1 x 1 plate takes the
tiles' place in the middle of each wall, flush with them, its stud standing up where a lift-off roof frame locates;
without, the top is smooth all round. Returns the z of the top (the tiles', under the lugs' studs).
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
    "3024",
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
    for side, part in (("front and back", runs[:2]), ("sides", runs[2:])):
        with m.step(f"{title}: tiles, {side}"):
            _tiles(m, part, color, lugs, z)
    return z + 2


def _tiles(m, runs, color, lugs, z):
    """Tiles along runs of the plates, and with lugs a 1 x 1 plate in each run's middle."""
    for (sx, sy), n, axis in runs:
        # The lug: a 1 x 1 plate in the run's middle, flush with the tiles, its stud up for the roof to grip.
        gap = n // 2 if lugs else n
        for start, k in ((0, gap), (gap + 1 if lugs else n, n - gap - 1)):
            if k > 0:
                at = (sx + start, sy) if axis == "x" else (sx, sy + start)
                m.row(Tile, color, at=(*at, z + 1), length=k, axis=axis)
        if lugs:
            at = (sx + gap, sy) if axis == "x" else (sx, sy + gap)
            m.add("3024", color, at=(*at, z + 1))


def example(m):
    """Four low plastered walls with a stone cornice."""
    m.wall(0, 0, 0, length=10, depth=7, rows=2, color="White")
    m.use("wall_top_cornice", x=0, y=0, z=6, length=10, depth=7)
