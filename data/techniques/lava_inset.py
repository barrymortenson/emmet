# SPDX-License-Identifier: Apache-2.0
"""Lava inset: a glowing square of lava set a plate down into a stone floor of blocks, for a forge or a cave.

m.use("lava_inset", x, y, z, length=2, depth=4, seed=1, stone=("Light Bluish Gray", "Dark Bluish Gray"),
lava="Orange", glow="Trans Orange", title="Lava") builds, on the studs at height z, a stone rim of 2 x 2 blocks two
studs wide round a length x depth pit, (length + 4) x (depth + 4) studs in all from (x, y). Each rim block is a 2 x 2
brick in one of the `stone` shades, by seed, with a 2 x 2 jumper on top, so the floor is smooth but for one stud a
block. In the pit, two layers of plates in the lava color and a layer of 1 x 2 tiles in the glow color, whose top is
a plate below the rim's: it reads as a pool. length and depth: even. Returns the z of the rim's top.
"""

import random

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3003", "87580", "3020", "3022", "3069b"]


def build(
    m,
    x,
    y,
    z,
    length=2,
    depth=4,
    seed=1,
    stone=("Light Bluish Gray", "Dark Bluish Gray"),
    lava="Orange",
    glow="Trans Orange",
    title="Lava",
):
    """The lava, then the rim around it; returns the z of the rim's top."""
    if length % 2 or depth % 2 or length < 2 or depth < 2:
        raise ValueError(
            f"lava_inset takes an even length and depth of 2 or more, not {length}, {depth}"
        )
    rng = random.Random(seed)
    px, py = x + 2, y + 2
    for k in range(2):
        with m.step(f"{title}, pit"):
            m.fill(Plate, lava, at=(px, py, z + k), size=(length, depth))
    # 1 x 2 tiles along y, eight a step.
    glow_at = [(px + i, py + 2 * j) for j in range(depth // 2) for i in range(length)]
    for s in range(0, len(glow_at), 8):
        with m.step(f"{title}, glow"):
            for gx, gy in glow_at[s : s + 8]:
                m.add("3069b", glow, at=(gx, gy, z + 2))
    rim = [
        (i, j)
        for j in range(0, depth + 4, 2)
        for i in range(0, length + 4, 2)
        if not (2 <= i < length + 2 and 2 <= j < depth + 2)
    ]
    for s in range(0, len(rim), 3):
        with m.step(f"{title}, rim"):
            for i, j in rim[s : s + 3]:
                color = rng.choice(stone)
                m.add("3003", color, at=(x + i, y + j, z))
                m.add("87580", color, at=(x + i, y + j, z + 3))
    return z + 4


def example(m):
    """A 2 x 4 lava square in a stone floor."""
    with m.step("Base"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("lava_inset", x=0, y=0, z=1, length=2, depth=4)
