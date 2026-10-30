# SPDX-License-Identifier: Apache-2.0
"""Curtain wall glazing: a modern glass building at micro scale, columns of transparent bricks between dark 1 x 1
mullions, with a dark plate band at every floor.

m.use("curtain_wall_glazing", x, y, z, length=7, depth=4, floors=2, rows=3, glass="Trans Light Blue", frame="Dark Bluish
Gray", roof="Light Bluish Gray", title="Glass block") builds, on the studs at height z, the four walls of a length x
depth box from (x, y), a stud thick. Along each wall a mullion (a 1 x 1 brick in the frame color) stands every third
stud, the corners included, and the 2-stud bays between take 1 x 2 transparent bricks (3065), `rows` courses of them a
floor (3 in the books), stacked so the glass runs in columns. A ring of plates in the frame color caps each floor as the
slab band and ties the walls; the top one is filled across and tiled in the roof color. length and depth are each 3k + 1
(4, 7, 10, 13). Returns the z of the roof's top.
"""

from emmet_dsl import Brick, Plate, Tile
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10272. Not derived from any
model file."""

PARTS = [
    "3065",
    "3005",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3020",
    "3022",
    "3068b",
    "87079",
    "3069b",
]


def _cells(n):
    """The pieces along a wall n studs long: ("m", i) a mullion at i, ("g", i) a 2-stud bay from i."""
    return [("m", i) if i % 3 == 0 else ("g", i) for i in range(n) if i % 3 != 2]


def build(
    m,
    x,
    y,
    z,
    length=7,
    depth=4,
    floors=2,
    rows=3,
    glass="Trans Light Blue",
    frame="Dark Bluish Gray",
    roof="Light Bluish Gray",
    title="Glass block",
):
    """The walls a course at a time, the slab bands, and the roof; returns the z of the top."""
    if (
        length % 3 != 1
        or depth % 3 != 1
        or length < 4
        or depth < 4
        or floors < 1
        or not 1 <= rows <= 4
    ):
        raise ValueError(
            "curtain_wall_glazing takes a length and depth of 3k + 1 (4, 7, 10, …), 1 or more floors, "
            "and 1 to 4 rows, "
            f"not {length}, {depth}, {floors}, {rows}"
        )
    zf = z
    for f in range(floors):
        for r in range(rows):
            zr = zf + 3 * r
            actions = []
            # Front and back own the corners; the sides fill between them.
            for wy in (y, y + depth - 1):
                for kind, i in _cells(length):
                    if kind == "m":
                        actions.append(
                            (
                                1,
                                lambda i=i, wy=wy: m.add(
                                    Brick(1, 1), frame, at=(x + i, wy, zr)
                                ),
                            )
                        )
                    else:
                        actions.append(
                            (
                                1,
                                lambda i=i, wy=wy: m.add(
                                    "3065", glass, at=(x + i, wy, zr), rot=1
                                ),
                            )
                        )
            for wx in (x, x + length - 1):
                for kind, j in _cells(depth)[1:-1]:
                    if kind == "m":
                        actions.append(
                            (
                                1,
                                lambda j=j, wx=wx: m.add(
                                    Brick(1, 1), frame, at=(wx, y + j, zr)
                                ),
                            )
                        )
                    else:
                        actions.append(
                            (
                                1,
                                lambda j=j, wx=wx: m.add(
                                    "3065", glass, at=(wx, y + j, zr)
                                ),
                            )
                        )
            in_steps(m, f"{title}, floor {f + 1}: row {r + 1}", actions)
        zs = zf + 3 * rows
        if f < floors - 1:
            with m.step(f"{title}, floor {f + 1}: slab"):
                m.row(Plate, frame, at=(x, y, zs), length=length)
                m.row(Plate, frame, at=(x, y + depth - 1, zs), length=length)
                m.row(Plate, frame, at=(x, y + 1, zs), length=depth - 2, axis="y")
                m.row(
                    Plate,
                    frame,
                    at=(x + length - 1, y + 1, zs),
                    length=depth - 2,
                    axis="y",
                )
        else:
            with m.step(f"{title}: roof plates"):
                m.fill(Plate, frame, at=(x, y, zs), size=(length, depth))
            with m.step(f"{title}: roof"):
                m.fill(Tile, roof, at=(x, y, zs + 1), size=(length, depth))
            return zs + 2
        zf = zs + 1


def example(m):
    """A two-floor glass block on a paved lot."""
    with m.step("Lot"):
        m.add(Plate(6, 8), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("curtain_wall_glazing", x=0, y=1, z=1, length=7, depth=4, floors=2, rows=2)
