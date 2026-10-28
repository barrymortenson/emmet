# SPDX-License-Identifier: Apache-2.0
"""Tiled floor: a layer of plates topped with tiles over a rectangle, smooth on top, plain, checkered, or bordered.

m.use("tiled_floor", x, y, z, length, depth, color="Tan", pattern="plain", color2=None, cell=2, plate_color=None,
title="Floor") covers the rectangle whose front-left corner is (x, y), `length` studs along x and `depth` along y,
standing on the studs at height z. First a layer of plates (in plate_color, or the floor's color) in strips 2 studs
wide along the rectangle's long side, then tiles over all of it, so the floor is 2 plates thick and has no studs on
top. pattern: "plain" tiles in `color`, laid in strips across the plates' strips so they tie the plates together;
"checker", squares of `cell` studs (1 or 2) alternating `color` and color2, the front-left one in `color` (a square at
an odd edge is cut to 1 stud); or "border", a 1-stud border in color2 around plain tiles in `color`. color2 defaults to
White. It brings its own steps, "<title>, plates" and "<title>, tiles" (split into parts of up to 8), the back rows
first for strips along x (left to right for strips along y), and returns the z of the floor's top, z + 2.
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3B), written from general building knowledge; not derived from any model
file."""

PARTS = [
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "4282",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3070b",
    "3069b",
    "63864",
    "2431",
    "6636",
    "4162",
    "3068b",
    "87079",
]


def _pieces(family, run, width):
    """A run of `run` studs, `width` across, as [(length along, part id)], longest first."""
    have = family.lengths(width)
    out, left = [], run
    for n in sorted(have, reverse=True):
        while left >= n:
            out.append((n, have[n]))
            left -= n
    return out


def _turn(axis, along, across):
    """The quarter turn that lays a piece `along` studs along the axis (a piece's long side runs along y at rot=0)."""
    if along == across:
        return 0
    return 1 if (axis == "x") == (along > across) else 0


def _widths(n):
    """Strips 2 studs wide, and one 1 wide when n is odd."""
    return [2] * (n // 2) + [1] * (n % 2)


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    color="Tan",
    pattern="plain",
    color2=None,
    cell=2,
    plate_color=None,
    title="Floor",
):
    """The floor; returns the z of its top."""
    if pattern not in ("plain", "checker", "border"):
        raise ValueError(
            f'tiled_floor: pattern must be "plain", "checker", or "border", not {pattern!r}'
        )
    if cell not in (1, 2):
        raise ValueError(f"tiled_floor: cell must be 1 or 2, not {cell!r}")
    if length < 1 or depth < 1 or (pattern == "border" and min(length, depth) < 3):
        raise ValueError(
            "tiled_floor needs a length and depth of 1 or more (3 or more with a border)"
        )
    color2 = color2 or "White"

    def strips(family, tint, x0, y0, run_x, run_y, axis, h):
        """Covers the run_x × run_y rectangle at (x0, y0) with strips along `axis`, the back ones first."""
        run, wide = (run_x, run_y) if axis == "x" else (run_y, run_x)
        out, v = [], 0
        for w in _widths(wide):
            pieces = _pieces(family, run, w)
            out.append((len(pieces), pieces, v, w))
            v += w
        if axis == "x":
            out.reverse()  # the strips along x go back to front
        actions = []
        for n, pieces, v, w in out:

            def add(pieces=pieces, v=v, w=w):
                u = 0
                for k, part in pieces:
                    at = (x0 + u, y0 + v, h) if axis == "x" else (x0 + v, y0 + u, h)
                    m.add(part, tint, at=at, rot=_turn(axis, k, w))
                    u += k

            actions.append((n, add))
        return actions

    along = "x" if length >= depth else "y"
    across = "y" if along == "x" else "x"
    in_steps(
        m,
        f"{title}, plates",
        strips(Plate, plate_color or color, x, y, length, depth, along, z),
    )
    top = z + 1
    if pattern == "plain":
        actions = strips(Tile, color, x, y, length, depth, across, top)
    elif pattern == "border":
        actions = strips(Tile, color2, x, y + depth - 1, length, 1, "x", top)
        actions += strips(Tile, color2, x, y + 1, 1, depth - 2, "y", top)
        actions += strips(Tile, color, x + 1, y + 1, length - 2, depth - 2, across, top)
        actions += strips(Tile, color2, x + length - 1, y + 1, 1, depth - 2, "y", top)
        actions += strips(Tile, color2, x, y, length, 1, "x", top)
    else:
        actions = []
        for j in reversed(range(0, depth, cell)):
            for i in range(0, length, cell):
                w, d = min(cell, length - i), min(cell, depth - j)
                tint = color if (i // cell + j // cell) % 2 == 0 else color2
                part = Tile(w, d)

                def add(part=part, i=i, j=j, w=w, d=d, tint=tint):
                    m.add(part, tint, at=(x + i, y + j, top), rot=1 if w > d else 0)

                actions.append((1, add))
    in_steps(m, f"{title}, tiles", actions)
    return z + 2


def example(m):
    """An 8 × 6 floor of tan and white squares."""
    m.use("tiled_floor", x=0, y=0, z=0, length=8, depth=6, pattern="checker")
