# SPDX-License-Identifier: Apache-2.0
"""White ashlar wall: a smooth stone wall of long, uneven courses with no repeating pattern, a few grooved and grille
blocks breaking up its face, and a tiled top with the odd stud left open.

m.use("white_ashlar_wall", x, y, z, length, rows=4, color="White", seed=1, cap=True, title="Ashlar wall") lays `rows`
courses along x from (x, y, z), facing front (-y), each of bricks 1 to 8 studs long picked by seed, every joint
missing the joints of the course below. In every fourth course (the third, the seventh, ...) one or two 1 x 2 or 1 x 4
pieces become a brick with a groove (4216, 2653) or a 1 x 2 grille brick (2877), small dark slots in the stone. With
cap, a course of tiles covers the top except two or three studs. Returns the z of the top.
"""

import random

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = [
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "4216",
    "2653",
    "2877",
    "3070b",
    "3069b",
    "63864",
    "2431",
]

# Brick 1 x n.
BRICKS = {1: "3005", 2: "3004", 3: "3622", 4: "3010", 6: "3009", 8: "3008"}
SLOTS = {2: ("4216", "2877"), 4: ("2653",)}  # the recessed pieces, by length
TILES = {1: "3070b", 2: "3069b", 3: "63864", 4: "2431"}


def _course(rng, length, below):
    """Piece lengths for one course, picked at random, no joint on a joint below where something else fits."""
    out, at = [], 0
    while at < length:
        fits = [n for n in BRICKS if at + n <= length]
        good = [n for n in fits if at + n == length or at + n not in below] or fits
        n = rng.choice(good)
        out.append(n)
        at += n
    return out


def build(
    m, x, y, z, length, rows=4, color="White", seed=1, cap=True, title="Ashlar wall"
):
    """The courses and the cap; returns the z of the top."""
    if length < 4 or rows < 1:
        raise ValueError(
            f"white_ashlar_wall needs a length of 4 or more and 1 row or more, not {length}, {rows}"
        )
    rng = random.Random(seed)
    below = set()
    for r in range(rows):
        sizes = _course(rng, length, below)
        slots = set()
        if r % 4 == 2:
            for _ in range(10):
                if any(n in SLOTS for n in sizes):
                    break
                sizes = _course(rng, length, below)
            fit = [k for k, n in enumerate(sizes) if n in SLOTS]
            slots = set(rng.sample(fit, min(len(fit), rng.choice((1, 2)))))
        with m.step(f"{title}, row {r + 1}"):
            at = x
            for k, n in enumerate(sizes):
                if k in slots:
                    # Turned so the grille or groove faces front.
                    m.add(rng.choice(SLOTS[n]), color, at=(at, y, z + 3 * r), rot=3)
                else:
                    m.add(BRICKS[n], color, at=(at, y, z + 3 * r), rot=1)
                at += n
        below = {sum(sizes[: k + 1]) for k in range(len(sizes))}
    top = z + 3 * rows
    if not cap:
        return top
    with m.step(f"{title}: cap"):
        open_ = set(rng.sample(range(length), min(length // 4, 3)))
        at = 0
        while at < length:
            if at in open_:
                at += 1
                continue
            n = 1
            while n < 4 and at + n < length and at + n not in open_:
                n += 1
            m.add(TILES[n], color, at=(x + at, y, top), rot=1)
            at += n
    return top + 1


def example(m):
    """A white stone wall 12 studs long and four courses tall, on a plate."""
    with m.step("Footing"):
        m.row(Plate, "Light Bluish Gray", at=(0, 0, 0), length=12)
    m.use("white_ashlar_wall", x=0, y=0, z=1, length=12)
