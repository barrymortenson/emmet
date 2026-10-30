# SPDX-License-Identifier: Apache-2.0
"""Wainscot split wall: a wall with its lower courses in brickwork and its upper courses in log siding or plain
color, split by a trim band of plates in an accent color.

m.use("wainscot_split_wall", x, y, z, length, rows=5, low=2, color="Medium Nougat", trim="Dark Red",
upper="Olive Green", siding="log", title="Wall") lays a wall along x from (x, y, z), facing front (-y): `low` courses
of 1 x 4 and 1 x 2 embossed bricks (15533, 98283) in bond, a course of 1 x n plates in the trim color, then the rest of
the `rows` courses in the upper color, as 1 x 2 log bricks (30136) with a 1 x 1 brick at each end of every other
course (siding="log") or as plain bricks (siding="plain"). Window sills set on the trim band line up with it. length
is even, 2 or more. Returns the z of the top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387. Not derived from any
model file."""

PARTS = [
    "15533",
    "98283",
    "30136",
    "3005",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
]

MASONRY = {4: "15533", 2: "98283"}


def build(
    m,
    x,
    y,
    z,
    length,
    rows=5,
    low=2,
    color="Medium Nougat",
    trim="Dark Red",
    upper="Olive Green",
    siding="log",
    title="Wall",
):
    """The brickwork, the band, and the upper wall; returns the z of the top."""
    if length < 2 or length % 2 or not 1 <= low < rows:
        raise ValueError(
            f"wainscot_split_wall needs an even length of 2 or more and 1 <= low < rows, not {length}, {low}, {rows}"
        )
    if siding not in ("log", "plain"):
        raise ValueError(f"siding={siding!r}: use log or plain")
    zr = z
    for r in range(low):
        with m.step(f"{title}, row {r + 1}: brickwork"):
            # Every other course starts with a 1 x 2, so its joints miss the course below.
            sizes = [2] if r % 2 and length > 2 else []
            left = length - sum(sizes)
            sizes += [4] * (left // 4) + ([2] if left % 4 else [])
            at = x
            for n in sizes:
                m.add(MASONRY[n], color, at=(at, y, zr), rot=1)
                at += n
        zr += 3
    with m.step(f"{title}: trim band"):
        m.row(
            Plate, trim, at=(x, y, zr), length=length, first=3 if length > 4 else None
        )
    zr += 1
    for r in range(low, rows):
        with m.step(f"{title}, row {r + 1}: siding"):
            odd = (r - low) % 2
            if siding == "plain":
                m.row(
                    Brick, upper, at=(x, y, zr), length=length, first=1 if odd else None
                )
            else:
                at = x
                if odd:
                    m.add("3005", upper, at=(x, y, zr))
                    at += 1
                while at + 2 <= x + length:
                    m.add("30136", upper, at=(at, y, zr), rot=1)
                    at += 2
                if at < x + length:
                    m.add("3005", upper, at=(at, y, zr))
        zr += 3
    return zr


def example(m):
    """A wall 8 studs long, two courses of brickwork under three of logs, on a plate."""
    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=8)
    m.use("wainscot_split_wall", x=0, y=0, z=1, length=8)
