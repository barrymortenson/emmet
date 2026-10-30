# SPDX-License-Identifier: Apache-2.0
"""Micro house on wall: tiny houses a stud or two wide standing on ledges in the face of a bigger wall, at different
heights, so a figure-scale city wall reads as a far-off town climbing behind it.

m.use("micro_house_on_wall", x, y, z, length=10, tiers=3, houses=5, seed=1, wall="Light Bluish Gray", house="White",
roofs=("Sand Green", "Reddish Brown", "Dark Bluish Gray"), title="Wall") builds a wall a stud thick and `length` studs
long along x, its face at y + 1, on the studs at height z: `tiers` tiers, each two courses of bricks laid in running
bond and a course of plates. In each plate course but the top one, a few cells take a plate a stud longer, reaching out
in front of the wall as a ledge, held down by the bricks over its back half. On each ledge stands a micro house facing
front: one stud wide, a brick with a headlight (4070, its recess the window) under a cheese slope; or two studs wide,
two of them under a 45 degree double slope (3044c) as a gable. Each roof is one of `roofs`. `houses` houses (or as many
as fit, a stud apart) go on the ledges at random. seed picks the places. Returns the z of the wall's top.
"""

import random

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = [
    "3004",
    "3010",
    "3009",
    "3008",
    "3622",
    "3005",
    "3023b",
    "3022",
    "3710",
    "3666",
    "3460",
    "4070",
    "54200",
    "3044c",
]

TIER = 7  # two courses of bricks and one of plates, in plates


def build(
    m,
    x,
    y,
    z,
    length=10,
    tiers=3,
    houses=5,
    seed=1,
    wall="Light Bluish Gray",
    house="White",
    roofs=("Sand Green", "Reddish Brown", "Dark Bluish Gray"),
    title="Wall",
):
    """The wall tiers, the ledges, and the houses; returns the z of the top."""
    if length < 4 or tiers < 2 or houses < 0 or not roofs:
        raise ValueError(
            f"micro_house_on_wall takes a length of 4 or more, 2 or more tiers, and a roof color, not {length}, {tiers}"
        )
    rng = random.Random(seed)
    # Pick the houses: (tier, x offset, width), a stud or more apart within a tier.
    picked, taken = [], {t: set() for t in range(tiers - 1)}
    for _ in range(houses):
        spots = [
            (t, i, w)
            for t in range(tiers - 1)
            for w in (1, 2)
            for i in range(length - w + 1)
            if not set(range(i - 1, i + w + 1)) & taken[t]
        ]
        if not spots:
            break
        t, i, w = rng.choice(spots)
        taken[t] |= set(range(i, i + w))
        picked.append((t, i, w))
    fy = y + 1  # the wall's face
    for t in range(tiers):
        zt = z + TIER * t
        for r in range(2):
            with m.step(f"{title}, tier {t + 1}: row {r + 1}"):
                m.wall(x, fy, zt + 3 * r, length=length, color=wall)
        zp = zt + 6
        ledges = sorted((i, w) for tt, i, w in picked if tt == t)
        with m.step(f"{title}, tier {t + 1}: plates"):
            start = 0
            for i, w in ledges + [(length, 0)]:
                if i > start:
                    m.row(Plate, wall, at=(x + start, fy, zp), length=i - start)
                if w:
                    m.add(Plate(w, 2), wall, at=(x + i, y, zp))
                start = i + w
        if ledges:
            with m.step(f"{title}, tier {t + 1}: houses"):
                for i, w in ledges:
                    hx, hz = x + i, zp + 1
                    for k in range(w):
                        m.add("4070", house, at=(hx + k, y, hz), rot=3)
                    if w == 1:
                        m.add("54200", rng.choice(roofs), at=(hx, y, hz + 3), rot=3)
                    else:
                        m.add("3044c", rng.choice(roofs), at=(hx, y, hz + 3))
    return z + TIER * tiers


def example(m):
    """A stretch of city wall three tiers tall with five houses on its face."""
    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=10, width=2)
    m.use("micro_house_on_wall", x=0, y=0, z=1, length=10, tiers=3, houses=5, seed=4)
