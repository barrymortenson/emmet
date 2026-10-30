# SPDX-License-Identifier: Apache-2.0
"""Bar truss brace: thin diagonal struts from ground anchors up to a tower leg, each a bar with a clip at both ends
snapped onto handles, so every brace makes a triangle.

m.use("bar_truss_brace", x, y, z, sides=2, rows=2, color="Light Bluish Gray", strut="Light Bluish Gray",
title="Brace") stands a leg 2 x 2 studs at (x, y) on the studs at z and braces it. Ten plates up, a plate 1 x 2 with a
handle on each end (18649) crosses the leg's front row, its handles half a stud out each side; on the ground, another
lies along x 4 studs left of the leg (and, with sides=2, one 4 studs right). A strut runs from each ground handle to
the leg's: a bar holder with a clip (11090) on each handle and a bar 3L (87994) between them, 10 LDU deep in each
holder. The handles are 3 studs apart across and 10 plates up, so the strut is exactly 100 LDU, a 3-4-5 triangle
(about 53 degrees): the one length these parts make on the grid, so the brace is that triangle, not a free angle.
Keep the ground under the ground handles clear (a stud there would meet the handle): x - 7 to x - 6 and x - 4 to x - 3,
and x + 5 to x + 6 and x + 8 to x + 9. The leg rises `rows` bricks over the handle plate. Returns the z of its top.
"""

import math

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10341. Not derived from any
model file."""

PARTS = ["18649", "11090", "87994", "3003", "3022", "3023b"]

RISE = 10  # plates from a ground handle up to the leg's, and 3 studs across: a strut of 100 LDU


def _ldraw(p):
    """A DSL point (studs, studs, plates) in LDraw's frame, in LDU."""
    x, y, z = p
    return (20 * y, -8 * z, -20 * x)


def _cross(a, b):
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def _raw(origin, ydir, zdir):
    """An LDraw line's 12 numbers: the part's origin, its local Y along `ydir`, its local Z along `zdir`."""
    xdir = _cross(ydir, zdir)
    rows = [(xdir[i], ydir[i], zdir[i]) for i in range(3)]
    return [round(v, 4) for v in origin] + [round(v, 6) for row in rows for v in row]


def _strut(m, a, b, color):
    """A holder with a clip on the handle through `a`, a bar, and a holder on the handle through `b` (DSL points)."""
    pa, pb = _ldraw(a), _ldraw(b)
    d = [pb[i] - pa[i] for i in range(3)]
    length = math.sqrt(sum(v * v for v in d))
    u = [v / length for v in d]
    across = (1, 0, 0)  # the handles run along DSL y, LDraw's X
    m.add_raw("11090", color, _raw(pa, u, across))
    m.add_raw("87994", color, _raw([pa[i] + 20 * u[i] for i in range(3)], u, across))
    m.add_raw("11090", color, _raw(pb, [-v for v in u], [-v for v in across]))


def build(
    m,
    x,
    y,
    z,
    sides=2,
    rows=2,
    color="Light Bluish Gray",
    strut="Light Bluish Gray",
    title="Brace",
):
    """The anchors, the leg with its handle plate, and the struts; returns the z of the leg's top."""
    if sides not in (1, 2) or not 0 <= rows <= 8:
        raise ValueError(
            f"bar_truss_brace takes 1 or 2 sides and 0 to 8 rows, not {sides}, {rows}"
        )
    with m.step(f"{title}, anchors and leg"):
        # A plate with a handle on each end, its plate 2 studs along x: the handles half a stud past its ends.
        m.add("18649", color, at=(x - 6.8, y, z), rot=1)
        if sides == 2:
            m.add("18649", color, at=(x + 5.2, y, z), rot=1)
        m.stack("3003", color, at=(x, y, z), count=3)
        m.add("3022", color, at=(x, y, z + 9))
    zh = z + RISE
    with m.step(f"{title}, handle plate"):
        # The plate with handles is 1 x 2 between handles 3.6 studs apart: its plate on the leg's front row.
        m.add("18649", color, at=(x - 0.8, y, zh), rot=1)
        m.add("3023b", color, at=(x, y + 1, zh), rot=1)
    hy = y + 0.5  # the struts' plane, on the handle plate's middle
    for s, (ax, bx) in enumerate(((x - 3.5, x - 0.5), (x + 5.5, x + 2.5))[:sides]):
        with m.step(f"{title}, strut {s + 1}"):
            _strut(m, (ax, hy, z + 0.75), (bx, hy, zh + 0.75), strut)
    with m.step(f"{title}, leg"):
        m.stack("3003", color, at=(x, y, zh + 1), count=rows)
    return zh + 1 + 3 * rows


def example(m):
    """A leg braced both ways, on three footings."""
    with m.step("Footings"):
        # Three footings, the ground clear under the handles (a stud there would meet them).
        for fx in (-6, 0, 6):  # the anchors and the leg, each 2 studs along x
            m.add("3022", "Dark Bluish Gray", at=(fx, 0, 0))
    m.use("bar_truss_brace", x=0, y=0, z=1)
