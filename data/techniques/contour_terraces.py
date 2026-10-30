# SPDX-License-Identifier: Apache-2.0
"""Contour terraces: a low mound that rises in soft rings, each terrace a layer of earth under a grass top with round
corners that overhangs it by a stud, every ring set back from the one below by a different amount on each side.

m.use("contour_terraces", x, y, z, length=14, depth=12, terraces=3, rise=3, seed=1, earth="Reddish Brown",
grass=("Bright Green", "Green"), title="Mound") builds, on the studs at height z (a baseplate is z = 0.5), a mound
inside the box `length` x `depth` studs from (x, y). Each terrace is `rise` - 1 layers of earth plates a stud in from
its outline, and on them a grass layer that fills the outline, with a round-cornered plate at each corner, its round
out: a Plate 4 x 4 with Corner Round (30565) where the terrace is 8 or more studs each way, a Plate 3 x 3 with 2 x 2
Corner Round (30357) where it is 6 or 7. The grass overhangs the earth by a stud all around, so each step shows a dark
band of earth in its shadow, and its plates run the other way from the earth's, so the seams cross. Each terrace's
outline is set back 2 or 3 studs from the one below on each side, chosen at random, so the rings aren't concentric. It
stops early when the next outline would be under 6 studs. rise: 2 or 3 plates a terrace. grass: a color, or colors the
terraces take in turn, so each ring reads against the one below. seed picks the setbacks. Returns the z of the top
terrace.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10354 and the botanical
display in the instruction book 6650754. Not derived from any model file."""

PARTS = [
    "30565",
    "30357",
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


def _fill(cells, along, support=None):
    """Covers a set of (i, j) studs with plates, the largest that fit first, their long side along `along` ("x" or "y")
    where they can be, each on at least one stud of `support` when it's given: a list of (part, (i, j), rot)."""
    free, out = set(cells), []
    for j0, i0 in sorted((j, i) for i, j in cells):
        if (i0, j0) not in free:
            continue
        for a, b in SIZES:
            for dx, dy in ((a, b), (b, a)) if along == "y" else ((b, a), (a, b)):
                block = {(i0 + p, j0 + q) for p in range(dx) for q in range(dy)}
                if block <= free and (support is None or block & support):
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
    length=14,
    depth=12,
    terraces=3,
    rise=3,
    seed=1,
    earth="Reddish Brown",
    grass=("Bright Green", "Green"),
    title="Mound",
):
    """The terraces, the lowest first; returns the z of the top one."""
    if (
        not 6 <= length <= 32
        or not 6 <= depth <= 32
        or not 1 <= terraces <= 5
        or rise not in (2, 3)
    ):
        raise ValueError(
            f"contour_terraces takes a length and depth of 6 to 32, 1 to 5 terraces, and a rise of 2 or "
            f"3, not {length} x {depth}, {terraces}, {rise}"
        )
    rng = random.Random(seed)
    greens = (grass,) if isinstance(grass, str) else tuple(grass)
    x0, y0, x1, y1 = x, y, x + length, y + depth
    for t in range(terraces):
        if t:
            x0, y0 = x0 + rng.randint(2, 3), y0 + rng.randint(2, 3)
            x1, y1 = x1 - rng.randint(2, 3), y1 - rng.randint(2, 3)
        w, d = x1 - x0, y1 - y0
        if min(w, d) < 6:
            break
        inner = {(i, j) for i in range(x0 + 1, x1 - 1) for j in range(y0 + 1, y1 - 1)}
        for k in range(rise - 1):
            in_steps(
                m,
                f"{title}, terrace {t + 1}, earth",
                [
                    (1, lambda p=p, at=(i, j, z), r=r: m.add(p, earth, at=at, rot=r))
                    for p, (i, j), r in _fill(inner, "x")
                ],
            )
            z += 1
        part, s = ("30565", 4) if min(w, d) >= 8 else ("30357", 3)
        corners = [
            ((x0, y0), 2),
            ((x1 - s, y0), 3),
            ((x1 - s, y1 - s), 0),
            ((x0, y1 - s), 1),
        ]
        top = {(i, j) for i in range(x0, x1) for j in range(y0, y1)}
        for (cx, cy), _ in corners:
            top -= {(cx + a, cy + b) for a in range(s) for b in range(s)}
        with m.step(f"{title}, terrace {t + 1}, corners"):
            for (cx, cy), q in corners:
                m.add(
                    part,
                    greens[t % len(greens)],
                    at=(cx, cy, z),
                    rot=(q - ROUND[part]) % 4,
                )
        in_steps(
            m,
            f"{title}, terrace {t + 1}, grass",
            [
                (
                    1,
                    lambda p=p, at=(i, j, z), r=r, c=greens[t % len(greens)]: m.add(
                        p, c, at=at, rot=r
                    ),
                )
                for p, (i, j), r in _fill(top, "y", inner)
            ],
        )
        z += 1
    return z


def example(m):
    """A mound of three terraces on a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("contour_terraces", x=0, y=0, z=0.5, length=16, depth=16, terraces=3, seed=2)
