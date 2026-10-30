# SPDX-License-Identifier: Apache-2.0
"""Curved booth: a diner booth, a U of padded seat around a table on a round pedestal, built apart and placed.

m.use("curved_booth", x, y, z, facing="-y", color="Red", padding="Red", table="Medium Lavender", title="Booth") stands a
booth 6 x 4 studs on the studs at height z, its footprint's min corner at (x, y) (4 x 6 facing "+x" or "-x"), its open
side toward `facing`. On a 4 x 6 base plate, a U of seat (plates under tiles, 1 stud deep, the seat 3 plates up) runs
round three sides inside a U of back two bricks tall; the back's back corners are round bricks, the open ends round
bricks as bolsters, and its upper course behind the back seat is bricks with studs on one side carrying 1 x 2 tiles in
`padding` as the padded stripes. A row of plates caps the back, a 1 x 6 across the back corners tying them. The table in
the middle is a round 2 x 2 brick as the pedestal, a round 2 x 2 plate and a round tile in `table`, its top 6 plates up,
3 over the seat. It's a submodel, built once in its own steps, "<title>, seat", "<title>, back", "<title>, padding and
cap", and "<title>, table", placed in a step titled "<title>"; it returns the placed copy's Handle.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10352. Not derived from any
model file."""

PARTS = [
    "3032",
    "3623",
    "3023b",
    "63864",
    "3062b",
    "3622",
    "3010",
    "11211",
    "3069b",
    "3666",
    "3941",
    "4032b",
    "14769",
]

PANEL = Rot(y=-90, z=90)  # a 1 x 2 tile on its back, its face to -y, long along x

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


def _booth(m, color, padding, table, title):
    """The booth in its own frame: 6 along x, 4 deep, its open side at y = 0."""
    with m.step(f"{title}, seat"):
        m.add("3032", color, at=(0, 0, 0), rot=1, id="base")
        m.add("3623", color, at=(1, 0, 1), id="seat/left")
        m.add("3623", color, at=(4, 0, 1), id="seat/right")
        m.add("3023b", color, at=(2, 2, 1), rot=1, id="seat/back")
        m.add("63864", color, at=(1, 0, 2), id="seat/left/tile")
        m.add("63864", color, at=(4, 0, 2), id="seat/right/tile")
        m.add("3069b", color, at=(2, 2, 2), rot=1, id="seat/back/tile")
    with m.step(f"{title}, back"):
        for side, bx in (("left", 0), ("right", 5)):
            m.add("3622", color, at=(bx, 0, 1), id=f"back/{side}/1")
            m.add("3062b", color, at=(bx, 3, 1), id=f"back/{side}/corner/1")
        m.add("3010", color, at=(1, 3, 1), rot=1, id="back/1")
    with m.step(f"{title}, back, upper course"):
        for side, bx in (("left", 0), ("right", 5)):
            m.add("3062b", color, at=(bx, 0, 4), id=f"back/{side}/bolster")
            m.add("3004", color, at=(bx, 1, 4), id=f"back/{side}/2")
            m.add("3062b", color, at=(bx, 3, 4), id=f"back/{side}/corner/2")
        m.add("11211", color, at=(1, 3, 4), rot=3, id="back/2/left")
        m.add("11211", color, at=(3, 3, 4), rot=3, id="back/2/right")
    with m.step(f"{title}, padding and cap"):
        m.add("3069b", padding, at=(1, 2.6, 4.5), rot=PANEL, id="padding/left")
        m.add("3069b", padding, at=(3, 2.6, 4.5), rot=PANEL, id="padding/right")
        m.add("3666", color, at=(0, 3, 7), rot=1, id="cap/back")
        for side, bx in (("left", 0), ("right", 5)):
            m.add("3623", color, at=(bx, 0, 7), id=f"cap/{side}")
    with m.step(f"{title}, table"):
        m.add("3941", "Light Bluish Gray", at=(2, 0, 1), id="table/pedestal")
        m.add("4032b", "Light Bluish Gray", at=(2, 0, 4), id="table/plate")
        m.add("14769", table, at=(2, 0, 5), id="table/top")


def build(
    m,
    x,
    y,
    z,
    facing="-y",
    color="Red",
    padding="Red",
    table="Medium Lavender",
    title="Booth",
):
    """The booth; returns the placed copy's Handle."""
    sub = m.submodel(
        "booth", _booth, color=color, padding=padding, table=table, title=title
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 6, 4))


def example(m):
    """A booth against the back of a diner floor."""
    with m.step("Floor"):
        m.add("3036", "White", at=(0, 0, 0), rot=1)
    m.use("curved_booth", x=1, y=2, z=1, padding="Bright Light Orange")
