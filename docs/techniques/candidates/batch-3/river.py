# SPDX-License-Identifier: Apache-2.0
"""River: a stream in a channel, its water clear tiles over a blue bed, a plate below the banks, whose grass overhangs
the water here and there.

m.use("river", x, y, z, length, width=2, seed=1, bed="Blue", water=("Trans-Light Blue", "Trans-Clear"),
mud="Dark Tan", grass="Green", title="River") builds a river running along x from x, `length` studs, on the studs at
height z: a bed of blue plates width + 2 studs deep (y to y + width + 2), tying the banks together under it; the water,
rows of 1-wide clear tiles over the channel (y + 1 to y + width + 1), their joints staggered row to row and mixed in
the two `water` colors so they read as ripples; and on each bank (the rows at y and y + width + 1) a plate of mud,
then grass plates that in some runs overhang the water by a stud and in others leave a smooth tile of mud where the
bank is worn. The water is a plate below the grass. seed picks the runs and colors. Returns the z of the grass's top.
"""

import random

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3460",
    "3666",
    "3710",
    "3623",
    "3023b",
    "3024",
    "3034",
    "3795",
    "3020",
    "3021",
    "3022",
    "2431",
    "3069b",
    "3070b",
    "2445",
    "3832",
    "4282",
    "4477",
    "60479",
    "91988",
]


def _runs(rng, length):
    """The bank's runs along x as [(start, n, kind)]: "grass", "over" (overhangs the water), or "worn"."""
    runs, k = [], 0
    while k < length:
        kind = rng.choice(("grass", "grass", "over", "worn"))
        n = min(
            length - k, rng.choice((2, 3)) if kind != "worn" else rng.choice((1, 2))
        )
        if kind == "over" and n < 2:
            kind = "grass"
        runs.append((k, n, kind))
        k += n
    return runs


def build(
    m,
    x,
    y,
    z,
    length,
    width=2,
    seed=1,
    bed="Blue",
    water=("Trans-Light Blue", "Trans-Clear"),
    mud="Dark Tan",
    grass="Green",
    title="River",
):
    """The river; returns the z of the grass's top."""
    if not 2 <= length <= 16 or not 2 <= width <= 6:
        raise ValueError(
            f"river takes a length of 2 to 16 and a width of 2 to 6, not {length}, {width}"
        )
    if not water:
        raise ValueError("river needs at least one water color")
    rng = random.Random(seed)
    with m.step(f"{title}, bed"):
        m.fill(Plate, bed, at=(x, y, z), size=(length, width + 2))
    actions = []
    for j in range(width):
        first = 2 if j % 2 else None
        for n_at, piece in _tiles(length, first):
            color = rng.choice(water)
            at = (x + n_at, y + 1 + j, z + 1)
            actions.append(
                (1, lambda p=piece, c=color, at=at: m.add(p, c, at=at, rot=1))
            )
    in_steps(m, f"{title}, water", actions)
    banks = []
    for yb in (y, y + width + 1):
        banks += row_actions(m, Plate, mud, (x, yb, z + 1), length)
    in_steps(m, f"{title}, mud", banks)
    for side, yb in (("near bank", y), ("far bank", y + width + 1)):
        actions = []
        for s, n, kind in _runs(rng, length):
            at = (x + s, yb, z + 2)
            if kind == "grass":
                actions += row_actions(m, Plate, grass, at, n)
            elif kind == "over":
                y0 = (
                    yb if yb == y else yb - 1
                )  # the near bank reaches back, the far one forward
                actions += row_actions(m, Plate, grass, (x + s, y0, z + 2), n, width=2)
            else:
                actions += row_actions(m, Tile, mud, at, n)
        in_steps(m, f"{title}, {side}", actions)
    return z + 3


def _tiles(length, first):
    """A row of clear tiles: [(offset, part)], 1 x 4s with a 1 x 2 to start every other row and 1 x 1s to fill."""
    out, k = [], 0
    sizes = {4: "2431", 2: "3069b", 1: "3070b"}
    if first:
        out.append((0, sizes[first]))
        k = first
    while k < length:
        n = next(s for s in (4, 2, 1) if k + s <= length)
        out.append((k, sizes[n]))
        k += n
    return out


def example(m):
    """A river 8 studs long, its channel 2 studs wide."""
    m.use("river", x=0, y=0, z=0, length=8, width=2)
