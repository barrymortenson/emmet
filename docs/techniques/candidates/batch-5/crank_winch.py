# SPDX-License-Identifier: Apache-2.0
"""Crank winch: a drum on an axle between two Technic bricks, turned by a crank arm with a handle on the axle's end.

m.use("crank_winch", x, y, z, crank=30, color="Light Bluish Gray", drum_color="Red", axle_color="Black",
title="Winch") builds, from z, two Technic bricks 1 x 2 with a hole (3700) along y from (x, y), at x and x + 3, and
runs an axle 6L (3706) along x through their holes, 1.75 plates above z. Between the bricks it threads the drum on the
axle: a 2 x 2 round plate (4032a) as a flange each side and a 2 x 2 round brick (3941) between, all on their sides. A
bush (3713) holds the axle's left end; on the right a half bush (32123a) spaces the crank off the brick: a thin
liftarm 1 x 2 (41677) with an axle pin (3749) in its outer hole as the handle, the arm turned `crank` degrees from
straight up toward the front (-y). The drum reaches 0.75 plates below z, so stand the bricks on something that leaves
it room. It brings its own steps and returns the z of the bricks' top (z + 3).
"""

import math

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3700", "3706", "3713", "4032a", "3941", "32123a", "41677", "3749"]

ON_SIDE = Rot(y=90)  # a round part on its side, its top to +x


def _mul(a, b):
    return tuple(
        tuple(sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3))
        for i in range(3)
    )


def _raw(m, part, color, center, r_dsl):
    """A part by its origin at `center` (DSL), turned by r_dsl (a rotation in DSL axes)."""
    p = frame.to_ldraw_point(center)
    r = frame.to_ldraw_rotation(r_dsl)
    return m.add_raw(part, color, [*p, *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    crank=30,
    color="Light Bluish Gray",
    drum_color="Red",
    axle_color="Black",
    title="Winch",
):
    """The frame, the axle and drum, and the crank; returns the z of the bricks' top."""
    if not -180 <= crank <= 180:
        raise ValueError(
            f"crank_winch takes a crank angle from -180 to 180, not {crank}"
        )
    yc, zc = y + 1, z + 1.75  # the axle's line
    # A part whose length is LDraw's X (an axle, an axle pin), turned to run along DSL x.
    along = frame.rotation(0, 0, 90)
    with m.step(f"{title}: frame"):
        m.add("3700", color, at=(x, y, z))
        m.add("3700", color, at=(x + 3, y, z))
    with m.step(f"{title}: axle and drum"):
        _raw(m, "3706", axle_color, (x + 2, yc, zc), along)
        m.add("4032a", drum_color, at=(x + 1, yc - 1, zc - 2.5), rot=ON_SIDE)
        m.add("3941", drum_color, at=(x + 1.4, yc - 1, zc - 2.5), rot=ON_SIDE)
        m.add("4032a", drum_color, at=(x + 2.6, yc - 1, zc - 2.5), rot=ON_SIDE)
    with m.step(f"{title}: bushes and crank"):
        _raw(m, "3713", axle_color, (x - 0.5, yc, zc), frame.rotation())
        _raw(m, "32123a", axle_color, (x + 4.25, yc, zc), frame.rotation())
        # The arm: its holes' axis along x, its length turned up, then `crank` degrees toward the front.
        arm = _mul(frame.rotation(crank, 0, 0), frame.rotation(0, 90, 0))
        _raw(m, "41677", color, (x + 4.75, yc, zc), arm)
        a = math.radians(crank)
        handle = (x + 5, yc - math.sin(a), zc + 2.5 * math.cos(a))
        _raw(m, "3749", axle_color, handle, along)
    return z + 3


def example(m):
    """A winch on a base, its frame a brick up so the drum turns clear."""
    with m.step("Base"):
        m.add("3020", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
        m.add("3004", "Light Bluish Gray", at=(0, 0, 1))
        m.add("3004", "Light Bluish Gray", at=(3, 0, 1))
    m.use("crank_winch", x=0, y=0, z=4)
