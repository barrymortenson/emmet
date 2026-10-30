# SPDX-License-Identifier: Apache-2.0
"""Front-hinged hood: a large car's hood built as its own sub-assembly that pivots on two Technic pins in line, at the
nose or at the windscreen, so it tips up to show the engine.

m.use("front_hinged_hood", x, y, z, length=6, width=6, hinge="front", color="Dark Green", frame="Light Bluish Gray",
title="Hood") puts a Technic brick 1 x 2 with a hole (3700) at each side of a body `width` studs along y from y, at z,
its hole across the car, at the nose (x, hinge="front") or at the back of the hood (x + length - 2, hinge="back"), with
a tile on it, and pushes a friction pin (2780) into each from outside. The hood, a submodel, is two layers of plates
`width` + 2 by `length`, their seams crossed, with tiles on them, resting on the body's tiles at z + 4, a stud wider
than the body each side, with a Technic brick 1 x 2 hanging under each side on a plate, its hole on the pin. Build and
tile the rest of the body's course from z to z + 3 first, so the hood rests on smooth tiles and swings. length and width
are even, 4 to 12 and 4 to 10. Returns the z of the hood's top (z + 7).
"""

from emmet_dsl import Plate, Tile, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11381, set 42213. Not derived
from any model file."""

PARTS = [
    "3700",
    "3069b",
    "2780",
    "3023b",
    "3020",
    "3795",
    "3034",
    "3032",
    "3036",
    "3068b",
    "87079",
    "69729",
]


def _put(m, part, color, point, r=None):
    """Places a part whose origin is its center (a pin) at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def _hood(m, length, width, hx, color, frame_color):
    """The hood in its own frame: its hinge bricks' bottoms at z = 0, its left edge at y = 0, its nose at x = 0."""
    with m.step("Hood: hinge bricks"):
        for side in (0, width + 1):
            m.add("3700", frame_color, at=(hx, side, 0), rot=1)
            m.add("3023b", color, at=(hx, side, 3), rot=1)
    wide = width + 2
    with m.step("Hood: plates"):
        m.fill(Plate, color, at=(0, 0, 4), size=(length, wide))
    # A second layer of plates across the first's seams (m.fill runs its strips along the long side), then tiles.
    with m.step("Hood: plates across"):
        if wide >= length:
            for k in range(0, wide, 2):
                m.row(Plate, color, at=(0, k, 5), length=length, axis="x", width=2)
        else:
            for k in range(0, length, 2):
                m.row(Plate, color, at=(k, 0, 5), length=wide, axis="y", width=2)
    with m.step("Hood: tiles"):
        m.fill(Tile, color, at=(0, 0, 6), size=(length, wide))


def build(
    m,
    x,
    y,
    z,
    length=6,
    width=6,
    hinge="front",
    color="Dark Green",
    frame="Light Bluish Gray",
    title="Hood",
):
    """The body's hinge bricks and pins, then the hood on them; returns the z of the hood's top."""
    if hinge not in ("front", "back"):
        raise ValueError(
            f"front_hinged_hood takes a hinge of 'front' or 'back', not {hinge!r}"
        )
    if length % 2 or width % 2 or not (4 <= length <= 12 and 4 <= width <= 10):
        raise ValueError(
            f"front_hinged_hood takes an even length 4 to 12 and width 4 to 10, not {length}, {width}"
        )
    hx = 0 if hinge == "front" else length - 2
    with m.step(f"{title}: hinge bricks"):
        for side in (y, y + width - 1):
            m.add("3700", frame, at=(x + hx, side, z), rot=1)
            m.add("3069b", color, at=(x + hx, side, z + 3), rot=1)
    with m.step(f"{title}: pins"):
        for side in (y, y + width):
            _put(m, "2780", "Black", (x + hx + 1, side, z + 1.75))
    hood = m.submodel(
        "hood",
        _hood,
        length=length,
        width=width,
        hx=hx,
        color=color,
        frame_color=frame,
    )
    with m.step(title):
        m.place(hood, at=(x, y - 1, z))
    return z + 7


def example(m):
    """A hood hinged at the nose of a body 6 x 6, the body tiled first."""
    with m.step("Floor"):
        m.add("3958", "Dark Bluish Gray", at=(0, 0, 0))
    with m.step("Body"):
        for side in (0, 5):
            m.add("3010", "Dark Green", at=(2, side, 1), rot=1)
        for row in (1, 3):
            m.add("2456", "Dark Bluish Gray", at=(0, row, 1), rot=1)
    with m.step("Body tiles"):
        for side in (0, 5):
            m.add("2431", "Dark Green", at=(2, side, 4), rot=1)
        for row in (1, 3):
            m.add("69729", "Dark Bluish Gray", at=(0, row, 4), rot=1)
    m.use("front_hinged_hood", x=0, y=0, z=1)
