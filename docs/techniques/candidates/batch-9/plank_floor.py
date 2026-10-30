# SPDX-License-Identifier: Apache-2.0
"""Plank floor: a wooden floor of long tile boards in two wood tones, every row's ends staggered from the row before.

m.use("plank_floor", x, y, z, length, depth, axis="x", seed=0, color="Reddish Brown", color2="Medium Nougat", sizes=(2,
3, 4, 6), title="Floor") covers the rectangle whose front-left corner is (x, y), `length` studs along x and `depth`
along y, standing on the studs at height z (a floor of plates, a baseplate). The boards are 1-wide tiles running along
`axis`, each row a run of lengths from `sizes`, longest-leaning, picked from `seed` so that no board end lines up with
an end in the row before, and each board is `color` or color2 at random, so the floor reads as old boards rather than a
grid. It's tiles only, one plate thick; for a floor that also ties the plates under it, lay tiled_floor's plates first
or use tiled_floor with its plain pattern. It brings its own steps, "<title>, boards" (split into parts of up to 8, the
back rows first), and returns the z of the floor's top, z + 1.
"""

import random

from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350, set 43268. Not derived
from any model file."""

PARTS = ["3069b", "63864", "2431", "6636", "4162"]

BOARDS = {2: "3069b", 3: "63864", 4: "2431", 6: "6636", 8: "4162"}


def _row(rng, run, sizes, below):
    """Board lengths for one row of `run` studs whose inner ends miss the ends in `below`."""
    best = None
    for _ in range(200):
        out, at = [], 0
        while at < run:
            fits = [n for n in sizes if n <= run - at]
            if not fits:
                break
            # Lean to the long boards, as real floors do.
            n = rng.choice(fits + [max(fits)] * 2)
            out.append(n)
            at += n
        if at != run:
            continue
        ends, at = set(), 0
        for n in out[:-1]:
            at += n
            ends.add(at)
        if not ends & below:
            return out, ends
        best = best or (out, ends)
    if best is None:
        raise ValueError(
            f"plank_floor: no boards from sizes {sizes} make a row of {run}"
        )
    return best


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    axis="x",
    seed=0,
    color="Reddish Brown",
    color2="Medium Nougat",
    sizes=(2, 3, 4, 6),
    title="Floor",
):
    """The boards; returns the z of the floor's top."""
    if axis not in ("x", "y"):
        raise ValueError(f'axis must be "x" or "y", not {axis!r}')
    for n in sizes:
        if n not in BOARDS:
            raise ValueError(
                f"a board is {', '.join(map(str, BOARDS))} studs long, not {n}"
            )
    rng = random.Random(seed)
    run, rows = (length, depth) if axis == "x" else (depth, length)
    actions, below = [], set()
    for r in reversed(range(rows)):
        boards, below = _row(rng, run, sorted(sizes), below)
        at = 0
        for n in boards:
            tone = color if rng.random() < 0.5 else color2
            pos = (x + at, y + r, z) if axis == "x" else (x + r, y + at, z)
            rot = 1 if axis == "x" else 0
            actions.append(
                (
                    1,
                    lambda n=n, tone=tone, pos=pos, rot=rot: m.add(
                        BOARDS[n], tone, at=pos, rot=rot
                    ),
                )
            )
            at += n
    in_steps(m, f"{title}, boards", actions)
    return z + 1


def example(m):
    """A floor of boards 10 x 6 on a plate."""
    with m.step("Plate"):
        m.add("3033", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("plank_floor", x=0, y=0, z=1, length=10, depth=6, seed=3)
