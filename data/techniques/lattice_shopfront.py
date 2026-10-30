# SPDX-License-Identifier: Apache-2.0
"""Lattice shopfront: a pub window, clear panes over a paneled base with diamond-lattice lights above them.

m.use("lattice_shopfront", x, y, z, panes=2, frame="Reddish Brown", lattice="Pearl Gold", panel="Dark Green",
plinth="Black", trim="Medium Nougat", title="Shopfront") builds, in a wall one stud thick along x facing front (-y), a
shopfront `panes` windows wide (2 to 4, 2 studs each) from (x, y): a Plate 2 x n in the plinth color at z, its back row
in the wall and its front row a stud out under a tile ledge; a course of Bricks 1 x 2 in the panel color on its back
row; a sill plate; a row of Windows 1 x 2 x 2 with clear glass (60592, 60601); a plate; a row of the same windows with a
diamond-lattice pane (38320) in the lattice color; and a plate in the trim color on top. It takes the wall's cells x to
x + 2 * panes - 1 from z to z + 19. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = [
    "60592",
    "60601",
    "38320",
    "3004",
    "3710",
    "3666",
    "3460",
    "3020",
    "3795",
    "3832",
    "2431",
    "6636",
    "4162",
]

PLATES = {4: "3710", 6: "3666", 8: "3460"}  # Plate 1 x n
WIDE = {4: "3020", 6: "3795", 8: "3832"}  # Plate 2 x n
TILES = {4: "2431", 6: "6636", 8: "4162"}  # Tile 1 x n


def _windows(m, x, y, z, n, frame, pane, lift):
    """A row of n Windows 1 x 2 x 2 from (x, y, z), each with `pane` set `lift` plates above the frame's bottom."""
    for k in range(n):
        f = m.add("60592", frame, at=(x + 2 * k, y, z), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            pane[0],
            pane[1],
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + lift),
            rot=1,
        )


def build(
    m,
    x,
    y,
    z,
    panes=2,
    frame="Reddish Brown",
    lattice="Pearl Gold",
    panel="Dark Green",
    plinth="Black",
    trim="Medium Nougat",
    title="Shopfront",
):
    """The plinth, the panel, and the two rows of windows; returns the z of the top."""
    n = 2 * panes
    if n not in PLATES:
        raise ValueError(f"lattice_shopfront takes 2 to 4 panes, not {panes}")
    with m.step(f"{title}: plinth and panel"):
        m.add(WIDE[n], plinth, at=(x, y - 1, z), rot=1)
        m.add(TILES[n], plinth, at=(x, y - 1, z + 1), rot=1)
        for k in range(panes):
            m.add("3004", panel, at=(x + 2 * k, y, z + 1), rot=1)
    with m.step(f"{title}: sill and lower windows"):
        m.add(PLATES[n], frame, at=(x, y, z + 4), rot=1)
        _windows(m, x, y, z + 5, panes, frame, ("60601", "Trans-Clear"), 0.875)
    with m.step(f"{title}: lattice lights"):
        m.add(PLATES[n], frame, at=(x, y, z + 11), rot=1)
        _windows(m, x, y, z + 12, panes, frame, ("38320", lattice), 0.9375)
    with m.step(f"{title}: head"):
        m.add(PLATES[n], trim, at=(x, y, z + 18), rot=1)
    return z + 19


def example(m):
    """A two-pane shopfront between two piers of a pub wall."""
    from emmet_dsl import Brick

    z = m.use("lattice_shopfront", x=0, y=0, z=0)
    for r in range(6):
        with m.step(f"Piers, row {r + 1}"):
            for cx in (-1, 4):
                m.add("3005", "Dark Green", at=(cx, 0, 3 * r))
    with m.step("Piers, top"):
        for cx in (-1, 4):
            m.add("3024", "Dark Green", at=(cx, 0, 18))
    with m.step("Lintel"):
        m.row(Brick, "Dark Green", at=(-1, 0, z), length=6)
