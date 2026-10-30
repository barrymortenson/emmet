# SPDX-License-Identifier: Apache-2.0
"""Vintage chassis: the long, narrow frame of a 1930s car, a 2-wide spine with a T across each axle, white-wall
wheels, and running boards hung between the wheels.

m.use("vintage_chassis", x, y, z, length=16, color="Light Bluish Gray", board="Black", rim="White", title="Chassis")
stands the frame of a car 6 studs wide (y to y + 6) and `length` long on the ground at height z, its front to -x: a
2-wide spine plate down the middle; under it a plate 2 x 2 with two wheel pins (4600) 2 studs from each end, a white
14 mm rim (11208) and its tire on each pin; over each axle, on two 2 x 2 plates that lift it clear of the tires, a 2 x
4 plate across (the T that carries the fenders); between the wheels 2 x 6 plates across the spine for the body's
floor; and under their ends, a plate below the floor, the running boards, 1-wide plates from the front wheels to the
rear. The spine stays 2 wide under the hood, so a hood on it can narrow toward the grille. Lengths 14 to 20. Returns
the z of the floor's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43277, set 43302. Not
derived from any model file."""

PARTS = [
    "4282",
    "4600",
    "11208",
    "11209",
    "3022",
    "3020",
    "3795",
    "3460",
    "3666",
    "3034",
    "3832",
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
SPINE = {14: "91988", 16: "4282"}  # Plate 2 x n
BOARDS = {4: "3710", 6: "3666", 8: "3460", 10: "4477"}  # Plate 1 x n


def _raw(m, part, color, c, turn):
    """A part placed by its LDraw origin at the DSL point c (studs, studs, plates)."""
    x, y, z = c
    return m.add_raw(part, color, (20 * y, -8 * z, -20 * x, *turn))


def build(
    m,
    x,
    y,
    z,
    length=16,
    color="Light Bluish Gray",
    board="Black",
    rim="White",
    title="Chassis",
):
    """The wheels, spine, axle T's, floor, and running boards; returns the z of the floor's top."""
    span = length - 10  # between the axle T's
    if span not in BOARDS:
        raise ValueError(
            f"vintage_chassis takes a length of 14, 16, 18, or 20, not {length}"
        )
    # The holders' top, a whole 4 plates up: the wheels ride 0.75 LDU low on their pins, within a pin's play, so the
    # tires stand on z and everything on the holders is on the plate grid.
    top = z + 4
    axles = (x + 2, x + length - 4)
    for k, ax in enumerate(axles):
        with m.step(f"{title}: {('front', 'rear')[k]} wheels"):
            m.add("4600", "Light Bluish Gray", at=(ax, y + 1.3, top - 1.25))
            for cy, turn in ((y + 1.375, OUT_LEFT), (y + 4.625, OUT_RIGHT)):
                c = (ax + 1, cy, z + TIRE)
                _raw(m, "11208", rim, c, turn)
                _raw(m, "11209", "Black", c, turn)
    with m.step(f"{title}: spine"):
        if length in SPINE:
            m.add(SPINE[length], color, at=(x, y + 2, top), rot=1)
        else:
            # Two plates that meet under the floor plates, which tie them.
            front = 8 if length == 18 else 10
            m.add(Plate(2, front), color, at=(x, y + 2, top), rot=1)
            m.add(Plate(2, 10), color, at=(x + front, y + 2, top), rot=1)
    with m.step(f"{title}: axle T's"):
        # Two plates up, so the T clears the tires.
        for ax in axles:
            m.stack("3022", color, at=(ax, y + 2, top + 1), count=2)
            m.add("3020", color, at=(ax, y + 1, top + 3), rot=0)
    with m.step(f"{title}: floor"):
        for a in range(x + 5, x + length - 5, 2):
            m.add("3795", color, at=(a, y, top + 1), rot=0)
    with m.step(f"{title}: running boards"):
        for side in (y, y + 5):
            m.add(BOARDS[span], board, at=(x + 5, side, top), rot=1)
    return top + 2


def example(m):
    """A vintage car's chassis 16 long."""
    m.use("vintage_chassis", x=0, y=0, z=0)
