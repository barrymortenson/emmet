# SPDX-License-Identifier: Apache-2.0
"""Stepped plank roof: a gable roof with no slopes, climbing in 1-stud steps of plank-faced courses to a ridge of
blocks, the blocky-world roof.

m.use("stepped_plank_roof", x, y, z, length, depth, seed=1, color="Tan", planks=("Tan", "Medium Nougat"),
title="Roof") builds a roof on the walls whose outside is length x depth from (x, y), on their top at z, its ridge
along x. Each course is a brick tall: a 2-wide run of 2 x n bricks in `color` along the front and another along the
back, each a stud further in than the one below, so the roof climbs a stud in for each brick up; the gable ends
between them are 1-wide bricks, a stepped triangle. The exposed front row of each run is tiled with 1 x n tiles in
the `planks` shades (mixed by seed) as plank treads. The last course is a single 2-wide run along the middle, topped
with 2 x 2 jumpers: a ridge of blocks, one stud each. Every course's joints miss the ones below. length and depth:
even, 4 or more. Returns the z of the ridge's top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = [
    "3003",
    "3002",
    "3001",
    "2456",
    "3007",
    "3005",
    "3004",
    "3622",
    "3010",
    "3069b",
    "63864",
    "2431",
    "6636",
    "3070b",
    "87580",
]

RUN = {2: "3003", 3: "3002", 4: "3001", 6: "2456", 8: "3007"}  # 2 x n bricks
GABLE = {1: "3005", 2: "3004", 3: "3622", 4: "3010"}  # 1 x n bricks
TREAD = {1: "3070b", 2: "3069b", 3: "63864", 4: "2431", 6: "6636"}  # 1 x n tiles


def _pieces(rng, length, below, sizes):
    """The (start, size) of a run of `length`: sizes by seed, no joint over a joint below where it can."""
    out, at = [], 0
    while at < length:
        left = length - at
        fit = [
            n for n in sizes if n <= left and (left - n == 0 or left - n >= min(sizes))
        ] or [left]
        good = [n for n in fit if at + n == length or at + n not in below] or fit
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
    depth,
    seed=1,
    color="Tan",
    planks=("Tan", "Medium Nougat"),
    title="Roof",
):
    """The courses, bottom up, then the ridge; returns the z of the ridge's top."""
    if length % 2 or depth % 2 or length < 4 or depth < 4:
        raise ValueError(
            f"stepped_plank_roof takes an even length and depth of 4 or more, not {length}, {depth}"
        )
    rng = random.Random(seed)
    below, gable_below = set(), set()
    k = 0
    while True:
        zk = z + 3 * k
        last = depth - 2 * k == 2
        run = _pieces(rng, length, below, (8, 6, 4, 3, 2))
        rows = [y + k] if last else [y + k, y + depth - k - 2]
        fill = depth - 2 * k - 4
        gable = _pieces(rng, fill, gable_below, (4, 3, 2, 1)) if fill > 0 else []
        # The front run and the left gable, then the back run and the right gable.
        for ry, gx in zip(rows, (x, x + length - 1)):
            with m.step(f"{title}, course {k + 1}"):
                for c, n in run:
                    m.add(RUN[n], color, at=(x + c, ry, zk), rot=1)
                for c, n in gable:
                    m.add(GABLE[n], color, at=(gx, y + k + 2 + c, zk))
        gable_below = {
            c + n - 1 for c, n in gable
        }  # in the next course's frame, which starts a stud further in
        below = {c + n for c, n in run}
        if last:
            break
        with m.step(f"{title}, course {k + 1}, treads"):
            for ty in (y + k, y + depth - k - 1):
                for c, n in _pieces(rng, length, set(), (6, 4, 3, 2)):
                    m.add(TREAD[n], rng.choice(planks), at=(x + c, ty, zk + 3), rot=1)
        k += 1
    top = z + 3 * (k + 1)
    with m.step(f"{title}, ridge"):
        for c in range(0, length, 2):
            m.add("87580", color, at=(x + c, y + k, top))
    return top + 1


def example(m):
    """A stepped roof on walls 8 x 8 studs, two bricks tall."""
    with m.step("Floor"):
        m.add("41539", "Dark Bluish Gray", at=(0, 0, 0))
    for r in range(2):
        with m.step(f"Walls, row {r + 1}"):
            m.wall(0, 0, 1 + 3 * r, 8, 8, color="Light Bluish Gray")
    m.use("stepped_plank_roof", x=0, y=0, z=7, length=8, depth=8)
