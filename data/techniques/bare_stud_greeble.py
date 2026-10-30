# SPDX-License-Identifier: Apache-2.0
"""Bare stud greeble: a hull surface tiled about 70 percent smooth, with small irregular patches of bare studs and a few
dark seams, so it reads as busy plating with no greebles bolted on.

m.use("bare_stud_greeble", x, y, z, length, depth, bare=0.3, seed=1, color="Light Bluish Gray", seam="Dark Tan",
seams=0.15, title="Plating") covers the studs at height z over length x depth studs from (x, y). It first picks bare
patches at random (rectangles of 2 to 8 studs, never touching each other, until about `bare` of the area is bare), then
tiles the rest, the largest tile that fits first (2 x 4, 2 x 3, 2 x 2, 1 x 4, 1 x 3, 1 x 2, 1 x 1), each turned either
way at random, so the seams never run in a grid. About `seams` of the narrow tiles are in `seam`, the panel lines. seed
picks the layout: the same seed gives the same surface. It lays no base: put it on plates (two layers, seams crossed,
for a big panel). Returns the z of the tiles' top.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10356. Not derived from any
model file."""

PARTS = ["87079", "26603", "3068b", "2431", "63864", "3069b", "3070b"]

# Tiles by size (across x, along y unturned), largest first.
TILES = [
    ((2, 4), "87079"),
    ((2, 3), "26603"),
    ((2, 2), "3068b"),
    ((1, 4), "2431"),
    ((1, 3), "63864"),
    ((1, 2), "3069b"),
    ((1, 1), "3070b"),
]
PATCHES = [
    (1, 2),
    (2, 1),
    (2, 2),
    (1, 3),
    (3, 1),
    (2, 3),
    (3, 2),
    (2, 4),
    (4, 2),
    (1, 4),
    (4, 1),
]


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    bare=0.3,
    seed=1,
    color="Light Bluish Gray",
    seam="Dark Tan",
    seams=0.15,
    title="Plating",
):
    """The bare patches and the tiles around them; returns the z of the tiles' top."""
    if length < 3 or depth < 3 or not 0 <= bare <= 0.5:
        raise ValueError(
            f"bare_stud_greeble takes an area of 3 x 3 or more and a bare share of 0 to 0.5, not {length} x {depth}, "
            f"{bare}"
        )
    rng = random.Random(seed)
    open_cells, near = set(), set()
    goal, tries = bare * length * depth, 0
    while len(open_cells) < goal and tries < 200:
        tries += 1
        w, d = rng.choice(PATCHES)
        i, j = rng.randrange(length - w + 1), rng.randrange(depth - d + 1)
        cells = {(i + a, j + b) for a in range(w) for b in range(d)}
        if cells & near:
            continue
        open_cells |= cells
        near |= {
            (a + p, b + q) for a, b in cells for p in (-1, 0, 1) for q in (-1, 0, 1)
        }
    covered = set(open_cells)
    actions = []
    for j in range(depth):
        for i in range(length):
            if (i, j) in covered:
                continue
            for (w, d), part in TILES:
                turns = [(w, d, 0), (d, w, 1)] if w != d else [(w, d, 0)]
                rng.shuffle(turns)
                fit = next(
                    (
                        (tw, td, t)
                        for tw, td, t in turns
                        if i + tw <= length
                        and j + td <= depth
                        and not {(i + a, j + b) for a in range(tw) for b in range(td)}
                        & covered
                    ),
                    None,
                )
                if fit:
                    tw, td, t = fit
                    covered |= {(i + a, j + b) for a in range(tw) for b in range(td)}
                    c = seam if w == 1 and rng.random() < seams else color
                    actions.append(
                        (
                            1,
                            lambda p=part, c=c, at=(x + i, y + j, z), t=t: m.add(
                                p, c, at=at, rot=t
                            ),
                        )
                    )
                    break
    in_steps(m, f"{title}, tiles", actions)
    return z + 1


def example(m):
    """A hull panel 10 x 8 studs on two layers of plates, its seams crossed."""
    with m.step("Panel, plates"):
        m.fill(Plate, "Light Bluish Gray", at=(0, 0, 0), size=(10, 8))  # strips along x
    with m.step("Panel, plates across"):
        m.grid("3034", "Light Bluish Gray", at=(0, 0, 1), nx=5)
    m.use("bare_stud_greeble", x=0, y=0, z=2, length=10, depth=8, seams=0.4)
