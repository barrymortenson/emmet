# SPDX-License-Identifier: Apache-2.0
"""Wall picture and clock: a framed picture or a round clock hung on side studs in a wall, facing into the room.

m.use("wall_picture_and_clock", x, y, z, kind="picture", facing="-y", picture="ship", clock="wood", color="White",
frame="Reddish Brown", title=None) sets a brick with studs on one side into a wall course at height z, in the wall's
cells from (x, y), its studs toward `facing`, and hangs the piece on them. kind="picture": a 1 x 2 brick with two side
studs, a 2 x 2 plate on them as the frame, and a printed 2 x 2 tile on the plate (picture="ship", "bridge", or
"drawing": a ship's portrait, a bridge, or a child's drawing), 2 studs wide and 5 plates tall, from the course's foot up
into the course above it. kind="clock": a 1 x 1 brick with a side stud and a round 2 x 2 clock tile centered on it
(clock="wood", Roman numerals on reddish brown, or "white"), reaching half a stud each side and 0.75 plates below the
course. The brick is in `color`, the wall's color: lay it in place of a wall brick. The piece stands 0.4 to 0.8 studs
proud of the wall. It's a submodel, built once in its own steps, "<title>, brick" and "<title>, hang it", placed in a
step titled "<title>" ("Picture" or "Clock" by default); it returns the placed copy's Handle.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335, set 43217, set 11387.
Not derived from any model file."""

PARTS = ["11211", "3022", "3068bp71", "87087", "14769p0a"]

FLAT = Rot(x=90)  # a plate or tile on its back, its face to -y
PICTURES = {
    "ship": ("3068bp71", "Dark Tan"),
    "bridge": ("3068bp74", "Reddish Brown"),
    "drawing": ("3068bp8g", "White"),
}
CLOCKS = {"wood": ("14769p0a", "Reddish Brown"), "white": ("14769p0f", "White")}

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


def _piece(m, kind, picture, clock, color, frame, title):
    """The piece in its own frame: its brick in the wall cells from (0, 0), its face toward -y."""
    with m.step(f"{title}, brick"):
        if kind == "picture":
            m.add("11211", color, at=(0, 0, 0), rot=3, id="brick")
        else:
            m.add("87087", color, at=(0, 0, 0), rot=3, id="brick")
    with m.step(f"{title}, hang it"):
        if kind == "picture":
            part, tint = PICTURES[picture]
            m.add("3022", frame, at=(0, -0.4, 0.5), rot=FLAT, id="frame")
            m.add(part, tint, at=(0, -0.8, 0.5), rot=FLAT, id="picture")
        else:
            part, tint = CLOCKS[clock]
            # The side stud is 1.75 plates up the brick: the round tile's middle goes on it.
            m.add(part, tint, at=(-0.5, -0.4, -0.75), rot=FLAT, id="clock")


def build(
    m,
    x,
    y,
    z,
    kind="picture",
    facing="-y",
    picture="ship",
    clock="wood",
    color="White",
    frame="Reddish Brown",
    title=None,
):
    """The picture or clock; returns the placed copy's Handle."""
    if kind not in ("picture", "clock"):
        raise ValueError(f'kind must be "picture" or "clock", not {kind!r}')
    if picture not in PICTURES:
        raise ValueError(f"picture is one of {', '.join(PICTURES)}, not {picture!r}")
    if clock not in CLOCKS:
        raise ValueError(f"clock is one of {', '.join(CLOCKS)}, not {clock!r}")
    title = title or kind.capitalize()
    sub = m.submodel(
        kind,
        _piece,
        kind=kind,
        picture=picture,
        clock=clock,
        color=color,
        frame=frame,
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 2 if kind == "picture" else 1, 1))


def example(m):
    """A wall with a picture and a clock in its second course."""
    with m.step("Floor"):
        m.add("3036", "Tan", at=(0, 0, 0), rot=1)
    with m.step("Wall, row 1"):
        m.wall(0, 5, 1, length=8, rows=1, color="White")
    with m.step("Wall, row 2"):
        m.add("3004", "White", at=(0, 5, 4), rot=1)
        m.add("3005", "White", at=(4, 5, 4))
        m.add("3004", "White", at=(6, 5, 4), rot=1)
    m.use("wall_picture_and_clock", x=2, y=5, z=4)
    m.use("wall_picture_and_clock", x=5, y=5, z=4, kind="clock")
    with m.step("Wall, rows 3 and 4"):
        m.wall(0, 5, 7, length=8, rows=2, color="White")
