# SPDX-License-Identifier: Apache-2.0
"""Lawn with plant sockets: a raised lawn with square holes framed in corner plates, each taking a plant built apart on
its own 2 x 2 plug, so trees and bushes can be built separately, moved, or left out.

m.use("lawn_with_plant_sockets", x, y, z, length=12, depth=8, sockets=None, count=3, seed=1, plants=("bush", "tree"),
lawn="Bright Green", rim="Dark Green", title="Lawn") lays a lawn of plates `length` x `depth` studs from (x, y) on the
studs at height z (a baseplate is z = 0.5), a plate thick, with sockets in it. Each socket is a 4 x 4 frame of four
Plates 2 x 2 Corner (2420) in `rim`, turned so they leave a 2 x 2 hole in its middle. sockets: the front-left corners of
the frames, as (dx, dy) from (x, y); by default `count` of them at random on a 5-stud grid. Each plug is a sub-assembly
that drops into a hole, its top flush with the lawn: plants cycles through "bush" (a 2 x 2 plate under a bush, as
m.use("bush") builds it) and "tree" (a 2 x 2 jumper plate in the lawn's color, its middle stud taking a small tree, as
m.use("tree") builds it), or None for a socket left empty; plants=() leaves them all empty. Returns the middle of each
socket, a list of (x, y), at the lawn's top, z + 1.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10315. Not derived from any
model file."""

PARTS = [
    "2420",
    "3022",
    "87580",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3021",
    "3020",
    "3795",
    "3034",
]

# The corner plate's missing stud is its back left at rot=0; this turn puts it at each quarter of the frame's hole:
# (quarter's offset in the frame, rot).
FRAME = [((0, 0), 3), ((2, 0), 0), ((2, 2), 1), ((0, 2), 2)]
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


def _plug(m, kind, lawn):
    """A plug 2 x 2 that drops into a socket, with its plant."""
    if kind == "bush":
        with m.step("Plug"):
            m.add("3022", lawn, at=(0, 0, 0))
        m.use("bush", x=0, y=0, z=1, tiers=1, title="Bush")
    else:
        with m.step("Plug"):
            m.add("87580", lawn, at=(0, 0, 0))
        m.use("tree", x=0.5, y=0.5, z=1, size="small", title="Tree")


def build(
    m,
    x,
    y,
    z,
    length=12,
    depth=8,
    sockets=None,
    count=3,
    seed=1,
    plants=("bush", "tree"),
    lawn="Bright Green",
    rim="Dark Green",
    title="Lawn",
):
    """The lawn, its socket frames, and the plugs; returns each socket's middle."""
    if not 4 <= length <= 32 or not 4 <= depth <= 32:
        raise ValueError(
            f"lawn_with_plant_sockets takes a length and depth of 4 to 32, not {length} x {depth}"
        )
    rng = random.Random(seed)
    if sockets is None:
        spots = [
            (i, j) for i in range(0, length - 3, 5) for j in range(0, depth - 3, 5)
        ]
        sockets = sorted(rng.sample(spots, min(count, len(spots))))
    taken = set()
    for dx, dy in sockets:
        frame = {(x + dx + a, y + dy + b) for a in range(4) for b in range(4)}
        if not 0 <= dx <= length - 4 or not 0 <= dy <= depth - 4 or frame & taken:
            raise ValueError(
                f"lawn_with_plant_sockets: the socket at {(dx, dy)} is off the lawn or on another"
            )
        taken |= frame
    lawn_cells = {
        (i, j) for i in range(x, x + length) for j in range(y, y + depth)
    } - taken
    in_steps(
        m,
        f"{title}, socket frames",
        [
            (
                1,
                lambda at=(x + dx + ox, y + dy + oy, z), r=r: m.add(
                    "2420", rim, at=at, rot=r
                ),
            )
            for dx, dy in sockets
            for (ox, oy), r in FRAME
        ],
    )
    in_steps(
        m,
        f"{title}, lawn",
        [
            (1, lambda p=p, at=(i, j, z), r=r: m.add(p, lawn, at=at, rot=r))
            for p, (i, j), r in _fill(lawn_cells)
        ],
    )
    middles = [(x + dx + 2, y + dy + 2) for dx, dy in sockets]
    for k, (mx, my) in enumerate(middles):
        if not plants:
            break
        kind = plants[k % len(plants)]
        if kind is None:
            continue
        plug = m.submodel(f"{title} plug, {kind}", _plug, kind=kind, lawn=lawn)
        lo, hi = plug.box
        # The plug's plate is the submodel's 2 x 2 at its own (0, 0, 0); the plant's leaves reach past it.
        with m.step(f"{title}, plant {k + 1}"):
            m.place(plug, at=(mx - 1 + lo[0], my - 1 + lo[1], z + lo[2]))
    return middles


def example(m):
    """A lawn with three sockets, two planted, one left empty, on a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use(
        "lawn_with_plant_sockets",
        x=1,
        y=2,
        z=0.5,
        length=14,
        depth=11,
        sockets=[(1, 1), (9, 1), (5, 6)],
        plants=("tree", "bush", None),
    )
