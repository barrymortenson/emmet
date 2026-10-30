# SPDX-License-Identifier: Apache-2.0
"""Reed stalks: tall lime stalks growing from a sand block in a square of water, the game's sugar-cane look.

m.use("reed_stalks", x, y, z, heights=(2, 3, 1), seed=1, water="Blue", surface="Trans Light Blue", sand="Tan",
stalk="Lime", title="Reeds") lays a 4 x 4 water square from (x, y), on the studs at height z: a 4 x 4 plate in the
water color, a 2 x 2 plate of sand in its middle, and around it a ring of tiles in the surface color, flush with the
sand. On three of the sand's four studs (the empty one by seed) stand stalks of 1 x 1 round bricks, as many as each
of `heights` (in turn, by seed), each topped with a 1 x 1 round plate. Set the square into a module's ground so its
tiles are flush with the grass. Returns the z of the water's surface.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3031", "3022", "2431", "3069b", "3062b", "6141"]


def build(
    m,
    x,
    y,
    z,
    heights=(2, 3, 1),
    seed=1,
    water="Blue",
    surface="Trans Light Blue",
    sand="Tan",
    stalk="Lime",
    title="Reeds",
):
    """The water square, then the stalks; returns the z of the surface."""
    if not 1 <= len(heights) <= 4 or min(heights) < 1:
        raise ValueError(
            f"reed_stalks takes 1 to 4 heights of 1 or more bricks, not {heights}"
        )
    rng = random.Random(seed)
    with m.step(f"{title}, water"):
        m.add("3031", water, at=(x, y, z))
        m.add("3022", sand, at=(x + 1, y + 1, z + 1))
        # The ring of tiles around the sand: 1 x 4 at the front and back, 1 x 2 at the sides.
        m.add("2431", surface, at=(x, y, z + 1), rot=1)
        m.add("2431", surface, at=(x, y + 3, z + 1), rot=1)
        m.add("3069b", surface, at=(x, y + 1, z + 1))
        m.add("3069b", surface, at=(x + 3, y + 1, z + 1))
    studs = [(x + 1 + k % 2, y + 1 + k // 2) for k in range(4)]
    rng.shuffle(studs)
    order = list(heights)
    rng.shuffle(order)
    for n, ((sx, sy), h) in enumerate(zip(studs, order)):
        with m.step(f"{title}, stalk {n + 1}"):
            m.stack("3062b", stalk, at=(sx, sy, z + 2), count=h)
            m.add("6141", stalk, at=(sx, sy, z + 2 + 3 * h))
    return z + 2


def example(m):
    """A reed square on a strip of grass."""
    with m.step("Ground"):
        m.add("3035", "Bright Green", at=(0, 0, 0), rot=1)
    m.use("reed_stalks", x=2, y=0, z=1, heights=(2, 3, 1))
