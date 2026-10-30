# SPDX-License-Identifier: Apache-2.0
"""Four-wide car: a whole small city car for one figure, sized by the 4-wide rules: wheels in arches on a 2-wide
spine, a floor, a windscreen with a roof, a seat, and lights front and back.

m.use("four_wide_car", x, y, z, color="Red", chassis="Black", glass="Trans-Clear", rim="Light Bluish Gray",
title="Car") stands a car 18 studs long (along x, its front to -x) and 4 wide (y to y + 4) on the ground at height
z. The running gear: a 2-wide spine of plates on two plates with two wheel pins (4600), 10 studs apart, each with a
14 mm rim (11208) and its 21 mm tire on each pin, and over each axle two 4 x 2.5 x 1 mudguards (98282) whose arches
wrap the tires. The body: 2 x 4 plates as the floor between and beyond the arches; at the front bricks with a
stud on the side carrying round clear lenses and a grille; behind the front arch a windscreen 2 x 4 x 2 with its
roof (windscreen) over a seat 2 x 2 and two courses of side bricks; at the back tail lights (tail_lights) and a
trunk plate. The whole car is 5 bricks tall to
the roof. Returns the z of the roof's top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from the width rules in the instruction books of set 60440, set 60509 (the
books have no 4-wide car; the sizes are the 4-wide arithmetic). Not derived from any model file."""

PARTS = [
    "4600",
    "11208",
    "11209",
    "98282",
    "3034",
    "3832",
    "3020",
    "87087",
    "98138",
    "2412b",
    "4079",
    "3004",
    "3823",
]

TIRE = 26.25 / 8  # the tire's radius (11209), in plates
OUT_LEFT = (
    0,
    0,
    1,
    0,
    1,
    0,
    -1,
    0,
    0,
)  # a wheel's turn, its hub on a pin that points to -y
OUT_RIGHT = (0, 0, -1, 0, 1, 0, 1, 0, 0)  # and to +y
FACE = Rot(y=-90)  # a tile on its back, its face to -x


def _raw(m, part, color, c, turn):
    """A part placed by its LDraw origin at the DSL point c (studs, studs, plates)."""
    x, y, z = c
    return m.add_raw(part, color, (20 * y, -8 * z, -20 * x, *turn))


def build(
    m,
    x,
    y,
    z,
    color="Red",
    chassis="Black",
    glass="Trans-Clear",
    rim="Light Bluish Gray",
    title="Car",
):
    """The running gear, the floor, the cab, and the lights; returns the z of the roof's top."""
    # The holders' top, a whole 4 plates up: the wheels ride 0.75 LDU low on their pins, within a pin's play, so the
    # tires stand on z and everything on the holders is on the plate grid.
    top = z + 4
    deck = top + 1
    axles = (x + 3, x + 13)
    for k, ax in enumerate(axles):
        with m.step(f"{title}: {('front', 'rear')[k]} wheels"):
            m.add("4600", "Light Bluish Gray", at=(ax, y + 0.3, top - 1.25))
            for cy, turn in ((y + 0.375, OUT_LEFT), (y + 3.625, OUT_RIGHT)):
                c = (ax + 1, cy, z + TIRE)
                _raw(m, "11208", rim, c, turn)
                _raw(m, "11209", "Black", c, turn)
    with m.step(f"{title}: spine"):
        m.add(Plate(2, 8), chassis, at=(x, y + 1, top), rot=1)
        m.add(Plate(2, 10), chassis, at=(x + 8, y + 1, top), rot=1)
    with m.step(f"{title}: arches"):
        for ax in axles:
            m.add("98282", color, at=(ax - 1, y - 0.443, deck), rot=3)
            m.add("98282", color, at=(ax - 1, y + 2, deck), rot=1)
    with m.step(f"{title}: floor"):
        for a in (x, x + 6, x + 8, x + 10, x + 16):
            m.add("3020", color, at=(a, y, deck), rot=0)
    with m.step(f"{title}: headlights"):
        # Bricks with a stud on the side facing the front: round clear lenses at the corners, a grille between.
        for k in range(4):
            m.add("87087", color, at=(x, y + k, deck + 1), rot=2)
        for side in (y, y + 3):
            m.add("98138", "Trans-Clear", at=(x - 0.4, side, deck + 1.5), rot=FACE)
        m.add("2412b", "Light Bluish Gray", at=(x - 0.4, y + 1, deck + 1.5), rot=FACE)
    with m.step(f"{title}: seat and cab sides"):
        m.add("4079", "Black", at=(x + 8, y + 1, deck + 1), rot=0)
        for side in (y, y + 3):
            m.add("3004", color, at=(x + 8, side, deck + 1), rot=1)
    with m.step(f"{title}: cab sides, row 2"):
        for side in (y, y + 3):
            m.add("3004", color, at=(x + 8, side, deck + 4), rot=1)
    roof = m.use(
        "windscreen",
        x=x + 6,
        y=y,
        z=deck + 1,
        color=glass,
        roof=color,
        title=f"{title}: windscreen",
    )
    m.use(
        "tail_lights",
        x=x + 17,
        y=y,
        z=deck + 1,
        width=4,
        color=color,
        title=f"{title}: tail lights",
    )
    with m.step(f"{title}: trunk"):
        m.add("3010", color, at=(x + 16, y, deck + 1), rot=0)
        m.add("3020", color, at=(x + 16, y, deck + 4), rot=0)
    return roof


def example(m):
    """A small red city car on the ground."""
    m.use("four_wide_car", x=0, y=0, z=0)
