# SPDX-License-Identifier: Apache-2.0
"""Side-stud lights: a 6-wide truck's front course, square headlights in its corners and a grille between, all on
bricks whose studs face the front.

m.use("side_stud_lights", x, y, z, width=6, color="White", lens="Trans-Clear", grille="Light Bluish Gray",
title="Front lights") lays a course across a vehicle's front, `width` studs along y from (x, y, z): a brick 1 x 1 with
a stud on one side (87087) at every stud, all facing the front (-x); a square 1 x 1 clear tile on each corner's stud
as the headlight, and 1 x 2 grille tiles across the studs between, so the whole face is flush, with no stud showing.
Even widths from 4. Returns the z of the course's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 60509. Not derived from any
model file."""

PARTS = ["87087", "3070b", "2412b"]

FACE = Rot(y=-90)  # a tile on its back, its face to -x, long along y


def build(
    m,
    x,
    y,
    z,
    width=6,
    color="White",
    lens="Trans-Clear",
    grille="Light Bluish Gray",
    title="Front lights",
):
    """The side-stud bricks, the lights, and the grille; returns the z of the top."""
    if width < 4 or width % 2:
        raise ValueError(
            f"side_stud_lights needs an even width of 4 or more, not {width}"
        )
    with m.step(f"{title}: bricks"):
        for k in range(width):
            m.add("87087", color, at=(x, y + k, z), rot=2)
    with m.step(f"{title}: lights and grille"):
        for side in (y, y + width - 1):
            m.add("3070b", lens, at=(x - 0.4, side, z + 0.5), rot=FACE)
        for k in range(1, width - 1, 2):
            m.add("2412b", grille, at=(x - 0.4, y + k, z + 0.5), rot=FACE)
    return z + 3


def example(m):
    """The front of a 6-wide truck: a plate, the light course, and a plate over it."""
    with m.step("Base"):
        m.add("3032", "Dark Bluish Gray", at=(0, 0, 0), rot=0)
    top = m.use("side_stud_lights", x=0, y=0, z=1, color="Blue")
    with m.step("Top"):
        m.add("3666", "Blue", at=(0, 0, top), rot=0)
