# SPDX-License-Identifier: Apache-2.0
"""Table and chairs: a small round table on one leg, or a long table on four, with up to four chairs set round it, built
apart and placed.

m.use("table_and_chairs", x, y, z, shape="round", chairs=2, facing="-y", wood="Reddish Brown", top=None, title="Table")
stands a table set on the studs at height z, its footprint's min corner at (x, y). shape="round" is a 2 x 2 round top on
a single round leg, centered on a round foot plate with one middle stud; the set is 4 x 4 studs. shape="long" is a 2 x 4
tile top on a plate and four round legs; the set is 6 x 4 (4 x 6 facing "+x" or "-x"). Either top is 5 plates up, 2
plates over a chair's seat. chairs (0 to 4) are set facing the table: left, right, front, then back. A chair is two
round plates as its leg and a bracket 1 x 1 - 1 x 1 as its seat and back, the seat 3 plates up, with a tile on the back,
which reaches 0.6 studs past the set's footprint; it's its own submodel. top colors the tabletop (wood by default). The
set is a submodel, built once in its own steps, "<title>, table" and "<title>, chairs" (each chair "Chair, leg and
seat"), placed in a step titled "<title>"; it returns the placed copy's Handle.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 43217, set 21335.
Not derived from any model file."""

PARTS = ["18674", "3062b", "14769", "3020", "87079", "6141", "36840", "3070b"]

BACK = Rot(x=-90)  # a 1 x 1 tile on its back, its face to +y

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


# Where each chair goes round each table, in the set's own frame: (x, y, facing toward the table).
SEATS = {
    "round": [(0, 1, "+x"), (3, 2, "-x"), (2, 0, "+y"), (1, 3, "-y")],
    "long": [(0, 1, "+x"), (5, 2, "-x"), (2, 0, "+y"), (3, 3, "-y")],
}


def _chair(m, wood):
    """A chair in its own frame: 1 x 1, its back at y = 1."""
    with m.step("Chair, leg and seat"):
        m.stack("6141", wood, at=(0, 0, 0), count=2, id="leg")
        # The bracket's upright is on its +x side; turned a quarter it stands at the back.
        m.add("36840", wood, at=(0, 0, 2), rot=1, id="seat")
        m.add("3070b", wood, at=(0, 1.2, 2), rot=BACK, id="back")


def _set(m, shape, chairs, wood, top, title):
    """The table and its chairs in the set's own frame."""
    with m.step(f"{title}, table"):
        if shape == "round":
            m.add("18674", wood, at=(1, 1, 0), id="foot")
            # The foot's one stud is in its middle, so the leg and the round top sit half a stud off the grid.
            m.add("3062b", wood, at=(1.5, 1.5, 1), id="leg")
            m.add("14769", top, at=(1, 1, 4), id="top")
        else:
            for i, (lx, ly) in enumerate(((1, 1), (4, 1), (1, 2), (4, 2))):
                m.add("3062b", wood, at=(lx, ly, 0), id=f"leg/{i + 1}")
            m.add("3020", wood, at=(1, 1, 3), rot=1, id="frame")
            m.add("87079", top, at=(1, 1, 4), rot=1, id="top")
    if chairs:
        chair = m.submodel("chair", _chair, wood=wood)
        with m.step(f"{title}, chairs"):
            for cx, cy, facing in SEATS[shape][:chairs]:
                _put(m, chair, cx, cy, 0, facing, (0, 0, 1, 1))


def build(
    m,
    x,
    y,
    z,
    shape="round",
    chairs=2,
    facing="-y",
    wood="Reddish Brown",
    top=None,
    title="Table",
):
    """The table set; returns the placed copy's Handle."""
    if shape not in SEATS:
        raise ValueError(f'shape must be "round" or "long", not {shape!r}')
    if not 0 <= chairs <= 4:
        raise ValueError(f"chairs is 0 to 4, not {chairs}")
    sub = m.submodel(
        "table set",
        _set,
        shape=shape,
        chairs=chairs,
        wood=wood,
        top=top or wood,
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 4 if shape == "round" else 6, 4))


def example(m):
    """A round table with two chairs and a long one with four."""
    with m.step("Floor"):
        m.add("3033", "Tan", at=(0, 0, 0), rot=1)
    m.use("table_and_chairs", x=0, y=1, z=1)
    m.use(
        "table_and_chairs",
        x=6,
        y=0,
        z=1,
        shape="long",
        chairs=4,
        facing="+x",
        top="Dark Blue",
    )
