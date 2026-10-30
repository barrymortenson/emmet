# SPDX-License-Identifier: Apache-2.0
"""Grass edge: a grassy top on a cliff or wall whose front edge wanders, overhanging the face in rounded tongues here
and stopping short of it there, locked by a second layer of plates.

m.use("grass_edge", x, y, z, length, depth=4, seed=1, color="Green", title="Grass") lays grass on the studs at height
z (the top of a cliff whose face is at y), `length` studs along x from x and `depth` studs back from y. Along the front,
the edge runs in stretches: flush with the face, stopping a stud short so the rock's top shows, or overhanging it by a
stud, a 2-wide tongue of plates whose two ends are 2 x 2 plates with a cut corner, so the tongue reads rounded. The
tongues' back halves sit on the cliff; a second layer from y + 1 to the back, its strips running the other way,
locks the first layer's rows together. seed picks the stretches. Returns the z of the grass's top.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 21335 (grass cap with an overhanging edge). Not derived from any model file."""

PARTS = [
    "26601",
    "3022",
    "3460",
    "3666",
    "3710",
    "3623",
    "3023b",
    "3024",
    "3795",
    "3020",
    "3021",
    "4477",
]

GREEN = (1, 2, 3, 4, 6, 8, 10)  # 1-wide plate lengths made in greens (no 1 x 12)


def _stretches(rng, length):
    """The front edge as [(start, n, kind)]: "flush", "short" (a stud back), or "over" (a stud out, n of 4 or 6)."""
    out, k, last = [], 0, None
    while k < length:
        kinds = [c for c in ("flush", "short", "over") if c != last]
        kind = rng.choice(kinds)
        if kind == "over":
            n = 6 if k + 6 <= length and rng.random() < 0.4 else 4
            if k + n > length:
                kind = "flush"
        if kind != "over":
            n = min(length - k, rng.choice((1, 2, 3)))
        out.append((k, n, kind))
        k += n
        last = kind
    return out


def build(m, x, y, z, length, depth=4, seed=1, color="Green", title="Grass"):
    """The grass's two layers; returns the z of its top."""
    if not 4 <= length <= 16 or not 2 <= depth <= 8:
        raise ValueError(
            f"grass_edge takes a length of 4 to 16 and a depth of 2 to 8, not {length}, {depth}"
        )
    rng = random.Random(seed)
    front = []
    for s, n, kind in _stretches(rng, length):
        x0 = x + s
        if kind == "over":
            front.append(
                (1, lambda x0=x0: m.add("26601", color, at=(x0, y - 1, z), rot=2))
            )
            if n == 6:
                front.append(
                    (1, lambda x0=x0: m.add("3022", color, at=(x0 + 2, y - 1, z)))
                )
            xe = x0 + n - 2
            front.append(
                (1, lambda xe=xe: m.add("26601", color, at=(xe, y - 1, z), rot=3))
            )
        elif kind == "flush":
            front += row_actions(m, Plate, color, (x0, y, z), n, sizes=GREEN)
    # Behind the front row, the first layer runs along x a row at a time.
    for j in range(1, depth):
        front += row_actions(m, Plate, color, (x, y + j, z), length, sizes=GREEN)
    in_steps(m, f"{title}, first layer", front)
    # The second layer runs along y, 2-wide strips (a 1-wide one at an odd end), from y + 1 back.
    lock = []
    for i in range(0, length, 2):
        w = min(2, length - i)
        lock += row_actions(
            m, Plate, color, (x + i, y + 1, z + 1), depth - 1, axis="y", width=w
        )
    in_steps(m, f"{title}, second layer", lock)
    return z + 2


def example(m):
    """Grass along the top of a short cliff 8 studs long."""
    with m.step("Cliff"):
        m.add("3035", "Dark Bluish Gray", at=(0, 1, 0), rot=1)
        m.add("3008", "Dark Bluish Gray", at=(0, 1, 1), rot=1)
        m.add("3001", "Dark Bluish Gray", at=(0, 2, 1), rot=1)
        m.add("3001", "Dark Bluish Gray", at=(4, 2, 1), rot=1)
        m.add("3008", "Dark Bluish Gray", at=(0, 4, 1), rot=1)
    m.use("grass_edge", x=0, y=1, z=4, length=8, depth=4, seed=7)
