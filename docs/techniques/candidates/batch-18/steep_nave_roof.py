# SPDX-License-Identifier: Apache-2.0
"""Steep nave roof: a long steep roof of 45 degree slopes built apart as a submodel, its seams staggered course to
course, with double slopes along the ridge, set on the walls so it lifts off to show the inside.

m.use("steep_nave_roof", x, y, z, length=8, width=6, color="Dark Bluish Gray", gable="Tan", frame="Dark Tan",
title="Roof") builds a roof `width` studs across (even, 4 to 10) and `length` long along x, and sets it with its min
corner at (x, y, z), on the studs of walls whose tops are at z. The roof is a frame of plates `width` x `length`, then
courses of slopes 2 deep (2 x 4, 2 x 3, 2 x 2, 2 x 1) facing front and back, each course a stud further in than the one
below, the bricks between them in `gable` so the ends read as gable walls, and a ridge of double slopes on top. Every
other course starts with a shorter piece, so no seam sits on a seam. It is a submodel with its own steps, held by its
frame on the wall tops only, so it lifts straight off. Returns the placed copy's Handle; its .max[2] is the ridge.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file."""

PARTS = [
    "3037",
    "3038",
    "3039",
    "3040b",
    "3041",
    "3042",
    "3043",
    "3044b",
    "3001",
    "3002",
    "3003",
    "3004",
    "3034",
]

SLOPE = {4: "3037", 3: "3038", 2: "3039", 1: "3040b"}
RIDGE = {4: "3041", 3: "3042", 2: "3043", 1: "3044b"}


def _pieces(length, first):
    """Piece lengths for a run, starting with `first` (when it's shorter than the run), longest after."""
    out, n = [], length
    if first and first < n:
        out, n = [first], n - first
    for size in (4, 3, 2, 1):
        while n >= size:
            out.append(size)
            n -= size
    return out


def _roof(m, length, width, color, gable, frame, title):
    """The roof in its own frame: the plates at z = 0, the ridge along x at y = width / 2."""
    with m.step(f"{title}, frame"):
        m.fill(Plate, frame, at=(0, 0, 0), size=(length, width))
    courses = width // 2 - 1
    for k in range(courses):
        zk = 1 + 3 * k
        first = 2 if k % 2 else None
        with m.step(f"{title}, course {k + 1}: front and back"):
            for yy, rot in ((k, 3), (width - 2 - k, 1)):
                a = 0
                for n in _pieces(length, first):
                    m.add(SLOPE[n], color, at=(a, yy, zk), rot=rot)
                    a += n
        fill = width - 4 - 2 * k
        if fill > 0:
            with m.step(f"{title}, course {k + 1}: between"):
                for j in range(0, fill, 2):
                    a = 0
                    for n in _pieces(length, first):
                        # A brick's long side runs along y unturned.
                        part, rot = (Brick(2, n), 1) if n > 1 else (Brick(1, 2), 0)
                        m.add(part, gable, at=(a, k + 2 + j, zk), rot=rot)
                        a += n
    with m.step(f"{title}, ridge"):
        a = 0
        for n in _pieces(length, 2 if courses % 2 else None):
            m.add(RIDGE[n], color, at=(a, width // 2 - 1, 1 + 3 * courses), rot=1)
            a += n


def build(
    m,
    x,
    y,
    z,
    length=8,
    width=6,
    color="Dark Bluish Gray",
    gable="Tan",
    frame="Dark Tan",
    title="Roof",
):
    """The roof, built apart and set on; returns the placed copy's Handle."""
    if width % 2 or not 4 <= width <= 10 or not 2 <= length <= 24:
        raise ValueError(
            f"steep_nave_roof takes an even width of 4 to 10 and a length of 2 to 24, not {width}, {length}"
        )
    roof = m.submodel(
        "nave_roof",
        _roof,
        length=length,
        width=width,
        color=color,
        gable=gable,
        frame=frame,
        title=title,
    )
    with m.step(title):
        return m.place(roof, at=(x, y, z))


def example(m):
    """A nave 8 x 6 studs, two bricks tall, with its roof set on."""
    with m.step("Ground"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(8, 6))
    for r in range(2):
        with m.step(f"Walls, row {r + 1}"):
            m.wall(0, 0, 1 + 3 * r, 8, depth=6, color="Tan")
    m.use("steep_nave_roof", x=0, y=0, z=7, length=8, width=6)
