# SPDX-License-Identifier: Apache-2.0
"""Scalloped awning: a short striped-looking shop awning, a row of curved slopes on a plate off the wall, round plates
hung under its edge as scallops.

m.use("scalloped_awning", x, y, z, length=4, color="Dark Red", trim="White", title="Awning") lays, over a window in a
wall one stud thick along x facing front (-y), a Plate 2 x `length` in the trim color from (x, y - 1) at z: its back row
takes the wall's cells x to x + length - 1 for that plate course, its front row stands a stud out; a second such plate
on that front row reaches two studs out. On it go `length` Slope Brick Curved 2 x 1 (11477), striped in the two colors,
their curves down to the street, and under its front row hang `length` round 1 x 1 plates in `color`, the scallops.
length is 2, 4, or 6. Returns the z of the plate's top, where the wall goes on.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["3021", "3020", "3795", "11477", "6141"]

PLATES = {2: "3021", 4: "3020", 6: "3795"}  # Plate 2 x n


def build(m, x, y, z, length=4, color="Dark Red", trim="White", title="Awning"):
    """The plate, the scallops under it, and the slopes; returns the z of the plate's top."""
    if length not in PLATES:
        raise ValueError(f"scalloped_awning takes a length of 2, 4, or 6, not {length}")
    with m.step(f"{title}: plates and scallops"):
        m.add(PLATES[length], trim, at=(x, y - 1, z), rot=1)
        m.add(PLATES[length], trim, at=(x, y - 2, z + 1), rot=1)
        for k in range(length):
            m.add("6141", color, at=(x + k, y - 2, z))
    with m.step(f"{title}: slopes"):
        for k in range(length):
            m.add(
                "11477", color if k % 2 == 0 else trim, at=(x + k, y - 2, z + 2), rot=3
            )
    return z + 1


def example(m):
    """An awning over a shop window in a short wall."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Sand Green", at=(0, 0, 0), length=6)
    with m.step("Window"):
        for k in range(2):
            f = m.add("60592", "White", at=(1 + 2 * k, 0, 3), rot=1)
            (x0, y0, z0), (x1, y1, _) = f.min, f.max
            m.add(
                "60601",
                "Trans-Clear",
                at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
                rot=1,
            )
    for r in range(2):
        with m.step(f"Wall beside the window, row {r + 2}"):
            for cx in (0, 5):
                m.add("3005", "Sand Green", at=(cx, 0, 3 + 3 * r))
    with m.step("Plates beside the awning"):
        for cx in (0, 5):
            m.add("3024", "Sand Green", at=(cx, 0, 9))
    z = m.use("scalloped_awning", x=1, y=0, z=9)
    with m.step("Wall over the awning"):
        m.row(Brick, "Sand Green", at=(0, 0, z), length=6)
