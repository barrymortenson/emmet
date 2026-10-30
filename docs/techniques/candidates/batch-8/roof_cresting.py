# SPDX-License-Identifier: Apache-2.0
"""Roof cresting: an ornamental rail along a roof's ridge or a flat roof's edge, spindled fences between two finials.

m.use("roof_cresting", x, y, z, length, color="Dark Bluish Gray", finial="Pearl Gold", title="Cresting") stands a
rail on the row of studs `length` long along x from (x, y), at height z: spindled fences 1 x 4 x 2 (30055) down the
middle, and at each end, and in the studs the fences leave over, a finial (a round plate 1 x 1 under a cone 1 x 1).
A ridge needs a row of studs: lay a row of 1-wide bricks between the two top courses of slopes (ridge_cap's row,
without its caps) and stand the cresting on it; a flat roof's edge has its studs already. (gable_roof's double slopes
have no studs on top.) length must be 2 or more. It brings its own steps and returns the z of the rail's top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["30055", "6141", "4589"]


def build(
    m, x, y, z, length, color="Dark Bluish Gray", finial="Pearl Gold", title="Cresting"
):
    """The fences and the finials; returns the z of the rail's top."""
    if length < 2:
        raise ValueError(f"roof_cresting needs a length of 2 or more, not {length}")
    fences = (length - 2) // 4
    left = (
        length - 4 * fences
    ) // 2  # the studs before the first fence, finials on each
    ends = [x + k for k in range(left)] + [
        x + left + 4 * fences + k for k in range(length - left - 4 * fences)
    ]
    with m.step(f"{title}: finials"):
        for fx in ends:
            m.add("6141", finial, at=(fx, y, z))
            m.add("4589", finial, at=(fx, y, z + 1))
    with m.step(f"{title}: fences"):
        for k in range(fences):
            m.add("30055", color, at=(x + left + 4 * k, y, z), rot=1)
    return z + 6 if fences else z + 4


def example(m):
    """A short roof: 2 x 2 slopes facing front and back, a row of bricks along the ridge between them, and the
    cresting on it."""
    with m.step("Deck"):
        m.fill(Plate, "Light Bluish Gray", at=(0, 0, 0), size=(10, 5))
    with m.step("Slopes, front"):
        for k in range(0, 10, 2):
            m.add("3039", "Sand Green", at=(k, 0, 1), rot=3)
    with m.step("Slopes, back"):
        for k in range(0, 10, 2):
            m.add("3039", "Sand Green", at=(k, 3, 1), rot=1)
    with m.step("Ridge row"):
        m.row(Brick, "Sand Green", at=(0, 2, 1), length=10)
    m.use("roof_cresting", x=0, y=2, z=4, length=10)
