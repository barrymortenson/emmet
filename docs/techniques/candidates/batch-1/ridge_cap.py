# SPDX-License-Identifier: Apache-2.0
"""Ridge cap: a rounded roll along a roof's ridge, curved-top bricks on a row of bricks between the two top slopes.

m.use("ridge_cap", x, y, z, length, color="Dark Red", cap="Dark Bluish Gray", title="Ridge") finishes a roof whose two
top courses of 45° slopes meet `length` studs long along x at y (the front slopes' top row ends at y, the back slopes'
starts at y + 1): a row of 1-wide bricks between them at z, and curved-top 1 x 1 bricks on it. Returns the z of its
top.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 21335, set 10350. Not derived
from any model file."""

PARTS = [
    "49307",
    "3010",
    "3009",
    "3008",
    "3004",
    "3005",
    "3622",
    "3037",
    "3039",
    "3038",
    "3040b",
]


def build(m, x, y, z, length, color="Dark Red", cap="Dark Bluish Gray", title="Ridge"):
    """The ridge row and its caps; returns the z of the top."""
    with m.step(f"{title}: bricks"):
        m.row(Brick, color, at=(x, y, z), length=length)
    with m.step(f"{title}: caps"):
        for a in range(length):
            m.add("49307", cap, at=(x + a, y, z + 3))
    return z + 5


def example(m):
    """A short roof: two 2 x 4 slopes facing front and back, and the ridge between them."""
    with m.step("Base"):
        m.add("3032", "Light Bluish Gray", at=(0, 0, 0))
    with m.step("Slopes"):
        m.add("3037", "Dark Red", at=(0, 0, 1), rot=3)
        m.add("3037", "Dark Red", at=(0, 3, 1), rot=1)
    m.use("ridge_cap", x=0, y=2, z=1, length=4)
