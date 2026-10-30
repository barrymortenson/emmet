# SPDX-License-Identifier: Apache-2.0
"""Pipe column cluster: a bank of big round pipes or reactor columns of uneven heights, banded and capped, packed side
by side.

m.use("pipe_column_cluster", x, y, z, cols=3, rows=1, height=4, seed=1, color="Light Bluish Gray",
band="Dark Bluish Gray", cap="Dark Bluish Gray", title="Pipes") stands `cols` x `rows` pipes 2 studs across, side
by side from (x, y), on the studs at z. Each is a stack of round bricks 2 x 2, plain (3941) or with a grille (92947),
with a round plate 2 x 2 in `band` over every second brick, and a cap: a round tile 2 x 2 or a dome (Dish 2 x 2
Inverted, 4740) in `cap`. Each pipe is `height` bricks tall, one brick less or more where seed says (never under 2), so
the bank's top is uneven. The pipes rise together, a course per step. Returns the z of the tallest pipe's top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75419. Not derived from any
model file."""

PARTS = ["3941", "92947", "4032b", "4150", "4740"]


def build(
    m,
    x,
    y,
    z,
    cols=3,
    rows=1,
    height=4,
    seed=1,
    color="Light Bluish Gray",
    band="Dark Bluish Gray",
    cap="Dark Bluish Gray",
    title="Pipes",
):
    """The pipes, a course at a time, and their caps; returns the z of the tallest top."""
    if not (1 <= cols <= 6 and 1 <= rows <= 3 and 2 <= height <= 10):
        raise ValueError(
            f"pipe_column_cluster takes 1 to 6 cols, 1 to 3 rows and a height of 2 to 10, not {cols}, {rows}, {height}"
        )
    rng = random.Random(seed)
    pipes = []
    for j in range(rows):
        for i in range(cols):
            # Each pipe as a list of (part, plates): bricks, a band over every second one.
            n = max(2, height + rng.choice((-1, 0, 0, 1)))
            pieces = []
            for b in range(n):
                pieces.append(("92947" if rng.random() < 0.3 else "3941", color, 3))
                if b % 2 and b < n - 1:
                    pieces.append(("4032b", band, 1))
            pieces.append(("4150" if rng.random() < 0.5 else "4740", cap, 1))
            pipes.append(((x + 2 * i, y + 2 * j), pieces))
    tops = {at: z for at, _ in pipes}
    course = 0
    while any(pieces for _, pieces in pipes):
        course += 1
        with m.step(f"{title}, course {course}"):
            for at, pieces in pipes:
                if pieces:
                    part, col, plates = pieces.pop(0)
                    m.add(part, col, at=(*at, tops[at]))
                    tops[at] += plates
    return max(tops.values())


def example(m):
    """A bank of three pipes by two on a floor plate."""
    with m.step("Floor"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
        m.add("3020", "Dark Bluish Gray", at=(4, 0, 0))
    m.use("pipe_column_cluster", x=0, y=0, z=1, cols=3, rows=2)
