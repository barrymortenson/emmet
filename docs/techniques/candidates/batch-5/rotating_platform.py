# SPDX-License-Identifier: Apache-2.0
"""Rotating platform: a deck on a 4 x 4 turntable, built as one unit on the turntable's top so the whole deck turns.

m.use("rotating_platform", x, y, z, size=6, angle=0, color="Dark Bluish Gray", base_color="Light Bluish Gray",
title="Platform") puts a turntable base 4 x 4 (3403, in base_color, like its top) on the studs at z, centered on the
size × size square whose front-left corner is (x, y), and on it the platform, built apart as a sub-assembly: the
turntable's top (3404) with a deck of plates `size` studs square over it and tiles around the edge, leaving the middle
studs to build on. It goes on turned `angle` degrees about the turntable's center (counterclockwise from above). size
is even, 4 to 10; the deck overhangs the turntable by (size - 4) / 2 studs all round. It brings its own steps and
returns the z of the plates' top inside the rim of tiles (z + 4), where a crane cab, a gun, or a display goes.
"""

from emmet_dsl import Plate, Rot, Tile, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = [
    "3403",
    "3404",
    "3020",
    "3795",
    "3034",
    "3832",
    "2431",
    "6636",
    "4162",
    "3069b",
    "63864",
    "3070b",
]


def _deck(m, size, color, turntable_color):
    """The platform in its own frame: the turntable's top centered under a deck of `size` × `size`, edge tiles."""
    c = (size - 4) / 2
    with m.step("Turntable top"):
        m.add("3404", turntable_color, at=(c, c, 0))
    with m.step("Deck"):
        m.fill(Plate, color, at=(0, 0, 2.25), size=(size, size))
    with m.step("Edge tiles"):
        m.row(Tile, color, at=(0, 0, 3.25), length=size)
        m.row(Tile, color, at=(0, size - 1, 3.25), length=size)
        m.row(Tile, color, at=(0, 1, 3.25), length=size - 2, axis="y")
        m.row(Tile, color, at=(size - 1, 1, 3.25), length=size - 2, axis="y")


def build(
    m,
    x,
    y,
    z,
    size=6,
    angle=0,
    color="Dark Bluish Gray",
    base_color="Light Bluish Gray",
    title="Platform",
):
    """The turntable base and the turning deck; returns the z of the deck's top."""
    if size < 4 or size > 10 or size % 2:
        raise ValueError(
            f"rotating_platform takes an even size from 4 to 10, not {size}"
        )
    c = (size - 4) / 2
    with m.step(f"{title}: turntable base"):
        m.add("3403", base_color, at=(x + c, y + c, z))
    sub = m.submodel(
        f"{title}: deck", _deck, size=size, color=color, turntable_color=base_color
    )
    # Put the deck's own center (size/2, size/2) on the turntable's, with the top's underside 0.75 plates up the base.
    pos, r, _ = frame.place(
        frame.box_in_ldraw(*sub.box), (0, 0, 0), frame.rotation(0, 0, angle)
    )
    p = frame.to_ldraw_point((size / 2, size / 2, 0))
    w = [sum(r[i][k] * p[k] for k in range(3)) + pos[i] for i in range(3)]
    landed = (-w[2] / frame.STUD, w[0] / frame.STUD, -w[1] / frame.PLATE)
    with m.step(f"{title}: deck"):
        m.place(
            sub,
            at=(
                x + size / 2 - landed[0],
                y + size / 2 - landed[1],
                z + 0.75 - landed[2],
            ),
            rot=Rot(z=angle),
        )
    return z + 4


def example(m):
    """A 6 x 6 platform on a base, turned a little."""
    with m.step("Base"):
        m.add("3958", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("rotating_platform", x=0, y=0, z=1, size=6, angle=20)
