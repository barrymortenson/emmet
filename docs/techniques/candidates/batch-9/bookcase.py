# SPDX-License-Identifier: Apache-2.0
"""Bookcase: a tall bookcase for a figure's room, its shelves full of books whose spines are 1 x 2 tiles standing on
side studs, built apart and placed against a wall.

m.use("bookcase", x, y, z, width=4, shelves=3, facing="-y", seed=0, color="Reddish Brown", books=None, title="Bookcase")
stands a bookcase `width` studs wide (even, 4 or more) and 2 deep on the studs at height z, its footprint's min corner
at (x, y), its books toward `facing`, so its back goes against a wall. Each of `shelves` shelves is a 2-deep plate, then
a side at each end (a brick 1 x 2 x 1 2/3 with studs on one side, studs out, under a 2 x 2 tile so the side is smooth)
and, along the back between them, bricks 1 x 2 x 1 2/3 with studs on one side, studs forward, carrying the books: a 1 x
2 tile standing on its end on each column of studs, 5 plates tall, in the colors of `books` (a list) or picked from
`seed`, with now and then a gap. A plate and tiles cap it, 20 plates up for 3 shelves. The side tiles stand 0.4 studs
proud of the footprint each side. It's a submodel, built once in its own steps, "<title>, shelf 1", "<title>, shelf 1,
books", and so on, and "<title>, top", placed in a step titled "<title>"; it returns the placed copy's Handle.
"""

import random

from emmet_dsl import Plate, Rot, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["3020", "22885", "3068b", "3069b", "87079"]

SPINE = Rot(x=90)  # a 1 x 2 tile on its back, its face to -y, its long side up
SIDE = {
    "left": (2, Rot(x=90, z=270)),
    "right": (0, Rot(x=90, z=90)),
}  # the side brick's turn; the tile facing out
BOOKS = [
    "Dark Red",
    "Dark Blue",
    "Dark Green",
    "Dark Tan",
    "Sand Blue",
    "Black",
    "Dark Orange",
    "Tan",
]

FACING = {
    "-y": 0,
    "+x": 1,
    "+y": 2,
    "-x": 3,
}  # quarter turns that point a front built toward -y the given way


def _turn(px, py, k):
    for _ in range(k):
        px, py = -py, px
    return px, py


def _put(m, sub, x, y, z, facing, foot):
    """Places sub so the rectangle foot = (x0, y0, x1, y1) of its own frame lands with its min corner at (x, y) and
    its own z = 0 at z, its front (built toward -y) toward `facing`."""
    if facing not in FACING:
        raise ValueError(f"facing must be one of {', '.join(FACING)}, not {facing!r}")
    k = FACING[facing]
    (bx0, by0, bz0), (bx1, by1, _) = sub.box
    box = [_turn(px, py, k) for px in (bx0, bx1) for py in (by0, by1)]
    feet = [_turn(px, py, k) for px in (foot[0], foot[2]) for py in (foot[1], foot[3])]
    lo = (min(p[0] for p in box), min(p[1] for p in box))
    fl = (min(p[0] for p in feet), min(p[1] for p in feet))
    return m.place(sub, at=(x - fl[0] + lo[0], y - fl[1] + lo[1], z + bz0), rot=k)


def _bookcase(m, width, shelves, seed, color, books, title):
    """The bookcase in its own frame: `width` along x, 2 deep, its books facing y = 0."""
    rng = random.Random(seed)
    for s in range(shelves):
        z = 6 * s
        with m.step(f"{title}, shelf {s + 1}"):
            m.row(Plate, color, at=(0, 0, z), length=width, width=2)
            for side, sx in (("left", 0), ("right", width - 1)):
                m.add(
                    "22885",
                    color,
                    at=(sx, 0, z + 1),
                    rot=SIDE[side][0],
                    id=f"side/{side}/{s + 1}",
                )
            for j in range((width - 2) // 2):
                m.add(
                    "22885",
                    color,
                    at=(1 + 2 * j, 1, z + 1),
                    rot=3,
                    id=f"back/{s + 1}/{j + 1}",
                )
        with m.step(f"{title}, shelf {s + 1}, books"):
            for side, sx in (("left", -0.4), ("right", width)):
                m.add(
                    "3068b",
                    color,
                    at=(sx, 0, z + 1),
                    rot=SIDE[side][1],
                    id=f"side/{side}/{s + 1}/tile",
                )
            for i in range(1, width - 1):
                if rng.random() < 0.15:
                    continue
                m.add(
                    "3069b",
                    rng.choice(books),
                    at=(i, 0.6, z + 1),
                    rot=SPINE,
                    id=f"book/{s + 1}-{i}",
                )
    with m.step(f"{title}, top"):
        m.row(Plate, color, at=(0, 0, 6 * shelves), length=width, width=2)
        m.fill(Tile, color, at=(0, 0, 6 * shelves + 1), size=(width, 2))


def build(
    m,
    x,
    y,
    z,
    width=4,
    shelves=3,
    facing="-y",
    seed=0,
    color="Reddish Brown",
    books=None,
    title="Bookcase",
):
    """The bookcase; returns the placed copy's Handle."""
    if width < 4 or width % 2:
        raise ValueError(f"width is even and 4 or more, not {width}")
    sub = m.submodel(
        "bookcase",
        _bookcase,
        width=width,
        shelves=shelves,
        seed=seed,
        color=color,
        books=tuple(books or BOOKS),
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, width, 2))


def example(m):
    """A bookcase 6 wide against the back of a study floor."""
    with m.step("Floor"):
        m.add("3036", "Medium Nougat", at=(0, 0, 0), rot=1)
    m.use("bookcase", x=1, y=4, z=1, width=6, seed=2)
