# SPDX-License-Identifier: Apache-2.0
"""Angled display stand: a low black stand that holds a model nose-up at 10 to 20 degrees on a cradle pinned to a short
pylon, so the model looks in flight and lifts off.

m.use("angled_display_stand", x, y, z, angle=15, length=6, ship=None, color="Black", title="Stand") builds, on the studs
at height z, a base centered on (x, y): a cross of two 2 x 8 plates, one over the other, their arms tiled smooth; a
pylon 2 x 2 studs in the middle, two courses of 2 x 2 bricks and a course of two Technic bricks 1 x 2, their holes along
x; and a cradle built flat as a submodel: two Technic bricks 1 x `length` (6 or 8) as rails, one each side of the pylon,
a 1 x `length` plate on each, and a 4 x `length` plate across them as the platform. A Technic axle 4 runs through the
rails' middle holes and the pylon's, and the cradle turns on it by `angle` degrees, nose (+y) up. ship: a submodel to
stand on the platform's studs, centered on it, which turns with the cradle and lifts off it. Returns the placed cradle's
Handle.
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10356. Not derived from any
model file."""

PARTS = [
    "3034",
    "26603",
    "3003",
    "3700",
    "3705",
    "3894",
    "3702",
    "3666",
    "3460",
    "3032",
    "3035",
]

RAIL = {6: ("3894", "3666", "3032"), 8: ("3702", "3460", "3035")}
HOLE = 1.75  # a Technic brick's holes are 1.75 plates up


def _cradle(m, length, color, ship, title):
    """The rails, their plates, and the platform, in their own frame: rails at x = 0 and x = 3, along y."""
    rail, plate, top = RAIL[length]
    with m.step(f"{title}, cradle: rails"):
        m.add(rail, color, at=(0, 0, 0))
        m.add(rail, color, at=(3, 0, 0))
        m.add(plate, color, at=(0, 0, 3))
        m.add(plate, color, at=(3, 0, 3))
    with m.step(f"{title}, cradle: platform"):
        m.add(top, color, at=(0, 0, 4))
    if ship is not None:
        (lo, hi) = ship.box
        w, d = hi[0] - lo[0], hi[1] - lo[1]
        with m.step(f"{title}, cradle: model"):
            m.place(ship, at=((4 - w) // 2, (length - d) // 2, 5))


def _at_for(sub, rot, p, target):
    """Where to place `sub` turned by `rot` so the point p of its own frame lands on `target` (DSL units)."""
    r = frame.to_ldraw_rotation(rot._matrix())
    body = frame.box_in_ldraw(*sub.box)
    lo, _ = frame.box_in_dsl(frame.transform_box(r, (0, 0, 0), body))
    pl, tl = frame.to_ldraw_point(p), frame.to_ldraw_point(target)
    o = [tl[i] - sum(r[i][k] * pl[k] for k in range(3)) for i in range(3)]
    origin = (-o[2] / frame.STUD, o[0] / frame.STUD, -o[1] / frame.PLATE)
    return tuple(origin[i] + lo[i] for i in range(3))


def build(m, x, y, z, angle=15, length=6, ship=None, color="Black", title="Stand"):
    """The base, the pylon, the axle, and the turned cradle; returns the cradle's Handle."""
    if not 10 <= angle <= 20 or length not in RAIL:
        raise ValueError(
            f"angled_display_stand takes an angle of 10 to 20 and a length of 6 or 8, not {angle}, {length}"
        )
    with m.step(f"{title}, base"):
        m.add("3034", color, at=(x - 4, y - 1, z), rot=1)
        m.add("3034", color, at=(x - 1, y - 4, z + 1))
    with m.step(f"{title}, base tiles"):
        m.add("26603", color, at=(x - 4, y - 1, z + 1), rot=1)
        m.add("26603", color, at=(x + 1, y - 1, z + 1), rot=1)
        m.add("26603", color, at=(x - 1, y - 4, z + 2))
        m.add("26603", color, at=(x - 1, y + 1, z + 2))
    zp = z + 2
    with m.step(f"{title}, pylon"):
        m.stack("3003", color, at=(x - 1, y - 1, zp), count=2)
        m.add("3700", color, at=(x - 1, y - 1, zp + 6))
        m.add("3700", color, at=(x, y - 1, zp + 6))
    pivot = zp + 6 + HOLE
    cradle = m.submodel(
        "cradle", _cradle, length=length, color=color, ship=ship, title=title
    )
    turn = Rot(x=angle)
    with m.step(f"{title}, axle"):
        m.add("3705", color, at=(x - 2, y - 0.3, pivot - 0.75), rot=1)
    with m.step(f"{title}, cradle"):
        at = _at_for(cradle, turn, (0, length // 2, HOLE), (x - 2, y, pivot))
        return m.place(cradle, at=at, rot=turn)


def example(m):
    """A small white ship on the stand, nose up 15 degrees."""

    def ship(s):
        with s.step("Ship, hull"):
            s.add("3032", "White", at=(0, 0, 0))
            s.add("3020", "Light Bluish Gray", at=(1, 0, 1))
        with s.step("Ship, nose"):
            s.add("15068", "White", at=(0, 4, 1), rot=1)
            s.add("15068", "White", at=(2, 4, 1), rot=1)
            s.add("3039", "White", at=(1, 2, 2), rot=1)
            s.add("3068b", "Light Bluish Gray", at=(1, 0, 2))

    craft = m.submodel("ship", ship)
    m.use("angled_display_stand", x=4, y=4, z=0, ship=craft)
