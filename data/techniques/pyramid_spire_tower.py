# SPDX-License-Identifier: Apache-2.0
"""Pyramid spire tower: a square brick tower 4 x 4 with a window in each face at the top, a corbel plate that
overhangs a stud all around, and a steep pyramid roof of slopes ending in a spire.

m.use("pyramid_spire_tower", x, y, z, rows=3, color="Reddish Brown", roof="Dark Bluish Gray",
trim="Light Bluish Gray", title="Tower") builds, on the studs at height z, a tower whose 4 x 4 walls cover x to x + 4
and y to y + 4: `rows` courses of walls (m.wall, corners interlocking), then a stage two bricks tall with a window
1 x 2 x 2 (and its glass) in the middle of each face between 1 x 1 brick corners, a plate 6 x 6 a stud out all around,
and the roof: a ring of slopes 6 wide (a corner slope 2 x 2 at each corner, a 2 x 2 slope in the middle of each side,
a 2 x 2 brick in the center), four corner slopes 4 wide over it, a quadruple convex slope 75 on their four studs, and
a spire (an antenna 4H on a round plate). It's hollow under the corbel. Returns the z of the spire's top.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = [
    "3005",
    "60592",
    "60601",
    "3958",
    "3045",
    "3039",
    "3003",
    "3688",
    "6141",
    "3957b",
]

# The quarter turn that points a slope down toward each side; a corner slope's turn for the corner it caps.
DOWN = {"+x": 0, "+y": 1, "-x": 2, "-y": 3}
CORNER = {"back-right": 0, "back-left": 1, "front-left": 2, "front-right": 3}


def _window(m, color, at, rot):
    """A window 1 x 2 x 2 with its glass, centered in the frame as roof_dormer sets it."""
    f = m.add("60592", color, at=at, rot=rot)
    (x0, y0, z0), (x1, y1, _) = f.min, f.max
    if rot % 2:
        glass = ((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875)
    else:
        glass = ((x0 + x1 - 0.2) / 2, (y0 + y1 - 1.65) / 2, z0 + 0.875)
    m.add("60601", "Trans-Clear", at=glass, rot=rot)


def _corners(m, part, color, x, y, z, w):
    """A corner slope on each corner of a w-wide square from (x, y)."""
    for name, (cx, cy) in (
        ("front-left", (0, 0)),
        ("front-right", (w - 2, 0)),
        ("back-right", (w - 2, w - 2)),
        ("back-left", (0, w - 2)),
    ):
        m.add(part, color, at=(x + cx, y + cy, z), rot=CORNER[name])


def build(
    m,
    x,
    y,
    z,
    rows=3,
    color="Reddish Brown",
    roof="Dark Bluish Gray",
    trim="Light Bluish Gray",
    title="Tower",
):
    """The walls, the window stage, the corbel, and the roof; returns the z of the spire's top."""
    for r in range(rows):
        with m.step(f"{title}, row {r + 1}"):
            m.wall(x, y, z + 3 * r, length=4, depth=4, color=color, kind=Brick)
    zw = z + 3 * rows
    with m.step(f"{title}, windows: corners"):
        for cx, cy in ((0, 0), (3, 0), (3, 3), (0, 3)):
            m.stack("3005", color, at=(x + cx, y + cy, zw), count=2)
    with m.step(f"{title}, windows"):
        _window(m, color, (x + 1, y, zw), 1)
        _window(m, color, (x + 1, y + 3, zw), 1)
        _window(m, color, (x, y + 1, zw), 0)
        _window(m, color, (x + 3, y + 1, zw), 0)
    zc = zw + 6
    with m.step(f"{title}: corbel"):
        m.add("3958", trim, at=(x - 1, y - 1, zc))
    zr = zc + 1
    with m.step(f"{title}, roof: first ring"):
        _corners(m, "3045", roof, x - 1, y - 1, zr, 6)
        m.add("3003", roof, at=(x + 1, y + 1, zr))
    with m.step(f"{title}, roof: first ring, sides"):
        m.add("3039", roof, at=(x + 1, y - 1, zr), rot=DOWN["-y"])
        m.add("3039", roof, at=(x + 3, y + 1, zr), rot=DOWN["+x"])
        m.add("3039", roof, at=(x + 1, y + 3, zr), rot=DOWN["+y"])
        m.add("3039", roof, at=(x - 1, y + 1, zr), rot=DOWN["-x"])
    with m.step(f"{title}, roof: second ring"):
        _corners(m, "3045", roof, x, y, zr + 3, 4)
    with m.step(f"{title}: spire"):
        m.add("3688", roof, at=(x + 1, y + 1, zr + 6))
        m.add("6141", trim, at=(x + 1.5, y + 1.5, zr + 12))
        top = m.add("3957b", trim, at=(x + 1.6, y + 1.6, zr + 13))
    return top.max[2]


def example(m):
    """A tower on a plate."""
    with m.step("Base"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("pyramid_spire_tower", x=0, y=0, z=1)
