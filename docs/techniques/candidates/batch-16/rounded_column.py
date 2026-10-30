# SPDX-License-Identifier: Apache-2.0
"""Rounded column: a short round cylinder from four corner round bricks a course, tied by round plates, with an optional
dome on the end: a muzzle, a leg, a foot, an eye pod, a pillar, upright or standing out of a face.

m.use("rounded_column", x, y, z, size=4, courses=2, cap="dome", facing="up", color="White", title="Column") builds a
cylinder `size` studs across (4, 6, or 8) and `courses` courses long (1 to 6). Each course is a ring of four corner
round bricks (2 x 2, 3 x 3, or 4 x 4), each a quarter turn from the last, with a round brick 2 x 2 in the middle of the
smaller rings' hole, then a round plate of the same size across them, which ties the four quarters (a course is 4
plates, 1.6 studs). cap "dome" ends it with dome_from_quarters' four quarter pieces of the same size (2, 3, or 8 plates
tall), "none" leaves the last round plate's studs. It's a submodel (one file per size, length, cap, and color) with its
own steps, placed in a step titled `title`. facing is the way it points: "up" stands it on the studs at height z with
its corner at (x, y); "front" (-y), "back" (+y), "left" (-x), and "right" (+x) lay it on its side, pressed onto the side
studs of a face, its lowest edge at z: for "front" and "back" x is its left end and y the face, for "left" and "right" x
is the face and y its front end. The side studs must sit on its grid, 1.25 plates above z and a stud (2.5 plates) apart.
Returns the placed copy's Handle; its .max is the far end.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21368 and 43279. Not derived
from any model file."""

PARTS = [
    "3063b",
    "5152",
    "2577",
    "3941",
    "60474",
    "11213",
    "74611",
    "5852",
    "76797",
    "49612",
]

# By size: the corner round brick, the round plate that ties a course, and the dome quarter and its height.
SIZES = {
    4: ("3063b", "60474", "5852", 2),
    6: ("5152", "11213", "76797", 3),
    8: ("2577", "74611", "49612", 8),
}
# A corner piece's round side points to +x +y at rot 0: the turn for the quarter in each half of x and y.
TURNS = {(1, 1): 0, (0, 1): 1, (0, 0): 2, (1, 0): 3}
FACING = {
    "up": Rot(),
    "front": Rot(x=90),
    "back": Rot(x=90, z=180),
    "left": Rot(x=90, z=-90),
    "right": Rot(x=90, z=90),
}


def _column(m, size, courses, cap, color):
    """The column standing up in its own frame, its corner at the origin."""
    corner, plate, dome, _ = SIZES[size]
    h = size // 2
    for c in range(courses):
        zc = 4 * c
        with m.step(f"Course {c + 1}"):
            for (i, j), rot in TURNS.items():
                m.add(corner, color, at=(i * h, j * h, zc), rot=rot)
            if size < 8:
                # The ring has a hole in the middle: a round brick fills it and holds the plate's middle.
                m.add("3941", color, at=(h - 1, h - 1, zc))
            m.add(plate, color, at=(0, 0, zc + 3))
    if cap == "dome":
        with m.step("Dome"):
            for (i, j), rot in TURNS.items():
                m.add(dome, color, at=(i * h, j * h, 4 * courses), rot=rot)


def build(
    m,
    x,
    y,
    z,
    size=4,
    courses=2,
    cap="dome",
    facing="up",
    color="White",
    title="Column",
):
    """The column, placed; returns the placed copy's Handle."""
    if size not in SIZES:
        raise ValueError(f"rounded_column takes a size of 4, 6, or 8, not {size}")
    if not 1 <= courses <= 6:
        raise ValueError(f"rounded_column takes 1 to 6 courses, not {courses}")
    if cap not in ("dome", "none"):
        raise ValueError(f"rounded_column takes a cap of 'dome' or 'none', not {cap!r}")
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a way: use up, front, back, left, or right"
        )
    sub = m.submodel(
        "rounded_column", _column, size=size, courses=courses, cap=cap, color=color
    )
    lo, hi = sub.box
    long = (hi[2] - lo[2]) * 0.4  # its length, in studs, lying on its side
    at = {
        "up": (x, y, z),
        "front": (x, y - long, z),
        "back": (x, y, z),
        "left": (x - long, y, z),
        "right": (x, y, z),
    }[facing]
    with m.step(title):
        return m.place(sub, at=at, rot=FACING[facing])


def example(m):
    """A leg standing on a foot plate, and a muzzle sticking out of a block's front, both 4 studs across."""
    with m.step("Base"):
        m.add("3958", "Dark Bluish Gray", at=(0, 0, 0))
        m.add("3958", "Dark Bluish Gray", at=(6, 0, 0))
    m.use(
        "rounded_column",
        x=1,
        y=1,
        z=1,
        size=4,
        courses=3,
        cap="none",
        color="Tan",
        title="Leg",
    )
    with m.step("Head block"):
        m.add("3001", "Tan", at=(7, 3, 1), rot=1)  # Brick 2 x 4, along x
        m.add("3001", "Tan", at=(7, 4, 4), rot=1)
        m.add("3710", "Tan", at=(7, 3, 4), rot=1)  # Plate 1 x 4
        m.add("3710", "Tan", at=(7, 3, 5), rot=1)
    with m.step("Head block, side studs"):
        m.add("22885", "Tan", at=(7, 3, 6), rot=3)  # studs to the front
        m.add("22885", "Tan", at=(9, 3, 6), rot=3)
        m.add("3001", "Tan", at=(7, 4, 7), rot=1)
        m.add("3710", "Tan", at=(7, 4, 10), rot=1)
    m.use(
        "rounded_column",
        x=7,
        y=3,
        z=1 + 5,
        size=4,
        courses=1,
        facing="front",
        color="Tan",
        title="Muzzle",
    )
