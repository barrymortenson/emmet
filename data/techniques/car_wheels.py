# SPDX-License-Identifier: Apache-2.0
"""Car wheels: a car's two axles of real rims and tires, tucked into wheel arches on the body's sides.

m.use("car_wheels", x, y, z, width=4, wheelbase=8, overhang=3, color="Red", chassis="Black",
rim="Light Bluish Gray", title="Wheels") stands a vehicle `width` studs wide (y to y + width: 4 for a city car, 6
for a truck, 8 for a racing-scale car) on the ground at height z, its front to -x: a front axle 2 studs long from x,
a rear axle `wheelbase` studs behind it, a spine of plates that runs `overhang` studs past each, and on the spine
over each axle a pair of mudguards whose arches wrap the tops of the tires, flush with the body's sides. Each wheel
is a 14 mm rim (11208) with its 21 mm tire (11209) on a wheel pin: for width 4 one plate with two pins (4600) under
a 2-wide spine; for 6 and 8 two plates with one pin each (4488) under a 4- or 6-wide spine, so the wheels stay at
the body's sides. The arches are 4 x 2.5 x 1 mudguards (98282) at 4 and 6 wide and the taller 4 x 2.5 x 2 (50745) at
8 wide. Returns the z of the spine's top (z + 5), where the body's floor plates go between and around the arches.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 60509, set 60440, set 77264.
Not derived from any model file."""

PARTS = ["4600", "4488", "11208", "11209", "98282", "50745", "3034", "3032", "3036"]

TIRE = 26.25 / 8  # the tire's radius (11209: 52.5 LDU across), in plates
# A wheel's turn, its hub's hole (along its own z) on a pin that points to -y (OUT_LEFT) or +y (OUT_RIGHT).
OUT_LEFT = (0, 0, 1, 0, 1, 0, -1, 0, 0)
OUT_RIGHT = (0, 0, -1, 0, 1, 0, 1, 0, 0)
# By width: the arch part, and how far its flare stands out past the body's side.
ARCH = {4: ("98282", 0.443), 6: ("98282", 0.443), 8: ("50745", 0.4)}


# The plates the library has, by width: lengths of Plate(2, n), Plate(4, n), and Plate(6, n).
LENGTHS = {
    2: (2, 3, 4, 6, 8, 10, 12, 14, 16),
    4: (4, 6, 8, 10, 12),
    6: (6, 8, 10, 12, 14, 16, 24),
}


def _split(across, length, least):
    """The spine's plates: one plate, or two that meet between the axles, each at least `least` long (so each
    carries an axle)."""
    have = LENGTHS[across]
    if length in have:
        return [length]
    pairs = [
        (f, length - f)
        for f in have
        if f >= least and length - f >= least and length - f in have
    ]
    if not pairs:
        raise ValueError(
            f"car_wheels can't lay a {across}-wide spine {length} long: change wheelbase or overhang"
        )
    return list(min(pairs, key=lambda p: abs(p[0] - p[1])))


def _raw(m, part, color, c, turn):
    """A part placed by its LDraw origin at the DSL point c (studs, studs, plates)."""
    x, y, z = c
    return m.add_raw(part, color, (20 * y, -8 * z, -20 * x, *turn))


def _axle(m, x, y, top, hub, width, rim, title):
    """One axle: its holder (or two) with the holder's top at `top`, and a wheel on each pin, its hub at `hub`."""
    with m.step(title):
        if width == 4:
            # Plate 2 x 2 with 2 Wheel Pins: its body is 3.4 studs across the pins and 1.25 plates deep.
            m.add("4600", "Light Bluish Gray", at=(x, y + 1 - 0.7, top - 1.25))
        else:
            # Two plates with one pin each, at the spine's edges, the pins out.
            m.add("4488", "Light Bluish Gray", at=(x, y + 0.3, top - 2), rot=2)
            m.add("4488", "Light Bluish Gray", at=(x, y + width - 3, top - 2), rot=0)
        for cy, turn in ((y + 0.375, OUT_LEFT), (y + width - 0.375, OUT_RIGHT)):
            c = (x + 1, cy, hub)
            _raw(m, "11208", rim, c, turn)
            _raw(m, "11209", "Black", c, turn)


def build(
    m,
    x,
    y,
    z,
    width=4,
    wheelbase=8,
    overhang=3,
    color="Red",
    chassis="Black",
    rim="Light Bluish Gray",
    title="Wheels",
):
    """The axles, the spine, and the arches; returns the z of the spine's top."""
    if width not in ARCH:
        raise ValueError(f"car_wheels takes a width of 4, 6, or 8, not {width}")
    if wheelbase < 4:
        raise ValueError(f"car_wheels needs a wheelbase of 4 or more, not {wheelbase}")
    # The holders' top, a whole 4 plates up: the wheels ride 0.75 LDU low on their pins, within a pin's play, so the
    # tires stand on z and everything on the holders is on the plate grid.
    top = z + 4
    for k, ax in enumerate((x, x + wheelbase)):
        _axle(
            m, ax, y, top, z + TIRE, width, rim, f"{title}: {('front', 'rear')[k]} axle"
        )
    across = width - 2
    length = wheelbase + 2 + 2 * overhang
    pieces = _split(across, length, overhang + 2)
    with m.step(f"{title}: spine"):
        at = x - overhang
        for n in pieces:
            m.add(Plate(across, n), chassis, at=(at, y + 1, top), rot=1)
            at += n
    arch, flare = ARCH[width]
    # The 50745's inner leg stands 2 plates above the bottom of its flare.
    at = top + 1 if arch == "98282" else top - 1
    with m.step(f"{title}: arches"):
        for ax in (x, x + wheelbase):
            m.add(arch, color, at=(ax - 1, y - flare, at), rot=3)
            m.add(arch, color, at=(ax - 1, y + width - 2, at), rot=1)
    return top + 1


def example(m):
    """A 4-wide city car's running gear, with a floor, a windscreen, and a seat on it."""
    deck = m.use("car_wheels", x=0, y=0, z=0, width=4)
    with m.step("Floor"):
        for x in (-3, 3, 5, 11):
            m.add("3020", "Red", at=(x, 0, deck), rot=0)
    with m.step("Cab"):
        m.add("3823", "Trans-Clear", at=(3, 0, deck + 1))
        m.add("4079", "Black", at=(5, 1, deck + 1))
