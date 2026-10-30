# SPDX-License-Identifier: Apache-2.0
"""Bookshelf block: a 2 x 2 game block of wood whose sides show rows of colored book spines.

m.use("bookshelf_block", x, y, z, count=1, axis="x", seed=1, wood="Tan", books=("Blue", "Red", "Green", "Lime",
"White", "Black"), title="Bookshelf") sets `count` blocks side by side along `axis` from (x, y), on the studs at
height z. Each is 2 x 2 studs and 5 plates, a cube on the block grid: a 2 x 2 plate in the wood color, four books
standing on it (each a 1 x 1 brick in one of `books`, or a stack of three 1 x 1 plates in three of them), and a 2 x 2
jumper in the wood color on top, so it ends in one stud like any block. Colors and which books are stacks are by
seed, so every side shows two or three spines at different heights. Three in a row fill a recess in a 1-stud wall.
Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3022", "3005", "3024", "87580"]


def build(
    m,
    x,
    y,
    z,
    count=1,
    axis="x",
    seed=1,
    wood="Tan",
    books=("Blue", "Red", "Green", "Lime", "White", "Black"),
    title="Bookshelf",
):
    """The blocks; returns the z of the top."""
    if count < 1 or axis not in ("x", "y"):
        raise ValueError(
            f"bookshelf_block needs a count of 1 or more and an axis of 'x' or 'y', not {count}, {axis!r}"
        )
    if len(books) < 3:
        raise ValueError("bookshelf_block needs at least three book colors")
    rng = random.Random(seed)
    for b in range(count):
        bx, by = (x + 2 * b, y) if axis == "x" else (x, y + 2 * b)
        cells = [(bx + k % 2, by + k // 2) for k in range(4)]
        for half in (0, 1):
            with m.step(f"{title}, block {b + 1}" if count > 1 else title):
                if half == 0:
                    m.add("3022", wood, at=(bx, by, z))
                for cx, cy in cells[2 * half : 2 * half + 2]:
                    if rng.random() < 0.5:
                        m.add("3005", rng.choice(books), at=(cx, cy, z + 1))
                    else:
                        for p, color in enumerate(rng.sample(list(books), 3)):
                            m.add("3024", color, at=(cx, cy, z + 1 + p))
                if half == 1:
                    m.add("87580", wood, at=(bx, by, z + 4))
    return z + 5


def example(m):
    """A shelf of three blocks between two posts, under a lintel."""
    with m.step("Floor"):
        m.add("3034", "Reddish Brown", at=(0, 0, 0), rot=1)
    with m.step("Wall, posts"):
        for zr in (1, 4):
            m.add("3005", "Reddish Brown", at=(0, 1, zr))
            m.add("3005", "Reddish Brown", at=(7, 1, zr))
    m.use("bookshelf_block", x=1, y=0, z=1, count=3, seed=4)
    with m.step("Wall, lintel"):
        m.add("3008", "Tan", at=(0, 1, 7), rot=1)
