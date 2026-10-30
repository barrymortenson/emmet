# SPDX-License-Identifier: Apache-2.0
"""Lily and reeds: life on a pond without covering it, lily pads with a flower and clumps of reeds, each on a single
open stud among the water's tiles.

m.use("lily_and_reeds", spots, z, seed=1, reeds=0.35, pad="Bright Green", flowers=("Dark Pink", "Medium Lavender",
"White"), reed="Olive Green", title="Lilies") stands one plant on each stud in `spots`, a list of (x, y) studs whose
tops are at height z (the open studs a tiled pond leaves, such as tiled_pond_water's pads, or round plates on a pond's
bed). Each is either a lily, a round 1 x 1 plate with tabs (33291) as the pad and a 1 x 1 plate with 5 petals (24866) in
one of `flowers` on it, or, for about `reeds` of them, a reed clump: a plant grass stem (15279) standing on the stud.
Keep the spots 3 to 6 studs apart, as a real pond's are, and a stud clear of walls for the pads' tabs. seed picks which
are reeds and the flowers' colors. Returns the z of the lilies' top.
"""

import random

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 10315 and 31218. Not derived
from any model file."""

PARTS = ["33291", "24866", "15279"]

# Each part's body box in LDU, about its origin, which is the middle of the stud it goes on.
BODIES = {
    "33291": [-12.0, 0.0, -12.0, 12.0, 8.0, 12.0],
    "24866": [-9.926, 0.0, -9.5, 9.926, 8.0, 10.043],
    "15279": [-8.0, -63.993, -19.391, 8.0, 0.0, 14.729],
}


def _on_stud(m, part, color, x, y, z, rot=0):
    """Puts a part centered on the stud at (x, y), its bottom at z."""
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), BODIES[part]))
    return m.add(part, color, at=(x + 0.5 + lo[0], y + 0.5 + lo[1], z), rot=rot)


def build(
    m,
    spots,
    z,
    seed=1,
    reeds=0.35,
    pad="Bright Green",
    flowers=("Dark Pink", "Medium Lavender", "White"),
    reed="Olive Green",
    title="Lilies",
):
    """A lily or a reed clump on each spot; returns the z of the lilies' top."""
    if not spots:
        raise ValueError("lily_and_reeds needs at least one stud in spots")
    rng = random.Random(seed)
    plants = [(sx, sy, rng.random() < reeds) for sx, sy in spots]
    lilies = [(sx, sy) for sx, sy, r in plants if not r]
    if lilies:
        with m.step(f"{title}, pads"):
            for sx, sy in lilies:
                _on_stud(m, "33291", pad, sx, sy, z)
        with m.step(f"{title}, flowers"):
            for sx, sy in lilies:
                _on_stud(
                    m,
                    "24866",
                    rng.choice(flowers),
                    sx,
                    sy,
                    z + 1,
                    rot=rng.randint(0, 3),
                )
    clumps = [(sx, sy) for sx, sy, r in plants if r]
    if clumps:
        with m.step(f"{title}, reeds"):
            for k, (sx, sy) in enumerate(clumps):
                _on_stud(m, "15279", reed, sx, sy, z, rot=rng.randint(0, 3))
    return z + 2


def example(m):
    """A small tiled pond with four open studs, a plant on each."""
    with m.step("Bed"):
        m.add("3033", "Blue", at=(0, 0, 0), rot=1)
    spots = [(1, 1), (5, 4), (8, 1), (3, 4)]
    with m.step("Open studs"):
        for sx, sy in spots:
            m.add("6141", "Blue", at=(sx, sy, 1))
    for j in range(6):
        with m.step(f"Water, row {j + 1}"):
            i = 0
            for cut in sorted([sx for sx, sy in spots if sy == j]) + [10]:
                while i < cut:
                    n = min(4 if (i + j) % 2 == 0 else 2, cut - i)
                    n = 4 if n >= 4 else 2 if n >= 2 else 1
                    m.add(
                        {4: "2431", 2: "3069b", 1: "3070b"}[n],
                        "Trans-Light Blue",
                        at=(i, j, 1),
                        rot=1,
                    )
                    i += n
                i = cut + 1
    m.use("lily_and_reeds", spots=spots, z=2, seed=3)
