# SPDX-License-Identifier: Apache-2.0
"""Flagstone paving: crazy paving of tiles in mixed sizes and two grays laid at random, with a few jumper plates and
bare studs as worn stones and places to stand things.

m.use("flagstone_paving", x, y, z, length, depth, seed=1, colors=("Light Bluish Gray", "Dark Bluish Gray"), bumps=0.08,
title="Paving") covers the rectangle `length` studs along x and `depth` along y from (x, y), on the studs at height z
(a baseplate or a layer of plates), with tiles 2 x 2, 1 x 4, 1 x 3, 1 x 2 (either way), and 1 x 1, each in one of
`colors`, packed front to back from randomly chosen sizes, so the joints wander and no long seam lines up. About
`bumps` of the stones are instead a jumper plate (2 x 2 or 1 x 2 with one center stud) or a bare stud, the surface's
small bumps. seed picks the sizes, colors, and bumps. Returns the z of the paving's top.
"""

import random

from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 21335 (crazy paving of tiles and jumpers). Not derived from any model file."""

PARTS = ["3068b", "2431", "63864", "3069b", "3070b", "87580", "3794b"]

# Each stone: (studs along x, studs along y, part, rot), the bigger ones first in the draw's weights.
STONES = [
    ((2, 2, "3068b", 0), 2),
    ((4, 1, "2431", 1), 1),
    ((1, 3, "63864", 0), 1),
    ((3, 1, "63864", 1), 1),
    ((2, 1, "3069b", 1), 3),
    ((1, 2, "3069b", 0), 3),
    ((1, 1, "3070b", 0), 2),
]
BUMPS = [(2, 2, "87580", 0), (2, 1, "3794b", 1), (1, 2, "3794b", 0), (1, 1, None, 0)]


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    seed=1,
    colors=("Light Bluish Gray", "Dark Bluish Gray"),
    bumps=0.08,
    title="Paving",
):
    """The paving; returns the z of its top."""
    if not 2 <= length <= 32 or not 2 <= depth <= 32:
        raise ValueError(
            f"flagstone_paving takes a length and depth of 2 to 32, not {length}, {depth}"
        )
    if not colors or not 0 <= bumps <= 0.5:
        raise ValueError("flagstone_paving needs a color, and bumps from 0 to 0.5")
    rng = random.Random(seed)
    used = set()

    def fits(i, j, w, d):
        return (
            i + w <= length
            and j + d <= depth
            and all((i + a, j + b) not in used for a in range(w) for b in range(d))
        )

    actions = []
    for j in range(depth):
        for i in range(length):
            if (i, j) in used:
                continue
            pool = BUMPS if rng.random() < bumps else None
            if pool:
                w, d, part, rot = rng.choice(
                    [s for s in pool if fits(i, j, s[0], s[1])]
                )
                color = colors[-1]
            else:
                shapes = [(s, wt) for s, wt in STONES if fits(i, j, s[0], s[1])]
                ((w, d, part, rot),) = rng.choices(
                    [s for s, _ in shapes], [wt for _, wt in shapes]
                )
                color = rng.choice(colors)
            used.update((i + a, j + b) for a in range(w) for b in range(d))
            if part:
                at = (x + i, y + j, z)
                actions.append(
                    (1, lambda p=part, c=color, at=at, r=rot: m.add(p, c, at=at, rot=r))
                )
    in_steps(m, title, actions)
    return z + 1


def example(m):
    """A patch of paving 8 x 6 studs on a plate."""
    with m.step("Base"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("flagstone_paving", x=0, y=0, z=1, length=8, depth=6, bumps=0.12)
