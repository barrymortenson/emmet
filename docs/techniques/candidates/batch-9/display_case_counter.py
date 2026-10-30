# SPDX-License-Identifier: Apache-2.0
"""Display case counter: a shop counter with a glass front and goods on show behind it, built apart and placed.

m.use("display_case_counter", x, y, z, length=4, facing="-y", color="White", goods=None, seed=0, title="Display case")
stands a counter `length` studs long (even, 4 or more) and 2 deep on the studs at height z, its footprint's min corner
at (x, y), its glass toward `facing` (the shopkeeper's side is behind it). On a 2-deep plate, the front row is windows 1
x 2 x 2 in `color` with clear glass; behind them, between a column of 1 x 1 bricks at each end, stand the goods, a round
1 x 1 brick on each stud in the colors of `goods` (a list) or picked from `seed`, seen through the glass. A course of
2-deep plates on top ties the frames to the columns, finished with tiles, 8 plates up. It's a submodel, built once in
its own steps, "<title>, glass", "<title>, goods", and "<title>, top", placed in a step titled "<title>"; it returns the
placed copy's Handle.
"""

import random

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["60592", "60601", "3005", "3062b", "3020", "87079"]

GOODS = [
    "Red",
    "Yellow",
    "Bright Pink",
    "Medium Azure",
    "Lime",
    "Orange",
    "Dark Brown",
    "White",
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


def _case(m, length, color, goods, title):
    """The counter in its own frame: `length` along x, 2 deep, its glass at y = 0."""
    with m.step(f"{title}, glass"):
        m.row(Plate, color, at=(0, 0, 0), length=length, width=2)
        for i in range(length // 2):
            f = m.add("60592", color, at=(2 * i, 0, 1), rot=1, id=f"window/{i + 1}")
            (x0, y0, z0), (x1, y1, _) = f.min, f.max
            m.add(
                "60601",
                "Trans-Clear",
                at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
                rot=1,
                id=f"window/{i + 1}/glass",
            )
    with m.step(f"{title}, goods"):
        for side, cx in (("left", 0), ("right", length - 1)):
            m.stack("3005", color, at=(cx, 1, 1), count=2, id=f"column/{side}")
        for i in range(1, length - 1):
            m.add("3062b", goods[i % len(goods)], at=(i, 1, 1), id=f"goods/{i}")
    with m.step(f"{title}, top"):
        m.row(Plate, color, at=(0, 0, 7), length=length, width=2)
        m.fill(Tile, color, at=(0, 0, 8), size=(length, 2))


def build(
    m,
    x,
    y,
    z,
    length=4,
    facing="-y",
    color="White",
    goods=None,
    seed=0,
    title="Display case",
):
    """The counter; returns the placed copy's Handle."""
    if length < 4 or length % 2:
        raise ValueError(f"length is even and 4 or more, not {length}")
    if goods is None:
        rng = random.Random(seed)
        goods = [rng.choice(GOODS) for _ in range(length)]
    sub = m.submodel(
        "display case",
        _case,
        length=length,
        color=color,
        goods=tuple(goods),
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, length, 2))


def example(m):
    """A display case 6 long along the back of a shop floor."""
    with m.step("Floor"):
        m.add("3036", "Sand Green", at=(0, 0, 0), rot=1)
    m.use("display_case_counter", x=1, y=3, z=1, length=6)
