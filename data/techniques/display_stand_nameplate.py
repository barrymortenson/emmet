# SPDX-License-Identifier: Apache-2.0
"""Display stand with a nameplate: a tiled base with a plaque on its front edge and a post that holds a display
motorbike up by its swingarm pivot, its wheels just clear of the base.

m.use("display_stand_nameplate", x, y, z=0, length=42, width=12, back=17, lift=26, under=None, color="Black",
top_color="Dark Bluish Gray", name_color="Light Bluish Gray", post_color="Black", title="Stand") builds a base of two
layers of plates, `length` studs along x from x - back and `width` across y centered on y, from z, bonded (the second
layer's joints over the first's middles), tiled on top. A nameplate, two tiles 1 x 6 (6636) in name_color, sits in the
middle of the front edge (-y). A post of bricks 1 x 2 stands on the base at x, 2 studs back of y (beside a center
shock, in the plane a `motorbike_frame` leaves free at its pivot), `lift` plates of bricks and plates topped by a
Technic brick 1 x 2 with a hole (3700) whose hole runs along y. It brings its own steps and returns a dict of "pivot"
(x, y, z) and "post" (the post's hole): build the `motorbike_frame` with its pivot at "pivot" and on="post", and its
pivot axle goes through the post first. `under` sets `lift` for you: how far the wheels' bottoms are below the pivot,
in studs (the axles' drop plus the wheels' radius), and they stand clear of the tiles by under a plate. At the
defaults a frame with `swingarm_shock` and `suspension_fork` wheels has both wheels within a plate of the base, clear
of it.
"""

import math

from emmet_dsl import Brick, Plate, Tile
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of display stands. Not derived from
any model file."""

PARTS = ["3004", "3023b", "3700", "6636"]

NAME = 6  # the nameplate's length, in studs


def _tiles(m, color, x0, x1, y, z, skip):
    """Tile actions for the row y from x0 to x1, around the cells in `skip`."""
    actions, x = [], x0
    while x < x1:
        if (x, y) in skip:
            x += 1
            continue
        end = x
        while end < x1 and (end, y) not in skip:
            end += 1
        actions += row_actions(m, Tile, color, (x, y, z), end - x)
        x = end
    return actions


def build(
    m,
    x,
    y,
    z=0,
    length=42,
    width=12,
    back=17,
    lift=26,
    under=None,
    color="Black",
    top_color="Dark Bluish Gray",
    name_color="Light Bluish Gray",
    post_color="Black",
    title="Stand",
):
    """The base, the nameplate, and the post; returns where the post's hole is."""
    if under is not None:
        # The wheels' bottoms `under` studs below the pivot, at least on the tiles' top (2 plates up and a tile).
        lift = math.ceil(under * 2.5 - 0.75 - 1e-6)
    x0, y0 = x - back, y - width // 2
    first = []
    for j in range(0, width, 2):
        first += row_actions(
            m, Plate, color, (x0, y0 + j, z), length, width=min(2, width - j)
        )
    in_steps(m, f"{title}: base", first)
    second = row_actions(m, Plate, color, (x0, y0, z + 1), length)
    for j in range(1, width - 1, 2):
        second += row_actions(
            m, Plate, color, (x0, y0 + j, z + 1), length, width=min(2, width - 1 - j)
        )
    if width % 2 == 0:
        second += row_actions(m, Plate, color, (x0, y0 + width - 1, z + 1), length)
    in_steps(m, f"{title}: base, second layer", second)
    name_x = x0 + (length - NAME) // 2
    post = {(x - 1, y + 2), (x, y + 2)}
    plaque = {(name_x + i, y0 + r) for i in range(NAME) for r in (0, 1)}
    with m.step(f"{title}: nameplate"):
        for r in (0, 1):
            m.add("6636", name_color, at=(name_x, y0 + r, z + 2), rot=1)
    tiles = []
    for j in range(width):
        tiles += _tiles(m, top_color, x0, x0 + length, y0 + j, z + 2, post | plaque)
    in_steps(m, f"{title}: tiles", tiles)
    bricks, plates = divmod(lift, 3)
    column = [
        (
            1,
            lambda k=k: m.add(
                Brick(1, 2), post_color, at=(x - 1, y + 2, z + 2 + 3 * k), rot=1
            ),
        )
        for k in range(bricks)
    ]
    column += [
        (
            1,
            lambda k=k: m.add(
                Plate(1, 2),
                post_color,
                at=(x - 1, y + 2, z + 2 + 3 * bricks + k),
                rot=1,
            ),
        )
        for k in range(plates)
    ]
    in_steps(m, f"{title}: post", column)
    pivot = (x, y, z + 2 + lift + 1.75)
    with m.step(f"{title}: post top"):
        top = m.add("3700", post_color, at=(x, y + 2.5, pivot[2]), rot=1, hole=1)
    return {"pivot": pivot, "post": top.hole(1)}


def example(m):
    """The stand with a motorbike frame on its post."""
    stand = m.use("display_stand_nameplate", x=17, y=6)
    x, y, z = stand["pivot"]
    m.use("motorbike_frame", x=x, y=y, z=z, on=stand["post"])
