# SPDX-License-Identifier: Apache-2.0
"""Two-tone masonry: an embossed brick wall with plain bricks in a neighboring tone mixed in and the odd brick
standing proud, so it reads as real, uneven brickwork.

m.use("two_tone_masonry", x, y, z, length, rows=4, color="Medium Nougat", tone="Nougat", plain=5, proud=2, seed=1,
title="Brick wall") lays `rows` courses along x from (x, y, z), facing front (-y), mostly in 1 x 4 and 1 x 2 embossed
bricks (15533, 98283), with about one plain brick (1 x 1 to 1 x 3) in every `plain`, in the wall color or the tone.
`proud` times a brick with two studs on one side (11211) carries a 1 x 2 tile in the tone, a brick sticking out a
plate. Pieces are picked by seed, and each course's joints miss the ones below. Returns the z of the top.
"""

import random

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302 and set 10350. Not
derived from any model file."""

PARTS = ["15533", "98283", "3005", "3004", "3622", "11211", "3069b"]

MASONRY = {4: "15533", 2: "98283"}
PLAIN = {1: "3005", 2: "3004", 3: "3622"}
FACE = Rot(y=-90, z=90)  # a 1 x 2 tile on its back, its face to -y, long along x


def _course(rng, length, below, proud, plain):
    """The pieces of one course, as (kind, size, start): kind "m" masonry, "p" plain, "s" a proud brick."""
    out, at = [], 0
    while at < length:
        if at in proud and length - at >= 2:
            out.append(("s", 2, at))
            at += 2
            continue
        stop = min([c for c in proud if c > at] + [length])
        cap = stop - at
        kind = (
            "p"
            if rng.random() < 1 / plain or cap == 1 or cap == 3 and rng.random() < 0.5
            else "m"
        )
        sizes = [n for n in ((1, 2, 3) if kind == "p" else (4, 2)) if n <= cap]
        if not sizes:
            kind, sizes = "p", [n for n in (1, 2, 3) if n <= cap]
        # Joints over joints only where nothing else fits.
        good = [n for n in sizes if at + n not in below or at + n == length] or sizes
        n = rng.choice(good) if kind == "p" else max(good)
        out.append((kind, n, at))
        at += n
    return out


def build(
    m,
    x,
    y,
    z,
    length,
    rows=4,
    color="Medium Nougat",
    tone="Nougat",
    plain=5,
    proud=2,
    seed=1,
    title="Brick wall",
):
    """The courses; returns the z of the top."""
    if length < 4 or rows < 1 or plain < 2:
        raise ValueError(
            "two_tone_masonry needs a length of 4 or more, 1 row or more, and plain of 2 or more, "
            f"not {length}, {rows}, {plain}"
        )
    rng = random.Random(seed)
    stick = {}
    for r in rng.sample(range(rows), min(proud, rows)):
        stick[r] = {rng.randrange(1, length - 2)}
    below = set()
    for r in range(rows):
        zr = z + 3 * r
        pieces = _course(rng, length, below, stick.get(r, set()), plain)
        with m.step(f"{title}, row {r + 1}"):
            for kind, n, c in pieces:
                if kind == "m":
                    m.add(MASONRY[n], color, at=(x + c, y, zr), rot=1)
                elif kind == "p":
                    m.add(PLAIN[n], rng.choice([color, tone]), at=(x + c, y, zr), rot=1)
                else:
                    m.add("11211", color, at=(x + c, y, zr), rot=3)
                    m.add("3069b", tone, at=(x + c, y - 0.4, zr + 0.5), rot=FACE)
        below = {c + n for _, n, c in pieces}
    return z + 3 * rows


def example(m):
    """A brick wall 10 studs long and five bricks tall, on a plate."""
    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=10)
    m.use("two_tone_masonry", x=0, y=0, z=1, length=10, rows=5)
