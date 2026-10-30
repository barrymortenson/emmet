# SPDX-License-Identifier: Apache-2.0
"""Rough stone ledge: a heavy stone ledge of plates in two close tones, its top course set half a stud over on jumper
plates and stopping short here and there, so its edge is ragged like cut stone.

m.use("rough_stone_ledge", x, y, z, length, tones=("Nougat", "Medium Nougat"), seed=1, title="Ledge") lays a ledge one
stud deep and a brick tall along x from (x, y, z), on the front row of a floor (facing front, -y): a course of 1 x n
plates, a course of 1 x 2 jumper plates (15573), then a course of plates on the jumpers' center studs, half a stud
over from the courses below, with the odd stud left bare between them. Each plate takes one of the tones at random
(seed). Build it on a floor's edge where a story overhangs or steps back, the next wall standing behind it. length
is even, 4 or more. Returns the z of the top.
"""

import random

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["3024", "3023b", "3623", "3710", "15573"]

PLATES = {1: "3024", 2: "3023b", 3: "3623", 4: "3710"}  # Plate 1 x n


def build(m, x, y, z, length, tones=("Nougat", "Medium Nougat"), seed=1, title="Ledge"):
    """The plates, the jumpers, and the offset course; returns the z of the top."""
    if length < 4 or length % 2:
        raise ValueError(
            f"rough_stone_ledge needs an even length of 4 or more, not {length}"
        )
    rng = random.Random(seed)
    with m.step(f"{title}: plates"):
        at = x
        while at < x + length:
            n = min(rng.choice((2, 3, 4)), x + length - at)
            m.add(PLATES[n], rng.choice(tones), at=(at, y, z), rot=1)
            at += n
    with m.step(f"{title}: jumpers"):
        for j in range(length // 2):
            m.add("15573", rng.choice(tones), at=(x + 2 * j, y, z + 1), rot=1)
    with m.step(f"{title}: offset course"):
        # Plates half a stud over, each over at least one jumper's stud (at x + 1, x + 3, ...), a stud bare now and
        # then; the course stops half a stud short of each end.
        at, end = x + 0.5 + rng.choice((0, 0, 1)), x + length - 0.5
        while end - at >= 2:
            n = min(rng.choice((2, 3, 4)), int(end - at))
            m.add(PLATES[n], rng.choice(tones), at=(at, y, z + 2), rot=1)
            at += n + (1 if rng.random() < 0.3 else 0)
    return z + 3


def example(m):
    """A ledge along the front of a floor, with the wall above set back behind it."""
    from emmet_dsl import Brick, Plate

    with m.step("Floor"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=10, width=2, first=4)
    m.use("rough_stone_ledge", x=0, y=0, z=1, length=10)
    for r in range(2):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Sand Green",
                at=(0, 1, 1 + 3 * r),
                length=10,
                first=2 if r else 4,
            )
