# SPDX-License-Identifier: Apache-2.0
"""Canopy: a roof on columns over a platform or a walkway, flat or gabled.

m.use("canopy", x, y, z, length, depth, spacing=8, height=6, style="flat", columns="Dark Green",
roof="Dark Bluish Gray", title="Canopy") covers the length × depth rectangle whose front-left corner is (x, y), standing
on studs at height z: a column `height` bricks tall (a Brick 1 x 1 x 5 with 1 x 1 bricks on it) every `spacing` studs
along its front and back rows and one at each end (the last bay a little longer, never too narrow to walk through), a beam of plates along each row on top of them, a deck of 2-wide
plates across from beam to beam, and then a roof: "flat" (tiles over the deck) or "gable" (a gable_roof). height is
the clear room under the beams: 6 (the default) clears a 1 x 4 x 6 door beside the canopy, and less than 5 can't be
walked under, so it's 5 to 12. spacing is 4 or more, so a figure fits between columns (docs/spec/building-rules.md).
length and depth must be even, depth 4 to 16. Leave the column spots free of tiles. It brings its own steps, a stage at
a time, and returns the z of the roof's top, with .columns (the (x, y) of every column, so benches can go between them)
and .underside (the z of the beams' bottom).
"""

from emmet_dsl import Plate
from emmet_dsl.core import fit, in_steps, row_actions

PROVENANCE = """Emmet's own (session 3B), written from general building knowledge; not derived from any model
file."""

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
    "87079",
    "3068b",
    "14716",
    "3005",
]


def column(height):
    """The parts of a column `height` bricks tall, bottom up: Bricks 1 x 1 x 5, then 1 x 1 x 3, then 1 x 1."""
    out = []
    for part, n in (("2453b", 5), ("14716", 3), ("3005", 1)):
        while height >= n:
            out.append((part, n))
            height -= n
    return out


def columns_at(x, length, spacing):
    """The x of each column along a row: every `spacing` studs and one at the far end, without a last bay too narrow
    for a figure (under 3 studs clear)."""
    xs = list(range(x, x + length - 1, spacing))
    if len(xs) > 1 and x + length - 1 - xs[-1] < 4:
        xs.pop()
    return xs + [x + length - 1]


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    spacing=8,
    height=6,
    style="flat",
    columns="Dark Green",
    roof="Dark Bluish Gray",
    title="Canopy",
):
    if length % 2 or depth % 2 or not 4 <= depth <= 16:
        raise ValueError(
            f"canopy needs an even length and an even depth of 4 to 16, not {length} × {depth}"
        )
    if not 5 <= height <= 12:
        raise ValueError(
            f"canopy height is 5 to 12 bricks (6 clears a door beside it), not {height}"
        )
    if spacing < 4:
        raise ValueError(
            f"canopy spacing is 4 studs or more, so a figure fits between columns, not {spacing}"
        )
    if style not in ("flat", "gable"):
        raise ValueError(f'style is "flat" or "gable", not {style!r}')
    rows = (y, y + depth - 1)
    spots = [(cx, cy) for cy in rows for cx in columns_at(x, length, spacing)]
    stack = column(height)

    def raise_column(c):
        h = z
        for part, n in stack:
            m.add(part, columns, at=(*c, h))
            h += 3 * n

    in_steps(
        m,
        f"{title}, columns",
        [(len(stack), lambda c=c: raise_column(c)) for c in spots],
    )
    top = z + 3 * height
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
        roof_top = m.use(
            "gable_roof",
            x=x,
            y=y,
            z=top + 2,
            length=length,
            depth=depth,
            color=roof,
            title=f"{title} roof",
        )
        return fit(roof_top, columns=spots, underside=top)
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
    return fit(top + 3, columns=spots, underside=top)


def example(m):
    """A flat canopy, 8 × 4, on a plate."""
    with m.step("Plate"):
        m.add("3035", "Light Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("canopy", x=0, y=0, z=1, length=8, depth=4, spacing=7)
