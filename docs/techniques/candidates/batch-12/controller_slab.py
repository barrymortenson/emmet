# SPDX-License-Identifier: Apache-2.0
"""Controller slab: a flat retro game controller, a slab of bricks with studs on the sides tiled smooth all round, a
cross pad, two round buttons, and two small pill buttons on a dark face.

m.use("controller_slab", x, y, z, body="Light Bluish Gray", face="Black", pad="Black", buttons="Red", pills="Dark
Bluish Gray", title="Controller") builds, on the studs at height z over (x, y) to (x + 10, y + 4), a controller 6
plates thick: a layer of plates; a course of bricks whose front and back rows are bricks with studs on one side (30414
and 11211), so 1 x 4 and 1 x 2 tiles in `body` stand on them as smooth sides; a layer of plates in `face`; and on it a
plate 3 x 3 cross (15397) as the direction pad at the left, two 1 x 2 tiles in `pills` in the middle, two round tiles
2 x 2 (14769) in `buttons` at the right, and `face` tiles everywhere else, the largest that fit. The side tiles stand
0.4 studs out front and back. Returns the z of the top.
"""

from emmet_dsl import Brick, Plate, Rot
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 71374. Not derived from any
model file."""

PARTS = ["30414", "11211", "2431", "3069b", "15397", "14769", "3068b", "3070b"]

SIDE = {
    "front": Rot(y=-90, z=90),
    "back": Rot(y=90, z=90),
}  # a tile on its back, face to -y or +y, long along x
OUT = 0.4  # a tile's thickness, in studs
LENGTH, DEPTH = 10, 4
TILES = [
    ((2, 2), "3068b", 0),
    ((1, 2), "3069b", 0),
    ((2, 1), "3069b", 1),
    ((1, 1), "3070b", 0),
]


def build(
    m,
    x,
    y,
    z,
    body="Light Bluish Gray",
    face="Black",
    pad="Black",
    buttons="Red",
    pills="Dark Bluish Gray",
    title="Controller",
):
    """The slab, its sides, and its face; returns the z of the top."""
    with m.step(f"{title}: bottom"):
        m.fill(Plate, body, at=(x, y, z), size=(LENGTH, DEPTH))
    with m.step(f"{title}: front and back bricks"):
        for side, row, turn in (("front", y, 3), ("back", y + DEPTH - 1, 1)):
            m.add("30414", body, at=(x, row, z + 1), rot=turn)
            m.add("30414", body, at=(x + 4, row, z + 1), rot=turn)
            m.add("11211", body, at=(x + 8, row, z + 1), rot=turn)
    with m.step(f"{title}: middle bricks"):
        m.row(Brick, body, at=(x, y + 1, z + 1), length=LENGTH, width=2, first=2)
    for side, ty in (("front", y - OUT), ("back", y + DEPTH)):
        with m.step(f"{title}: {side} side"):
            # A side stud's middle is 1.75 plates up the brick; a tile on its side is 2.5 plates tall.
            m.add("2431", body, at=(x, ty, z + 1.5), rot=SIDE[side])
            m.add("2431", body, at=(x + 4, ty, z + 1.5), rot=SIDE[side])
            m.add("3069b", body, at=(x + 8, ty, z + 1.5), rot=SIDE[side])
    with m.step(f"{title}: face"):
        m.fill(Plate, face, at=(x, y, z + 4), size=(LENGTH, DEPTH))
    cells = {(c, r) for c in range(LENGTH) for r in range(DEPTH)}
    special = [("15397", pad, (0, 0), 0)]
    cells -= {(c, r) for c in range(3) for r in range(3)}
    for k in range(2):
        special.append(("3069b", pills, (4, 1 + k), 1))
        cells -= {(4, 1 + k), (5, 1 + k)}
        special.append(("14769", buttons, (6 + 2 * k, 1 if k else 0), 0))
        cells -= {(6 + 2 * k + i, (1 if k else 0) + j) for i in (0, 1) for j in (0, 1)}
    tiles = []
    for c, r in sorted(cells, key=lambda p: (p[1], p[0])):
        if (c, r) not in cells:
            continue
        for (tw, th), part, rot in TILES:
            patch = {(c + i, r + j) for i in range(tw) for j in range(th)}
            if patch <= cells:
                cells -= patch
                tiles.append((part, face, (c, r), rot))
                break
    in_steps(
        m,
        f"{title}: controls",
        [
            (
                1,
                lambda t=t: m.add(
                    t[0], t[1], at=(x + t[2][0], y + t[2][1], z + 5), rot=t[3]
                ),
            )
            for t in special + tiles
        ],
    )
    return z + 6


def example(m):
    """A controller on a mat."""
    with m.step("Mat"):
        m.fill(Plate, "Tan", at=(-1, -1, 0), size=(12, 6))
    m.use("controller_slab", x=0, y=0, z=1)
