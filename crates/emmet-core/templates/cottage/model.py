# SPDX-License-Identifier: Apache-2.0
from typing import Annotated

from emmet_dsl import Color, Model, Range, Step, model

# The baseplate is half a plate thick, so everything on it starts at z = 0.5.
GROUND = 0.5

BRICKS = {4: "3010", 3: "3622", 2: "3004", 1: "3005"}  # Brick 1 x n
PLATES = {16: "4282", 12: "2445", 10: "3832", 8: "3034", 6: "3795", 4: "3020"}
PLATES |= {3: "3021", 2: "3022"}  # Plate 2 x n
SLOPES = {4: "3037", 3: "3038", 2: "3039", 1: "3040b"}  # Slope 45 2 x n
RIDGES = {4: "3041", 2: "3043", 1: "3044c"}  # Slope 45 2 x n double


def lengths(n, sizes):
    """Splits a run of n studs into the longest pieces available, longest first."""
    out = []
    for size in sorted(sizes, reverse=True):
        while n >= size:
            out.append(size)
            n -= size
    return out


def runs(cells, axis):
    """Groups cells into unbroken runs along the axis: [(first cell, length), ...]."""
    out = []
    for c in cells:
        if out:
            (lx, ly), n = out[-1]
            if c == ((lx + n, ly) if axis == "x" else (lx, ly + n)):
                out[-1] = ((lx, ly), n + 1)
                continue
        out.append((c, 1))
    return out


@model
def cottage(
    width: Annotated[int, Range(6, 16)] = 8,
    depth: Annotated[int, Range(4, 12), Step(2)] = 6,
    wall_rows: Annotated[int, Range(6, 9)] = 6,
    wall_color: Color = "Tan",
    roof_color: Color = "Dark Red",
    chimney: bool = True,
):
    """A small cottage on a green 32 x 32 baseplate.
    width: how many studs wide the house is
    depth: how many studs deep the house is
    wall_rows: how many bricks tall the walls are
    wall_color: the color of the walls
    roof_color: the color of the roof
    chimney: add a brick chimney at the back
    """
    m = Model()
    # The house's front-left and back-right corners, centered on the baseplate.
    x0, y0 = (32 - width) // 2, (32 - depth) // 2
    x1, y1 = x0 + width - 1, y0 + depth - 1

    with m.step("Baseplate"):
        m.add("3811", "Green", at=(0, 0, 0), id="base")

    # The door in the middle of the front wall, a window in each side wall and the back.
    door_x = x0 + (width - 4) // 2
    side_y = y0 + (depth - 2) // 2
    back_x = x0 + (width - 2) // 2
    door = {(x, y0) for x in range(door_x, door_x + 4)}
    windows = {(x0, side_y), (x0, side_y + 1), (x1, side_y), (x1, side_y + 1)}
    windows |= {(back_x, y1), (back_x + 1, y1)}

    for row in range(wall_rows):
        z = GROUND + 3 * row
        with m.step(f"Walls, row {row + 1}"):
            if row == 0:
                m.add("60596", "White", at=(door_x, y0, z), rot=3, id="door")
            if row == 2:
                m.add("60592", "White", at=(x0, side_y, z), rot=2, id="window/left")
                m.add("60592", "White", at=(x1, side_y, z), rot=0, id="window/right")
                m.add("60592", "White", at=(back_x, y1, z), rot=1, id="window/back")
            # Corners alternate between the long and the short walls, so the joints
            # overlap from row to row.
            if row % 2 == 0:
                walls = [((x0, y0), width, "x"), ((x0, y1), width, "x")]
                walls += [
                    ((x0, y0 + 1), depth - 2, "y"),
                    ((x1, y0 + 1), depth - 2, "y"),
                ]
            else:
                walls = [((x0, y0), depth, "y"), ((x1, y0), depth, "y")]
                walls += [
                    ((x0 + 1, y0), width - 2, "x"),
                    ((x0 + 1, y1), width - 2, "x"),
                ]
            gaps = (door if row < 6 else set()) | (windows if 2 <= row < 4 else set())
            for (sx, sy), n, axis in walls:
                cells = [
                    (sx + i, sy) if axis == "x" else (sx, sy + i) for i in range(n)
                ]
                for (x, y), run in runs([c for c in cells if c not in gaps], axis):
                    for size in lengths(run, [2, 1]):
                        turn = 1 if axis == "x" else 0
                        m.add(BRICKS[size], wall_color, at=(x, y, z), rot=turn)
                        x, y = (x + size, y) if axis == "x" else (x, y + size)

    top = GROUND + 3 * wall_rows
    with m.step("Ceiling"):
        for y in range(y0, y0 + depth, 2):
            x = x0
            for size in lengths(width, PLATES):
                m.add(PLATES[size], wall_color, at=(x, y, top), rot=1)
                x += size

    # A gable roof: each course steps in one stud from the front and the back, until
    # the two sides meet under a ridge of double slopes.
    courses = (depth - 4) // 2 + 1
    for k in range(courses + 1):
        z = top + 1 + 3 * k
        with m.step(f"Roof, course {k + 1}"):
            x = x0
            if k == courses:
                for size in lengths(width, RIDGES):
                    m.add(RIDGES[size], roof_color, at=(x, y0 + k, z), rot=1)
                    x += size
                continue
            for size in lengths(width, SLOPES):
                # rot=3 slopes toward -y (the front side), rot=1 toward +y (the back).
                m.add(SLOPES[size], roof_color, at=(x, y0 + k, z), rot=3)
                m.add(SLOPES[size], roof_color, at=(x, y1 - 1 - k, z), rot=1)
                x += size
            for y in range(y0 + k + 2, y1 - 1 - k):  # bricks under the next course
                x = x0
                for size in lengths(width, BRICKS):
                    m.add(BRICKS[size], roof_color, at=(x, y, z), rot=1)
                    x += size

    if chimney:
        ridge_top = top + 1 + 3 * (courses + 1)
        with m.step("Chimney"):
            z = GROUND
            while z < ridge_top + 3:
                m.add("3004", "Reddish Brown", at=(x0 + 1, y1 + 1, z), rot=1)
                z += 3
    return m
