# SPDX-License-Identifier: Apache-2.0
"""Wing plate porch roof: a light porch roof over a side door, one tapered wing plate off the wall on turned posts and a
corbel.

m.use("wing_plate_porch_roof", x, y, z, ground=0, color="Light Bluish Gray", posts="Reddish Brown", wall="Tan",
title="Porch roof") lays a Wing 3 x 12 (47397), studs up, along a wall one stud thick along x facing front (-y), from x
to x + 12 at z, the top of the wall: its straight back row on the wall and its wide end at x + 12, two studs out. Two
posts hold its front row at the wide end, at x + 7 and x + 11, from the ground at `ground` up to z, each turned from
round 1 x 1 bricks with a round plate on each; an Inverted Slope 45 2 x 1 under its narrow end, in the wall's cell x + 2
of the course under the wing (z - 3 to z), is the corbel. Lay that course without that cell. Returns the z of the wing's
top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["47397", "3665b", "3062b", "6141", "3024"]


def _post(m, at, height, color):
    """A turned post `height` plates tall: round bricks, each with a round plate on it, and plates to make up."""
    x, y, z = at
    top = z + height
    while top - z >= 4:
        m.add("3062b", color, at=(x, y, z))
        m.add("6141", color, at=(x, y, z + 3))
        z += 4
    if top - z == 3:
        m.add("3062b", color, at=(x, y, z))
    else:
        for k in range(top - z):
            m.add("6141", color, at=(x, y, z + k))


def build(
    m,
    x,
    y,
    z,
    ground=0,
    color="Light Bluish Gray",
    posts="Reddish Brown",
    wall="Tan",
    title="Porch roof",
):
    """The posts, the corbel, and the wing; returns the z of the wing's top."""
    if z - ground < 3:
        raise ValueError(
            f"wing_plate_porch_roof needs the wing 3 plates or more over the ground, not {z - ground}"
        )
    for k, px in enumerate((x + 7, x + 11)):
        with m.step(f"{title}: post {k + 1}"):
            _post(m, (px, y - 2, ground), z - ground, posts)
    with m.step(f"{title}: corbel and wing"):
        m.add("3665b", wall, at=(x + 2, y - 1, z - 3), rot=1)
        m.add("47397", color, at=(x, y - 2, z), rot=2)
    return z + 1


def example(m):
    """A wall with a side door's porch roof over it, the corbel in its top course."""
    from emmet_dsl import Brick

    for r in range(5):
        with m.step(f"Wall, row {r + 1}"):
            first = 2 if r % 2 else None
            if r < 4:
                m.row(Brick, "Tan", at=(0, 0, 3 * r), length=12, first=first)
            else:
                m.row(Brick, "Tan", at=(0, 0, 3 * r), length=2)
                m.row(Brick, "Tan", at=(3, 0, 3 * r), length=9)
    m.use("wing_plate_porch_roof", x=0, y=0, z=15)
