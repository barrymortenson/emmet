# SPDX-License-Identifier: Apache-2.0
"""Buttress: a stepped pier against the face of a wall, bonded into it course by course, its two set-backs weathered
with slopes.

m.use("buttress", x, y, z, length, rows=6, offsets=None, color="Light Bluish Gray", title="Buttress") builds a wall 1
stud thick and `length` studs long along x from (x, y), `rows` bricks tall from z, with a buttress 2 studs wide at each
of `offsets` (studs from the wall's start; by default one in the middle) standing out from its face (-y). The buttress
stands out 2 studs for all but the top three courses, its bricks bonded into the wall: a 2 x 3 brick that takes the
wall's own stud on one course, a 2 x 2 in front of an unbroken wall on the next. Then a slope 2 x 2 sets it back a stud,
a 2 x 2 brick (bonded into the wall) carries it a course, and a last slope sets it back into the wall's face. rows must
be 4 or more. Returns the z of the top.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3002",
    "3003",
    "3039",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
]

BRICKS = {8: "3008", 6: "3009", 4: "3010", 3: "3622", 2: "3004", 1: "3005"}


def _course(m, color, x, y, z, length, gaps, shift):
    """One course of the wall along x, 1 x n bricks around the gaps [(start, end)], the first brick of each run
    shorter when shift (the running bond)."""
    u = 0
    for start, end in sorted(gaps) + [(length, length)]:
        n = start - u
        if n > 0:
            first = max(k for k in BRICKS if k <= n) // 2 if shift and n > 2 else None
            m.row(Brick, color, at=(x + u, y, z), length=n, first=first)
        u = end


def build(
    m,
    x,
    y,
    z,
    length,
    rows=6,
    offsets=None,
    color="Light Bluish Gray",
    title="Buttress",
):
    """The wall and its buttresses, a course at a time; returns the z of the top."""
    offsets = [(length - 2) // 2] if offsets is None else sorted(offsets)
    if rows < 4:
        raise ValueError(f"buttress needs 4 rows or more, not {rows}")
    if (
        any(offsets[i + 1] - offsets[i] < 3 for i in range(len(offsets) - 1))
        or offsets[0] < 0
        or offsets[-1] > length - 2
    ):
        raise ValueError(
            f"buttress needs its buttresses inside the wall and a stud of wall between them, not at {offsets}"
        )
    for r in range(rows):
        zr = z + 3 * r
        # 0 at the first slope, 1 at the bonded course over it, 2 at the last slope.
        set_back = r - (rows - 3)
        bonded = set_back in (1, 2) or (set_back < 0 and r % 2 == 0)
        with m.step(f"{title}, row {r + 1}"):
            for u in offsets:
                if set_back < 0:
                    if bonded:
                        m.add("3002", color, at=(x + u, y - 2, zr))
                    else:
                        m.add("3003", color, at=(x + u, y - 2, zr))
                elif set_back == 0:
                    m.add("3039", color, at=(x + u, y - 2, zr), rot=3)
                elif set_back == 1:
                    m.add("3003", color, at=(x + u, y - 1, zr))
                else:
                    m.add("3039", color, at=(x + u, y - 1, zr), rot=3)
            gaps = [(u, u + 2) for u in offsets] if bonded else []
            _course(m, color, x, y, zr, length, gaps, r % 2)
    return z + 3 * rows


def example(m):
    """A wall 8 studs long with a buttress in the middle."""
    m.use("buttress", x=0, y=0, z=0, length=8)
