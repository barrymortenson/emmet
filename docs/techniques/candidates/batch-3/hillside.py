# SPDX-License-Identifier: Apache-2.0
"""Hillside: ground that rises toward the back in terraces two plates high, each terrace's front softened by cheese
slopes, so the hill reads as a smooth bank rather than steps.

m.use("hillside", x, y, z, length, steps=3, tread=2, seed=1, color="Green", earth="Dark Tan", title="Hill") builds,
on the studs at height z, a hill `length` studs along x from x, rising toward +y in `steps` terraces, each 2 plates
higher than the one before and `tread` studs deep, with tread studs of flat top behind the last: steps x tread + tread
studs from y in all. Each terrace is two layers of plates, the first in strips along x and the second along y, so the
seams cross and the layers lock. Along its front edge, on the studs of the terrace below, stand 1 x 2 and 1 x 1 cheese
slopes facing front (2 plates tall, the terrace's height), most in `color` and a few in `earth` where the bank shows
bare. seed picks the cheese slopes. Returns the z of the top terrace.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "85984",
    "54200",
    "3034",
    "3832",
    "3795",
    "3020",
    "3021",
    "3022",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3024",
    "2445",
    "4282",
    "4477",
    "60479",
    "91988",
]


def build(
    m,
    x,
    y,
    z,
    length,
    steps=3,
    tread=2,
    seed=1,
    color="Green",
    earth="Dark Tan",
    title="Hill",
):
    """The terraces and their banks; returns the z of the top terrace."""
    if not 2 <= length <= 16 or not 1 <= steps <= 5 or not 2 <= tread <= 4:
        raise ValueError(
            f"hillside takes a length of 2 to 16, 1 to 5 steps, and a tread of 2 to 4, not {length}, {steps}, {tread}"
        )
    rng = random.Random(seed)
    depth = (steps + 1) * tread
    for k in range(steps):
        zk = z + 2 * k
        front = y + k * tread  # the bank's row; the terrace starts a stud behind it
        rows = depth - k * tread - 1
        bank = []
        i = 0
        while i < length:
            c = earth if rng.random() < 0.2 else color
            at = (x + i, front, zk)
            if i + 2 <= length and rng.random() < 0.7:
                bank.append((1, lambda c=c, at=at: m.add("85984", c, at=at, rot=3)))
                i += 2
            else:
                bank.append((1, lambda c=c, at=at: m.add("54200", c, at=at, rot=3)))
                i += 1
        in_steps(m, f"{title}, terrace {k + 1}: bank", bank)
        lower = []
        for j in range(0, rows, 2):
            w = min(2, rows - j)
            lower += row_actions(
                m, Plate, color, (x, front + 1 + j, zk), length, width=w
            )
        in_steps(m, f"{title}, terrace {k + 1}: first layer", lower)
        upper = []
        for i in range(0, length, 2):
            w = min(2, length - i)
            upper += row_actions(
                m, Plate, color, (x + i, front + 1, zk + 1), rows, axis="y", width=w
            )
        in_steps(m, f"{title}, terrace {k + 1}: second layer", upper)
    return z + 2 * steps


def example(m):
    """A hill 4 studs long rising in three terraces."""
    with m.step("Ground"):
        m.add("3035", "Green", at=(0, 0, 0))
    m.use("hillside", x=0, y=0, z=1, length=4, steps=3, tread=2)
