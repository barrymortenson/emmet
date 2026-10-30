# SPDX-License-Identifier: Apache-2.0
"""Vent fins: a row of thin upright fins with gaps between, like a heat sink or a console's vent, from tiles on the
side studs of brackets.

m.use("vent_fins", x, y, z, count=5, color="Dark Bluish Gray", fins="Light Bluish Gray", base="Dark Bluish Gray",
title="Vent") builds, on the studs at height z over (x, y) to (x + 2 * count, y + 2), a plate 2 wide under `count`
fins 2 studs apart along x: each a bracket 1 x 2 - 2 x 2 up (99207) on a 1 x 2 plate, its upright face turned to +x,
and a 2 x 2 tile in `fins` on that face's four studs, standing over a 1 x 2 tile that smooths the stud between fins.
Each fin is 2 studs tall and wide and a plate and a tile thick; the gaps are 1.2 studs. Returns the z of the fins'
tops.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 71374. Not derived from any
model file."""

PARTS = ["99207", "3023b", "3069b", "3068b"]

FIN = Rot(y=90)  # a tile on its back, its face to +x
LIP = 0.2  # how far a bracket's upright face stands past its plate, in studs


def build(
    m,
    x,
    y,
    z,
    count=5,
    color="Dark Bluish Gray",
    fins="Light Bluish Gray",
    base="Dark Bluish Gray",
    title="Vent",
):
    """The base and the fins; returns the z of the fins' tops."""
    if not 1 <= count <= 8:
        raise ValueError(f"vent_fins takes a count of 1 to 8, not {count}")
    with m.step(f"{title}: base"):
        m.row(Plate, base, at=(x, y, z), length=2 * count, width=2)
    for k in range(count):
        fx = x + 2 * k
        with m.step(f"{title}: fin {k + 1}"):
            m.add("3023b", color, at=(fx, y, z + 1))
            m.add("3069b", color, at=(fx + 1, y, z + 1))
            m.add("99207", color, at=(fx, y, z + 2), rot=0)
            m.add("3068b", fins, at=(fx + 1 + LIP, y, z + 2), rot=FIN)
    return z + 7


def example(m):
    """A vent block of five fins on a slab."""
    with m.step("Slab"):
        m.fill(Plate, "Light Bluish Gray", at=(-1, -1, 0), size=(12, 4))
    m.use("vent_fins", x=0, y=0, z=1)
