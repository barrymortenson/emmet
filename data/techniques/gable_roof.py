# SPDX-License-Identifier: Apache-2.0
"""Gable roof: two sides of 45° slopes that meet at a ridge, over a rectangle of any length.

m.use("gable_roof", x, y, z, length, depth, color="Dark Red", title="Roof") covers the length × depth rectangle
whose front-left corner is (x, y), starting at height z: the top of the walls, with a layer of plates (a ceiling)
under the whole rectangle for the first course to stand on. The slopes run along x and face the front and the back;
each course steps in one stud on both sides, with bricks behind the slopes, until a ridge of double slopes closes it.
depth must be even (4 or more). It brings its own steps, a side of a course at a time, titled "<title>, course 1:
front" and so on, and returns the z of the ridge's top.
"""

from emmet_dsl import Brick, Plate, Slope
from emmet_dsl.core import in_steps

PARTS = [
    "3037",
    "3038",
    "3039",
    "3040b",
    "3041",
    "3043",
    "3044c",
    "3010",
    "3622",
    "3004",
    "3005",
]

# Double slopes for the ridge, by length (the library has no 3-long one).
RIDGES = {4: "3041", 2: "3043", 1: "3044c"}


def _lengths(n, sizes):
    out = []
    for size in sorted(sizes, reverse=True):
        while n >= size:
            out.append(size)
            n -= size
    return out


def build(m, x, y, z, length, depth, color="Dark Red", title="Roof"):
    """The roof; returns the z of the ridge's top."""
    if depth < 4 or depth % 2:
        raise ValueError(f"gable_roof needs an even depth of 4 or more, not {depth}")
    slopes = {n: Slope(45, 2, n) for n in (4, 3, 2, 1)}
    bricks = {n: Brick(1, n) for n in (4, 3, 2, 1)}
    courses = (depth - 4) // 2 + 1
    y1 = y + depth - 1
    for k in range(courses):
        zk = z + 3 * k
        inner = list(
            range(y + k + 2, y1 - 1 - k)
        )  # rows of bricks behind this course's slopes
        half = (len(inner) + 1) // 2
        for side, sy, turn, rows in (
            ("front", y + k, 3, inner[:half]),
            ("back", y1 - 1 - k, 1, inner[half:]),
        ):

            def slope_row(sy=sy, turn=turn, zk=zk):
                sx = x
                for n in _lengths(length, slopes):
                    # rot=3 slopes down toward −y (the front), rot=1 toward +y (the back).
                    m.add(slopes[n], color, at=(sx, sy, zk), rot=turn)
                    sx += n

            def brick_row(ry, zk=zk):
                bx = x
                for n in _lengths(length, bricks):
                    m.add(bricks[n], color, at=(bx, ry, zk), rot=1)
                    bx += n

            actions = [(len(_lengths(length, slopes)), slope_row)]
            actions += [
                (len(_lengths(length, bricks)), lambda ry=ry: brick_row(ry))
                for ry in rows
            ]
            in_steps(m, f"{title}, course {k + 1}: {side}", actions)
    top = z + 3 * courses
    with m.step(f"{title}, ridge"):
        rx = x
        for n in _lengths(length, RIDGES):
            m.add(RIDGES[n], color, at=(rx, y + courses, top), rot=1)
            rx += n
    return top + 3


def example(m):
    """A 6 × 6 gable roof on a plate deck."""
    with m.step("Deck"):
        m.fill(Plate, "Light Bluish Gray", at=(0, 0, 0), size=(6, 6))
    m.use("gable_roof", x=0, y=0, z=1, length=6, depth=6, color="Dark Red")
