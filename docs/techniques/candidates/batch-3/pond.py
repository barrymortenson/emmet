# SPDX-License-Identifier: Apache-2.0
"""Pond: a small pond sunk a plate into the ground, its water clear tiles with rounded corners over a blue bed, lily
pads on the water, and stones and reeds on its rim.

m.use("pond", x, y, z, length=6, depth=4, seed=1, bed="Blue", water="Trans-Light Blue", ground="Green",
stone="Light Bluish Gray", title="Pond") builds, on the studs at height z, a pond whose water covers length x depth
studs from (x + 1, y + 1), inside a 1-stud rim of `ground` plates: a bed of blue plates under it all ties the rim
together, the water is 1-wide clear tiles with a 1 x 1 corner-round tile at each of its four corners, so the pond
reads round, and two or three studs of the water are left for lily pads (1 x 1 round plates with tabs, bright green)
standing on the bed. On the rim, a few round 1 x 1 plates and tiles are stones, and grass stems stand at one back
corner as reeds. The rim is (length + 2) x (depth + 2) over all. seed picks where pads and stones go. Returns the z
of the rim's top.
"""

import random

from emmet_dsl import Plate, frame
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3034",
    "3795",
    "3020",
    "3832",
    "3460",
    "3666",
    "3710",
    "3623",
    "3023b",
    "3024",
    "2431",
    "3069b",
    "3070b",
    "25269",
    "33291",
    "6141",
    "98138",
    "15279",
    "2445",
    "4477",
    "60479",
    "91988",
]

TILES = {4: "2431", 2: "3069b", 1: "3070b"}
GRASS = [
    -8.0,
    -63.993,
    -19.391,
    8.0,
    0.0,
    14.729,
]  # Plant Grass Stem's body, in LDU; its origin is its base


def _on_stud(m, part, color, body, cx, cy, z, rot=0):
    """Puts a plant part whose origin is its base on the stud whose center is (cx, cy) and whose base is at z."""
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), body))
    return m.add(part, color, at=(cx + lo[0], cy + lo[1], z + lo[2]), rot=rot)


def build(
    m,
    x,
    y,
    z,
    length=6,
    depth=4,
    seed=1,
    bed="Blue",
    water="Trans-Light Blue",
    ground="Green",
    stone="Light Bluish Gray",
    title="Pond",
):
    """The pond; returns the z of the rim's top."""
    if not 3 <= length <= 12 or not 3 <= depth <= 10:
        raise ValueError(
            f"pond takes a length of 3 to 12 and a depth of 3 to 10, not {length}, {depth}"
        )
    rng = random.Random(seed)
    with m.step(f"{title}, bed"):
        m.fill(Plate, bed, at=(x, y, z), size=(length + 2, depth + 2))
    rim = row_actions(m, Plate, ground, (x, y, z + 1), length + 2)
    rim += row_actions(m, Plate, ground, (x, y + depth + 1, z + 1), length + 2)
    rim += row_actions(m, Plate, ground, (x, y + 1, z + 1), depth, axis="y")
    rim += row_actions(
        m, Plate, ground, (x + length + 1, y + 1, z + 1), depth, axis="y"
    )
    in_steps(m, f"{title}, rim", rim)
    # Lily pads take a few studs inside the water's edge; the water's tiles fill around them.
    inner = [(i, j) for i in range(1, length - 1) for j in range(1, depth - 1)]
    pads = set(rng.sample(inner, min(len(inner), 2 + (length * depth >= 24))))
    corners = {
        (0, 0): 2,
        (length - 1, 0): 3,
        (length - 1, depth - 1): 0,
        (0, depth - 1): 1,
    }
    actions = []
    for j in range(depth):
        i = 0
        while i < length:
            at = (x + 1 + i, y + 1 + j, z + 1)
            if (i, j) in corners:
                rot = corners[(i, j)]
                actions.append(
                    (1, lambda at=at, rot=rot: m.add("25269", water, at=at, rot=rot))
                )
                i += 1
                continue
            if (i, j) in pads:
                actions.append(
                    (
                        1,
                        lambda at=at: m.add(
                            "33291",
                            "Bright Green",
                            at=(at[0] - 0.1, at[1] - 0.1, at[2]),
                        ),
                    )
                )
                i += 1
                continue
            n = 1
            while (
                n < 4
                and i + n < length
                and (i + n, j) not in pads
                and (i + n, j) not in corners
            ):
                n += 1
            n = max(s for s in TILES if s <= n)
            part = TILES[n]
            actions.append((1, lambda p=part, at=at: m.add(p, water, at=at, rot=1)))
            i += n
    in_steps(m, f"{title}, water", actions)
    ring = [(i, 0) for i in range(length + 2)] + [(0, j) for j in range(1, depth + 2)]
    ring += [(length + 1, j) for j in range(1, depth + 1)]
    reeds = [(length + 1, depth + 1), (length, depth + 1), (length + 1, depth)]
    stones = rng.sample([c for c in ring if c not in reeds], 4)
    actions = []
    for k, (i, j) in enumerate(stones):
        part = "6141" if k % 2 else "98138"
        at = (x + i, y + j, z + 2)
        actions.append((1, lambda p=part, at=at: m.add(p, stone, at=at)))
    for k, (i, j) in enumerate(reeds):
        cx, cy = x + i + 0.5, y + j + 0.5
        actions.append(
            (
                1,
                lambda cx=cx, cy=cy, k=k: _on_stud(
                    m, "15279", "Bright Green", GRASS, cx, cy, z + 2, rot=k
                ),
            )
        )
    in_steps(m, f"{title}, stones and reeds", actions)
    return z + 2


def example(m):
    """A pond 6 x 4 studs of water."""
    m.use("pond", x=0, y=0, z=0, length=6, depth=4)
