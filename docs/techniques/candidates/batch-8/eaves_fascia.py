# SPDX-License-Identifier: Apache-2.0
"""Eaves fascia: a finished eave edge for a roof that overhangs its walls, a strip along the eave in front of the
roof's first slopes: a contrasting tile strip, a dark fascia under the edge, a kicked row of cheese slopes, or a
rounded lip of curved slopes.

m.use("eaves_fascia", x, y, z, length, depth, style="fascia", sides="both", color="Dark Red", trim="Dark Brown",
title="Eaves") finishes the eaves of a deck (a ceiling or a lift-off frame) length x depth from (x, y) whose top is
at z and which overhangs the walls along its front and back edges (sides: "front", "back", or "both"). Along each
eave it lays a strip `length` long on the deck's edge rows, and the roof goes on behind it: style "tile" is a tile
strip in `trim` (1 deep); "fascia" is tiles in `color` with `trim` plates hung under the edge, a dark band below the
eave (1 deep; the deck has to overhang the walls a stud there); "cheese" is a row of 1 x 1 slopes 30 in `color`
facing out, which carries the roof's line down past its first slopes (1 deep); "curve" is curved slopes 2 x 1
(11477) in `trim` facing out, a rounded lip, as snow or a thatched edge (2 deep). It brings its own steps and returns
(y, depth) for the roof behind it: pass them to gable_roof (or ridge_cap's slopes) with the same x, length, and z.
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217, set 11387, set 21335.
Not derived from any model file."""

PARTS = [
    "2431",
    "3069b",
    "3070b",
    "63864",
    "6636",
    "4162",
    "3710",
    "3623",
    "3023b",
    "3024",
    "3666",
    "3460",
    "54200",
    "11477",
]

DOWN = {"+x": 0, "+y": 1, "-x": 2, "-y": 3}
REACH = {"tile": 1, "fascia": 1, "cheese": 1, "curve": 2}


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    style="fascia",
    sides="both",
    color="Dark Red",
    trim="Dark Brown",
    title="Eaves",
):
    """The eave strips; returns (y, depth) of the roof behind them."""
    if style not in REACH or sides not in ("front", "back", "both"):
        raise ValueError(
            'style is "tile", "fascia", "cheese" or "curve", and sides "front", "back" or "both"'
        )
    reach = REACH[style]
    edges = [("front", y, "-y"), ("back", y + depth - reach, "+y")]
    edges = [e for e in edges if sides in ("both", e[0])]
    for name, ey, out in edges:
        if style == "tile":
            actions = row_actions(m, Tile, trim, (x, ey, z), length)
        elif style == "fascia":
            actions = row_actions(m, Plate, trim, (x, ey, z - 2), length) + row_actions(
                m, Tile, color, (x, ey, z), length
            )
        elif style == "cheese":
            actions = [
                (
                    1,
                    lambda ex=ex, ey=ey, out=out: m.add(
                        "54200", color, at=(ex, ey, z), rot=DOWN[out]
                    ),
                )
                for ex in range(x, x + length)
            ]
        else:
            actions = [
                (
                    1,
                    lambda ex=ex, ey=ey, out=out: m.add(
                        "11477", trim, at=(ex, ey, z), rot=DOWN[out]
                    ),
                )
                for ex in range(x, x + length)
            ]
        in_steps(m, f"{title}: {name}", actions)
    front = reach if sides in ("front", "both") else 0
    back = reach if sides in ("back", "both") else 0
    return y + front, depth - front - back


def example(m):
    """Walls 8 x 4, a ceiling a stud wider at the front and back, a fascia along both eaves, and the roof."""
    with m.step("Base"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 1, 0), size=(8, 4))
    for r in range(2):
        with m.step(f"Walls, row {r + 1}"):
            m.wall(0, 1, 1 + 3 * r, length=8, depth=4, color="White")
    with m.step("Ceiling"):
        m.fill(Plate, "White", at=(0, 0, 7), size=(8, 6))
    ry, rd = m.use("eaves_fascia", x=0, y=0, z=8, length=8, depth=6)
    m.use("gable_roof", x=0, y=ry, z=8, length=8, depth=rd)
