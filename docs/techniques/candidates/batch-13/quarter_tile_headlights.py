# SPDX-License-Identifier: Apache-2.0
"""Quarter-tile headlights: the slim swept lights of a racing-scale car's nose, clear tiles with a rounded outer
corner on brackets facing the front, a grille between them.

m.use("quarter_tile_headlights", x, y, z, width=8, color="Red", lens="Trans-Clear", grille="Black",
title="Headlights") puts a row of brackets 1 x 2 - 1 x 2 up (99780) across a nose whose front edge is at x and whose
top is z, `width` studs along y from y, their faces a stud tall to the front (-x). On the two end brackets a light:
a 1 x 1 tile and a 1 x 1 quarter-round tile (25269) whose round corner is outside and on top, so the light sweeps
back at its outer edge; on the brackets between, 1 x 2 grille tiles as the intake. Even widths from 4. Returns the
z of the brackets' top, where the hood goes.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 77264. Not derived from any
model file."""

PARTS = ["99780", "25269", "3070b", "2412b"]

FACE = Rot(y=-90)  # a tile on its back, its face to -x, long along y
# The quarter tile facing -x with its round corner at the top: at the +y end (FACE itself), and at the -y end.
CORNER_PLUS = FACE
CORNER_MINUS = Rot(x=90, y=0, z=-90)


def build(
    m,
    x,
    y,
    z,
    width=8,
    color="Red",
    lens="Trans-Clear",
    grille="Black",
    title="Headlights",
):
    """The brackets, the lights, and the grille; returns the z of the brackets' top."""
    if width < 4 or width % 2:
        raise ValueError(
            f"quarter_tile_headlights needs an even width of 4 or more, not {width}"
        )
    with m.step(f"{title}: brackets"):
        for b in range(0, width, 2):
            m.add("99780", color, at=(x - 0.2, y + b, z), rot=2)
    with m.step(f"{title}: lights"):
        m.add("25269", lens, at=(x - 0.6, y, z), rot=CORNER_MINUS)
        m.add("3070b", lens, at=(x - 0.6, y + 1, z), rot=FACE)
        m.add("3070b", lens, at=(x - 0.6, y + width - 2, z), rot=FACE)
        m.add("25269", lens, at=(x - 0.6, y + width - 1, z), rot=CORNER_PLUS)
    with m.step(f"{title}: grille"):
        for b in range(2, width - 2, 2):
            m.add("2412b", grille, at=(x - 0.6, y + b, z), rot=FACE)
    return z + 1


def example(m):
    """The nose of an 8-wide car: a plate, the lights, and a hood of curved slopes."""
    with m.step("Nose"):
        m.add("3035", "Red", at=(0, 0, 0), rot=0)
    top = m.use("quarter_tile_headlights", x=0, y=0, z=1)
    with m.step("Hood: filler"):
        m.add("3460", "Red", at=(1, 0, 1), rot=0)
    with m.step("Hood: curved slopes"):
        for k in range(8):
            m.add("11477", "Red", at=(0, k, top), rot=0)
