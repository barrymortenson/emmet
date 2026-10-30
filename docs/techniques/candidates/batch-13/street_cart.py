# SPDX-License-Identifier: Apache-2.0
"""Street cart: a food or popcorn cart on four small wheels, a counter with a glass display, and a peaked canopy on
four posts.

m.use("street_cart", x, y, z, color="Red", canopy="White", trim="White", rim="Light Bluish Gray", title="Cart")
stands a cart 6 studs long (along x) and 4 wide on the ground at height z: two plates with two wheel pins (4600)
under 2 x 2 bricks, each with a 14 mm rim and its tire on each pin; a 4 x 6 plate on the bricks; a counter of 1 x 4
bricks around its edge and a 4 x 6 plate on it; a post of two round 1 x 1 bricks at each corner, with windows
1 x 2 x 2 between them along both sides as the glass display; and a 4 x 6 plate on the posts carrying six curved
slopes 2 x 2 that fall to both sides, the canopy. Returns the z of the canopy's ridge.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302, set 60440. Not derived
from any model file."""

PARTS = ["4600", "11208", "11209", "3003", "3032", "3010", "3062b", "60592c01", "15068"]

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
DOWN_LEFT, DOWN_RIGHT = 3, 1  # a curved slope 2 x 2 falling to -y, and to +y


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
    canopy="White",
    trim="White",
    rim="Light Bluish Gray",
    title="Cart",
):
    """The wheels, the base and counter, the posts and display, and the canopy; returns the ridge's z."""
    # The holders' top, a whole 4 plates up: the wheels ride 0.75 LDU low on their pins, within a pin's play, so the
    # tires stand on z and everything on the holders is on the plate grid.
    top = z + 4
    for k, ax in enumerate((x, x + 4)):
        with m.step(f"{title}: {('front', 'rear')[k]} wheels"):
            m.add("4600", "Light Bluish Gray", at=(ax, y + 0.3, top - 1.25))
            for cy, turn in ((y + 0.375, OUT_LEFT), (y + 3.625, OUT_RIGHT)):
                c = (ax + 1, cy, z + TIRE)
                _raw(m, "11208", rim, c, turn)
                _raw(m, "11209", "Black", c, turn)
    with m.step(f"{title}: base"):
        for ax in (x, x + 4):
            m.add("3003", color, at=(ax, y + 1, top))
        m.add("3032", color, at=(x, y, top + 3), rot=1)
    with m.step(f"{title}: counter"):
        for end in (x, x + 5):
            m.add("3010", color, at=(end, y, top + 4), rot=0)
        for side in (y, y + 3):
            m.add("3010", color, at=(x + 1, side, top + 4), rot=1)
    with m.step(f"{title}: counter top"):
        m.add("3032", trim, at=(x, y, top + 7), rot=1)
    for r in range(2):
        with m.step(f"{title}: {'posts and display' if r == 0 else 'posts, row 2'}"):
            for px in (x, x + 5):
                for py in (y, y + 3):
                    m.add("3062b", trim, at=(px, py, top + 8 + 3 * r))
            if r == 0:
                for side in (y, y + 3):
                    for k in range(2):
                        m.add(
                            "60592c01", trim, at=(x + 1 + 2 * k, side, top + 8), rot=1
                        )
    with m.step(f"{title}: canopy"):
        m.add("3032", canopy, at=(x, y, top + 14), rot=1)
        for k in range(3):
            m.add("15068", canopy, at=(x + 2 * k, y, top + 15), rot=DOWN_LEFT)
            m.add("15068", canopy, at=(x + 2 * k, y + 2, top + 15), rot=DOWN_RIGHT)
    return top + 17


def example(m):
    """A street cart on the ground."""
    m.use("street_cart", x=0, y=0, z=0)
