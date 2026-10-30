# SPDX-License-Identifier: Apache-2.0
"""Dresser shelf: a kitchen dresser, a cupboard with drawers and a worktop below and an open rack of shelves above with
jars and cups in its cubbies, built apart and placed against a wall.

m.use("dresser_shelf", x, y, z, width=4, shelves=2, facing="-y", seed=0, color="Reddish Brown", drawers="Medium Nougat",
title="Dresser") stands a dresser `width` studs wide (4 or 6) and 2 deep on the studs at height z, its footprint's min
corner at (x, y), its front toward `facing`, so its back goes against a wall. The cupboard is a course of 2-deep bricks
and a course whose front is bricks with studs on the side carrying a 1 x 2 tile in `drawers` per drawer; a 2-deep plate
ties it, and tiles on its front row make the worktop, 8 plates up. The rack stands on the back row: a 1 x 1 brick at
each end and a plate across them for each of `shelves` shelves, a cubby a brick tall between, with a jar (a round brick)
or a cup on each stud, picked from `seed`, and a tile on top. It's a submodel, built once in its own steps, "<title>,
cupboard", "<title>, drawers", "<title>, worktop", and "<title>, shelf 1" and so on, placed in a step titled "<title>";
it returns the placed copy's Handle.
"""

import random

from emmet_dsl import Brick, Plate, Rot, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10354, set 21363. Not derived
from any model file."""

PARTS = [
    "3001",
    "30414",
    "3010",
    "3069b",
    "3020",
    "2431",
    "3005",
    "3710",
    "3062b",
    "3899",
]

PANEL = Rot(y=-90, z=90)  # a 1 x 2 tile on its back, its face to -y, long along x
JARS = ["Sand Blue", "Dark Orange", "Tan", "Sand Green", "White"]

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


def _dresser(m, width, shelves, seed, color, drawers, title):
    """The dresser in its own frame: `width` along x, 2 deep, its front at y = 0."""
    rng = random.Random(seed)
    with m.step(f"{title}, cupboard"):
        m.row(Brick, color, at=(0, 0, 0), length=width, width=2)
        m.add("30414", color, at=(0, 0, 3), rot=3, id="front/1")
        if width == 6:
            m.add("11211", color, at=(4, 0, 3), rot=3, id="front/2")
        m.row(Brick, color, at=(0, 1, 3), length=width)
    with m.step(f"{title}, drawers"):
        for i in range(width // 2):
            m.add(
                "3069b", drawers, at=(2 * i, -0.4, 3.5), rot=PANEL, id=f"drawer/{i + 1}"
            )
    with m.step(f"{title}, worktop"):
        m.row(Plate, color, at=(0, 0, 6), length=width, width=2)
        m.row(Tile, color, at=(0, 0, 7), length=width)
    z = 7
    for s in range(shelves):
        with m.step(f"{title}, shelf {s + 1}"):
            m.add("3005", color, at=(0, 1, z), id=f"rack/{s + 1}/left")
            m.add("3005", color, at=(width - 1, 1, z), id=f"rack/{s + 1}/right")
            for i in range(1, width - 1):
                if rng.random() < 0.6:
                    m.add(
                        "3062b", rng.choice(JARS), at=(i, 1, z), id=f"jar/{s + 1}-{i}"
                    )
                else:
                    # The cup's handle reaches 0.7 studs to its left: turned a quarter, it points out of the cubby.
                    m.add("3899", "White", at=(i, 0.3, z), rot=1, id=f"cup/{s + 1}-{i}")
            m.row(Plate, color, at=(0, 1, z + 3), length=width)
        z += 4
    with m.step(f"{title}, top"):
        m.row(Tile, color, at=(0, 1, z), length=width)


def build(
    m,
    x,
    y,
    z,
    width=4,
    shelves=2,
    facing="-y",
    seed=0,
    color="Reddish Brown",
    drawers="Medium Nougat",
    title="Dresser",
):
    """The dresser; returns the placed copy's Handle."""
    if width not in (4, 6):
        raise ValueError(f"width is 4 or 6, not {width}")
    sub = m.submodel(
        "dresser",
        _dresser,
        width=width,
        shelves=shelves,
        seed=seed,
        color=color,
        drawers=drawers,
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, width, 2))


def example(m):
    """A dresser 6 wide against the back of a kitchen floor."""
    with m.step("Floor"):
        m.add("3036", "Tan", at=(0, 0, 0), rot=1)
    m.use("dresser_shelf", x=1, y=4, z=1, width=6, seed=4)
