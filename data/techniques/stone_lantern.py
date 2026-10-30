# SPDX-License-Identifier: Apache-2.0
"""Stone lantern: a garden lantern of stacked stone, a round plinth, a slim pillar, a glowing light chamber, and a
pyramid roof with a finial, built apart and placed beside a path.

m.use("stone_lantern", x, y, z, pillar=1, color="Dark Bluish Gray", light="Trans-Yellow", title="Lantern") stands a
lantern on the studs at height z (a baseplate is z = 0.5) over the 2 x 2 from (x, y): a round brick 2 x 2 for the
plinth, a jumper 2 x 2 that brings its single stud to the middle, `pillar` round 1 x 1 bricks, a plate 2 x 2 centered on
the pillar's stud, a round brick 2 x 2 in `light` for the light chamber, a plate 4 x 4 for the roof's eaves, four Slope
Brick 45 2 x 2 Double Convex (3045) on it in a pyramid, another jumper on their four top studs, and a cone 1 x 1 for the
finial. The roof reaches a stud past the plinth each way. pillar: 1 to 3. It's a submodel (one file for each height and
colors); returns the placed copy's Handle, whose .max[2] is the top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10315. Not derived from any
model file."""

PARTS = ["3941", "87580", "3062b", "3022", "3031", "3045", "4589"]


def _lantern(m, pillar, color, light, title):
    """The lantern in its own frame, its plinth's corner at the origin."""
    with m.step(f"{title}, plinth and pillar"):
        m.add("3941", color, at=(0, 0, 0))
        m.add("87580", color, at=(0, 0, 3))
        m.stack("3062b", color, at=(0.5, 0.5, 4), count=pillar)
    z = 4 + 3 * pillar
    with m.step(f"{title}, light"):
        # A 2 x 2 plate centered on one stud: the stud goes in its middle tube.
        m.add("3022", color, at=(0, 0, z))
        m.add("3941", light, at=(0, 0, z + 1))
    z += 4
    with m.step(f"{title}, roof"):
        m.add("3031", color, at=(-1, -1, z))
        # A double-convex slope faces back right at rot=0; each turn moves it a quarter on, counterclockwise.
        for (cx, cy), rot in (((-1, -1), 2), ((1, -1), 3), ((1, 1), 0), ((-1, 1), 1)):
            m.add("3045", color, at=(cx, cy, z + 1), rot=rot)
    with m.step(f"{title}, finial"):
        m.add("87580", color, at=(0, 0, z + 4))
        m.add("4589", color, at=(0.5, 0.5, z + 5))


def build(
    m,
    x,
    y,
    z,
    pillar=1,
    color="Dark Bluish Gray",
    light="Trans-Yellow",
    title="Lantern",
):
    """The lantern; returns the placed copy's Handle."""
    if not 1 <= pillar <= 3:
        raise ValueError(f"stone_lantern takes a pillar of 1 to 3 bricks, not {pillar}")
    lantern = m.submodel(
        "stone_lantern", _lantern, pillar=pillar, color=color, light=light, title=title
    )
    lo = lantern.box[0]
    with m.step(title):
        return m.place(lantern, at=(x + lo[0], y + lo[1], z + lo[2]))


def example(m):
    """Two lanterns either side of a path on a baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    with m.step("Path"):
        m.add("3034", "Tan", at=(7, 4, 0.5))
    m.use("stone_lantern", x=4, y=6, z=0.5, pillar=1)
    m.use("stone_lantern", x=10, y=6, z=0.5, pillar=2, color="Light Bluish Gray")
