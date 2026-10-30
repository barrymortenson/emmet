# SPDX-License-Identifier: Apache-2.0
"""Checker light wall: a control wall of red and white light squares in a black frame, a tiled plate stood upright on
side studs.

m.use("checker_light_wall", x, y, z, width=6, height=6, seed=1, frame="Black", lights=("Red", "White"),
title="Light wall") builds, on the studs at height z, a wall `width` studs long along x from (x, y), facing front (-y),
and `height` studs tall (2.5 plates a stud): a backing of bricks 1 2/3 tall with studs on one side (22885, and 32952 at
the ends of every other row, so the joints cross), a column of 1 x 2 bricks at each end (x - 1 and x + width), and a
lintel plate over all of it. The lights are built flat as a submodel: a plate `width` x `height` covered in 2 x 2
squares of tiles, the two `lights` colors in a checkerboard, some squares (seed picks which) split into four 1 x 1
tiles, also checkered. It stands upright on the backing's side studs, 0.8 studs proud of it, between the columns.
width: 2 to 8, height: 2 to 8, both even. The columns stand on y - 1 to y + 1, so give it a floor 2 studs deep.
Returns the z of the lintel's top.
"""

import random

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75419. Not derived from any
model file."""

PARTS = [
    "22885",
    "32952",
    "3004",
    "3068b",
    "3070b",
    "3958",
    "3036",
    "3020",
    "3795",
    "3832",
    "3034",
    "3460",
]

UPRIGHT = Rot(x=90)  # a part built flat, stood up with its studs to -y


def _lights(m, width, height, seed, lights):
    """The plate and its squares, flat, its corner at the origin."""
    rng = random.Random(seed)
    with m.step("Plate"):
        m.fill(Plate, "Black", at=(0, 0, 0), size=(width, height))
    squares = [(i, j) for j in range(0, height, 2) for i in range(0, width, 2)]
    for k in range(0, len(squares), 2):
        with m.step(f"Lights {k // 2 + 1}"):
            for i, j in squares[k : k + 2]:
                first = lights[(i // 2 + j // 2) % 2]
                if rng.random() < 0.3:
                    for a in (0, 1):
                        for b in (0, 1):
                            m.add(
                                "3070b",
                                lights[(i // 2 + j // 2 + a + b) % 2],
                                at=(i + a, j + b, 1),
                            )
                else:
                    m.add("3068b", first, at=(i, j, 1))


def build(
    m,
    x,
    y,
    z,
    width=6,
    height=6,
    seed=1,
    frame="Black",
    lights=("Red", "White"),
    title="Light wall",
):
    """The backing, the columns, the lights, and the lintel; returns the z of the lintel's top."""
    if not (2 <= width <= 8 and 2 <= height <= 8) or width % 2 or height % 2:
        raise ValueError(
            f"checker_light_wall takes an even width and height of 2 to 8 studs, not {width}, {height}"
        )
    rows = height // 2  # rows of bricks 5 plates tall: 2 studs each
    for r in range(rows):
        zr = z + 5 * r
        with m.step(f"{title}, backing row {r + 1}"):
            k = 0
            if r % 2:
                m.add("32952", frame, at=(x, y, zr), rot=3)
                m.add("32952", frame, at=(x + width - 1, y, zr), rot=3)
                k = 1
            while k + 2 <= width - (r % 2):
                m.add("22885", frame, at=(x + k, y, zr), rot=3)
                k += 2
    zt = z + 5 * rows
    for side, cx in (("left", x - 1), ("right", x + width)):
        with m.step(f"{title}, {side} column"):
            # Bricks to just under the top; the last plates make up the odd plate.
            n = (zt - z) // 3
            m.stack("3004", frame, at=(cx, y - 1, z), count=n)
            for p in range(int(zt - z - 3 * n)):
                m.add("3023b", frame, at=(cx, y - 1, z + 3 * n + p))
    panel = m.submodel(
        "checker_lights",
        _lights,
        width=width,
        height=height,
        seed=seed,
        lights=tuple(lights),
    )
    with m.step(f"{title}, lights"):
        m.place(panel, at=(x, y - 0.8, z), rot=UPRIGHT)
    with m.step(f"{title}, lintel"):
        m.row(Plate, frame, at=(x - 1, y - 1, zt), length=width + 2, width=2)
    return zt + 1


def example(m):
    """A 6 x 6 light wall on a strip of floor."""
    with m.step("Floor"):
        m.row(Plate, "Dark Bluish Gray", at=(-1, -1, 0), length=8, width=2)
    m.use("checker_light_wall", x=0, y=0, z=1)
