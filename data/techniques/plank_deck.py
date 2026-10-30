# SPDX-License-Identifier: Apache-2.0
"""Plank deck: a wooden deck or floor of long boards with staggered butt joints, laid over plates on cross beams.

m.use("plank_deck", x, y, z, length=12, width=4, beam_every=4, seed=1, bare=0.05, planks=("Reddish Brown",), beam="Dark
Brown", title="Deck") builds, on the studs at z, a deck `length` studs along x (the boards' direction) and `width`
across (y) from (x, y): beams, a brick 1 x `width` across the deck every `beam_every` studs and at the far end, in
`beam`; on them a course of 1-wide plates along x in the first planks color, each row started with a different length so
their joints miss; and on those the boards, tiles 1 x 1 to 1 x 8 along x, each row started with another length (seed
picks it, never the row beside's), so no two butt joints line up, as on a laid deck. Some 1 x 1 boards (about `bare` of
the studs, where seed says) are left out, bare studs for fittings: a capstan, a barrel, a figure. With several planks
colors, seed mixes them board by board. length: 4 to 16; width: 2 to 8. Returns the z of the deck's top.
"""

import random

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10335, set 21363. Not derived
from any model file."""

PARTS = [
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3070b",
    "3069b",
    "63864",
    "2431",
    "6636",
    "4162",
]

BOARDS = {
    8: "4162",
    6: "6636",
    4: "2431",
    3: "63864",
    2: "3069b",
    1: "3070b",
}  # Tile 1 x n


def _boards(rng, run):
    """A run of `run` studs as board lengths, longest first, from the boards there are."""
    out = []
    for n in sorted(BOARDS, reverse=True):
        while run >= n:
            out.append(n)
            run -= n
    rng.shuffle(out)
    return out


def build(
    m,
    x,
    y,
    z,
    length=12,
    width=4,
    beam_every=4,
    seed=1,
    bare=0.05,
    planks=("Reddish Brown",),
    beam="Dark Brown",
    title="Deck",
):
    """The beams, the plates, and the boards; returns the z of the top."""
    if not (4 <= length <= 16 and 2 <= width <= 8 and 2 <= beam_every <= length - 1):
        raise ValueError(
            f"plank_deck takes a length of 4 to 16, a width of 2 to 8 and beams 2 to length - 1 apart, "
            f"not {length}, {width}, {beam_every}"
        )
    rng = random.Random(seed)
    beams = sorted(set(range(0, length - 1, beam_every)) | {length - 1})
    for k in range(0, len(beams), 4):
        with m.step(f"{title}, beams {k // 4 + 1}"):
            for bx in beams[k : k + 4]:
                m.row(Brick, beam, at=(x + bx, y, z), length=width, axis="y")
    zp = z + 3
    for j in range(0, width, 2):
        with m.step(f"{title}, plates, rows {j + 1} to {min(j + 2, width)}"):
            for r in range(j, min(j + 2, width)):
                m.row(
                    Plate,
                    planks[0],
                    at=(x, y + r, zp),
                    length=length,
                    first=2 if r % 2 else None,
                )
    zt = zp + 1
    last = None
    rows = []
    for r in range(width):
        # The row's first board: any length but the row beside's, so no joint lines up with one beside it.
        first = rng.choice([n for n in (1, 2, 3, 4, 6) if n != last and n < length])
        last = first
        lengths = [first] + _boards(rng, length - first)
        rows.append(lengths)
    for j in range(0, width, 2):
        with m.step(f"{title}, boards, rows {j + 1} to {min(j + 2, width)}"):
            for r in range(j, min(j + 2, width)):
                at = 0
                for n in rows[r]:
                    if n == 1 and rng.random() < bare * 4:
                        at += 1  # a bare stud
                        continue
                    m.add(BOARDS[n], rng.choice(planks), at=(x + at, y + r, zt), rot=1)
                    at += n
    return zt + 1


def example(m):
    """A deck 12 x 4 studs on three beams."""
    m.use(
        "plank_deck",
        x=0,
        y=0,
        z=0,
        planks=("Reddish Brown", "Reddish Brown", "Dark Tan"),
    )
