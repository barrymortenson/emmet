# SPDX-License-Identifier: Apache-2.0
"""Micro tower: a slender skyscraper of tall 1 x 1 bricks standing side by side on a plate and capped by another, the
seams between them reading as the tower's piers.

m.use("micro_tower", x, y, z, width=3, depth=1, height=30, color="Light Bluish Gray", title="Tower") stands, on the
studs at height z, a width x depth plate (1 to 3 studs each way, along x and y) and on each of its studs a column of 1 x
1 x 5 and 1 x 1 x 3 bricks (2453b, 14716), topped up with plain 1 x 1 bricks, `height` plates tall (a multiple of 3, 9
or more); the columns start with a different brick in turn, so their joints never line up across the tower. A plate of
the same size caps them and ties the columns together. At micro scale a plate is about a story, so 30 plates reads as a
30-story tower. Returns the z of the cap's top, where a crown goes.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = ["2453b", "14716", "3005", "3024", "3023b", "3623", "3022", "3021"]

TALL = {15: "2453b", 9: "14716", 3: "3005"}  # 1 x 1 bricks by height in plates


def _column(height, start):
    """The heights of a column's bricks, the first one picked by `start` so neighbors' joints differ."""
    first = (15, 9, 3)[start % 3]
    if first > height:
        first = 9 if height >= 9 else 3
    out, left = [first], height - first
    for h in (15, 9, 3):
        while left >= h:
            out.append(h)
            left -= h
    return out


def _plate(m, color, x, y, z, width, depth):
    if width == 1 and depth == 1:
        return m.add(Plate(1, 1), color, at=(x, y, z))
    if width >= depth:
        return m.add(Plate(depth, width), color, at=(x, y, z), rot=1)
    return m.add(Plate(width, depth), color, at=(x, y, z))


def build(
    m, x, y, z, width=3, depth=1, height=30, color="Light Bluish Gray", title="Tower"
):
    """The plate, the columns, and the cap; returns the z of the top."""
    if not (1 <= width <= 3 and 1 <= depth <= 3) or height < 9 or height % 3:
        raise ValueError(
            "micro_tower takes a width and depth of 1 to 3 and a height of 9 or more plates in whole bricks, "
            f"not {width}, {depth}, {height}"
        )
    with m.step(f"{title}: base"):
        _plate(m, color, x, y, z, width, depth)
    columns = []
    for j in range(depth):
        for i in range(width):
            columns.append((x + i, y + j, _column(height, i + 2 * j)))
    # A course at a time: every column's next brick, bottom up, so a step never builds one column past the others.
    level = 0
    while any(columns_left for _, _, columns_left in columns):
        with m.step(f"{title}: shaft, {level + 1}"):
            for cx, cy, left in columns:
                if left:
                    h = left.pop(0)
                    zc = z + 1 + height - sum(left) - h
                    m.add(TALL[h], color, at=(cx, cy, zc))
        level += 1
    with m.step(f"{title}: cap"):
        _plate(m, color, x, y, z + 1 + height, width, depth)
    return z + 2 + height


def example(m):
    """Three towers of staggered heights on a dark plate, the middle one the tallest."""
    with m.step("Ground"):
        m.add(Plate(4, 8), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use(
        "micro_tower",
        x=0,
        y=1,
        z=1,
        width=2,
        depth=2,
        height=24,
        color="Dark Bluish Gray",
        title="Left tower",
    )
    m.use(
        "micro_tower", x=3, y=1, z=1, width=3, depth=1, height=36, title="Middle tower"
    )
    m.use(
        "micro_tower",
        x=6,
        y=1,
        z=1,
        width=1,
        depth=2,
        height=18,
        color="White",
        title="Right tower",
    )
    with m.step("Crowns"):
        m.add("37352", "Light Bluish Gray", at=(3, 1, 39), rot=1)
        m.add("49307", "White", at=(6, 2, 21))
        m.add(Brick(1, 1), "White", at=(6, 1, 21))
        m.add("49307", "White", at=(6, 1, 24))
