# SPDX-License-Identifier: Apache-2.0
"""Plank and stone wall: a blocky house wall, light wood planks over a speckled stone footing, with dark log posts at
its ends and a plain window set in the planks.

m.use("plank_and_stone_wall", x, y, z, length=8, stone_rows=2, plank_rows=4, window=None, seed=1,
stone=("Light Bluish Gray", "Dark Bluish Gray"), plank="Tan", log="Reddish Brown", title="Wall") lays a wall along x
from (x, y, z), facing front (-y), `length` studs long and stone_rows + plank_rows bricks tall. A 1 x 1 post in the
log color runs the full height at each end, a brick a course. Between the posts, the first `stone_rows` courses are
1 x 1 to 1 x 4 bricks in the `stone` shades, mixed by seed; above them, log bricks (1 x 2 and 1 x 4) in the plank
color, whose grooves read as planks. Every course's joints miss the ones below. window: None, or the x offset from x
of a 1 x 2 x 2 window (a frame in the plank color with its glass, no sill, the blocky style's plain opening) in the
second and third plank courses, and the course over it spans it. Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3005", "3004", "3622", "3010", "30136", "30137", "60592", "60601"]

STONE = {1: "3005", 2: "3004", 3: "3622", 4: "3010"}
PLANK = {
    1: "3005",
    2: "30136",
    3: "3622",
    4: "30137",
}  # logs where there's a log that long
GLASS = (
    0.175,
    0.65,
    0.875,
)  # where the glass sits in its frame (turned along x), from the frame's corner


def _course(rng, start, stop, below, prefer):
    """The (start, size) of the pieces from start to stop: lengths from `prefer` where they fit, no joint over a joint
    below where it can."""
    out, at = [], start
    while at < stop:
        left = stop - at
        sizes = [n for n in prefer if n <= left and left - n != 1 or n == left] or [
            min(left, 4)
        ]
        good = [n for n in sizes if at + n == stop or at + n not in below] or sizes
        n = rng.choice(good)
        out.append((at, n))
        at += n
    return out


def build(
    m,
    x,
    y,
    z,
    length=8,
    stone_rows=2,
    plank_rows=4,
    window=None,
    seed=1,
    stone=("Light Bluish Gray", "Dark Bluish Gray"),
    plank="Tan",
    log="Reddish Brown",
    title="Wall",
):
    """The courses, posts in each; returns the z of the top."""
    if length < 4 or stone_rows < 0 or plank_rows < 1:
        raise ValueError(
            f"plank_and_stone_wall needs a length of 4 or more and 1 plank row or more, not {length}, {plank_rows}"
        )
    if window is not None and (plank_rows < 4 or not 1 <= window <= length - 3):
        raise ValueError(
            f"plank_and_stone_wall's window needs 4 plank rows and an offset from 1 to {length - 3}, not {window}"
        )
    rng = random.Random(seed)
    below = set()
    for r in range(stone_rows + plank_rows):
        zr = z + 3 * r
        p = r - stone_rows  # the plank course, or negative in the stone
        gap = window is not None and p in (1, 2)
        spans = [(1, window), (window + 2, length - 1)] if gap else [(1, length - 1)]
        pieces = []
        for a, b in spans:
            pieces += _course(rng, a, b, below, (2, 3, 4) if p < 0 else (4, 2))
        with m.step(f"{title}, row {r + 1}"):
            m.add("3005", log, at=(x, y, zr))
            m.add("3005", log, at=(x + length - 1, y, zr))
            for c, n in pieces:
                if p < 0:
                    m.add(STONE[n], rng.choice(stone), at=(x + c, y, zr), rot=1)
                else:
                    m.add(PLANK[n], plank, at=(x + c, y, zr), rot=1)
            if gap and p == 1:
                wx = x + window
                m.add("60592", plank, at=(wx, y, zr), rot=1)
                m.add(
                    "60601",
                    "Trans Clear",
                    at=(wx + GLASS[0], y + GLASS[1], zr + GLASS[2]),
                    rot=1,
                )
        below = {c + n for c, n in pieces} | ({window, window + 2} if gap else set())
    return z + 3 * (stone_rows + plank_rows)


def example(m):
    """A wall 8 studs long with a window, on a footing plate."""
    with m.step("Footing"):
        m.add("3034", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("plank_and_stone_wall", x=0, y=0, z=1, length=8, window=3)
