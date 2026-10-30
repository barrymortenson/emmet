# SPDX-License-Identifier: Apache-2.0
"""Crop field: a tilled field of crops in rows on dark soil, a water channel down its middle and a rim of smooth
blocks around it.

m.use("crop_field", x, y, z, length=10, depth=8, seed=1, soil="Reddish Brown", rim="Tan", water="Trans Light Blue",
crop="Bright Green", title="Field") lays a length x depth field from (x, y), on the studs at height z, all of it one
plate deep so it lies flush: a ring of 2 x 2 jumpers in the rim color two studs wide around the edge, one stud a block;
down the middle, along x, a channel of 2 x 2 tiles in the water color two studs wide; and soil plates either side of
it. On the soil, crops stand in rows one stud apart, on every other stud along each row (their blades are wider
than a stud), each a 1 x 1 round plate with a grass stem (15279) on it, or, about one in four by seed, a young one:
a lime round plate alone. length and depth: even, at least 6 and 8. Returns the z of the field's surface.
"""

import random

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["87580", "3068b", "3666", "6141", "15279"]

# The grass stem's body starts this far left of and in front of the stud it stands on, in studs.
STEM = (0.736, 0.4)


def build(
    m,
    x,
    y,
    z,
    length=10,
    depth=8,
    seed=1,
    soil="Reddish Brown",
    rim="Tan",
    water="Trans Light Blue",
    crop="Bright Green",
    title="Field",
):
    """The rim, the channel and soil, then the crops; returns the z of the surface."""
    if length % 2 or depth % 2 or length < 6 or depth < 8:
        raise ValueError(
            f"crop_field takes an even length of 6 or more and an even depth of 8 or more, not {length}, {depth}"
        )
    rng = random.Random(seed)
    rim_cells = [
        (i, j)
        for j in range(0, depth, 2)
        for i in range(0, length, 2)
        if i in (0, length - 2) or j in (0, depth - 2)
    ]
    for s in range(0, len(rim_cells), 8):
        with m.step(f"{title}, rim"):
            for i, j in rim_cells[s : s + 8]:
                m.add("87580", rim, at=(x + i, y + j, z))
    inner, rows = (
        length - 4,
        (depth - 6) // 2,
    )  # the inner length, and the soil rows each side of the channel
    cy = y + 2 + rows  # the channel's front
    with m.step(f"{title}, channel and soil"):
        m.grid("3068b", water, at=(x + 2, cy, z), nx=inner // 2)
        m.fill(Plate, soil, at=(x + 2, y + 2, z), size=(inner, rows))
        m.fill(Plate, soil, at=(x + 2, cy + 2, z), size=(inner, rows))
    soil_rows = [y + 2 + r for r in range(rows)] + [cy + 2 + r for r in range(rows)]
    for n, sy in enumerate(soil_rows):
        with m.step(f"{title}, crops, row {n + 1}"):
            for sx in range(x + 2 + n % 2, x + 2 + inner, 2):
                if rng.random() < 0.25:
                    m.add("6141", "Lime", at=(sx, sy, z + 1))
                else:
                    m.add("6141", "Green", at=(sx, sy, z + 1))
                    m.add(
                        "15279",
                        crop,
                        at=(sx + 0.5 - STEM[0], sy + 0.5 - STEM[1], z + 2),
                    )
    return z + 1


def example(m):
    """A field 12 x 8 on grass."""
    with m.step("Ground"):
        m.add("3036", "Bright Green", at=(0, 0, 0))
        m.add("3036", "Bright Green", at=(6, 0, 0))
    m.use("crop_field", x=0, y=0, z=1, length=12, depth=8)
