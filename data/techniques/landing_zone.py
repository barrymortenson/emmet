# SPDX-License-Identifier: Apache-2.0
"""Landing zone: a field of open studs the exact size of a building or module that's built apart and dropped on, framed
by a smooth ring of tiles with cut corners, so the seam is hidden and the module finds its place.

m.use("landing_zone", x, y, z, length=4, depth=4, seat="Dark Bluish Gray", path=("Tan", "Dark Tan"), seed=1,
title="Landing zone") lays, on the studs at height z (a baseplate or ground plates), a seat of plates length x depth
studs (even sizes, 2 to 12) from (x + 2, y + 2), its studs open for the module, and round it a ring of tiles 2 studs
wide, in strips of the longest 2-wide tiles, each a color from `path` at random, so the ring reads as a worn path. At
each of the ring's four outer corners a triangular tile 2 x 2 cuts the corner off, its square corner inward, leaving
the ground's studs showing on the outside half. The ring and the seat's top are level, so the module's base plate
sits flush with the path. It covers (length + 4) x (depth + 4) studs. seed picks the colors. Returns the z of the
seat's studs (where the module goes) and the (x, y) of its front-left corner, as (x, y, z).
"""

import random

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 21335 (a tiled landing zone for a drop-in module). Not derived from any model file."""

PARTS = [
    "3022",
    "3020",
    "3795",
    "3034",
    "3832",
    "3068b",
    "87079",
    "69729",
    "35787",
    "2445",
]

# A triangular tile's turn with its square corner at a ring corner's inner side, and the nudge that seats it
# (it's 1.85 studs, not 2, along its short sides).
CORNERS = {
    "front-left": (1, (0.15, 0.15)),
    "front-right": (2, (0, 0.15)),
    "back-right": (3, (0, 0)),
    "back-left": (0, (0.15, 0)),
}


def build(
    m,
    x,
    y,
    z,
    length=4,
    depth=4,
    seat="Dark Bluish Gray",
    path=("Tan", "Dark Tan"),
    seed=1,
    title="Landing zone",
):
    """The seat and its ring; returns the (x, y, z) where the module goes."""
    if length % 2 or depth % 2 or not 2 <= length <= 12 or not 2 <= depth <= 12:
        raise ValueError(
            f"landing_zone takes an even length and depth of 2 to 12, not {length}, {depth}"
        )
    if not path:
        raise ValueError("landing_zone needs at least one path color")
    rng = random.Random(seed)
    with m.step(f"{title}, seat"):
        m.fill(Plate, seat, at=(x + 2, y + 2, z), size=(length, depth))
    ring = []
    for at, n, axis in (
        ((x + 2, y, z), length, "x"),
        ((x + 2, y + depth + 2, z), length, "x"),
        ((x, y + 2, z), depth, "y"),
        ((x + length + 2, y + 2, z), depth, "y"),
    ):
        ring += row_actions(m, Tile, rng.choice(path), at, n, axis=axis, width=2)
    corners = {
        "front-left": (x, y),
        "front-right": (x + length + 2, y),
        "back-right": (x + length + 2, y + depth + 2),
        "back-left": (x, y + depth + 2),
    }
    for name, (cx, cy) in corners.items():
        rot, (dx, dy) = CORNERS[name]
        c = rng.choice(path)
        at = (cx + dx, cy + dy, z)
        ring.append((1, lambda c=c, at=at, rot=rot: m.add("35787", c, at=at, rot=rot)))
    in_steps(m, f"{title}, ring", ring)
    return (x + 2, y + 2, z + 1)


def example(m):
    """A 4 x 4 landing zone on grass, with a small module set on it."""
    with m.step("Grass"):
        m.add("41539", "Green", at=(0, 0, 0))
    sx, sy, sz = m.use("landing_zone", x=0, y=0, z=1, length=4, depth=4)

    def hut(s):
        with s.step("Base"):
            s.add("3031", "Tan", at=(0, 0, 0))
        s.wall(
            0, 0, 1, length=4, depth=4, color="White", rows=1, openings=[(1, 0), (2, 0)]
        )

    module = m.submodel("Module", hut)
    with m.step("Module on"):
        m.place(module, at=(sx, sy, sz))
