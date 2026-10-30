# SPDX-License-Identifier: Apache-2.0
"""Axle round stack: a thin round booster, tank, or pole, 2 x 2 round bricks threaded on a Technic axle, with dark round
plates as bands and a cone for a nose.

m.use("axle_round_stack", x, y, z, bricks=6, band_every=2, color="White", band="Black", nose="White", title="Booster")
stands, on the studs at height z with its corner at (x, y), a stack of `bricks` 2 x 2 round bricks (3941), a 2 x 2 round
plate (4032a) in `band` after every `band_every` of them (0 for none) as a ring, and a cone 2 x 2 x 2 (3942c) on top as
the nose. The longest Technic axle that fits runs up the middle through all their axle holes from the bottom, so the
stack stays straight and can be clipped or laid on its side. Returns the z of the top of the cone.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10341 and set 42221. Not
derived from any model file."""

PARTS = ["3941", "4032a", "3942c", "3705", "3706", "3707", "3737", "3708"]

AXLES = {12: "3708", 10: "3737", 8: "3707", 6: "3706", 4: "3705"}  # by length in studs
UP = Rot(x=90)  # an axle stood on end
AXLE = 0.6  # an axle's thickness, in studs


def build(
    m,
    x,
    y,
    z,
    bricks=6,
    band_every=2,
    color="White",
    band="Black",
    nose="White",
    title="Booster",
):
    """The round bricks and bands, the cone, and the axle; returns the z of the top of the cone."""
    if not 2 <= bricks <= 16:
        raise ValueError(f"axle_round_stack takes 2 to 16 bricks, not {bricks}")
    zz = z
    for i in range(0, bricks, 2):
        with m.step(f"{title}, stack {i // 2 + 1}"):
            for k in range(i, min(bricks, i + 2)):
                m.add("3941", color, at=(x, y, zz))
                zz += 3
                if band_every and (k + 1) % band_every == 0 and k + 1 < bricks:
                    m.add("4032a", band, at=(x, y, zz))
                    zz += 1
    with m.step(f"{title}, axle and nose"):
        # An axle is 2.5 plates a stud: the longest that stays inside the stack.
        length = max((n for n in AXLES if 2.5 * n <= zz - z), default=None)
        if length:
            m.add(
                AXLES[length],
                "Black",
                at=(x + 1 - AXLE / 2, y + 1 - AXLE / 2, z),
                rot=UP,
            )
        m.add("3942c", nose, at=(x, y, zz))
    return zz + 6


def example(m):
    """A core stage and a shorter booster beside it, on a plate."""
    with m.step("Pad"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("axle_round_stack", x=0, y=1, z=1, bricks=6, title="Core")
    m.use(
        "axle_round_stack",
        x=2,
        y=1,
        z=1,
        bricks=4,
        band_every=0,
        color="Light Bluish Gray",
        title="Booster",
    )
