# SPDX-License-Identifier: Apache-2.0
"""Greeble strip: a band of small surface detail (grilles, round plates, tiles, a stud or two) on side studs, the busy
machine texture of a spaceship hull or a space-station wall.

m.use("greeble_strip", x, y, z, length, rows=1, facing="front", color="Dark Bluish Gray", detail="Light Bluish Gray",
seed=1, title="Greebles") builds a wall one stud thick and `length` studs long from (x, y), `rows` courses tall from
z, of bricks with studs on one side (1 x 4 30414, 1 x 2 11211, 1 x 1 87087), and puts a detail on each side stud: a
grille tile, a plain 1 x 2 tile, or a jumper plate (one stud showing) across two studs, or a 1 x 1 tile, round plate,
or round tile on one, chosen at random from `seed`, so the same seed gives the same strip. facing is the side the
details face, which also sets the wall's direction: "front" (-y) and "back" (+y) walls run along x, "left" (-x) and
"right" (+x) ones along y. The details stand out a plate (0.4 studs) from the wall. Every other course starts with a 1
x 2, so the joints don't line up. It brings its own steps, a course and then its details at a time, and returns the z
of the strip's top.
"""

import random

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["30414", "11211", "87087", "2412b", "3069b", "3070b", "6141", "98138", "3794b"]

SNOT = {4: "30414", 2: "11211", 1: "87087"}
# By facing: the way the wall runs, the quarter turn that points a side-stud brick's studs out, and the extra turn
# about z that takes a detail made for the front (-y) round to face that side.
FACING = {
    "front": ("x", 3, 0),
    "back": ("x", 1, 180),
    "left": ("y", 2, 270),
    "right": ("y", 0, 90),
}
OUT = 0.4  # a plate or tile on its back stands out this far, in studs
WIDE = ["2412b", "2412b", "3069b"]  # details across two studs, grilles twice as often
SMALL = ["3070b", "6141", "98138", "6141"]  # details on one stud
JUMPER = "3794b"  # a 1 x 2 plate with its one stud between the two it sits on


def _lengths(n, stagger=False):
    """Side-stud bricks for a course n long, longest first; a staggered course starts with a 1 x 2, so its joints
    fall between the ones below."""
    out = []
    if stagger and n > 2:
        out, n = [2], n - 2
    for size in (4, 2, 1):
        while n >= size:
            out.append(size)
            n -= size
    return out


def build(
    m,
    x,
    y,
    z,
    length,
    rows=1,
    facing="front",
    color="Dark Bluish Gray",
    detail="Light Bluish Gray",
    seed=1,
    title="Greebles",
):
    """The wall of side-stud bricks and its details; returns the z of the top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if length < 1 or rows < 1:
        raise ValueError(
            f"greeble_strip needs a length and rows of 1 or more, not {length}, {rows}"
        )
    axis, turn, spin = FACING[facing]
    # A tile or plate on its back, its long side along the wall.
    flat = Rot(y=-90, z=90 + spin)
    rng = random.Random(seed)

    def cell(a):
        return (x + a, y) if axis == "x" else (x, y + a)

    def face_at(a, zr):
        """The min corner of a detail starting `a` studs along the wall, on the course at zr."""
        cx, cy = cell(a)
        if facing == "front":
            cy -= OUT
        elif facing == "back":
            cy += 1
        elif facing == "left":
            cx -= OUT
        else:
            cx += 1
        return (cx, cy, zr + 0.5)

    for r in range(rows):
        zr = z + 3 * r
        with m.step(f"{title}, row {r + 1}: bricks"):
            a = 0
            for n in _lengths(length, stagger=r % 2 == 1):
                m.add(SNOT[n], color, at=(*cell(a), zr), rot=turn)
                a += n
        # Fill the row's studs left to right: a two-stud detail where two are free, else a one-stud one.
        details, a = [], 0
        while a < length:
            if a + 1 < length and rng.random() < 0.6:
                details.append((a, rng.choice(WIDE + [JUMPER])))
                a += 2
            else:
                details.append((a, rng.choice(SMALL)))
                a += 1
        for i in range(0, len(details), 8):
            part = f" ({i // 8 + 1})" if len(details) > 8 else ""
            with m.step(f"{title}, row {r + 1}: details{part}"):
                for a, part_id in details[i : i + 8]:
                    m.add(part_id, detail, at=face_at(a, zr), rot=flat)
    return z + 3 * rows


def example(m):
    """A strip 8 studs long and two courses tall, on a plate."""
    with m.step("Base"):
        m.add("3034", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("greeble_strip", x=0, y=0, z=1, length=8, rows=2)
