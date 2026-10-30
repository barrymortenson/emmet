# SPDX-License-Identifier: Apache-2.0
"""Stepped block terrain: a hill or dune that rises in square terraces on the 2-stud grid, never in slopes, its top in
square patches of two or three materials.

m.use("stepped_block_terrain", x, y, z, length=8, depth=8, peak=3, plate_steps=0.3, seed=1, body="Reddish Brown",
tops=("Bright Green", "Bright Green", "Tan"), title="Terrain") raises the length x depth area from (x, y), on the
studs at height z, as a grid of 2 x 2 columns. Each column is a whole number of bricks high, highest (`peak` bricks)
at a cell picked by seed and a brick lower for each cell away, give or take one, so the ground climbs in terraces
whose edges all sit on the block grid. Each course pairs neighboring columns into 2 x 4 bricks, along x on one course
and along y on the next, so the hill is tied together; 2 x 2 bricks fill in. About `plate_steps` of the columns get a
plate more, so some steps are a plate high and some a brick. Every column, even a flat one, ends in a 2 x 2 plate in
one of `tops`, by seed: repeat a color to weight it. For a dune use body and tops of Tan. length and depth: even.
Returns {(x, y): z of the top} for each 2 x 2 cell, so props can stand on it.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3001", "3003", "3022"]


def build(
    m,
    x,
    y,
    z,
    length=8,
    depth=8,
    peak=3,
    plate_steps=0.3,
    seed=1,
    body="Reddish Brown",
    tops=("Bright Green", "Bright Green", "Tan"),
    title="Terrain",
):
    """The columns, a course at a time, then the tops; returns each cell's top."""
    if length % 2 or depth % 2 or length < 2 or depth < 2 or peak < 0:
        raise ValueError(
            "stepped_block_terrain takes an even length and depth and a peak of 0 or more, "
            f"not {length}, {depth}, {peak}"
        )
    if not tops:
        raise ValueError("stepped_block_terrain needs at least one top color")
    rng = random.Random(seed)
    nx, ny = length // 2, depth // 2
    px, py = rng.randrange(nx), rng.randrange(ny)
    height = {}
    for j in range(ny):
        for i in range(nx):
            d = max(abs(i - px), abs(j - py))
            height[(i, j)] = max(0, min(peak, peak - d + rng.choice((-1, 0, 0, 1))))
    for level in range(peak):
        zl = z + 3 * level
        left = {c for c, h in height.items() if h > level}
        pieces = []
        dx, dy = (1, 0) if level % 2 == 0 else (0, 1)
        for j in range(ny):
            for i in range(nx):
                if (i, j) not in left:
                    continue
                left.discard((i, j))
                if (i + dx, j + dy) in left:
                    left.discard((i + dx, j + dy))
                    pieces.append(("3001", i, j, 1 if dx else 0))
                else:
                    pieces.append(("3003", i, j, 0))
        for s in range(0, len(pieces), 6):
            with m.step(f"{title}, course {level + 1}"):
                for part, i, j, turn in pieces[s : s + 6]:
                    m.add(part, body, at=(x + 2 * i, y + 2 * j, zl), rot=turn)
    out, caps = {}, []
    for (i, j), h in sorted(height.items(), key=lambda c: (c[0][1], c[0][0])):
        zc = z + 3 * h
        extra = rng.random() < plate_steps
        caps.append((i, j, zc, extra))
        out[(x + 2 * i, y + 2 * j)] = zc + 1 + extra
    for s in range(0, len(caps), 4):
        with m.step(f"{title}, tops"):
            for i, j, zc, extra in caps[s : s + 4]:
                if extra:
                    m.add("3022", body, at=(x + 2 * i, y + 2 * j, zc))
                m.add("3022", rng.choice(tops), at=(x + 2 * i, y + 2 * j, zc + extra))
    return out


def example(m):
    """An 8 x 8 grass hill with sand patches, on a plate."""
    with m.step("Ground"):
        m.add("41539", "Bright Green", at=(0, 0, 0))
    m.use("stepped_block_terrain", x=0, y=0, z=1, length=8, depth=8, peak=3, seed=4)
