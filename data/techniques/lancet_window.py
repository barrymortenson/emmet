# SPDX-License-Identifier: Apache-2.0
"""Lancet window: a row of tall round-headed Gothic windows at midi scale, each a 1-wide stack of glass centered on a
jumper in a 2-stud bay, lit below and dark above, between stone piers.

m.use("lancet_window", x, y, z, bays=6, stone="Tan", sill="Dark Tan", lit="Trans Clear", dark="Trans Brown",
head="Black", title="Lancets") builds, on the studs at height z, a nave wall 1 stud thick along x from (x, y): a course
of bricks, then `bays` bays of 3 studs each, a 1 x 1 pier of three bricks and a 2-stud window bay: a jumper (1 x 2 with
one center stud) in `sill`, and on its stud a 1 x 1 brick in `lit` (the glass lit from inside), a 1 x 1 brick in `dark`
(the upper glass), and a 1 x 1 x 2/3 brick with a curved top in `head` as the round head, which leaves a half-stud
shadow each side of the glass; the window is a submodel, built once. A last pier closes the run, and a course of plates
and tiles over the piers ties them. The wall is 3 x bays + 1 studs long. Returns the z of the top.
"""

from emmet_dsl import Brick, Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file."""

PARTS = [
    "15573",
    "3005",
    "49307",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "3623",
    "3666",
    "3069b",
    "6636",
]


def _lancet(m, sill, lit, dark, head, title):
    """One window in its own frame: the jumper along x, the glass and head on its middle stud."""
    with m.step(f"{title}, window"):
        m.add("15573", sill, at=(0, 0, 0), rot=1)
        m.add("3005", lit, at=(0.5, 0, 1))
        m.add("3005", dark, at=(0.5, 0, 4))
        m.add("49307", head, at=(0.5, 0, 7))


def build(
    m,
    x,
    y,
    z,
    bays=6,
    stone="Tan",
    sill="Dark Tan",
    lit="Trans Clear",
    dark="Trans Brown",
    head="Black",
    title="Lancets",
):
    """The course, the piers, the windows, and the course over them; returns the z of the top."""
    if not 1 <= bays <= 16:
        raise ValueError(f"lancet_window takes 1 to 16 bays, not {bays}")
    length = 3 * bays + 1
    with m.step(f"{title}, sill course"):
        m.row(Brick, stone, at=(x, y, z), length=length)
    zw = z + 3
    lancet = m.submodel(
        "lancet", _lancet, sill=sill, lit=lit, dark=dark, head=head, title=title
    )
    for b0 in range(0, bays, 2):
        with m.step(f"{title}, bays {b0 + 1}-{min(bays, b0 + 2)}"):
            for b in range(b0, min(bays, b0 + 2)):
                m.stack("3005", stone, at=(x + 3 * b, y, zw), count=3)
                m.place(lancet, at=(x + 3 * b + 1, y, zw))
    with m.step(f"{title}, last pier"):
        m.stack("3005", stone, at=(x + length - 1, y, zw), count=3)
    zt = zw + 9
    with m.step(f"{title}, cornice"):
        m.row(Plate, stone, at=(x, y, zt), length=length)
        m.row(Tile, sill, at=(x, y, zt + 1), length=length)
    return zt + 2


def example(m):
    """A nave wall of six lancets on a plate."""
    with m.step("Ground"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=19, width=2)
    m.use("lancet_window", x=0, y=1, z=1, bays=6)
