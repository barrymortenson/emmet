# SPDX-License-Identifier: Apache-2.0
"""Wood stove: a black iron stove on short legs with a glowing grate door, a hotplate, a kettle, and a stovepipe, built
apart and placed.

m.use("wood_stove", x, y, z, facing="-y", pipe=4, kettle=True, color="Black", glow="Trans-Orange", title="Stove") stands
a stove 2 x 2 studs on the studs at height z, its footprint's min corner at (x, y), its door toward `facing`: four round
plates as legs, a body of a plate, a brick with studs on one side and a plain brick, a grille tile in `glow` standing on
the side studs as the fire door, and a top plate with a grille hotplate, a kettle, and `pipe` round bricks rising from
the back corner as the stovepipe (0 for none). The door stands 0.4 studs proud of the footprint. It's a submodel, built
once in its own steps, "<title>, body" and "<title>, top", placed in a step titled "<title>"; it returns the placed
copy's Handle.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335. Not derived from any
model file."""

PARTS = ["6141", "3022", "11211", "3004", "2412b", "3062b", "3899"]

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


def _stove(m, pipe, kettle, color, glow, title):
    """The stove in its own frame: 2 x 2, its door at y = 0."""
    with m.step(f"{title}, body"):
        for i, (lx, ly) in enumerate(((0, 0), (1, 0), (0, 1), (1, 1))):
            m.add("6141", color, at=(lx, ly, 0), id=f"leg/{i + 1}")
        m.add("3022", color, at=(0, 0, 1), id="floor")
        m.add("11211", color, at=(0, 0, 2), rot=3, id="front")
        m.add("3004", color, at=(0, 1, 2), rot=1, id="back")
        m.add("2412b", glow, at=(0, -0.4, 2.5), rot=PANEL, id="door")
    with m.step(f"{title}, top"):
        m.add("3022", color, at=(0, 0, 5), id="top")
        m.add("2412b", "Dark Bluish Gray", at=(0, 0, 6), rot=1, id="hotplate")
        if kettle:
            # The cup's handle reaches 0.7 studs to its left: on the back right stud, turned to the right.
            m.add("3899", "Pearl Silver", at=(1, 1, 6), rot=2, id="kettle")
        if pipe:
            m.stack("3062b", color, at=(0, 1, 6), count=pipe, id="pipe")


def build(
    m,
    x,
    y,
    z,
    facing="-y",
    pipe=4,
    kettle=True,
    color="Black",
    glow="Trans-Orange",
    title="Stove",
):
    """The stove; returns the placed copy's Handle."""
    sub = m.submodel(
        "wood stove",
        _stove,
        pipe=pipe,
        kettle=kettle,
        color=color,
        glow=glow,
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 2, 2))


def example(m):
    """A stove in a corner of a floor, its door to the front, and one turned to face right."""
    with m.step("Floor"):
        m.add("3031", "Reddish Brown", at=(0, 0, 0))
    m.use("wood_stove", x=0, y=2, z=1)
    m.use("wood_stove", x=2, y=0, z=1, facing="+x", pipe=2, kettle=False)
