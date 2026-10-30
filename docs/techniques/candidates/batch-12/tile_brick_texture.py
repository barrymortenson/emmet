# SPDX-License-Identifier: Apache-2.0
"""Tile brick texture: a pixel-art brick wall or ground block, drawn by the seams between small tiles on a darker
backing, the mortar lines showing where the tiles meet.

m.use("tile_brick_texture", x, y, z, length=8, rows=4, bond="running", brick="Dark Orange", mortar="Reddish Brown",
upright=True, title="Brick texture") builds a panel `length` studs wide and `rows` tall: two crossed layers of plates
in `mortar`, then tiles in `brick`, one course a row: bond="running" lays 1 x 2 tiles with every other course starting
with a 1 x 1 (a running bond, each brick two pixels long), bond="grid" lays 1 x 1 tiles (square blocks). upright=True
stands it face front (-y) on brackets 1 x 2 - 1 x 2 (99780) on the studs at height z, row y from x, like
pixel_mosaic_panel: the panel covers y - 1.4 to y - 0.2, from z up 2.5 plates a row, resting on the 2 rows in front of
the brackets (keep them smooth at z, or empty). upright=False lays it studs up on the studs at (x, y, z) as a floor or
the top of a ground block. Returns the z of the panel's top.
"""

from emmet_dsl import Plate, Rot
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 71374. Not derived from any
model file."""

PARTS = ["99780", "3069b", "3070b"]

STAND = Rot(
    x=90
)  # the panel lying studs up, turned up about x: its studs face front (-y), its top row up
PANEL = 1.2  # the panel's thickness: two plates and a tile, in studs
LIP = 0.2  # how far a bracket's side face stands in front of its plate, in studs
BONDS = ("running", "grid")


def _panel(m, length, rows, bond, brick, mortar):
    """The panel lying studs up: x across, y up the wall."""
    with m.step("Backing, up"):
        for c in range(0, length, 2):
            m.row(
                Plate,
                mortar,
                at=(c, 0, 0),
                length=rows,
                axis="y",
                width=min(2, length - c),
            )
    with m.step("Backing, across"):
        for r in range(0, rows, 2):
            m.row(
                Plate,
                mortar,
                at=(0, r, 1),
                length=length,
                axis="x",
                width=min(2, rows - r),
            )
    actions = []
    for r in range(rows):
        c = 0
        if bond == "running" and r % 2:
            actions.append((1, lambda r=r: m.add("3070b", brick, at=(0, r, 2))))
            c = 1
        while c < length:
            if bond == "running" and c + 2 <= length:
                actions.append(
                    (1, lambda c=c, r=r: m.add("3069b", brick, at=(c, r, 2), rot=1))
                )
                c += 2
            else:
                actions.append(
                    (1, lambda c=c, r=r: m.add("3070b", brick, at=(c, r, 2)))
                )
                c += 1
    in_steps(m, "Tiles", actions)


def build(
    m,
    x,
    y,
    z,
    length=8,
    rows=4,
    bond="running",
    brick="Dark Orange",
    mortar="Reddish Brown",
    upright=True,
    title="Brick texture",
):
    """The panel and, standing, its brackets; returns the z of the panel's top."""
    if bond not in BONDS:
        raise ValueError(
            f"tile_brick_texture: bond must be one of {', '.join(BONDS)}, not {bond!r}"
        )
    if length < 2 or rows < 1:
        raise ValueError(
            f"tile_brick_texture needs a length of 2 or more and 1 or more rows, not {length}, {rows}"
        )
    sub = m.submodel(
        f"{title}: panel",
        _panel,
        length=length,
        rows=rows,
        bond=bond,
        brick=brick,
        mortar=mortar,
    )
    if not upright:
        with m.step(title):
            m.place(sub, at=(x, y, z))
        return z + 3
    with m.step(f"{title}: brackets"):
        for c in range(0, length - 1, 2):
            m.add("99780", mortar, at=(x + c, y - LIP, z), rot=3)
    with m.step(title):
        m.place(sub, at=(x, y - LIP - PANEL, z), rot=STAND)
    return z + 2.5 * rows


def example(m):
    """A standing brick wall and a floor of square blocks in front of it."""
    from emmet_dsl import Tile

    with m.step("Base"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, -4, 0), size=(8, 7))
    with m.step("Base top"):
        m.row(Tile, "Dark Bluish Gray", at=(0, 0, 1), length=8, width=2)
        m.row(Plate, "Dark Bluish Gray", at=(0, 2, 1), length=8)
    m.use("tile_brick_texture", x=0, y=2, z=2, rows=4)
    m.use(
        "tile_brick_texture",
        x=0,
        y=-4,
        z=1,
        length=8,
        rows=4,
        bond="grid",
        upright=False,
        title="Floor",
    )
