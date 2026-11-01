# SPDX-License-Identifier: Apache-2.0
"""Wing plate porch roof: a light porch roof over a side door, one tapered wing plate off the wall on turned posts and a
corbel.

m.use("wing_plate_porch_roof", x, y, z, ground=0, corbel=True, color="Light Bluish Gray", posts="Reddish Brown",
wall="Tan", title="Porch roof") lays a Wing 3 x 12 (47397), studs up, along a wall one stud thick along x facing front (-y), from x
to x + 12 at z, the top of the wall: its straight back row on the wall and its wide end at x + 12, two studs out. Two
posts hold its front row at the wide end, at x + 7 and x + 11, from the ground at `ground` up to z, each turned from
round 1 x 1 bricks with a round plate on each; an Inverted Slope 45 2 x 1 under its narrow end, in the wall's cell x + 2
of the course under the wing (z - 3 to z), is the corbel: lay that course without that cell, or pass corbel=False on a
wall that's whole (a door_frame's). The door goes under the wing in the wall's cells x + 3 to x + 7, clear of the
corbel and the posts, and z is at or above its top: 5 bricks over the ground at least, 6 over a 1 x 4 x 6 door's sill
(docs/spec/building-rules.md, rule 1). Returns the z of the wing's top, with .posts (the (x, y) of each post) and
.underside (z).
"""

from emmet_dsl.core import HEADROOM, fit, in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["47397", "3665b", "3062b", "6141", "3024"]


def _post(m, at, height, color):
    """A turned post `height` plates tall, as in_steps actions: round bricks, each with a round plate on it (a tier an
    action), and plates to make up."""
    x, y, z = at
    top = z + height
    out = []
    while top - z >= 4:
        out.append(
            (
                2,
                lambda z=z: (
                    m.add("3062b", color, at=(x, y, z)),
                    m.add("6141", color, at=(x, y, z + 3)),
                ),
            )
        )
        z += 4
    if top - z == 3:
        out.append((1, lambda z=z: m.add("3062b", color, at=(x, y, z))))
    else:
        for k in range(round(top - z)):
            out.append((1, lambda k=k, z=z: m.add("6141", color, at=(x, y, z + k))))
    return out


def build(
    m,
    x,
    y,
    z,
    ground=0,
    corbel=True,
    color="Light Bluish Gray",
    posts="Reddish Brown",
    wall="Tan",
    title="Porch roof",
):
    """The posts, the corbel, and the wing; returns the z of the wing's top."""
    if z - ground < HEADROOM:
        raise ValueError(
            f"wing_plate_porch_roof needs the wing 5 bricks ({HEADROOM} plates) or more over the ground, so a "
            f"figure walks under it, not {z - ground}"
        )
    spots = [(x + 7, y - 2), (x + 11, y - 2)]
    for k, (px, py) in enumerate(spots):
        in_steps(
            m, f"{title}: post {k + 1}", _post(m, (px, py, ground), z - ground, posts)
        )
    with m.step(f"{title}: corbel and wing" if corbel else f"{title}: wing"):
        if corbel:
            m.add("3665b", wall, at=(x + 2, y - 1, z - 3), rot=1)
        m.add("47397", color, at=(x, y - 2, z), rot=2)
    return fit(z + 1, posts=spots, underside=z)


def example(m):
    """A wall with a side door's porch roof over it, the corbel in its top course."""
    from emmet_dsl import Brick

    for r in range(6):
        with m.step(f"Wall, row {r + 1}"):
            first = 2 if r % 2 else None
            if r < 5:
                m.row(Brick, "Tan", at=(0, 0, 3 * r), length=12, first=first)
            else:
                m.row(Brick, "Tan", at=(0, 0, 3 * r), length=2)
                m.row(Brick, "Tan", at=(3, 0, 3 * r), length=9)
    m.use("wing_plate_porch_roof", x=0, y=0, z=18)
