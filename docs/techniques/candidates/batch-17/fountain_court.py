# SPDX-License-Identifier: Apache-2.0
"""Fountain court: a small walled-city courtyard at mini scale, a round fountain at its heart, low hedges, a tapering
stone path from the gate, and a white tree.

m.use("fountain_court", x, y, z, tree=True, lawn="Bright Green", hedge="Green", stone="White", path="Tan",
title="Court") lays, on the studs at z, a court 10 x 10 studs from (x, y): a lawn of plates; a gate of two round bricks
1 x 1 with cones at the front edge; a path of two wedge plates 2 x 4 (41770a, 41769a) side by side, 4 studs wide at the
gate and narrowing to 2 at the fountain, which reads as a long avenue at this scale; the fountain, 4 x 4 in the middle,
two round plates 4 x 4 with a 2 x 2 hole (11833) as its rim with round 1 x 1 tiles in Trans-Light Blue as the water
inside, a plate below the rim's top; a block of round 1 x 1 plates in `hedge` each side of it as clipped hedges; and
with tree, a white tree at the back right, a trunk of round bricks with layers of leaves (2423) spiraling up it. Returns
the z of the lawn's top.
"""

from emmet_dsl import Plate, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = [
    "3958",
    "3832",
    "3034",
    "3062b",
    "4589",
    "41769a",
    "41770a",
    "11833",
    "98138",
    "6141",
    "2423",
]

LEAVES = [
    -26.0,
    0.0,
    -68.0,
    26.0,
    8.0,
    10.0,
]  # Plant Leaves 4 x 3's body (LDU); its origin is the stud it hangs on


def _leaves(m, color, cx, cy, z, turn):
    """A layer of leaves on the stud whose center is (cx, cy), its base at z."""
    r = frame.to_ldraw_rotation(frame.quarter_turns(turn))
    lo, _ = frame.box_in_dsl(frame.transform_box(r, (0, 0, 0), LEAVES))
    m.add("2423", color, at=(cx + lo[0], cy + lo[1], z + 1 + lo[2]), rot=turn)


def build(
    m,
    x,
    y,
    z,
    tree=True,
    lawn="Bright Green",
    hedge="Green",
    stone="White",
    path="Tan",
    title="Court",
):
    """The lawn, the gate and path, the fountain, the hedges, and the tree; returns the z of the lawn's top."""
    with m.step(f"{title}, lawn"):
        m.fill(Plate, lawn, at=(x, y, z), size=(10, 10))
    zl = z + 1
    with m.step(f"{title}, gate and path"):
        for gx in (x + 2, x + 7):
            m.add("3062b", stone, at=(gx, y, zl))
            m.add("4589", stone, at=(gx, y, zl + 3))
        # Square ends at the gate, tips at the fountain, each wedge's full edge on the path's middle line.
        m.add("41770a", path, at=(x + 3, y, zl), rot=1)
        m.add("41769a", path, at=(x + 5, y, zl), rot=1)
    with m.step(f"{title}, fountain"):
        m.add("11833", stone, at=(x + 3, y + 4, zl))
        for i in (0, 1):
            for j in (0, 1):
                m.add("98138", "Trans-Light Blue", at=(x + 4 + i, y + 5 + j, zl))
        m.add("11833", stone, at=(x + 3, y + 4, zl + 1))
    for side, hx in (("left", x), ("right", x + 8)):
        with m.step(f"{title}, {side} hedge"):
            for i in (0, 1):
                for j in range(4):
                    m.add("6141", hedge, at=(hx + i, y + 4 + j, zl))
    if tree:
        with m.step(f"{title}, tree"):
            m.stack("3062b", stone, at=(x + 8, y + 8, zl), count=2)
            for k in range(3):
                _leaves(m, stone, x + 8.5, y + 8.5, zl + 6 + k, k)
    return zl


def example(m):
    """The court on its own."""
    m.use("fountain_court", x=0, y=0, z=0)
