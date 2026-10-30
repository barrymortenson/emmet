# SPDX-License-Identifier: Apache-2.0
"""Wall shelf: a shelf set into a wall as a plate course that reaches a stud into the room, with bottles, cheese, cups,
or rows of sweet jars on it.

m.use("wall_shelf", x, y, z, length=4, facing="-y", items="pantry", seed=0, color="Reddish Brown", title="Shelf") lays a
plate 2 x length in `color` at height z, on top of a wall course: its back row is a course of the wall, whose cells
start at (x, y) and run `length` studs along it (along x for facing "-y" or "+y", along y for "+x" or "-x"), and its
front row reaches one stud out of the wall toward `facing`, into the room. Lay the rest of the wall's course to the same
height (a plate) on each side, then carry the wall on up on the shelf's back row. On the front row's studs:
items="pantry" sets a bottle, a wedge of cheese, a cup, and a jar in an order from `seed`; items="jars" sets a jar of
sweets on every stud (a round plate in a bright color under a clear round brick), as on a sweet shop's shelves;
items="none" leaves it bare. It's a submodel, built once in its own steps, "<title>, plate" and "<title>, things" (for
jars, "<title>, sweets" and "<title>, jars"), placed in a step titled "<title>"; it returns the z of the plate's top, z
+ 1, where the wall goes on.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335, set 11387, set 43302.
Not derived from any model file."""

PARTS = ["3020", "95228", "54200", "3899", "3062b", "6141"]

SWEETS = ["Bright Pink", "Yellow", "Lime", "Orange", "Medium Azure", "Red"]
ITEMS = ("pantry", "jars", "none")

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


def _pantry(m, i, thing):
    if thing == "bottle":
        m.add("95228", "Trans-Green", at=(i, 0, 1), id=f"bottle/{i + 1}")
    elif thing == "cheese":
        m.add("54200", "Yellow", at=(i, 0, 1), rot=3, id=f"cheese/{i + 1}")
    elif thing == "cup":
        # The cup's handle reaches 0.7 studs to its left: turned a quarter, it points out into the room.
        m.add("3899", "White", at=(i, -0.7, 1), rot=1, id=f"cup/{i + 1}")
    else:
        m.add("3062b", "Trans-Clear", at=(i, 0, 1), id=f"jar/{i + 1}")


def _shelf(m, length, items, seed, color, title):
    """The shelf in its own frame: `length` along x, its front row at y = 0 and its back row (in the wall) at y = 1."""
    from emmet_dsl import Plate

    with m.step(f"{title}, plate"):
        m.row(Plate, color, at=(0, 0, 0), length=length, width=2)
    if items == "none":
        return
    rng = random.Random(seed)
    if items == "pantry":
        things = ["bottle", "cheese", "cup", "jar"]
        rng.shuffle(things)
        with m.step(f"{title}, things"):
            for i in range(length):
                _pantry(m, i, things[i % len(things)])
        return
    with m.step(f"{title}, sweets"):
        for i in range(length):
            m.add("6141", rng.choice(SWEETS), at=(i, 0, 1), id=f"jar/{i + 1}/sweets")
    with m.step(f"{title}, jars"):
        for i in range(length):
            m.add("3062b", "Trans-Clear", at=(i, 0, 2), id=f"jar/{i + 1}")


def build(
    m,
    x,
    y,
    z,
    length=4,
    facing="-y",
    items="pantry",
    seed=0,
    color="Reddish Brown",
    title="Shelf",
):
    """The shelf; returns the z of its plate's top."""
    if items not in ITEMS:
        raise ValueError(f"items is one of {', '.join(ITEMS)}, not {items!r}")
    sub = m.submodel(
        "shelf", _shelf, length=length, items=items, seed=seed, color=color, title=title
    )
    with m.step(title):
        _put(m, sub, x, y, z, facing, (0, 1, length, 2))
    return z + 1


def example(m):
    """A pantry shelf and a shelf of sweet jars set into a wall."""
    from emmet_dsl import Plate

    with m.step("Floor"):
        m.add("3036", "Tan", at=(0, 0, 0), rot=1)
    with m.step("Wall"):
        m.wall(0, 5, 1, length=8, rows=2, color="White")
    with m.step("Wall, plate course"):
        m.add(Plate(1, 1), "White", at=(0, 5, 7))
        m.add(Plate(1, 1), "White", at=(7, 5, 7))
    top = m.use("wall_shelf", x=1, y=5, z=7, length=6, seed=2)
    with m.step("Wall, over the shelf"):
        m.wall(0, 5, top, length=8, rows=1, color="White")
    with m.step("Wall, sweet shelf course"):
        m.add(Plate(1, 1), "White", at=(0, 5, top + 3))
        m.add(Plate(1, 2), "White", at=(6, 5, top + 3), rot=1)
    top = m.use(
        "wall_shelf",
        x=1,
        y=5,
        z=top + 3,
        length=5,
        items="jars",
        seed=1,
        title="Sweet shelf",
    )
    with m.step("Wall, top"):
        m.wall(0, 5, top, length=8, rows=1, color="White")
