# SPDX-License-Identifier: Apache-2.0
"""Organic ground outline: a patch of raised lawn whose edge is a chain of rounded lobes, never a rectangle: a layer of
soil plates with round corners and tongues, and a grass layer set in from it with its own, so a soil lip shows around the
edge.

m.use("organic_ground_outline", x, y, z, length=12, depth=10, seed=1, soil="Reddish Brown", grass="Green", title="Lawn")
lays, on the studs at height z (a baseplate is z = 0.5), two layers of plates inside the box `length` x `depth` studs
from (x, y). Each layer is a core rectangle a stud in from its own box, with a Plate 4 x 4 with Corner Round (30565) on
each corner, its round out, and along each side, at random, tongues that reach the box's edge: a Plate 2 x 2 Round
(4032b) for a small bump, or two Plates 3 x 3 with 2 x 2 Corner Round (30357) back to back for a long lobe, with gaps
between where the edge falls back a stud. The soil layer fills the box; the grass layer, a plate up, fills the box a
stud smaller all around and its plates run the other way, so its seams cross the soil's. Where a soil tongue meets a
grass gap, two studs of soil show; where a grass tongue meets a soil gap, the grass comes to the edge. A few round 1 x 1
plates of grass dot the bare soil, as tufts past the edge. length and depth: 10 to 32. seed picks the tongues and tufts.
Returns the z of the grass's top.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21360, 10354, 43268, 75936,
and 31109. Not derived from any model file."""

PARTS = [
    "30565",
    "30357",
    "4032b",
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
    "3031",
    "3032",
    "3035",
]

# Which corner of each shaped plate is round at rot=0, as a quarter: 0 is back right (+x, +y), 1 back left, 2 front
# left, 3 front right. A turn of k quarters moves the round corner k quarters on, counterclockwise.
ROUND = {"30565": 0, "30357": 1}
# Plate sizes the fill uses, (across, along) the way it prefers, largest first.
SIZES = [
    (4, 8),
    (4, 6),
    (4, 4),
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


def _turn(part, quarter):
    """The rot that puts `part`'s round corner at `quarter`."""
    return (quarter - ROUND[part]) % 4


def _fill(cells, along):
    """Covers a set of (i, j) studs with plates, the largest that fit first, their long side along `along` ("x" or "y")
    where they can be: a list of (part, (i, j), rot)."""
    free, out = set(cells), []
    for j0, i0 in sorted((j, i) for i, j in cells):
        if (i0, j0) not in free:
            continue
        for a, b in SIZES:
            for dx, dy in ((a, b), (b, a)) if along == "y" else ((b, a), (a, b)):
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


def _layer(x0, y0, length, depth, rng):
    """One layer's shaped pieces, (part, (i, j), rot), and the studs its plain plates cover."""
    core = {
        (i, j)
        for i in range(x0 + 1, x0 + length - 1)
        for j in range(y0 + 1, y0 + depth - 1)
    }
    shaped, used = [], set()

    def put(part, i, j, rot, w):
        shaped.append((part, (i, j), rot))
        used.update((i + p, j + q) for p in range(w) for q in range(w))

    x1, y1 = x0 + length, y0 + depth
    put("30565", x0, y0, _turn("30565", 2), 4)
    put("30565", x1 - 4, y0, _turn("30565", 3), 4)
    put("30565", x1 - 4, y1 - 4, _turn("30565", 0), 4)
    put("30565", x0, y1 - 4, _turn("30565", 1), 4)
    # Each side: (start of the stretch between the corners, its length, the way along it, where a tongue's 2 or 3
    # studs inward start, and the quarters a long lobe's two ends round toward).
    sides = [
        ((x0 + 4, y0), length - 8, (1, 0), (0, 0), (2, 3)),
        ((x0 + 4, y1), length - 8, (1, 0), (0, -1), (1, 0)),
        ((x0, y0 + 4), depth - 8, (0, 1), (0, 0), (2, 1)),
        ((x1, y0 + 4), depth - 8, (0, 1), (-1, 0), (3, 0)),
    ]
    for (sx, sy), run, (ax, ay), (ox, oy), (q0, q1) in sides:
        k = rng.randint(0, 1)
        while k < run:
            roll = rng.random()
            if roll < 0.45 and k + 6 <= run:
                for n, q in enumerate((q0, q1)):
                    put(
                        "30357",
                        sx + ax * (k + 3 * n) + 3 * ox,
                        sy + ay * (k + 3 * n) + 3 * oy,
                        _turn("30357", q),
                        3,
                    )
                k += 6 + rng.randint(1, 2)
            elif roll < 0.8 and k + 2 <= run:
                put("4032b", sx + ax * k + 2 * ox, sy + ay * k + 2 * oy, 0, 2)
                k += 2 + rng.randint(1, 2)
            else:
                k += 1
    return shaped, (core - used)


def build(
    m,
    x,
    y,
    z,
    length=12,
    depth=10,
    seed=1,
    soil="Reddish Brown",
    grass="Green",
    title="Lawn",
):
    """The soil and grass layers and the tufts; returns the z of the grass's top."""
    if not 10 <= length <= 32 or not 10 <= depth <= 32:
        raise ValueError(
            f"organic_ground_outline takes a length and depth of 10 to 32, not {length} x {depth}"
        )
    rng = random.Random(seed)
    layers = [
        (x, y, length, depth, soil, "x"),
        (x + 1, y + 1, length - 2, depth - 2, grass, "y"),
    ]
    covered = []
    for k, (x0, y0, w, d, color, along) in enumerate(layers):
        shaped, plain = _layer(x0, y0, w, d, rng)
        name = "soil" if k == 0 else "grass"
        at_z = z + k
        in_steps(
            m,
            f"{title}, {name} edge",
            [
                (
                    1,
                    lambda p=p, i=i, j=j, r=r, c=color: m.add(
                        p, c, at=(i, j, at_z), rot=r
                    ),
                )
                for p, (i, j), r in shaped
            ],
        )
        in_steps(
            m,
            f"{title}, {name}",
            [
                (
                    1,
                    lambda p=p, i=i, j=j, r=r, c=color: m.add(
                        p, c, at=(i, j, at_z), rot=r
                    ),
                )
                for p, (i, j), r in _fill(plain, along)
            ],
        )
        covered.append((shaped, plain))
    # Tufts go only on bare studs of the soil's plain plates, a stud or more apart.
    (_, soil_plain), (grass_shaped, grass_plain) = covered
    over = set(grass_plain)
    for p, (i, j), _ in grass_shaped:
        w = 4 if p == "30565" else 3 if p == "30357" else 2
        over.update((i + a, j + b) for a in range(w) for b in range(w))
    bare = sorted(soil_plain - over)
    rng.shuffle(bare)
    tufts = []
    for c in bare:
        if len(tufts) >= max(2, (length + depth) // 6):
            break
        if all(abs(c[0] - t[0]) > 1 or abs(c[1] - t[1]) > 1 for t in tufts):
            tufts.append(c)
    with m.step(f"{title}, tufts"):
        for i, j in tufts:
            m.add("6141", grass, at=(i, j, z + 1))
    return z + 2


def example(m):
    """A lawn 14 x 10 studs on a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("organic_ground_outline", x=1, y=3, z=0.5, length=14, depth=10, seed=3)
