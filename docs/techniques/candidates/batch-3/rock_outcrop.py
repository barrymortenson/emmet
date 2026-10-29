# SPDX-License-Identifier: Apache-2.0
"""Rock outcrop: a lump of rock, a core of bricks leaned on all round by slopes of different steepness, turned to face
out every way, with double-convex corner slopes and cheese chips on top.

m.use("rock_outcrop", x, y, z, height=2, seed=1, colors=("Dark Bluish Gray", "Light Bluish Gray"), title="Rock")
builds a rock 6 x 6 studs over the rectangle from (x, y), on the studs at height z: a 2 x 2 core in the middle,
`height` bricks tall (1 to 3), and on each side two slopes facing out, each 45° (3 plates tall), 65° (6), or 75° (9)
at random but never taller than the core, so the outline rises and falls around it; a 2 x 2 double-convex slope rounds
each corner. Each slope sits on the ground's studs with its top stud against the core. Two 1 x 1 cheese slopes on the
core's top, turned different ways, break the top. seed picks the slopes, turns, and colors (one of `colors` a part).
Returns the z of the core's top.
"""

import random

from emmet_dsl import Brick
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["3003", "3040b", "60481a", "4460b", "3045", "54200"]

# A slope 2 x 1 and its height in plates.
SLOPES = (("3040b", 3), ("60481a", 6), ("4460b", 9))


def build(
    m,
    x,
    y,
    z,
    height=2,
    seed=1,
    colors=("Dark Bluish Gray", "Light Bluish Gray"),
    title="Rock",
):
    """The rock; returns the z of the core's top."""
    if height not in (1, 2, 3):
        raise ValueError(f"rock_outcrop takes a height of 1 to 3 bricks, not {height}")
    if not colors:
        raise ValueError("rock_outcrop needs at least one color")
    rng = random.Random(seed)
    top = z + 3 * height
    choices = [part for part, h in SLOPES if h <= 3 * height]

    def piece(part, at, rot=0):
        color = rng.choice(colors)
        return (1, lambda: m.add(part, color, at=at, rot=rot))

    with m.step(f"{title}, core"):
        m.stack(Brick(2, 2), colors[0], at=(x + 2, y + 2, z), count=height)
    # (at, the slope's turn to face out): front, right, back, left, two cells a side.
    cells = []
    for i in range(2):
        cells += [
            ((x + 2 + i, y, z), 3),
            ((x + 4, y + 2 + i, z), 0),
            ((x + 2 + i, y + 4, z), 1),
            ((x, y + 2 + i, z), 2),
        ]
    in_steps(
        m, f"{title}, sides", [piece(rng.choice(choices), at, rot) for at, rot in cells]
    )
    corners = [
        ((x, y, z), 2),
        ((x + 4, y, z), 3),
        ((x + 4, y + 4, z), 0),
        ((x, y + 4, z), 1),
    ]
    chips = rng.sample([(0, 0), (1, 0), (0, 1), (1, 1)], 2)
    in_steps(
        m,
        f"{title}, corners and top",
        [piece("3045", at, rot) for at, rot in corners]
        + [
            piece("54200", (x + 2 + dx, y + 2 + dy, top), rng.randrange(4))
            for dx, dy in chips
        ],
    )
    return top


def example(m):
    """An outcrop three bricks tall on a gray plate."""
    with m.step("Base"):
        m.add("3958", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("rock_outcrop", x=0, y=0, z=1, height=3)
