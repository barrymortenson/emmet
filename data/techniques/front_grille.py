# SPDX-License-Identifier: Apache-2.0
"""Front grille: a truck's grille, grille tiles on bricks whose studs face the front, between the headlights.

m.use("front_grille", x, y, z, width=4, rows=2, color="Light Bluish Gray", grille="Light Bluish Gray", title="Grille")
builds `rows` courses `width` studs along y from (x, y, z), the face to the front (-x): bricks 1 x 2 with two studs on
one side (11211) facing front with a 1 x 2 grille tile on each. Even widths. Returns the z of the top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["11211", "2412b"]

FACE = Rot(y=-90)  # a tile on its back, its face to -x, long along y


def build(
    m,
    x,
    y,
    z,
    width=4,
    rows=2,
    color="Light Bluish Gray",
    grille="Light Bluish Gray",
    title="Grille",
):
    """The studded courses and their grilles; returns the z of the top."""
    if width < 2 or width % 2:
        raise ValueError(f"front_grille needs an even width, not {width}")
    for r in range(rows):
        with m.step(f"{title}, row {r + 1}"):
            for b in range(0, width, 2):
                m.add("11211", color, at=(x, y + b, z + 3 * r), rot=2)
                m.add("2412b", grille, at=(x - 0.4, y + b, z + 3 * r + 0.5), rot=FACE)
    return z + 3 * rows


def example(m):
    """A grille two courses tall on a plate."""
    with m.step("Base"):
        m.add("3031", "Blue", at=(0, 0, 0))
    m.use("front_grille", x=0, y=0, z=1)
