# SPDX-License-Identifier: Apache-2.0
"""Kitchen counter: a run of kitchen units 2 deep, each 2 studs wide, a cabinet, a stove, or a sink, under one worktop,
built apart and placed along a wall.

m.use("kitchen_counter", x, y, z, units=("cabinet", "stove", "sink"), facing="-y", color="White", doors="Sand Blue",
top="Light Bluish Gray", title="Counter") stands a counter on the studs at height z, its footprint's min corner at (x,
y), its doors toward `facing`: 2 × len(units) studs along the wall and 2 deep. Each unit is a plain brick at the back
and a brick with studs on one side at the front carrying its door: a 1 x 2 tile in `doors` for a cabinet or a sink, a
black grille tile for the stove's oven. A course of plates in `top` ties the units, and each unit tops it with its own:
a 2 x 2 tile for a cabinet, a black tile and two round burners for the stove, a trans-light-blue tile as the basin and a
tap for the sink. The worktop is 5 plates up, 2 plates over a figure's seat. It's a submodel, built once in its own
steps, "<title>, units", "<title>, doors", "<title>, worktop", and "<title>, tops" (split into parts of up to 8), placed
in a step titled "<title>"; it returns the placed copy's Handle.
"""

from emmet_dsl import Plate, Rot
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43268, set 10352. Not derived
from any model file."""

PARTS = ["3004", "11211", "3069b", "2412b", "3795", "3068b", "98138", "4599b"]

PANEL = Rot(y=-90, z=90)  # a 1 x 2 tile on its back, its face to -y, long along x
UNITS = ("cabinet", "stove", "sink")

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


def _unit(m, i, unit, color):
    """One unit's body: a brick at the back and a brick with side studs at the front."""
    ux = 2 * i
    m.add("3004", color, at=(ux, 1, 0), rot=1, id=f"{unit}/{i + 1}/back")
    m.add("11211", color, at=(ux, 0, 0), rot=3, id=f"{unit}/{i + 1}/front")


def _door(m, i, unit, doors):
    """One unit's door, on the side studs of its front brick."""
    ux = 2 * i
    door = ("2412b", "Black") if unit == "stove" else ("3069b", doors)
    m.add(door[0], door[1], at=(ux, -0.4, 0.5), rot=PANEL, id=f"{unit}/{i + 1}/door")


def _top(m, i, unit, top):
    """One unit's top, on the worktop's plates."""
    ux = 2 * i
    if unit == "cabinet":
        m.add("3068b", top, at=(ux, 0, 4), id=f"cabinet/{i + 1}/top")
    elif unit == "stove":
        m.add("3069b", "Black", at=(ux, 0, 4), rot=1, id=f"stove/{i + 1}/top")
        m.add("98138", "Black", at=(ux, 1, 4), id=f"stove/{i + 1}/burner/1")
        m.add("98138", "Black", at=(ux + 1, 1, 4), id=f"stove/{i + 1}/burner/2")
    else:
        m.add(
            "3069b", "Trans-Light Blue", at=(ux, 0, 4), rot=1, id=f"sink/{i + 1}/basin"
        )
        # The tap's spout reaches a stud to its left: turned a quarter, it reaches forward over the basin.
        m.add(
            "4599b",
            "Light Bluish Gray",
            at=(ux + 0.1, 0.5, 4),
            rot=1,
            id=f"sink/{i + 1}/tap",
        )
        m.add("98138", top, at=(ux + 1, 1, 4), id=f"sink/{i + 1}/tile")


def _counter(m, units, color, doors, top, title):
    """The counter in its own frame: 2 studs a unit along x, 2 deep, its doors at y = 0."""
    in_steps(
        m,
        f"{title}, units",
        [(2, lambda i=i, u=u: _unit(m, i, u, color)) for i, u in enumerate(units)],
    )
    in_steps(
        m,
        f"{title}, doors",
        [(1, lambda i=i, u=u: _door(m, i, u, doors)) for i, u in enumerate(units)],
    )
    with m.step(f"{title}, worktop"):
        m.row(Plate, top, at=(0, 0, 3), length=2 * len(units), width=2)
    sizes = {"cabinet": 1, "stove": 3, "sink": 3}
    in_steps(
        m,
        f"{title}, tops",
        [(sizes[u], lambda i=i, u=u: _top(m, i, u, top)) for i, u in enumerate(units)],
    )


def build(
    m,
    x,
    y,
    z,
    units=UNITS,
    facing="-y",
    color="White",
    doors="Sand Blue",
    top="Light Bluish Gray",
    title="Counter",
):
    """The counter; returns the placed copy's Handle."""
    units = tuple(units)
    for unit in units:
        if unit not in UNITS:
            raise ValueError(f"a unit is one of {', '.join(UNITS)}, not {unit!r}")
    sub = m.submodel(
        "kitchen counter",
        _counter,
        units=units,
        color=color,
        doors=doors,
        top=top,
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 2 * len(units), 2))


def example(m):
    """A counter of four units along the back of a kitchen floor."""
    with m.step("Floor"):
        m.add("3036", "White", at=(0, 0, 0), rot=1)
    m.use(
        "kitchen_counter", x=0, y=4, z=1, units=("cabinet", "stove", "sink", "cabinet")
    )
