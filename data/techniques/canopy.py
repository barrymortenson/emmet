# SPDX-License-Identifier: Apache-2.0
"""Canopy: a roof on columns over a platform or a walkway, flat or gabled.

m.use("canopy", x, y, z, length, depth, spacing=8, style="flat", columns="Dark Green", roof="Dark Bluish Gray",
title="Canopy") covers the length × depth rectangle whose front-left corner is (x, y), standing on studs at height z:
a column (Brick 1 x 1 x 5) every `spacing` studs along its front and back rows and one at each end, a beam of plates
along each row on top of them, a deck of 2-wide plates across from beam to beam, and then a roof: "flat" (tiles over
the deck) or "gable" (a gable_roof). length and depth must be even, depth 4 to 16. Leave the column spots free of
tiles. It brings its own steps, a stage at a time, and returns the z of the roof's top.
"""

from emmet_dsl import Plate
from emmet_dsl.core import in_steps, row_actions

PARTS = [
    "2453b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3020",
    "3795",
    "3034",
]
PARTS += ["87079", "3068b"]

COLUMN = 15  # a Brick 1 x 1 x 5, in plates


def columns_at(x, length, spacing):
    """The x of each column along a row: every `spacing` studs and one at the far end."""
    xs = list(range(x, x + length - 1, spacing))
    return xs + [x + length - 1]


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    spacing=8,
    style="flat",
    columns="Dark Green",
    roof="Dark Bluish Gray",
    title="Canopy",
):
    if length % 2 or depth % 2 or not 4 <= depth <= 16:
        raise ValueError(
            f"canopy needs an even length and an even depth of 4 to 16, not {length} × {depth}"
        )
    if style not in ("flat", "gable"):
        raise ValueError(f'style is "flat" or "gable", not {style!r}')
    rows = (y, y + depth - 1)
    spots = [(cx, cy) for cy in rows for cx in columns_at(x, length, spacing)]
    in_steps(
        m,
        f"{title}, columns",
        [(1, lambda c=c: m.add("2453b", columns, at=(*c, z))) for c in spots],
    )
    top = z + COLUMN
    beams = [
        a for cy in rows for a in row_actions(m, Plate, roof, (x, cy, top), length)
    ]
    in_steps(m, f"{title}, beams", beams)
    deck = [
        lambda cx=cx: m.add(Plate(2, depth), roof, at=(cx, y, top + 1))
        for cx in range(x, x + length, 2)
    ]
    in_steps(m, f"{title}, deck", [(1, add) for add in deck])
    if style == "gable":
        return m.use(
            "gable_roof",
            x=x,
            y=y,
            z=top + 2,
            length=length,
            depth=depth,
            color=roof,
            title=f"{title} roof",
        )
    tiles = []
    for cx in range(x, x + length, 2):
        ty = y
        for n in [4] * (depth // 4) + [2] * (depth % 4 // 2):
            tiles.append(
                (
                    1,
                    lambda cx=cx, ty=ty, n=n: m.add(
                        "87079" if n == 4 else "3068b", roof, at=(cx, ty, top + 2)
                    ),
                )
            )
            ty += n
    in_steps(m, f"{title}, roof", tiles)
    return top + 3


def example(m):
    """A flat canopy, 8 × 4, on a plate."""
    with m.step("Plate"):
        m.add("3035", "Light Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("canopy", x=0, y=0, z=1, length=8, depth=4, spacing=7)
