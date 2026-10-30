# SPDX-License-Identifier: Apache-2.0
"""Tiled pond water: still water of long tiles laid in a running bond over a blue bed, so it shimmers in staggered rows,
with koi among the tiles, lily pads on open studs, and its front and back rows drawn in so the pond reads round.

m.use("tiled_pond_water", x, y, z, length=10, depth=6, seed=1, bed="Blue", water="Trans-Light Blue", fish=2, pads=2,
title="Pond") lays, on the studs at height z (a baseplate is z = 0.5), a pond inside `length` x `depth` studs from (x,
y): a bed of plates in `bed`, and over it rows of 1 x 2 and 1 x 4 tiles along x in `water`, each row's joints clear of
the row below's (a 1 x 1 tile shifts a row when it must), so no seam runs straight across. The front and back rows stop
a stud short at each end, with a 1 x 1 corner-round tile there, so the outline is rounded. `fish` of the 1 x 2 tiles are
koi (3069bp0p and 3069bp0q, made only in Trans-Light Blue; other water colors get none), and `pads` studs inside are
left open with a round 1 x 1 plate of bright green on them: lily pads, and places for lily_and_reeds. length: 4 to 24;
depth: 3 to 16. seed picks the tiles, fish, and pads. Returns the pads' studs, a list of (x, y), whose top is at z + 2
like the water's.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10315. Not derived from any
model file."""

PARTS = [
    "3069b",
    "2431",
    "3070b",
    "25269",
    "3069bp0p",
    "3069bp0q",
    "6141",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
]

TILES = {
    1: "3070b",
    2: "3069b",
    4: "2431",
}  # no 1 x 3: it was never made in Trans-Light Blue
KOI = ("3069bp0p", "3069bp0q")
SIZES = [
    (2, 8),
    (2, 6),
    (2, 4),
    (2, 3),
    (2, 2),
    (1, 8),
    (1, 6),
    (1, 4),
    (1, 3),
    (1, 2),
    (1, 1),
]


def _fill(cells):
    """Covers a set of (i, j) studs with plates, the largest that fit first, long along y where they can be."""
    free, out = set(cells), []
    for j0, i0 in sorted((j, i) for i, j in cells):
        if (i0, j0) not in free:
            continue
        for a, b in SIZES:
            for dx, dy in ((a, b), (b, a)):
                block = {(i0 + p, j0 + q) for p in range(dx) for q in range(dy)}
                if block <= free:
                    free -= block
                    out.append(
                        (
                            Plate(min(dx, dy), max(dx, dy)),
                            (i0, j0),
                            0 if dx <= dy else 1,
                        )
                    )
                    break
            else:
                continue
            break
    return out


def build(
    m,
    x,
    y,
    z,
    length=10,
    depth=6,
    seed=1,
    bed="Blue",
    water="Trans-Light Blue",
    fish=2,
    pads=2,
    title="Pond",
):
    """The bed, the water rows, the koi, and the pads; returns the pads' studs."""
    if not 4 <= length <= 24 or not 3 <= depth <= 16:
        raise ValueError(
            f"tiled_pond_water takes a length of 4 to 24 and a depth of 3 to 16, not {length} x {depth}"
        )
    rng = random.Random(seed)
    rows = [
        (x + 1, x + length - 1) if j in (0, depth - 1) else (x, x + length)
        for j in range(depth)
    ]
    cells = {(i, y + j) for j in range(depth) for i in range(x, x + length)}
    # Pads: inner studs, not on a row's ends, no two side by side.
    inner = [
        (i, y + j)
        for j, (a, b) in enumerate(rows)
        if 0 < j < depth - 1
        for i in range(a + 1, b - 1)
    ]
    rng.shuffle(inner)
    open_ = []
    for c in inner:
        if len(open_) < pads and all(
            abs(c[0] - o[0]) > 1 or abs(c[1] - o[1]) > 1 for o in open_
        ):
            open_.append(c)
    in_steps(
        m,
        f"{title}, bed",
        [
            (1, lambda p=p, at=(i, j, z), r=r: m.add(p, bed, at=at, rot=r))
            for p, (i, j), r in _fill(cells)
        ],
    )
    z += 1
    below, laid = set(), []
    for j, (a, b) in enumerate(rows):
        row, joints, p = [], set(), a
        if j in (0, depth - 1):
            # The rounded ends: a corner-round tile, its round to the outside corner. At rot=0 it rounds back right.
            front = j == 0
            row.append(("25269", (a - 1, y + j), 2 if front else 1))
            row.append(("25269", (b, y + j), 3 if front else 0))
        while p < b:
            stop = min([o[0] for o in open_ if o[1] == y + j and o[0] >= p] + [b])
            if p == stop:
                p += 1
                continue
            sizes = [
                n
                for n in (4, 2)
                if p + n <= stop and (p + n == stop or p + n not in below)
            ]
            n = rng.choice(sizes) if sizes else 1
            row.append((n, (p, y + j), 1))
            p += n
            joints.add(p)
        below = joints
        laid.append(row)
    twos = [
        (r, k) for r, row in enumerate(laid) for k, t in enumerate(row) if t[0] == 2
    ]
    koi = (
        set(rng.sample(twos, min(fish, len(twos))))
        if water == "Trans-Light Blue"
        else set()
    )
    for r, row in enumerate(laid):
        actions = []
        for k, (n, (i, j), rot) in enumerate(row):
            part = (
                n
                if isinstance(n, str)
                else KOI[len(actions) % 2]
                if (r, k) in koi
                else TILES[n]
            )
            actions.append(
                (
                    1,
                    lambda part=part, at=(i, j, z), rot=rot: m.add(
                        part, water, at=at, rot=rot
                    ),
                )
            )
        in_steps(m, f"{title}, water row {r + 1}", actions)
    with m.step(f"{title}, lily pads"):
        for i, j in open_:
            m.add("6141", "Bright Green", at=(i, j, z))
    return open_


def example(m):
    """A pond 10 x 6 studs, with two koi and two pads, on a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("tiled_pond_water", x=3, y=4, z=0.5, length=10, depth=6, seed=2)
