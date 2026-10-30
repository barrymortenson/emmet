# SPDX-License-Identifier: Apache-2.0
"""Belfry louvers: the slatted openings of a bell tower, a column of bricks with studs on four sides whose side studs
carry cheese slopes as slats, so each opening is filled with sloping louvers.

m.use("belfry_louvers", x, y, z, slats=4, faces=("front", "back", "left", "right"), color="Dark Bluish Gray",
title="Louvers") stands, on the stud at (x, y) at height z, a column of `slats` 1 x 1 bricks with studs on four sides
(4733), and on each brick's side stud toward each of `faces` a 1 x 1 cheese slope (54200) laid on its back, its slope
falling outward and down like a louver blade. The slats stand out about 0.8 studs from the column. Set the column in the
middle of a tower so the louvers fill tall 1-stud openings in its walls, then close the top over it. Returns the z of
the column's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file."""

PARTS = ["4733", "54200"]

# A cheese slope on a side stud: its back to the column, its slope falling out and down; turned about z to each face.
SPIN = {"front": 0, "right": 90, "back": 180, "left": 270}
DEEP = 0.78  # a cheese slope's height (1.95 plates), standing out from the column, in studs


def build(
    m,
    x,
    y,
    z,
    slats=4,
    faces=("front", "back", "left", "right"),
    color="Dark Bluish Gray",
    title="Louvers",
):
    """The column and its slats; returns the z of the column's top."""
    if not 1 <= slats <= 12 or not faces or any(f not in SPIN for f in faces):
        raise ValueError(
            f"belfry_louvers takes 1 to 12 slats and faces from {', '.join(SPIN)}, not {slats}, {faces!r}"
        )
    where = {
        "front": (x, y - DEEP),
        "back": (x, y + 1),
        "left": (x - DEEP, y),
        "right": (x + 1, y),
    }
    for k in range(slats):
        zk = z + 3 * k
        with m.step(f"{title}, slat {k + 1}"):
            m.add("4733", color, at=(x, y, zk))
            for f in faces:
                # A side stud is 1.75 plates up the brick, a part's anti-stud 1.25 in from its edge.
                m.add(
                    "54200",
                    color,
                    at=(*where[f], zk + 0.5),
                    rot=Rot(x=90, y=90, z=SPIN[f]),
                )
    return z + 3 * slats


def example(m):
    """A small bell tower 3 x 3 studs with a louvered opening in each wall."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
    top = m.use("belfry_louvers", x=1, y=1, z=1)
    for r in range(4):
        with m.step(f"Tower, corners {r + 1}"):
            for cx, cy in ((0, 0), (2, 0), (0, 2), (2, 2)):
                m.add("3005", "Tan", at=(cx, cy, 1 + 3 * r))
    with m.step("Tower, top"):
        m.add(Plate(2, 3), "Tan", at=(0, 0, top), rot=1)
        m.add(Plate(1, 3), "Tan", at=(0, 2, top), rot=1)
    with m.step("Tower, spire"):
        m.add("3062b", "Tan", at=(1, 1, top + 1))
        m.add("4589", "Dark Bluish Gray", at=(1, 1, top + 4))
