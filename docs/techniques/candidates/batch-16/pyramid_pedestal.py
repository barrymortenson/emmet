# SPDX-License-Identifier: Apache-2.0
"""Pyramid pedestal: a dark tapered stand that holds a bust, a head, or a helmet up off a square plinth, with a
nameplate set flush in the plinth's front.

m.use("pyramid_pedestal", x, y, z, height=3, color="Black", name_color="Pearl Gold", title="Pedestal") builds, on the
studs at height z, a plinth 12 x 12 studs from (x, y): a layer of plates, a layer a stud in from the edge with its seams
crossed, and a ring of tiles round its top edge with a tile 1 x 4 in `name_color` in the middle of the front, the
nameplate. On the plinth a ring of 45-degree slopes (double convex slopes at the corners, slopes 2 x 2 between) falls
outward round a 4 x 4 core of bricks, and on the core four double convex slopes meet in a low pyramid whose four studs
carry a column of bricks 2 x 2, `height` bricks tall (1 to 8), its top 4 studs open for the sculpture (on a plate 2 x 2
or a turntable). The column stands over (x + 5, y + 5). Returns the z of the column's top.
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43014. Not derived from any
model file."""

PARTS = [
    "3045",
    "3039",
    "3001",
    "3003",
    "2431",
    "3069b",
    "3070b",
    "6636",
    "3958",
    "3036",
    "3032",
    "3795",
]

# A double convex slope's peak is at its -x -y corner at rot 0, so it falls to +x and +y; a quarter turn further for
# each corner going round counterclockwise. A slope 2 x 2 falls toward +x at rot 0.
TURNS = {(1, 1): 0, (0, 1): 1, (0, 0): 2, (1, 0): 3}
FALL = {"front": 3, "back": 1, "left": 2, "right": 0}


def build(
    m, x, y, z, height=3, color="Black", name_color="Pearl Gold", title="Pedestal"
):
    """The plinth, the slope ring, the pyramid, and the column; returns the z of the top."""
    if not 1 <= height <= 8:
        raise ValueError(
            f"pyramid_pedestal takes a height of 1 to 8 bricks, not {height}"
        )
    with m.step(f"{title}: plinth"):
        m.fill(Plate, color, at=(x, y, z), size=(12, 12))
    with m.step(f"{title}: plinth, second layer"):
        # Strips the other way, a stud in, so they cross the first layer's seams.
        for i in range(1, 11, 2):
            m.row(Plate, color, at=(x + i, y + 1, z + 1), length=10, axis="y", width=2)
    # The ring of tiles round the slopes, the nameplate in the middle of the front.
    ring = row_actions(m, Tile, color, (x + 1, y + 1, z + 2), 3)
    ring.append((1, lambda: m.add("2431", name_color, at=(x + 4, y + 1, z + 2), rot=1)))
    ring += row_actions(m, Tile, color, (x + 8, y + 1, z + 2), 3)
    ring += row_actions(m, Tile, color, (x + 1, y + 10, z + 2), 10)
    ring += row_actions(m, Tile, color, (x + 1, y + 2, z + 2), 8, axis="y")
    ring += row_actions(m, Tile, color, (x + 10, y + 2, z + 2), 8, axis="y")
    in_steps(m, f"{title}: plinth, tiles and nameplate", ring)
    zs = z + 2
    with m.step(f"{title}: core"):
        m.add("3001", color, at=(x + 4, y + 4, zs), rot=1)  # Brick 2 x 4, along x
        m.add("3001", color, at=(x + 4, y + 6, zs), rot=1)
    with m.step(f"{title}: slope ring, corners"):
        for (i, j), rot in TURNS.items():
            m.add("3045", color, at=(x + 2 + 6 * i, y + 2 + 6 * j, zs), rot=rot)
    with m.step(f"{title}: slope ring, sides"):
        for a in (4, 6):
            m.add("3039", color, at=(x + a, y + 2, zs), rot=FALL["front"])
            m.add("3039", color, at=(x + a, y + 8, zs), rot=FALL["back"])
            m.add("3039", color, at=(x + 2, y + a, zs), rot=FALL["left"])
            m.add("3039", color, at=(x + 8, y + a, zs), rot=FALL["right"])
    zp = zs + 3
    with m.step(f"{title}: pyramid"):
        for (i, j), rot in TURNS.items():
            m.add("3045", color, at=(x + 4 + 2 * i, y + 4 + 2 * j, zp), rot=rot)
    # The four peaks' studs meet in the middle: the column stands on them.
    zc = zp + 3
    with m.step(f"{title}: column"):
        m.stack("3003", color, at=(x + 5, y + 5, zc), count=height)
    return zc + 3 * height


def example(m):
    """A pedestal three bricks tall holding a small round head."""
    top = m.use("pyramid_pedestal", x=0, y=0, z=0, height=3)
    with m.step("Head"):
        m.add("3941", "White", at=(5, 5, top))  # Round brick 2 x 2 as a neck
        m.add("60474", "White", at=(4, 4, top + 3))  # Round plate 4 x 4
    with m.step("Head, dome"):
        for (i, j), rot in TURNS.items():
            m.add(
                "5852", "White", at=(4 + 2 * i, 4 + 2 * j, top + 4), rot=rot
            )  # Quarter circle 2 x 2
