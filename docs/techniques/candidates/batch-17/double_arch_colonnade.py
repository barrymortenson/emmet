# SPDX-License-Identifier: Apache-2.0
"""Double arch colonnade: an open arcade of tall arches on slender round columns, paired where two arches meet, with a
cornice line over them.

m.use("double_arch_colonnade", x, y, z, bays=3, height=3, color="White", trim="Light Bluish Gray",
title="Colonnade") builds, on the studs at z, an arcade one stud deep and 6 * bays studs long along x from (x, y): a
plinth of a plate course in `trim` and one in `color` over it, its joints crossed; columns of round bricks 1 x 1
(3062b) `height` bricks tall under each arch's feet, so a single column stands at each end and a pair where two arches
meet; a plate over each column or pair as its capital (1 x 2 over a pair, which ties it); an Arch 1 x 6 x 2 with Thin
Top (15254) over each bay; and a cornice of a plate course in `trim` and one in `color`, their joints off the arches'.
bays: 1 to 4; height: 2 to 6. Returns the z of the cornice's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = ["15254", "3062b", "3024", "3023b", "3710", "3666", "3460", "3623", "4477"]


def build(
    m,
    x,
    y,
    z,
    bays=3,
    height=3,
    color="White",
    trim="Light Bluish Gray",
    title="Colonnade",
):
    """The plinth, the columns, the capitals, the arches, and the cornice; returns the z of the top."""
    if not 1 <= bays <= 4 or not 2 <= height <= 6:
        raise ValueError(
            f"double_arch_colonnade takes 1 to 4 bays and a height of 2 to 6, not {bays}, {height}"
        )
    length = 6 * bays
    with m.step(f"{title}, plinth"):
        m.row(Plate, trim, at=(x, y, z), length=length)
        m.row(Plate, color, at=(x, y, z + 1), length=length, first=3)
    zc = z + 2
    # Each arch's feet: its first and last stud; neighbors' feet stand side by side.
    feet = sorted({6 * k for k in range(bays)} | {6 * k + 5 for k in range(bays)})
    for r in range(height):
        with m.step(f"{title}, columns, course {r + 1}"):
            for f in feet:
                m.add("3062b", color, at=(x + f, y, zc + 3 * r))
    za = zc + 3 * height
    with m.step(f"{title}, capitals"):
        for k in range(bays + 1):
            if k == 0:
                m.add("3024", color, at=(x, y, za))
            elif k == bays:
                m.add("3024", color, at=(x + length - 1, y, za))
            else:
                m.add("3023b", color, at=(x + 6 * k - 1, y, za), rot=1)
    with m.step(f"{title}, arches"):
        for k in range(bays):
            m.add("15254", color, at=(x + 6 * k, y, za + 1), rot=1)
    zt = za + 7
    with m.step(f"{title}, cornice"):
        m.row(Plate, trim, at=(x, y, zt), length=length, first=3)
        m.row(Plate, color, at=(x, y, zt + 1), length=length)
    return zt + 2


def example(m):
    """Three bays of arcade on a terrace."""
    with m.step("Terrace"):
        m.fill(Plate, "Tan", at=(-1, -1, 0), size=(20, 3))
    m.use("double_arch_colonnade", x=0, y=0, z=1)
