# SPDX-License-Identifier: Apache-2.0
"""Stave cylinder: a smooth round column about 6 studs across, four staves of curved slopes clipped around a core with
studs on all four sides.

m.use("stave_cylinder", x, y, z, units=2, color="White", core="Light Bluish Gray", band="Orange", band_every=0,
title="Cylinder") stands a core 4 x 4 studs on the studs at (x, y, z): in each unit, a course of side-stud bricks (a
1 x 1 with studs on four sides at each corner and a 1 x 2 with studs on one side between) and two 4 x 4 plates, 5 plates
in all, so the side studs of one unit and the next are two studs apart. Each face then takes a stave built flat as a
submodel: a 4 x (2 x units) plate with a pair of curved slopes 2 x 1 on each row, back to back, so its outer face is a
shallow arc. Stood up on the core's side studs, the four arcs make a round section 6.4 studs across. Every
`band_every`-th row of slopes (0 for none) is in `band`, which reads as a ring around the cylinder. A 4 x 4 round plate
caps the core. The staves start half a plate up the core and reach half a plate up its cap. Returns the z of the cap's
top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10341. Not derived from any
model file."""

PARTS = [
    "4733",
    "11211",
    "3031",
    "11477",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "60474",
]

# A stave built flat, its plate's underside to the core: turned about x it faces front (-y), then about z round to
# the core's other faces.
FACES = {
    "front": Rot(x=90),
    "right": Rot(x=90, z=90),
    "back": Rot(x=90, z=180),
    "left": Rot(x=90, z=270),
}
THICK = 1.2  # a stave's plate and slopes, 3 plates, in studs
LIFT = 0.5  # side studs are 1.75 plates up a brick, a stave's first anti-studs 1.25 plates up the stave


def _stave(m, rows, color, band, band_every, title):
    """One stave in its own frame: the plate along y, slopes falling away to x = 0 and x = 4."""
    with m.step(f"{title}, stave: plate"):
        m.fill(Plate, color, at=(0, 0, 0), size=(4, rows))
    for r0 in range(0, rows, 4):
        with m.step(f"{title}, stave: slopes {r0 // 4 + 1}"):
            for r in range(r0, min(rows, r0 + 4)):
                c = band if band_every and r % band_every == band_every - 1 else color
                m.add("11477", c, at=(0, r, 1), rot=2)
                m.add("11477", c, at=(2, r, 1), rot=0)


def build(
    m,
    x,
    y,
    z,
    units=2,
    color="White",
    core="Light Bluish Gray",
    band="Orange",
    band_every=0,
    title="Cylinder",
):
    """The core, its four staves, and the cap; returns the z of the cap's top."""
    if not 1 <= units <= 6:
        raise ValueError(f"stave_cylinder takes 1 to 6 units, not {units}")
    for u in range(units):
        zu = z + 5 * u
        with m.step(f"{title}, core {u + 1}: bricks"):
            for cx, cy in ((0, 0), (3, 0), (0, 3), (3, 3)):
                m.add("4733", core, at=(x + cx, y + cy, zu))
            m.add("11211", core, at=(x + 1, y, zu), rot=3)
            m.add("11211", core, at=(x + 1, y + 3, zu), rot=1)
            m.add("11211", core, at=(x, y + 1, zu), rot=2)
            m.add("11211", core, at=(x + 3, y + 1, zu), rot=0)
        with m.step(f"{title}, core {u + 1}: plates"):
            m.add("3031", core, at=(x, y, zu + 3))
            m.add("3031", core, at=(x, y, zu + 4))
    top = z + 5 * units
    with m.step(f"{title}, cap"):
        m.add("60474", core, at=(x, y, top))
    stave = m.submodel(
        "stave",
        _stave,
        rows=2 * units,
        color=color,
        band=band,
        band_every=band_every,
        title=title,
    )
    at = {
        "front": (x, y - THICK, z + LIFT),
        "right": (x + 4, y, z + LIFT),
        "back": (x, y + 4, z + LIFT),
        "left": (x - THICK, y, z + LIFT),
    }
    for pair in (("front", "right"), ("back", "left")):
        with m.step(f"{title}, staves: {' and '.join(pair)}"):
            for face in pair:
                m.place(
                    stave,
                    at=at[face],
                    rot=FACES[face],
                    id=f"{title.lower()}/stave-{face}",
                )
    return top + 1


def example(m):
    """A cylinder three units tall with an orange ring, on a plate."""
    with m.step("Base"):
        m.add("3958", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("stave_cylinder", x=1, y=1, z=1, units=3, band_every=3)
