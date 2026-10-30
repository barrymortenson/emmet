# SPDX-License-Identifier: Apache-2.0
"""Bamboo grove: a clump of tall, segmented canes of round bricks with darker rings at the joints and sprays of narrow
leaves, four canes of different heights on one 2 x 2 base, the tallest at the back.

m.use("bamboo_grove", x, y, z, height=6, seed=1, cane="Bright Green", joint="Green", leaves="Lime", base="Green",
title="Bamboo") stands a clump on the studs at height z (a baseplate is z = 0.5) over the 2 x 2 from (x, y): a plate 2 x
2 in `base`, and on each of its studs a cane: round 1 x 1 bricks in `cane`, with a round 1 x 1 plate in `joint` between
every two as the cane's joint ring, and here and there, and at the top, a Plant Brick 1 x 1 Round with 3 Bamboo Leaves
(30176) in `leaves` in place of a brick, turned so its leaves reach away from the clump. The back canes are `height`
bricks of cane, the front ones a brick or two shorter. The leaves spread about 2.5 studs round the base. height: 3 to
10. seed picks the heights and where the leaves are. Returns the z of the tallest cane's top.
"""

import random

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10315. Not derived from any
model file."""

PARTS = ["3022", "3062b", "6141", "30176"]

LEAVES = [
    -10.0,
    0.0,
    -38.011,
    48.88,
    24.0,
    38.011,
]  # the leaf brick's body in LDU; its origin is the stud under it
# The base's four studs, from its front-left corner, the turn that points a leaf brick there outward (a quarter less
# points it out the other way), and whether the cane is a back one.
CANES = (((0, 1), 1, True), ((1, 1), 0, True), ((0, 0), 2, False), ((1, 0), 3, False))


def _leaf(m, color, sx, sy, z, rot):
    """A leaf brick on the stud whose corner is (sx, sy), its bottom at z."""
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), LEAVES))
    m.add("30176", color, at=(sx + 0.5 + lo[0], sy + 0.5 + lo[1], z), rot=rot)


def build(
    m,
    x,
    y,
    z,
    height=6,
    seed=1,
    cane="Bright Green",
    joint="Green",
    leaves="Lime",
    base="Green",
    title="Bamboo",
):
    """The base and the canes; returns the z of the tallest cane's top."""
    if not 3 <= height <= 10:
        raise ValueError(f"bamboo_grove takes a height of 3 to 10 bricks, not {height}")
    rng = random.Random(seed)
    with m.step(f"{title}, base"):
        m.add("3022", base, at=(x, y, z))
    top = z + 1
    for k, ((sx, sy), rot, back) in enumerate(CANES):
        n = height - (0 if back else rng.randint(1, 2)) - (k % 2 if back else 0)
        sprays = set(rng.sample(range(2, n - 1), min(2, max(0, n - 3)))) | {n - 1}
        zc, actions = z + 1, []
        for i in range(n):
            if i in sprays:
                actions.append(
                    (
                        1,
                        lambda zc=zc, sx=sx, sy=sy, r=rot if i % 2 else (rot - 1) % 4: (
                            _leaf(m, leaves, x + sx, y + sy, zc, r)
                        ),
                    )
                )
            else:
                actions.append(
                    (
                        1,
                        lambda zc=zc, sx=sx, sy=sy: m.add(
                            "3062b", cane, at=(x + sx, y + sy, zc)
                        ),
                    )
                )
            zc += 3
            if i % 2 == 1 and i < n - 1:
                actions.append(
                    (
                        1,
                        lambda zc=zc, sx=sx, sy=sy: m.add(
                            "6141", joint, at=(x + sx, y + sy, zc)
                        ),
                    )
                )
                zc += 1
        top = max(top, zc)
        name = ("back" if back else "front") + (" left" if sx == 0 else " right")
        for s in range(0, len(actions), 6):
            with m.step(
                f"{title}, {name} cane"
                + (f" ({s // 6 + 1})" if len(actions) > 6 else "")
            ):
                for _, add in actions[s : s + 6]:
                    add()
    return top


def example(m):
    """A clump of bamboo six bricks tall on a baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("bamboo_grove", x=7, y=7, z=0.5, height=6, seed=2)
