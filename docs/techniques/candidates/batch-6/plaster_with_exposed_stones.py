# SPDX-License-Identifier: Apache-2.0
"""Plaster with exposed stones: a whitewashed wall with a few stones showing where the plaster has fallen away, each a
tile on a side stud standing a plate proud.

m.use("plaster_with_exposed_stones", x, y, z, length, rows=4, color="White", stone="Tan", stones=5, seed=1,
title="Plaster wall") lays `rows` courses of bricks along x from (x, y, z), facing front (-y), and swaps `stones` of
them (picked by seed) for a brick with studs on one side (87087 for a 1 x 1 stone, 11211 for a 1 x 2) carrying a tile
in the stone color. Stones fall more often low down and near the ends, never two side by side or one straight over
another. Returns the z of the top.
"""

import random

from emmet_dsl import Brick, Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335. Not derived from any
model file."""

PARTS = [
    "87087",
    "11211",
    "3070b",
    "3069b",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
]

FACE = Rot(y=-90, z=90)  # a tile on its back, its face to -y, long along x
# Stone width: the brick and its tile.
SNOT = {1: ("87087", "3070b"), 2: ("11211", "3069b")}
OUT = 0.4  # a tile's thickness, in studs


def _pick(rng, length, rows, count):
    """(row, start, width) for each stone: low rows and the ends weigh more; none touch side by side or above."""
    cells = [
        (r, c, w) for r in range(rows) for w in (1, 2) for c in range(length - w + 1)
    ]
    weight = [
        1
        + 2 * (rows - 1 - r) / max(1, rows - 1)
        + (2 if min(c, length - c - w) < 2 else 0)
        for r, c, w in cells
    ]
    out = []
    for _ in range(count * 20):
        if len(out) == count:
            break
        r, c, w = rng.choices(cells, weight)[0]
        if all(
            not (
                abs(r - r2) <= 1 and c < c2 + w2 + (r == r2) and c2 < c + w + (r == r2)
            )
            for r2, c2, w2 in out
        ):
            out.append((r, c, w))
    return out


def build(
    m,
    x,
    y,
    z,
    length,
    rows=4,
    color="White",
    stone="Tan",
    stones=5,
    seed=1,
    title="Plaster wall",
):
    """The courses with their stones; returns the z of the top."""
    if length < 3 or rows < 1:
        raise ValueError(
            f"plaster_with_exposed_stones needs a length of 3 or more and 1 row or more, not {length}, {rows}"
        )
    picked = _pick(random.Random(seed), length, rows, stones)
    for r in range(rows):
        zr = z + 3 * r
        here = sorted((c, w) for r2, c, w in picked if r2 == r)
        with m.step(f"{title}, row {r + 1}"):
            start = 0
            for c, w in here + [(length, 0)]:
                if c > start:
                    # Every other course starts a stud short, so its joints miss the course below.
                    first = 1 if r % 2 and start == 0 and c - start > 1 else None
                    m.row(
                        Brick,
                        color,
                        at=(x + start, y, zr),
                        length=c - start,
                        first=first,
                    )
                if w:
                    m.add(SNOT[w][0], color, at=(x + c, y, zr), rot=3)
                start = c + w
        if here:
            with m.step(f"{title}, row {r + 1}: stones"):
                for c, w in here:
                    m.add(SNOT[w][1], stone, at=(x + c, y - OUT, zr + 0.5), rot=FACE)
    return z + 3 * rows


def example(m):
    """A plastered wall 10 studs long and four bricks tall, on a plate."""
    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=10)
    m.use("plaster_with_exposed_stones", x=0, y=0, z=1, length=10, rows=4, stones=6)
