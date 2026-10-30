# SPDX-License-Identifier: Apache-2.0
"""Speckled block wall: a wall whose texture is two or three shades of one material mixed at random, the way a game
draws cobblestone, moss, or planks.

m.use("speckled_block_wall", x, y, z, length, rows=4, width=1, colors=("Light Bluish Gray", "Dark Bluish Gray"),
accent=0.33, seed=1, title="Wall") lays `rows` courses along x from (x, y, z), facing front (-y), `width` studs
thick: 1 (1 x 2, 1 x 3, and 1 x 4 bricks) or 2 (2 x 2, 2 x 3, and 2 x 4 bricks, a wall of whole blocks). Lengths are
picked by seed, and each course's joints miss the ones below, so no seams line up and no rows show. Each brick takes
the first color, or about `accent` of the time one of the others: one in three dark for stone; add Green for moss;
("Tan", "Medium Nougat", "Reddish Brown") for wood. Keep length on the 2-stud grid. Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3005", "3004", "3622", "3010", "3003", "3002", "3001"]

BRICKS = {
    1: {1: "3005", 2: "3004", 3: "3622", 4: "3010"},
    2: {1: "3004", 2: "3003", 3: "3002", 4: "3001"},
}


def _course(rng, length, below):
    """The (start, size) of one course's pieces: 2 to 4 long by seed, no joint over a joint below where it can."""
    out, at = [], 0
    while at < length:
        sizes = [n for n in (2, 3, 4) if at + n <= length] or [length - at]
        good = [n for n in sizes if at + n == length or at + n not in below] or sizes
        n = rng.choice(good)
        out.append((at, n))
        at += n
    return out


def build(
    m,
    x,
    y,
    z,
    length,
    rows=4,
    width=1,
    colors=("Light Bluish Gray", "Dark Bluish Gray"),
    accent=0.33,
    seed=1,
    title="Wall",
):
    """The courses; returns the z of the top."""
    if length < 2 or rows < 1 or width not in (1, 2):
        raise ValueError(
            "speckled_block_wall needs a length of 2 or more, 1 row or more, and a width of 1 or 2, "
            f"not {length}, {rows}, {width}"
        )
    if not colors:
        raise ValueError("speckled_block_wall needs at least one color")
    rng = random.Random(seed)
    below = set()
    for r in range(rows):
        zr = z + 3 * r
        pieces = _course(rng, length, below)
        for s in range(0, len(pieces), 6):
            with m.step(f"{title}, row {r + 1}"):
                for c, n in pieces[s : s + 6]:
                    color = colors[0]
                    if len(colors) > 1 and rng.random() < accent:
                        color = rng.choice(colors[1:])
                    # A brick's long side runs along y at rot 0; turn it to run along x (a 2 x 1 is a 1 x 2 along y).
                    long_x = not (width == 2 and n == 1)
                    m.add(
                        BRICKS[width][n],
                        color,
                        at=(x + c, y, zr),
                        rot=1 if long_x else 0,
                    )
        below = {c + n for c, n in pieces}
    return z + 3 * rows


def example(m):
    """A cobblestone wall 10 studs long and four bricks tall, and a mossy block wall beside it."""
    from emmet_dsl import Plate

    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=10, width=2)
        m.row(Plate, "Dark Bluish Gray", at=(12, 0, 0), length=6, width=2)
    m.use("speckled_block_wall", x=0, y=0, z=1, length=10, rows=4)
    m.use(
        "speckled_block_wall",
        x=12,
        y=0,
        z=1,
        length=6,
        rows=3,
        width=2,
        colors=("Light Bluish Gray", "Dark Bluish Gray", "Green"),
        accent=0.45,
        seed=3,
        title="Mossy wall",
    )
