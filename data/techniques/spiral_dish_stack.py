# SPDX-License-Identifier: Apache-2.0
"""Spiral dish stack: a round micro museum whose tiers grow wider going up, inverted dishes stacked on round plates,
their rims reading as the ramps of a spiral.

m.use("spiral_dish_stack", x, y, z, tiers=3, gap=1, color="White", core="Light Bluish Gray", skylight="Trans Clear",
title="Museum") stands, on the stud at (x, y) at height z, a round 1 x 1 brick in the core color, and on it `tiers`
inverted dishes centered on that stud, a 2 x 2 (4740) at the bottom and 4 x 4s (3960) over it, each on `gap` round 1 x 1
plates (6141) in the core color stacked on the center stud of the one below, so every tier stands clear of the next and
the stack widens going up. A round plate in the skylight color tops it. tiers is 2 to 4; the 4 x 4 dishes reach 1.5
studs past the stud each way, so keep that clear. (The 3 x 3 and 6 x 6 dishes, 43898 and 44375b, have no socket over a
stud in the parts data, so they aren't used.) Returns the z of the skylight's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = ["3062b", "4740", "3960", "6141"]

DISHES = [
    ("4740", 2, 1),
    ("3960", 4, 2),
    ("3960", 4, 2),
    ("3960", 4, 2),
]  # part, width, height in plates


def build(
    m,
    x,
    y,
    z,
    tiers=3,
    gap=1,
    color="White",
    core="Light Bluish Gray",
    skylight="Trans Clear",
    title="Museum",
):
    """The core, the dishes, and the skylight; returns the z of the top."""
    if not 2 <= tiers <= 4 or not 1 <= gap <= 3:
        raise ValueError(
            f"spiral_dish_stack takes 2 to 4 tiers and a gap of 1 to 3, not {tiers}, {gap}"
        )
    cx, cy = x + 0.5, y + 0.5
    with m.step(f"{title}: core"):
        m.add("3062b", core, at=(x, y, z))
    zt = z + 3
    for t, (part, w, h) in enumerate(DISHES[:tiers]):
        with m.step(f"{title}: tier {t + 1}"):
            if t:
                m.stack("6141", core, at=(x, y, zt), count=gap)
                zt += gap
            m.add(part, color, at=(cx - w / 2, cy - w / 2, zt))
            zt += h
    with m.step(f"{title}: skylight"):
        m.add("6141", skylight, at=(x, y, zt))
    return zt + 1


def example(m):
    """A three-tier museum on a jumper plate in the middle of a tiled plaza."""
    from emmet_dsl import Plate, Tile

    with m.step("Plaza"):
        m.add(Plate(8, 8), "Dark Bluish Gray", at=(0, 0, 0))
    with m.step("Paving"):
        m.fill(Tile, "Light Bluish Gray", at=(0, 0, 1), size=(8, 3))
        m.fill(Tile, "Light Bluish Gray", at=(0, 5, 1), size=(8, 3))
        m.fill(Tile, "Light Bluish Gray", at=(0, 3, 1), size=(3, 2))
        m.fill(Tile, "Light Bluish Gray", at=(5, 3, 1), size=(3, 2))
    with m.step("Base"):
        m.add("87580", "Light Bluish Gray", at=(3, 3, 1))
    m.use("spiral_dish_stack", x=3.5, y=3.5, z=2, tiers=3, gap=2)
