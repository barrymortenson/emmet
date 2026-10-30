# SPDX-License-Identifier: Apache-2.0
"""Floor rug: a flat rug of tiles on a room's floor studs, round or rectangular with a border.

m.use("floor_rug", x, y, z, shape="round", length=4, depth=2, color="Dark Red", border="Tan", title="Rug") lays a rug on
the studs at height z (a floor of plates, a baseplate) with its min corner at (x, y). shape="round" is a round 3 x 3
tile (it sits on nine studs, the middle one at (x + 1.5, y + 1.5)). shape="rect" covers `length` studs along x and
`depth` along y: tiles in `color` inside a 1-stud border of 1-wide tiles in `border` (border=None for a plain rug; a rug
2 deep or less is all border), the border's long sides laid first so its corners belong to them. The rug is one plate
thick, so furniture stands beside it, not on it. It brings its own steps, "<title>, border" and "<title>, middle" (split
into parts of up to 8), and returns the z of the rug's top, z + 1.
"""

from emmet_dsl import Tile
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 43268. Not derived
from any model file."""

PARTS = ["67095", "3069b", "2431", "3068b", "87079"]


def build(
    m,
    x,
    y,
    z,
    shape="round",
    length=4,
    depth=2,
    color="Dark Red",
    border="Tan",
    title="Rug",
):
    """The rug; returns the z of its top."""
    if shape == "round":
        with m.step(title):
            m.add("67095", color, at=(x, y, z))
        return z + 1
    if shape != "rect":
        raise ValueError(f'shape must be "round" or "rect", not {shape!r}')
    if border is None:
        with m.step(title):
            m.fill(Tile, color, at=(x, y, z), size=(length, depth))
        return z + 1
    actions = row_actions(m, Tile, border, (x, y + depth - 1, z), length)
    if depth > 1:
        actions += row_actions(m, Tile, border, (x, y, z), length)
    if depth > 2:
        actions += row_actions(m, Tile, border, (x, y + 1, z), depth - 2, axis="y")
        actions += row_actions(
            m, Tile, border, (x + length - 1, y + 1, z), depth - 2, axis="y"
        )
    in_steps(m, f"{title}, border", actions)
    if length > 2 and depth > 2:
        with m.step(f"{title}, middle"):
            m.fill(Tile, color, at=(x + 1, y + 1, z), size=(length - 2, depth - 2))
    return z + 1


def example(m):
    """A round rug and a bordered one on a plank-colored floor."""
    with m.step("Floor"):
        m.add("3036", "Reddish Brown", at=(0, 0, 0), rot=1)
    m.use("floor_rug", x=0, y=1, z=1)
    m.use(
        "floor_rug",
        x=4,
        y=0,
        z=1,
        shape="rect",
        length=4,
        depth=6,
        color="Dark Blue",
        border="Tan",
    )
