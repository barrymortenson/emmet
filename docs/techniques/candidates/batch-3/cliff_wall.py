# SPDX-License-Identifier: Apache-2.0
"""Cliff wall: a natural cliff face of ordinary bricks, its courses stepping in and out by a stud, in mixed grays, with
a ragged top.

m.use("cliff_wall", x, y, z, length, rows=4, seed=1, colors=("Dark Bluish Gray", "Light Bluish Gray"), cap="Tan",
title="Cliff") builds a cliff `length` studs along x from (x, y), 2 studs deep (y to y + 2), `rows` bricks tall from z,
its face to the front (-y). Each course is runs of 2-wide bricks that stand out to y and 1-wide bricks set back a stud,
so the face ledges in and out; where it can, no joint sits on a joint in the course under it, and every set-back run
sits on the back half of what's below, so a ledge can overhang. The top course leaves out its set-back runs, so the
top is ragged, and a few 1 x 1 cheese slopes facing front cap it. seed picks the runs and the colors (one of `colors`
per run). Returns the z of the top (the highest course).
"""

import random

from emmet_dsl import Brick
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 21335 (cliff wall with offset courses). Not derived from any model file."""

PARTS = ["3003", "3002", "3001", "3005", "3004", "3622", "54200"]


def _course(rng, length, below):
    """[(start, n, out)] runs for one course, none ending where a run in `below` ends (but the wall's own ends)."""
    joints = {s + n for s, n, _ in below}
    for _ in range(50):
        runs, k = [], 0
        while k < length:
            out = rng.random() < 0.55
            n = min(length - k, rng.choice((2, 3, 4) if out else (1, 2, 3)))
            if n == 1 and out:
                out = False
            runs.append((k, n, out))
            k += n
        if not any(s + n in joints and s + n < length for s, n, _ in runs):
            return runs
    return runs


def build(
    m,
    x,
    y,
    z,
    length,
    rows=4,
    seed=1,
    colors=("Dark Bluish Gray", "Light Bluish Gray"),
    cap="Tan",
    title="Cliff",
):
    """The cliff; returns the z of the top."""
    if not 4 <= length <= 16 or not 2 <= rows <= 10:
        raise ValueError(
            f"cliff_wall takes a length of 4 to 16 and 2 to 10 rows, not {length}, {rows}"
        )
    if not colors:
        raise ValueError("cliff_wall needs at least one color")
    rng = random.Random(seed)
    below = []
    for r in range(rows):
        zr = z + 3 * r
        runs = _course(rng, length, below)
        top = r == rows - 1
        if top:
            # The top course keeps only its outstanding runs, so the skyline steps.
            runs = [run for run in runs if run[2]] or [runs[0]]
        actions = []
        for s, n, out in runs:
            at = (x + s, y if out else y + 1, zr)
            width = 2 if out else 1
            actions += row_actions(m, Brick, rng.choice(colors), at, n, width=width)
        in_steps(m, f"{title}, row {r + 1}", actions)
        below = runs
    caps = []
    for s, n, out in below:
        if out and n >= 3:
            at = (x + s + rng.randrange(n), y, zr + 3)
            caps.append((1, lambda at=at: m.add("54200", cap, at=at, rot=3)))
    in_steps(m, f"{title}, top", caps)
    return z + 3 * rows


def example(m):
    """A cliff 8 studs long and 4 bricks tall."""
    with m.step("Base"):
        m.add("3034", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("cliff_wall", x=0, y=0, z=1, length=8, rows=4)
