# SPDX-License-Identifier: Apache-2.0
"""Raised module base: a slab of game terrain on dirt-colored legs, one brick off the table, like a chunk cut out of a
block world.

m.use("raised_module_base", x, y, z, length=16, depth=8, top="Bright Green", dirt="Reddish Brown", spacing=4, seed=1,
title="Module base") builds a length x depth slab from (x, y), standing on the table (or studs) at height z: 2 x 2
bricks in the dirt color as legs, one every `spacing` studs around the edge and across the middle, and on them the
top, the largest plates in the top color that fit (up to 8 x 16), every seam between them on a leg. Edge legs are set
back a stud here and there, by seed, so the dark edge under the top reads ragged. length and depth: even, 4 to 32.
Swap top for Tan (sand) or Light Bluish Gray (stone). Where two modules meet, their edges butt on the 2-stud grid.
Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = [
    "3003",
    "92438",
    "41539",
    "3036",
    "3035",
    "3034",
    "3958",
    "3032",
    "3795",
    "3031",
    "3020",
    "3022",
]

# Plate (short side, long side) -> id: the top's pieces.
PLATES = {
    (8, 16): "92438",
    (8, 8): "41539",
    (6, 8): "3036",
    (4, 8): "3035",
    (2, 8): "3034",
    (6, 6): "3958",
    (4, 6): "3032",
    (2, 6): "3795",
    (4, 4): "3031",
    (2, 4): "3020",
    (2, 2): "3022",
}


def _spans(n, big):
    """Splits n studs into runs of `big` and a remainder, the seams on even studs."""
    out, at = [], 0
    while at < n:
        out.append((at, min(big, n - at)))
        at += big
    return out


def _legs(n, spacing):
    """Where legs go along one side: the two ends and every `spacing` studs between."""
    return sorted({0, n} | set(range(spacing, n, spacing)))


def build(
    m,
    x,
    y,
    z,
    length=16,
    depth=8,
    top="Bright Green",
    dirt="Reddish Brown",
    spacing=4,
    seed=1,
    title="Module base",
):
    """The legs, then the top; returns the z of the top."""
    if length % 2 or depth % 2 or not 4 <= length <= 32 or not 4 <= depth <= 32:
        raise ValueError(
            f"raised_module_base takes an even length and depth of 4 to 32, not {length}, {depth}"
        )
    if spacing not in (4, 8):
        raise ValueError(f"raised_module_base takes a spacing of 4 or 8, not {spacing}")
    rng = random.Random(seed)
    # The top's plates: strips 8 wide across the short side, cut along the long side into runs of 16 (8 where a
    # strip is narrower, since there's no 4 x 16 plate).
    long_n, short_n = (length, depth) if length >= depth else (depth, length)
    pieces = []
    for b, lb in _spans(short_n, 8):
        for a, la in _spans(long_n, 16 if lb == 8 else 8):
            pieces.append((a, b, la, lb) if length >= depth else (b, a, lb, la))
    # Legs at every seam and every `spacing` studs: a leg's 2 x 2 sits across the line it marks. Corners first, then
    # the rest, each left out where it would clash with one already placed (on a small module).
    xs = sorted(set(_legs(length, spacing)) | {a for a, _, _, _ in pieces})
    ys = sorted(set(_legs(depth, spacing)) | {b for _, b, _, _ in pieces})
    spots = sorted(
        ((i, j) for i in xs for j in ys),
        key=lambda p: p[0] not in (0, length) or p[1] not in (0, depth),
    )
    legs = []
    for i, j in spots:
        edge_x, edge_y = i in (0, length), j in (0, depth)
        lx = 0 if i == 0 else length - 2 if i == length else i - 1
        ly = 0 if j == 0 else depth - 2 if j == depth else j - 1
        # Set an edge leg (not a corner) back a stud now and then.
        if edge_x != edge_y and rng.random() < 0.5:
            if edge_x:
                lx += 1 if i == 0 else -1
            else:
                ly += 1 if j == 0 else -1
        if all(abs(lx - ox) >= 2 or abs(ly - oy) >= 2 for ox, oy in legs):
            legs.append((lx, ly))
    legs.sort(key=lambda p: (p[1], p[0]))
    for s in range(0, len(legs), 8):
        with m.step(f"{title}, legs {s // 8 + 1}"):
            for lx, ly in legs[s : s + 8]:
                m.add("3003", dirt, at=(x + lx, y + ly, z))
    with m.step(f"{title}, top"):
        for a, b, la, lb in pieces:
            short, long = sorted((la, lb))
            part = PLATES[(short, long)]
            # A plate's long side runs along y at rot 0.
            m.add(part, top, at=(x + a, y + b, z + 3), rot=0 if lb >= la else 1)
    return z + 4


def example(m):
    """A grass module 16 x 8 beside a stone one 8 x 8, their edges butting."""
    m.use("raised_module_base", x=0, y=0, z=0, length=16, depth=8)
    m.use(
        "raised_module_base",
        x=16,
        y=0,
        z=0,
        length=8,
        depth=8,
        top="Light Bluish Gray",
        dirt="Dark Bluish Gray",
        seed=2,
        title="Stone module",
    )
