# SPDX-License-Identifier: Apache-2.0
"""Triangle tile floor: a patterned stone floor of 2 x 2 triangular tiles in two colors, set so the triangles meet in
diamonds and stars, for the floor of a hall or a cathedral at midi scale.

m.use("triangle_tile_floor", x, y, z, blocks=(2, 2), inner="White", outer="Black", open_cells=(), title="Floor") tiles
the studs at height z over blocks[0] x blocks[1] blocks of 4 x 4 studs from (x, y). Each 2 x 2 cell takes two triangular
tiles (35787) along its diagonal: the one whose right angle points to its block's middle in `inner`, the other in
`outer`, so the inner halves make a diamond in each block and the outer halves make diamonds where four blocks meet.
open_cells: (column, row) cells, counted in 2-stud cells from (x, y), left bare for columns to stand on. Returns the z
of the tiles' top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file."""

PARTS = ["35787"]

# A triangular tile's body is 1.85 studs square; where its min corner goes in a 2 x 2 cell, by quarter turn, so the
# right angle sits in a corner of the cell: turn 0 at +x -y, 1 at +x +y, 2 at -x +y, 3 at -x -y.
SEAT = {0: (0.15, 0), 1: (0.15, 0.15), 2: (0, 0.15), 3: (0, 0)}
# By a cell's place in its block: the turns of its inner and outer triangle.
TURNS = {(0, 0): (1, 3), (1, 0): (2, 0), (0, 1): (0, 2), (1, 1): (3, 1)}


def build(
    m,
    x,
    y,
    z,
    blocks=(2, 2),
    inner="White",
    outer="Black",
    open_cells=(),
    title="Floor",
):
    """The triangles, a block at a time; returns the z of the tiles' top."""
    bx, by = blocks
    if not (1 <= bx <= 8 and 1 <= by <= 8):
        raise ValueError(
            f"triangle_tile_floor takes 1 to 8 blocks each way, not {blocks}"
        )
    skip = {tuple(c) for c in open_cells}
    for j in range(by):
        for i in range(bx):
            with m.step(f"{title}, block {j * bx + i + 1}"):
                for (a, b), (t_in, t_out) in TURNS.items():
                    cell = (2 * i + a, 2 * j + b)
                    if cell in skip:
                        continue
                    cx, cy = x + 2 * cell[0], y + 2 * cell[1]
                    for turn, color in ((t_in, inner), (t_out, outer)):
                        dx, dy = SEAT[turn]
                        m.add("35787", color, at=(cx + dx, cy + dy, z), rot=turn)
    return z + 1


def example(m):
    """A floor of 2 x 2 blocks with two cells left bare for columns."""
    from emmet_dsl import Plate

    with m.step("Floor plates"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(8, 8))
    m.use("triangle_tile_floor", x=0, y=0, z=1, open_cells=[(0, 1), (3, 2)])
    with m.step("Columns"):
        for cx, cy in ((0, 2), (6, 4)):
            m.stack("3941", "Tan", at=(cx, cy, 1), count=2)
