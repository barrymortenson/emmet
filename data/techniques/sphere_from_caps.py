# SPDX-License-Identifier: Apache-2.0
"""Sphere from caps: a ball about 9 studs across, a cube-shaped core with a domed cap of round plates on each of its six
faces: a ball, a planet, a round head, a piece of fruit.

m.use("sphere_from_caps", x, y, z, color="Orange", core="Black", title="Ball") builds, on the studs at height z, a ball
whose 6 x 6 stud core has its front-left corner at (x, y), its bottom cap's round plate 2 x 2 on the studs 2 in from (x,
y) both ways: it covers x - 1.6 to x + 7.6 and y - 1.6 to y + 7.6, and stands 22 plates (8.8 studs) tall. The core is 6
studs wide and 15 plates (6 studs) tall: a plate, a ring of bricks, and a plate; then a ring with studs on all four
sides (bricks 1 x 2 x 1 2/3 with studs on the side, two to a face, plain columns at the corners); then a plate, a ring,
and a plate. Each face takes a cap of round plates 6, 4, and 2 studs across under a round tile 2 x 2, the four side caps
on the ring's side studs; the bottom cap is built smallest first, so the ball rests on a round plate 2 x 2. Between the
caps the core's edges show in the `core` color, which reads as the seams of a ball (black on orange makes a basketball).
Each cap is a submodel. It brings its own steps and returns the z of the top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43008 (the caps on a hub
there are on a Technic core; this one is all bricks). Not derived from any model file."""

PARTS = [
    "11213",
    "60474",
    "4032a",
    "14769",
    "3958",
    "22885",
    "3005",
    "3024",
    "3009",
    "3010",
    "3622",
    "3004",
]

ROUND = {6: "11213", 4: "60474", 2: "4032a"}
SIDES = {
    # The cap's turn, and where its base's corner goes, relative to the core's front-left-bottom corner.
    "front": (Rot(x=90), (0, -1.6)),
    "back": (Rot(x=90, z=180), (0, 6)),
    "left": (Rot(x=90, z=-90), (-1.6, 0)),
    "right": (Rot(x=90, z=90), (6, 0)),
}
STUDS_OUT = {
    "front": 3,
    "back": 1,
    "left": 2,
    "right": 0,
}  # a side-stud brick's quarter turn


def _cap(m, color, down):
    """A cap, its base's corner at the origin: bulging up, or built smallest first to hang down."""
    sizes = [6, 4, 2] if not down else [2, 4, 6]
    with m.step("Plates" if not down else "Plates, smallest first"):
        for k, n in enumerate(sizes):
            off = (6 - n) / 2
            m.add(ROUND[n], color, at=(off, off, k))
    if not down:
        with m.step("Top"):
            m.add("14769", color, at=(2, 2, 3))


def _ring(m, cx, cy, z, core, title):
    """A course of bricks around the edge of the 6 x 6 core."""
    with m.step(title):
        m.wall(cx, cy, z, 6, 6, color=core, rows=1)


def build(m, x, y, z, color="Orange", core="Black", title="Ball"):
    """The bottom cap, the core, and the other five caps; returns the z of the top."""
    cx, cy = x, y
    down = m.submodel("sphere_cap_under", _cap, color=color, down=True)
    up = m.submodel("sphere_cap", _cap, color=color, down=False)
    with m.step(f"{title}: bottom cap"):
        m.place(down, at=(cx, cy, z))
    z0 = z + 3
    # The core's lower third: a plate, a ring, a plate.
    with m.step(f"{title}: core, plate 1"):
        m.add("3958", core, at=(cx, cy, z0))
    _ring(m, cx, cy, z0 + 1, core, f"{title}: core, ring 1")
    with m.step(f"{title}: core, plate 2"):
        m.add("3958", core, at=(cx, cy, z0 + 4))
    # The middle third: studs on all four faces, two bricks 1 x 2 x 1 2/3 a face, and a plain column at each corner.
    zs = z0 + 5
    for half, corners in (("back", ((5, 5), (0, 5))), ("front", ((0, 0), (5, 0)))):
        with m.step(f"{title}: core, studded ring, {half} corners"):
            for i, j in corners:
                m.add("3005", core, at=(cx + i, cy + j, zs))
                m.add("3024", core, at=(cx + i, cy + j, zs + 3))
                m.add("3024", core, at=(cx + i, cy + j, zs + 4))
    for side in ("front", "back", "left", "right"):
        with m.step(f"{title}: core, studded ring, {side}"):
            for a in (1, 3):
                i, j = {
                    "front": (a, 0),
                    "back": (a, 5),
                    "left": (0, a),
                    "right": (5, a),
                }[side]
                m.add("22885", core, at=(cx + i, cy + j, zs), rot=STUDS_OUT[side])
    # The upper third: a plate, a ring, a plate.
    with m.step(f"{title}: core, plate 3"):
        m.add("3958", core, at=(cx, cy, zs + 5))
    _ring(m, cx, cy, zs + 6, core, f"{title}: core, ring 2")
    with m.step(f"{title}: core, plate 4"):
        m.add("3958", core, at=(cx, cy, zs + 9))
    top = zs + 10
    with m.step(f"{title}: top cap"):
        m.place(up, at=(cx, cy, top))
    for side, (turn, (dx, dy)) in SIDES.items():
        with m.step(f"{title}: {side} cap"):
            m.place(up, at=(cx + dx, cy + dy, z0), rot=turn)
    return top + 4


def example(m):
    """A ball on a small stand."""
    with m.step("Stand"):
        m.add("3958", "Dark Bluish Gray", at=(2, 2, 0))
        m.add("3062b", "Dark Bluish Gray", at=(4, 4, 1))  # Round brick 1 x 1
        m.add("3062b", "Dark Bluish Gray", at=(5, 4, 1))
        m.add("3062b", "Dark Bluish Gray", at=(4, 5, 1))
        m.add("3062b", "Dark Bluish Gray", at=(5, 5, 1))
    m.use("sphere_from_caps", x=2, y=2, z=4)
