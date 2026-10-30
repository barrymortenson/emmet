# SPDX-License-Identifier: Apache-2.0
"""Pixel mosaic panel: any pixel picture, given as rows of color letters, built flat in tiles on plates and stood up
studs-out, one stud a pixel.

m.use("pixel_mosaic_panel", x, y, z, rows, palette, backing="Black", upright=True, title="Mosaic") builds the picture
`rows` describes: a list of strings, the top row first, one letter a pixel, each letter a key of `palette` (a dict of
letter to color); "." (or a space) is no pixel, which leaves the backing's stud bare as texture. The picture is
len(rows[0]) studs wide and len(rows) tall. It is tiled the way the books do it, the largest tile that fits a patch of
one color first (2 x 4, 2 x 2, 1 x 8, 1 x 6, 1 x 4, 1 x 3, 1 x 2), 1 x 1 tiles only where a shape's edge needs them,
on two layers of plates whose seams cross (strips up the picture, then strips across it). The panel is a submodel
built lying down. upright=True stands it face front (-y) on a row of brackets 1 x 2 - 1 x 2 (99780) on the studs at
height z, row y from x: the panel covers y - 1.4 to y - 0.2, from z up 2.5 plates a pixel, resting on the 2 rows in
front of the brackets: keep them smooth at z (tiles) or empty. upright=False lays it studs up on the studs at (x, y,
z) as floor art, its top row at the back. Returns the z of the panel's top.
"""

from emmet_dsl import Plate, Rot
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 71374. Not derived from any
model file."""

PARTS = ["99780", "3070b", "3069b", "63864", "2431", "6636", "4162", "3068b", "87079"]

# Tiles by (width, height) in pixels, the largest first; a tile's long side runs along y at rot=0.
TILES = [
    ((2, 4), "87079", 0),
    ((4, 2), "87079", 1),
    ((2, 2), "3068b", 0),
    ((1, 8), "4162", 0),
    ((8, 1), "4162", 1),
    ((1, 6), "6636", 0),
    ((6, 1), "6636", 1),
    ((1, 4), "2431", 0),
    ((4, 1), "2431", 1),
    ((1, 3), "63864", 0),
    ((3, 1), "63864", 1),
    ((1, 2), "3069b", 0),
    ((2, 1), "3069b", 1),
    ((1, 1), "3070b", 0),
]
STAND = Rot(
    x=90
)  # the panel lying studs up, turned up about x: its studs face front (-y), its top row up
BRACKET = 3  # a bracket 1 x 2 - 1 x 2 turned so its side studs face front (-y)
PANEL = 1.2  # the panel's thickness: two plates and a tile, in studs
LIP = 0.2  # how far a bracket's side face stands in front of its plate, in studs


def _cells(rows, palette):
    """The picture as {(column, row up from the bottom): color}."""
    width = len(rows[0])
    cells = {}
    for r, line in enumerate(rows):
        if len(line) != width:
            raise ValueError(
                f"pixel_mosaic_panel: row {r + 1} is {len(line)} pixels, not {width} like the first"
            )
        for c, ch in enumerate(line):
            if ch in ". ":
                continue
            if ch not in palette:
                raise ValueError(
                    f"pixel_mosaic_panel: {ch!r} in row {r + 1} isn't in the palette"
                )
            cells[(c, len(rows) - 1 - r)] = palette[ch]
    return cells


def _tiles(cells):
    """Covers the picture with tiles, the largest that fits one color first: [(part, color, (c, r), rot)]."""
    left, out = dict(cells), []
    for c, r in sorted(cells, key=lambda p: (-p[1], p[0])):
        if (c, r) not in left:
            continue
        color = left[(c, r)]
        for (w, h), part, rot in TILES:
            # The tile's top-left pixel is (c, r): it reaches right and down.
            patch = [(c + i, r - j) for i in range(w) for j in range(h)]
            if all(left.get(p) == color for p in patch):
                for p in patch:
                    del left[p]
                out.append((part, color, (c, r - h + 1), rot))
                break
    return out


def _panel(m, rows, palette, backing):
    """The panel lying studs up, its bottom-left pixel's corner at the origin."""
    width, height = len(rows[0]), len(rows)
    # Each layer's strips are 2 wide, so their sides fall on even studs; each run starts with a 1-long piece, so its
    # joints fall on odd ones. No joint in one layer then lies along a side in the other, whatever the picture's size,
    # and every plate is held by plates of the other layer (longest first, a 5-wide picture's last column was loose).

    def pieces(n):
        """How many plates a run of n lays: the 1-long one, then the rest longest first, as m.row lays them."""
        count, left = 1, n - 1
        for size in (16, 14, 12, 10, 8, 6, 4, 3, 2, 1):
            count, left = count + left // size, left % size
        return count

    up = [
        (
            pieces(height),
            lambda c=c: m.row(
                Plate,
                backing,
                at=(c, 0, 0),
                length=height,
                axis="y",
                width=min(2, width - c),
                first=1 if height > 1 else None,
            ),
        )
        for c in range(0, width, 2)
    ]
    in_steps(m, "Backing, up", up)
    across = [
        (
            pieces(width),
            lambda r=r: m.row(
                Plate,
                backing,
                at=(0, r, 1),
                length=width,
                axis="x",
                width=min(2, height - r),
                first=1 if width > 1 else None,
            ),
        )
        for r in range(0, height, 2)
    ]
    in_steps(m, "Backing, across", across)
    tiles = _tiles(_cells(rows, dict(palette)))
    actions = [
        (1, lambda t=t: m.add(t[0], t[1], at=(t[2][0], t[2][1], 2), rot=t[3]))
        for t in tiles
    ]
    in_steps(m, "Tiles", actions)


def build(m, x, y, z, rows, palette, backing="Black", upright=True, title="Mosaic"):
    """The panel and, standing, its brackets; returns the z of the panel's top."""
    if not rows or not rows[0]:
        raise ValueError("pixel_mosaic_panel needs at least one row of pixels")
    width, height = len(rows[0]), len(rows)
    if upright and width < 2:
        raise ValueError(
            "pixel_mosaic_panel stands on brackets 2 studs wide: make the picture 2 or more pixels wide, "
            "or lay it flat (upright=False)"
        )
    sub = m.submodel(
        f"{title}: panel",
        _panel,
        rows=tuple(rows),
        palette=tuple(sorted(palette.items())),
        backing=backing,
    )
    if not upright:
        with m.step(title):
            m.place(sub, at=(x, y, z))
        return z + 3
    with m.step(f"{title}: brackets"):
        for c in range(0, width - 1, 2):
            m.add("99780", backing, at=(x + c, y - LIP, z), rot=BRACKET)
    with m.step(title):
        m.place(sub, at=(x, y - LIP - PANEL, z), rot=STAND)
    return z + 2.5 * height


def example(m):
    """A heart on a sky panel, standing on a base."""
    from emmet_dsl import Tile

    with m.step("Base"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(10, 3))
    with m.step("Base top"):
        m.row(Tile, "Dark Bluish Gray", at=(0, 0, 1), length=10, width=2)
        m.row(Plate, "Dark Bluish Gray", at=(0, 2, 1), length=10)
    m.use(
        "pixel_mosaic_panel",
        x=0,
        y=2,
        z=2,
        rows=[
            "BBBBBBBBBB",
            "BBKKBBKKBB",
            "BKRRKKRRKB",
            "BKRWRRRRKB",
            "BKRRRRRRKB",
            "BBKRRRRKBB",
            "BBBKRRKBBB",
            "BBBBKKBBBB",
        ],
        palette={"B": "Medium Blue", "K": "Black", "R": "Red", "W": "White"},
    )
