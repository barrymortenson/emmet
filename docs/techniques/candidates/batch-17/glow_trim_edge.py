# SPDX-License-Identifier: Apache-2.0
"""Glow trim edge: a dark platform with a thin lit line all round its edge and indicator lights along its front.

m.use("glow_trim_edge", x, y, z, length=8, depth=4, spacing=2, color="Black", glow="Red", title="Platform")
builds a platform `length` x `depth` studs from (x, y) on the studs at z, three plates thick: a course of plates in
`color`; a course whose outer ring, a stud wide, is plates in `glow` (the inside is `color`), so a thin lit line runs
round the edge under the dark top; and tiles in `color` over it, with a round 1 x 1 tile in `glow` every `spacing`
studs along the front row as indicator lights, starting a stud in from each end. length: 4 to 16; depth: 3 to 12.
Returns the z of the top, where figures and consoles stand.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75419. Not derived from any
model file."""

PARTS = [
    "3023b",
    "3710",
    "3666",
    "3460",
    "3020",
    "3795",
    "3832",
    "3069b",
    "2431",
    "6636",
    "4162",
    "3068b",
    "87079",
    "98138",
]


def build(
    m,
    x,
    y,
    z,
    length=8,
    depth=4,
    spacing=2,
    color="Black",
    glow="Red",
    title="Platform",
):
    """The three courses and the lights; returns the z of the top."""
    if not (4 <= length <= 16 and 3 <= depth <= 12) or spacing < 1:
        raise ValueError(
            f"glow_trim_edge takes a length of 4 to 16, a depth of 3 to 12 and a spacing of 1 or more, "
            f"not {length}, {depth}, {spacing}"
        )
    with m.step(f"{title}, base"):
        m.fill(Plate, color, at=(x, y, z), size=(length, depth))
    z1 = z + 1
    with m.step(f"{title}, lit ring: front and back"):
        m.row(Plate, glow, at=(x, y, z1), length=length)
        m.row(Plate, glow, at=(x, y + depth - 1, z1), length=length)
    with m.step(f"{title}, lit ring: sides"):
        m.row(Plate, glow, at=(x, y + 1, z1), length=depth - 2, axis="y")
        m.row(Plate, glow, at=(x + length - 1, y + 1, z1), length=depth - 2, axis="y")
    if length > 2 and depth > 2:
        with m.step(f"{title}, inside"):
            m.fill(Plate, color, at=(x + 1, y + 1, z1), size=(length - 2, depth - 2))
    z2 = z1 + 1
    lights = list(range(1, length - 1, spacing))
    with m.step(f"{title}, front tiles and lights"):
        start = 0
        for i in lights + [length]:
            if i > start:
                m.row(Tile, color, at=(x + start, y, z2), length=i - start)
            if i < length:
                m.add("98138", glow, at=(x + i, y, z2))
            start = i + 1
    with m.step(f"{title}, top tiles"):
        m.fill(Tile, color, at=(x, y + 1, z2), size=(length, depth - 1))
    return z2 + 1


def example(m):
    """A command platform 8 x 4 studs on a floor."""
    with m.step("Floor"):
        m.fill(Plate, "Dark Bluish Gray", at=(-1, -1, 0), size=(10, 6))
    m.use("glow_trim_edge", x=0, y=0, z=1)
