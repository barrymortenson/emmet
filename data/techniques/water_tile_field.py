# SPDX-License-Identifier: Apache-2.0
"""Water tile field: a harbor or river at micro scale, pale tiles in long runs with their seams offset row to row, a few
dark patches flush in the water as shadows, and a dark rim so the water sits a plate down.

m.use("water_tile_field", x, y, z, length=10, depth=6, seed=1, shadows=1, studs=(), rim=True, water="Light Aqua",
shadow="Dark Bluish Gray", edge="Dark Bluish Gray", title="Water") covers, on the studs at height z, a length x depth
field from (x, y) with 1-wide rows of tiles along x (1 x 8, 1 x 4, 1 x 2, 1 x 1), each row starting with a piece of a
random length so the seams never line up. `shadows` 2 x 4 tiles in the shadow color lie flush among them, where the
shade of a bridge or a building over the water would fall, and the cells in `studs`, (x, y) pairs, stay bare for a boat
to stand on. With rim, a 1-stud ring of plates under tiles in the edge color frames the field, a plate higher than the
water. seed picks the seams and the shadows. Returns the z of the water's top.
"""

import random

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21067 and set 21066. Not
derived from any model file."""

PARTS = [
    "4162",
    "2431",
    "3069b",
    "3070b",
    "87079",
    "3666",
    "3460",
    "3710",
    "3623",
    "3023b",
    "3024",
    "6636",
    "63864",
]

SIZES = (8, 4, 2, 1)  # tile lengths made in both pale aqua and transparent light blue


def build(
    m,
    x,
    y,
    z,
    length=10,
    depth=6,
    seed=1,
    shadows=1,
    studs=(),
    rim=True,
    water="Light Aqua",
    shadow="Dark Bluish Gray",
    edge="Dark Bluish Gray",
    title="Water",
):
    """The rim, the shadows, and the water rows; returns the z of the water's top."""
    if length < 2 or depth < 1 or shadows < 0:
        raise ValueError(
            f"water_tile_field takes a length of 2 or more and a depth of 1 or more, not {length}, {depth}"
        )
    rng = random.Random(seed)
    used = {(round(a) - x, round(b) - y) for a, b in studs}
    patches = []
    for _ in range(shadows):
        spots = [
            (i, j)
            for j in range(depth - 1)
            for i in range(length - 3)
            if not {(i + a, j + b) for a in range(-1, 5) for b in range(-1, 3)} & used
        ]
        if not spots:
            break
        i, j = rng.choice(spots)
        patches.append((i, j))
        used |= {(i + a, j + b) for a in range(4) for b in range(2)}
    if rim:
        ring = [
            ("x", (x - 1, y - 1), length + 2),
            ("x", (x - 1, y + depth), length + 2),
            ("y", (x - 1, y), depth),
            ("y", (x + length, y), depth),
        ]
        with m.step(f"{title}: rim"):
            for axis, (rx, ry), n in ring:
                m.row(Plate, edge, at=(rx, ry, z), length=n, axis=axis)
    if patches:
        with m.step(f"{title}: shadows"):
            for i, j in patches:
                m.add("87079", shadow, at=(x + i, y + j, z), rot=1)
    for j0 in range(0, depth, 2):
        with m.step(f"{title}: rows {j0 + 1} to {min(j0 + 2, depth)}"):
            for j in range(j0, min(j0 + 2, depth)):
                first = rng.choice((1, 2, 4))
                start = 0
                for i in [i for i in range(length) if (i, j) in used] + [length]:
                    if i > start:
                        n = i - start
                        m.row(
                            Tile,
                            water,
                            at=(x + start, y + j, z),
                            length=n,
                            sizes=SIZES,
                            first=first if first and n > first else None,
                        )
                        first = None
                    start = i + 1
    if rim:
        with m.step(f"{title}: rim tiles"):
            for axis, (rx, ry), n in ring:
                m.row(Tile, edge, at=(rx, ry, z + 1), length=n, axis=axis, sizes=SIZES)
    return z + 1


def example(m):
    """A harbor basin 10 x 6 with a shadow and two bare studs for a boat."""
    with m.step("Base"):
        m.add(Plate(8, 8), "Dark Bluish Gray", at=(0, 0, 0))
        m.add(Plate(4, 8), "Dark Bluish Gray", at=(8, 0, 0))
    m.use(
        "water_tile_field",
        x=1,
        y=1,
        z=1,
        length=10,
        depth=6,
        seed=2,
        studs=[(8, 5), (9, 5)],
    )
