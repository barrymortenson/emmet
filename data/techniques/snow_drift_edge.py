# SPDX-License-Identifier: Apache-2.0
"""Snow drift edge: a snowy patch whose edges are soft drifts, not a plate's edge: round corners, curved slopes laid
along the rim with their low side out, a deeper drift piled against a wall, and loose snow on top.

m.use("snow_drift_edge", x, y, z, length=12, depth=8, wall="back", seed=1, color="White", title="Snow") lays, on the
studs at height z (a baseplate is z = 0.5), a layer of snow plates `length` x `depth` studs from (x, y), with a Plate 4
x 4 with Corner Round (30565) at each corner, its round out. On its studs, along each open side between the corners,
drifts at random: Slope Brick Curved 2 x 2 (15068) and 2 x 1 (11477), their low edge on the patch's edge, with gaps
between. Along the `wall` side ("front", "back", "left", "right", or None), the drift faces the other way, low edge in
and high edge against the wall, a continuous bank where snow piles up against a house. A few round 1 x 1 plates are
loose snow. length and depth: 8 to 32. seed picks the drifts and the loose snow. Returns the z of the drifts' top.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387. Not derived from any
model file."""

PARTS = [
    "30565",
    "15068",
    "11477",
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
# A curved slope faces +x at rot=0, its low edge there, and a quarter on, counterclockwise, with each turn. By side:
# the turn that faces it out, and whether the side runs along x.
SIDES = {"front": (3, True), "back": (1, True), "left": (2, False), "right": (0, False)}


def _fill(cells):
    """Covers a set of (i, j) studs with plates, the largest that fit first, long along x where they can be."""
    free, out = set(cells), []
    for j0, i0 in sorted((j, i) for i, j in cells):
        if (i0, j0) not in free:
            continue
        for a, b in SIZES:
            for dx, dy in ((b, a), (a, b)):
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
    m, x, y, z, length=12, depth=8, wall="back", seed=1, color="White", title="Snow"
):
    """The snow layer, the drifts, and the loose snow; returns the z of the drifts' top."""
    if not 8 <= length <= 32 or not 8 <= depth <= 32:
        raise ValueError(
            f"snow_drift_edge takes a length and depth of 8 to 32, not {length} x {depth}"
        )
    if wall is not None and wall not in SIDES:
        raise ValueError(
            f"wall={wall!r} isn't a side: use front, back, left, right, or None"
        )
    rng = random.Random(seed)
    x1, y1 = x + length, y + depth
    corners = [((x, y), 2), ((x1 - 4, y), 3), ((x1 - 4, y1 - 4), 0), ((x, y1 - 4), 1)]
    rest = {(i, j) for i in range(x, x1) for j in range(y, y1)}
    for (cx, cy), _ in corners:
        rest -= {(cx + a, cy + b) for a in range(4) for b in range(4)}
    with m.step(f"{title}, corners"):
        # The corner plate's round is back right at rot=0, a quarter on with each turn.
        for (cx, cy), q in corners:
            m.add("30565", color, at=(cx, cy, z), rot=q)
    in_steps(
        m,
        f"{title}, layer",
        [
            (1, lambda p=p, at=(i, j, z), r=r: m.add(p, color, at=at, rot=r))
            for p, (i, j), r in _fill(rest)
        ],
    )
    z += 1
    used = set()
    for side, (rot, along_x) in SIDES.items():
        run = (length if along_x else depth) - 8
        actions, k = [], 0
        against = side == wall
        while k < run:
            if not against and rng.random() < 0.3:
                k += 1
                continue
            w = 2 if k + 2 <= run and rng.random() < 0.7 else 1
            # Each slope is 2 deep, from the side's edge in; against the wall it turns round, low edge in.
            at = {
                "front": (x + 4 + k, y),
                "back": (x + 4 + k, y1 - 2),
                "left": (x, y + 4 + k),
                "right": (x1 - 2, y + 4 + k),
            }[side]
            r = (rot + 2) % 4 if against else rot
            part = "15068" if w == 2 else "11477"
            actions.append(
                (
                    1,
                    lambda part=part, at=(*at, z), r=r: m.add(
                        part, color, at=at, rot=r
                    ),
                )
            )
            used |= {
                (at[0] + a, at[1] + b)
                for a in range(w if along_x else 2)
                for b in range(2 if along_x else w)
            }
            k += w
        if actions:
            in_steps(m, f"{title}, {side} drifts", actions)
    # Loose snow: a few round plates on open studs inside the drifts, none side by side.
    inner = sorted(
        (i, j)
        for i, j in rest
        if x + 2 <= i < x1 - 2 and y + 2 <= j < y1 - 2 and (i, j) not in used
    )
    rng.shuffle(inner)
    loose = []
    for c in inner:
        if len(loose) < max(2, length * depth // 24) and all(
            abs(c[0] - o[0]) > 1 or abs(c[1] - o[1]) > 1 for o in loose
        ):
            loose.append(c)
    with m.step(f"{title}, loose snow"):
        for i, j in loose:
            m.add("6141", color, at=(i, j, z))
    return z + 2


def example(m):
    """A snowy patch 14 x 11 studs, banked against a wall at the back, on a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    with m.step("Wall"):
        m.add("3008", "Reddish Brown", at=(1, 13, 0.5), rot=1)
        m.add("3008", "Reddish Brown", at=(9, 13, 0.5), rot=1)
        m.add("3008", "Reddish Brown", at=(5, 13, 3.5), rot=1)
    m.use("snow_drift_edge", x=1, y=2, z=0.5, length=14, depth=11, wall="back", seed=2)
