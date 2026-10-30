# SPDX-License-Identifier: Apache-2.0
"""Fender crown: the hood and front fenders of a classic sports car, the fenders riding higher than the hood and both
rolling down to a small nose in curved slopes, with inverted curves under the nose for the bottom of the roll.

m.use("fender_crown", x, y, z, length=10, width=6, color="Dark Green", title="Hood") builds a hood `length` studs long
along x from x, its nose at x (to -x), `width` studs along y from y, on plates at z: along each side a fender, a plate 1
x `length` with a curved slope 6 x 1 (42022) at the nose and a brick and tile behind it, its top 5 plates up; between
them the hood, plates and tiles 2 plates up, rolled down at the nose by curved slopes 2 x 2 (15068); and under the nose
a row of inverted curved slopes 2 x 1 (24201) that hangs 2 plates below z, so leave the body under the nose's first 2
studs that much lower. It's built apart as a submodel and set on. length: 8, 10, or 12; width: 4, 6, 8, or 10. Returns
the z of the fenders' top (z + 5).
"""

from emmet_dsl import Brick, Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11381. Not derived from any
model file."""

PARTS = [
    "42022",
    "15068",
    "24201",
    "3005",
    "3004",
    "3010",
    "3009",
    "3023b",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3069b",
    "2431",
    "6636",
]


def _hood(m, length, width, color):
    """The hood in its own frame: the base's front-left corner at the origin, its nose to -x."""
    inner = width - 2
    with m.step("Hood: base"):
        m.fill(Plate, color, at=(0, 0, 0), size=(length, width))
    with m.step("Hood: under the nose"):
        for k in range(width):
            # Its curve to the nose, under the base's edge.
            m.add("24201", color, at=(0, k, -2), rot=2)
    back = length - 6
    for side in (0, width - 1):
        with m.step(f"Hood: fender, {'left' if side == 0 else 'right'}"):
            m.add(Plate(1, length), color, at=(0, side, 1), rot=1)
            m.add("42022", color, at=(0, side, 2), rot=2)  # falls to the nose
            m.add(Brick(1, back), color, at=(6, side, 2), rot=1)
            m.add(Tile(1, back), color, at=(6, side, 5), rot=1)
    with m.step("Hood: hood"):
        m.fill(Plate, color, at=(2, 1, 1), size=(length - 2, inner))
        for k in range(0, inner, 2):
            m.add("15068", color, at=(0, 1 + k, 1), rot=2)
    with m.step("Hood: hood tiles"):
        m.fill(Tile, color, at=(2, 1, 2), size=(length - 2, inner))


def build(m, x, y, z, length=10, width=6, color="Dark Green", title="Hood"):
    """The hood, built apart and set on; returns the z of the fenders' top."""
    if length not in (8, 10, 12):
        raise ValueError(f"fender_crown takes a length of 8, 10, or 12, not {length}")
    if width not in (4, 6, 8, 10):
        raise ValueError(f"fender_crown takes a width of 4, 6, 8, or 10, not {width}")
    hood = m.submodel(
        "fender_crown_hood", _hood, length=length, width=width, color=color
    )
    (bx, by, bz), _ = hood.box
    with m.step(title):
        m.place(hood, at=(x + bx, y + by, z + bz))
    return z + 5


def example(m):
    """A hood on the front of a car's body."""
    with m.step("Floor"):
        m.add("3033", "Black", at=(0, 0, 0), rot=1)
    with m.step("Body"):
        m.row(Brick, "Dark Green", at=(2, 0, 1), length=8, width=2)
        m.row(Brick, "Dark Green", at=(2, 4, 1), length=8, width=2)
        m.row(Brick, "Black", at=(2, 2, 1), length=8, width=2)
    m.use("fender_crown", x=0, y=0, z=4)
