# SPDX-License-Identifier: Apache-2.0
"""Saucer petal panels: a round disc 12 studs across whose top is broken into tiled panels with radial seams and a few
bare patches, like the plating of a display saucer, around a small domed hub.

m.use("saucer_petal_panels", x, y, z, bare=0.25, seed=1, color="Light Bluish Gray", seam="Tan", seams=0.2, hub="Light
Bluish Gray", title="Saucer") lays, on the studs at height z, a disc whose min corner is (x, y): four 6 x 6 plates with
a 4 x 4 round corner (6003), each turned so its round corner faces out, so the four make a rounded disc, and a 4 x 4
round plate across their meeting corners that ties them, with a 2 x 2 round brick and a dish on it as the hub dome. Each
quarter is then panelled grid-friendly: about `bare` of its studs left bare in small patches, the rest in tiles whose
long sides run out from the middle (along x near the x axis, along y near the y axis), so the seams radiate; about
`seams` of the narrow tiles are in `seam`, the panel lines. Real petals tilt a few degrees to the rim on hinges; these
lie flat. seed picks the layout. Returns the z of the panels' top (the hub stands 5 plates above it).
"""

import random

from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10356. Not derived from any
model file."""

PARTS = [
    "6003",
    "60474",
    "3941",
    "4740",
    "87079",
    "3068b",
    "2431",
    "63864",
    "3069b",
    "3070b",
]

# The studs of a 6 x 6 plate with a 4 x 4 round corner at +x +y, less the 2 x 2 under the hub.
ROWS = {
    0: range(2, 6),
    1: range(2, 6),
    2: range(6),
    3: range(6),
    4: range(5),
    5: range(3),
}
CELLS = {(u, v) for v, us in ROWS.items() for u in us}
# Tiles by size (width, length): the length runs out from the middle.
TILES = [
    ((2, 4), "87079"),
    ((2, 2), "3068b"),
    ((1, 4), "2431"),
    ((1, 3), "63864"),
    ((1, 2), "3069b"),
    ((1, 1), "3070b"),
]
PATCHES = [(1, 2), (2, 1), (2, 2), (1, 3), (3, 1)]


def _quarter(rng, bare):
    """The tiles of one quarter in its own frame (round corner at +u +v): (u, v, width along u, depth along v,
    part, narrow)."""
    open_cells, goal = set(), bare * len(CELLS)
    for _ in range(40):
        if len(open_cells) >= goal:
            break
        w, d = rng.choice(PATCHES)
        cells = {
            (rng.randrange(6) + a, rng.randrange(6) + b)
            for a in range(w)
            for b in range(d)
        }
        cells = {(u, v) for u, v in cells}
        if cells <= CELLS and not cells & open_cells:
            open_cells |= cells
    covered, out = set(open_cells), []
    for v in range(6):
        for u in range(6):
            if (u, v) not in CELLS or (u, v) in covered:
                continue
            along_u = u >= v  # nearer the u axis: the long side runs along u
            for (w, l), part in TILES:
                tw, td = (l, w) if along_u else (w, l)
                cells = {(u + a, v + b) for a in range(tw) for b in range(td)}
                if cells <= CELLS and not cells & covered:
                    covered |= cells
                    out.append((u, v, tw, td, part, w == 1))
                    break
    return out


def build(
    m,
    x,
    y,
    z,
    bare=0.25,
    seed=1,
    color="Light Bluish Gray",
    seam="Tan",
    seams=0.2,
    hub="Light Bluish Gray",
    title="Saucer",
):
    """The four quarters, the hub that ties them, and the panels; returns the z of the panels' top."""
    if not 0 <= bare <= 0.5:
        raise ValueError(
            f"saucer_petal_panels takes a bare share of 0 to 0.5, not {bare}"
        )
    rng = random.Random(seed)
    cx, cy = x + 6, y + 6
    # A quarter turned k times about the middle: where a cell rectangle of its frame lands in the model.
    turn = [
        lambda u, v: (u, v),
        lambda u, v: (-v, u),
        lambda u, v: (-u, -v),
        lambda u, v: (v, -u),
    ]

    def place(k, u, v, w, d):
        xs, ys = zip(turn[k](u, v), turn[k](u + w, v + d))
        return cx + min(xs), cy + min(ys)

    with m.step(f"{title}, quarters"):
        for k in range(4):
            m.add("6003", color, at=(*place(k, 0, 0, 6, 6), z), rot=k)
    with m.step(f"{title}, hub"):
        m.add("60474", hub, at=(cx - 2, cy - 2, z + 1))
        m.add("3941", hub, at=(cx - 1, cy - 1, z + 2))
        m.add("4740", hub, at=(cx - 1, cy - 1, z + 5))
    for k in range(4):
        actions = []
        for u, v, w, d, part, narrow in _quarter(rng, bare):
            c = seam if narrow and rng.random() < seams else color
            # A tile's long side runs along y unturned: turn it when its long side runs along x in the model.
            rot = 1 if (w > d) != (k % 2 == 1) else 0
            at = (*place(k, u, v, w, d), z + 1)
            actions.append(
                (
                    1,
                    lambda part=part, c=c, at=at, rot=rot: m.add(
                        part, c, at=at, rot=rot
                    ),
                )
            )
        in_steps(m, f"{title}, panels {k + 1}", actions)
    return z + 2


def example(m):
    """A saucer on a short round pylon."""
    with m.step("Pylon"):
        m.add("3031", "Black", at=(4, 4, 0))
        m.stack("3941", "Dark Bluish Gray", at=(5, 5, 1), count=2)
    m.use("saucer_petal_panels", x=0, y=0, z=7)
