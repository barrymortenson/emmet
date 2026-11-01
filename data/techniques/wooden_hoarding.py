# SPDX-License-Identifier: Apache-2.0
"""Wooden hoarding: a covered timber gallery built out from a wall's top on beams through the wall, with a log front,
loopholes, and a sloped roof against the parapet.

m.use("wooden_hoarding", x, y, z, length, rows=5, color="Light Bluish Gray", timber="Reddish Brown",
roof="Dark Bluish Gray", title="Hoarding") builds on a wall 1 stud thick along x from (x, y), its top at z, `length`
studs long (3n + 1). The wall's next course is laid with beams: a 1 x 3 brick in timber every 3 studs, through the wall
and 2 studs out front (-y), with the wall's bricks between. A floor of 2-wide plates lies on the beams' ends; on its
front edge stand `rows` courses of log bricks, the next to top with a loophole in each bay, at a figure's eyes;
behind, the wall rises `rows` courses and a plate as a parapet. The roof is plates across the front, the gallery and
the parapet, and 2 x 2 slopes over the front two rows. The gallery between the front and the parapet is a stud deep,
room for a figure to stand, and rows is its headroom: 5 (the default) is the least a figure stands under, so it's 5 to
8 (docs/spec/building-rules.md, rule 1). Returns the z of the roof plates' top, with .floor (the z of the gallery
floor's top) and .underside (the z of the roof plates' bottom).
"""

from emmet_dsl import Brick, Plate
from emmet_dsl.core import fit

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3622",
    "30136",
    "30137",
    "3005",
    "3004",
    "3039",
    "3040b",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "91988",
    "4282",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
]

LOGS = {4: "30137", 2: "30136"}  # Brick 1 x n Log


def _logs(m, color, x, y, z, n):
    """A run of n studs of log bricks along x, the longest first, and a plain 1 x 1 brick for an odd stud."""
    for k in (4, 2):
        while n >= k:
            m.add(LOGS[k], color, at=(x, y, z), rot=1)
            x += k
            n -= k
    if n:
        m.add("3005", color, at=(x, y, z))


def build(
    m,
    x,
    y,
    z,
    length,
    rows=5,
    color="Light Bluish Gray",
    timber="Reddish Brown",
    roof="Dark Bluish Gray",
    title="Hoarding",
):
    """The beam course, the floor, the front and parapet, and the roof; returns the z of the roof plates' top, with
    .floor and .underside."""
    if length < 4 or (length - 1) % 3:
        raise ValueError(
            f"wooden_hoarding needs a length of 3n + 1 (4, 7, 10, ...), not {length}"
        )
    if not 5 <= rows <= 8:
        raise ValueError(
            f"wooden_hoarding takes rows of 5 to 8 (5 bricks over the gallery floor, the least a figure stands "
            f"under), not {rows}"
        )
    beams = range(0, length, 3)
    with m.step(f"{title}: beams"):
        for u in beams:
            m.add("3622", timber, at=(x + u, y - 2, z))
            if u + 3 < length:
                m.add("3004", color, at=(x + u + 1, y, z), rot=1)
    zf = z + 3
    with m.step(f"{title}: floor"):
        m.row(Plate, timber, at=(x, y - 2, zf), length=length, width=2)
    for r in range(rows):
        with m.step(f"{title}: front and parapet, course {r + 1}"):
            if r == rows - 2:
                # A loophole, a stud wide, in the middle of each bay.
                u = 0
                for gap in [b + 1 for b in beams[:-1]] + [length]:
                    _logs(m, timber, x + u, y - 2, zf + 1 + 3 * r, gap - u)
                    u = gap + 1
            else:
                _logs(m, timber, x, y - 2, zf + 1 + 3 * r, length)
            m.row(
                Brick,
                color,
                at=(x, y, zf + 3 * r),
                length=length,
                first=None if r % 2 else 2,
            )
            if r == rows - 1:
                m.row(
                    Plate,
                    color,
                    at=(x, y, zf + 3 * rows),
                    length=length,
                    first=None if rows % 2 else 2,
                )
    zr = zf + 3 * rows + 1
    with m.step(f"{title}: roof plates"):
        # One-wide over the front, two-wide over the gallery and the parapet, so the roof ties the two.
        m.row(Plate, roof, at=(x, y - 2, zr), length=length)
        m.row(Plate, roof, at=(x, y - 1, zr), length=length, width=2)
    with m.step(f"{title}: roof"):
        for u in range(0, length - 1, 2):
            m.add("3039", roof, at=(x + u, y - 2, zr + 1), rot=3)
        if length % 2:
            m.add("3040b", roof, at=(x + length - 1, y - 2, zr + 1), rot=3)
    return fit(zr + 1, floor=zf + 1, underside=zr)


def example(m):
    """A short wall with a hoarding along its top."""
    for r in range(2):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Light Bluish Gray",
                at=(0, 0, 3 * r),
                length=7,
                first=2 if r % 2 else None,
            )
    m.use("wooden_hoarding", x=0, y=0, z=6, length=7)
