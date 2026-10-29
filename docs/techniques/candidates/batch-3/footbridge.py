# SPDX-License-Identifier: Apache-2.0
"""Footbridge: a plank bridge over a stream, two stringers spanning stone footings, a deck of tile planks across them in
mixed browns, and a handrail on posts at each end.

m.use("footbridge", x, y, z, span=6, width=3, seed=1, wood=("Reddish Brown", "Dark Brown"), stone="Dark Bluish Gray",
title="Bridge") builds a bridge crossing along y from (x, y), on the studs at height z of the two banks, `width` studs
wide (2 to 4) and span + 2 studs long: a footing of `stone` bricks across each end (at y and y + span + 1), the gap of
`span` studs between them clear for the water. On the footings, two 1-wide plates run the full length at the bridge's
sides as stringers; across them, one plank a stud, tiles in the middle and plates at the two ends, each plank in one of
`wood` at random so the deck reads as boards. The end planks' studs carry round 1 x 1 brick posts two bricks tall, and
a plate the bridge's length on each side's posts is the handrail. span: 2 to 10, even. seed picks the planks' colors.
Returns the z of the deck's top.
"""

import random

from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3004",
    "3622",
    "3010",
    "3069b",
    "63864",
    "2431",
    "3023b",
    "3623",
    "3062b",
]

LONG = {4: "3710", 6: "3666", 8: "3460", 10: "4477", 12: "60479"}  # Plate 1 x n
FOOTING = {2: "3004", 3: "3622", 4: "3010"}  # Brick 1 x n
TILE = {2: "3069b", 3: "63864", 4: "2431"}  # Tile 1 x n
PLATE = {2: "3023b", 3: "3623", 4: "3710"}  # Plate 1 x n


def build(
    m,
    x,
    y,
    z,
    span=6,
    width=3,
    seed=1,
    wood=("Reddish Brown", "Dark Brown"),
    stone="Dark Bluish Gray",
    title="Bridge",
):
    """The bridge; returns the z of its deck's top."""
    if span + 2 not in LONG or width not in TILE:
        raise ValueError(
            f"footbridge takes an even span of 2 to 10 and a width of 2 to 4, not {span}, {width}"
        )
    if not wood:
        raise ValueError("footbridge needs at least one wood color")
    rng = random.Random(seed)
    n = span + 2
    sides = (x, x + width - 1)
    with m.step(f"{title}, footings and stringers"):
        for yf in (y, y + n - 1):
            m.add(FOOTING[width], stone, at=(x, yf, z), rot=1)
        for xs in sides:
            m.add(LONG[n], wood[0], at=(xs, y, z + 3))
    deck = []
    for j in range(n):
        part = PLATE[width] if j in (0, n - 1) else TILE[width]
        c = rng.choice(wood)
        at = (x, y + j, z + 4)
        deck.append((1, lambda p=part, c=c, at=at: m.add(p, c, at=at, rot=1)))
    in_steps(m, f"{title}, deck", deck)
    with m.step(f"{title}, posts"):
        for xs in sides:
            for yp in (y, y + n - 1):
                m.stack("3062b", wood[0], at=(xs, yp, z + 5), count=2)
    with m.step(f"{title}, handrails"):
        for xs in sides:
            m.add(LONG[n], wood[0], at=(xs, y, z + 11))
    return z + 5


def example(m):
    """A bridge over a stream 2 studs wide, between two grassy banks."""
    with m.step("Banks and stream"):
        m.add("3036", "Blue", at=(0, 0, 0), rot=1)
        m.add("3034", "Green", at=(0, 0, 1), rot=1)
        m.add("3034", "Green", at=(0, 4, 1), rot=1)
        for k in (0, 4):
            m.add("2431", "Trans-Light Blue", at=(k, 2, 1), rot=1)
            m.add("2431", "Trans-Light Blue", at=(k, 3, 1), rot=1)
    m.use("footbridge", x=2, y=0, z=2, span=4, width=3)
