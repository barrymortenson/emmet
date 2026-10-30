# SPDX-License-Identifier: Apache-2.0
"""Garden boulder: a smooth, faceted stone with a ridge along its back, from 33-degree slopes set in pairs that lean
together, rounded at its ends by double-convex corners, under a smaller cap of 45-degree slopes crested with cheese
slopes.

m.use("garden_boulder", x, y, z, length=2, crest=True, seed=1, colors=("Light Bluish Gray", "Dark Bluish Gray"),
pebbles=2, title="Boulder") builds, on the studs at height z (a baseplate is z = 0.5), a stone 2 x length + 6 studs
along x from x and 6 studs deep from y: `length` pairs of Slope Brick 33 3 x 2 (3298), one facing front and one back,
their high edges meeting in a ridge two studs wide along the middle, and at each end two Slope Brick 33 3 x 3 Double
Convex (3675) that round the ends off. On the ridge's studs, a cap of the same shape a brick up and a stud in from each
end: Slope Brick 45 2 x 2 Double Convex (3045) at its corners and Slope Brick 45 2 x 2 (3039) in pairs between, its low
edges hanging over the skirt. With crest=True, the cap's ridge carries cheese slopes 1 x 2 facing out each way, so the
top is soft, not studded. Most pieces are the first of `colors`, a few the second at random, so the stone is mottled.
`pebbles` round 1 x 1 plates lie at its foot. length: 1 to 5. seed picks the colors and pebbles. Returns the z of its
top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 10315 and 21363. Not derived
from any model file."""

PARTS = ["3298", "3675", "3045", "3039", "85984", "6141"]


def build(
    m,
    x,
    y,
    z,
    length=2,
    crest=True,
    seed=1,
    colors=("Light Bluish Gray", "Dark Bluish Gray"),
    pebbles=2,
    title="Boulder",
):
    """The slopes, the crest, and the pebbles; returns the z of the top."""
    if not 1 <= length <= 5:
        raise ValueError(f"garden_boulder takes a length of 1 to 5 pairs, not {length}")
    rng = random.Random(seed)

    def color():
        return (
            colors[0]
            if len(colors) == 1 or rng.random() < 0.65
            else rng.choice(colors[1:])
        )

    end = x + 3 + 2 * length  # where the right-hand corners start
    with m.step(f"{title}, left end"):
        # A double-convex corner faces back right at rot=0; each turn moves it a quarter on, counterclockwise.
        m.add("3675", color(), at=(x, y, z), rot=2)
        m.add("3675", color(), at=(x, y + 3, z), rot=1)
    with m.step(f"{title}, slopes"):
        for k in range(length):
            m.add("3298", color(), at=(x + 3 + 2 * k, y, z), rot=3)
            m.add("3298", color(), at=(x + 3 + 2 * k, y + 3, z), rot=1)
    with m.step(f"{title}, right end"):
        m.add("3675", color(), at=(end, y, z), rot=3)
        m.add("3675", color(), at=(end, y + 3, z), rot=0)
    # The cap: a narrower ridged unit of 45-degree slopes on the ridge's studs, a stud in from each end, its low
    # edges hanging over the skirt below.
    top = z + 3
    with m.step(f"{title}, cap"):
        m.add("3045", color(), at=(x + 2, y + 1, top), rot=2)
        m.add("3045", color(), at=(x + 2, y + 3, top), rot=1)
        for k in range(length - 1):
            m.add("3039", color(), at=(x + 4 + 2 * k, y + 1, top), rot=3)
            m.add("3039", color(), at=(x + 4 + 2 * k, y + 3, top), rot=1)
        m.add("3045", color(), at=(end - 1, y + 1, top), rot=3)
        m.add("3045", color(), at=(end - 1, y + 3, top), rot=0)
    top += 3
    if crest:
        # The cap's ridge: two rows of studs from its left corners' inner studs to its right ones'.
        with m.step(f"{title}, crest"):
            for k in range(length):
                # A cheese slope 1 x 2 runs along x facing front (-y) at rot=3, and back (+y) at rot=1.
                m.add("85984", color(), at=(x + 3 + 2 * k, y + 2, top), rot=3)
                m.add("85984", color(), at=(x + 3 + 2 * k, y + 3, top), rot=1)
        top += 2
    if pebbles:
        spots = [
            (x + 1 + 2 * k, py) for k in range(length + 2) for py in (y - 1, y + 6)
        ]
        with m.step(f"{title}, pebbles"):
            for px, py in rng.sample(spots, min(pebbles, len(spots))):
                m.add("6141", colors[-1], at=(px, py, z))
    return top


def example(m):
    """A boulder of one pair, with its crest and two pebbles, on a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("garden_boulder", x=4, y=5, z=0.5, length=1, seed=3)
