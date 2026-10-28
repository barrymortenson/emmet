# SPDX-License-Identifier: Apache-2.0
"""Platform edge: the edge of a station platform beside the track, with a yellow safety line.

m.use("platform_edge", x, y, z, length, color="Light Bluish Gray", line="Yellow", title="Platform edge") finishes the
2-stud band along x from (x, y), at height z on top of the platform: a row of 1-wide tiles in the line color at y,
the edge nearest the track, and a row in the platform's color behind it at y + 1, the longest tiles first. Underneath
must be studs (a plate deck). It brings its own steps and returns the z of the tiles' top. A platform beside
plastic track stands 4 plates above the track's bottom or more (the rails are 3 plates up).
"""

from emmet_dsl import Brick, Tile
from emmet_dsl.core import in_steps, row_actions

PARTS = ["2431", "63864", "3069b", "3070b"]


def build(
    m, x, y, z, length, color="Light Bluish Gray", line="Yellow", title="Platform edge"
):
    sizes = (4, 3, 2, 1)
    in_steps(m, title, row_actions(m, Tile, line, (x, y, z), length, sizes=sizes))
    in_steps(m, title, row_actions(m, Tile, color, (x, y + 1, z), length, sizes=sizes))
    return z + 1


def example(m):
    """A short platform: a deck of plates on a course of bricks at the front and back, with its edge."""
    with m.step("Platform walls"):
        m.row(Brick, "Light Bluish Gray", at=(0, 0, 0), length=12)
        m.row(Brick, "Light Bluish Gray", at=(0, 3, 0), length=12)
    with m.step("Platform deck"):
        m.repeat(lambda i: m.add("3020", "Light Bluish Gray", at=(2 * i, 0, 3)), 6)
    m.use("platform_edge", x=0, y=0, z=4, length=12)
