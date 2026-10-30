# SPDX-License-Identifier: Apache-2.0
"""Irregular stone wall: a cellar or cave wall of uneven stone blocks in a broken bond, with gray rock showing
through in steep slopes and a ragged top.

m.use("irregular_stone_wall", x, y, z, length, rows=5, color="Tan", tone="Dark Tan", rock="Light Bluish Gray",
rocks=3, seed=1, title="Stone wall") lays a wall 2 studs deep along x from (x, y, z), facing front (-y), `rows`
courses tall, of 2 x 2 and 1 x 2 bricks, mostly in the stone color and one in five in the tone. `rocks` times (placed
by seed) a 75-degree slope 2 x 1 x 3, upright or inverted (4460b, 2449), in the rock color stands in the wall three
courses tall, its face to the front, so the stone seems to come out of the rock. The top course stops and starts, so
the top is ragged. Returns the z of the top of the full courses.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21363. Not derived from any
model file."""

PARTS = ["3003", "3004", "4460b", "2449"]

ROCKS = ("4460b", "2449")  # Slope Brick 75 2 x 1 x 3, upright and inverted


def build(
    m,
    x,
    y,
    z,
    length,
    rows=5,
    color="Tan",
    tone="Dark Tan",
    rock="Light Bluish Gray",
    rocks=3,
    seed=1,
    title="Stone wall",
):
    """The courses and the rocks; returns the z of the top of the full courses."""
    if length < 4 or rows < 4:
        raise ValueError(
            f"irregular_stone_wall needs a length of 4 or more and 4 rows or more, not {length}, {rows}"
        )
    rng = random.Random(seed)
    # Each rock: (column, bottom course), never in the top course, two studs or more from the next one.
    placed = []
    for _ in range(rocks * 20):
        if len(placed) == rocks:
            break
        c, r = rng.randrange(length), rng.randrange(rows - 3)
        if all(abs(c - c2) >= 2 or abs(r - r2) >= 3 for c2, r2 in placed):
            placed.append((c, r))
    for r in range(rows):
        zr = z + 3 * r
        taken = {c for c, r2 in placed if r2 <= r < r2 + 3}
        with m.step(f"{title}, row {r + 1}"):
            for c, r2 in placed:
                if r2 == r:
                    m.add(rng.choice(ROCKS), rock, at=(x + c, y, zr), rot=3)
            at = 0
            while at < length:
                if at in taken:
                    at += 1
                    continue
                room = 1
                while at + room < length and at + room not in taken and room < 2:
                    room += 1
                # Odd courses start a segment with a 1 x 2, so the joints shift.
                n = 1 if room == 1 or (r % 2 and (at == 0 or at - 1 in taken)) else 2
                if r < rows - 1 or rng.random() < 0.6:
                    stone = tone if rng.random() < 0.2 else color
                    if n == 2:
                        m.add("3003", stone, at=(x + at, y, zr))
                    else:
                        m.add("3004", stone, at=(x + at, y, zr))
                at += n
    return z + 3 * (rows - 1)


def example(m):
    """A stone wall 10 studs long on a dark tan plate."""
    from emmet_dsl import Plate

    with m.step("Footing"):
        m.fill(Plate, "Dark Tan", at=(0, 0, 0), size=(10, 2))
    m.use("irregular_stone_wall", x=0, y=0, z=1, length=10)
