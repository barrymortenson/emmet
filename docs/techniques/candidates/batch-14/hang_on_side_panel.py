# SPDX-License-Identifier: Apache-2.0
"""Hang-on side panel: a large car's long, flat, stud-free side, a panel of plates and tiles built on its side that
hangs by clips on bars along the body's top edge, so it clicks on and comes off.

m.use("hang_on_side_panel", x, y, z, length=8, color="Light Bluish Gray", panel="Red", title="Side panel") lays a course
along the top of a body side at z, `length` studs along x from x, its outer face at y (the side faces -y): plates 1 x 2
with a handle (2540) whose bars stand out half a stud from the face, at x + 1, x + length - 2 and, from 8 long, the
middle, with plates between. The panel is two studs tall and hangs below the bars: an inner layer of plates 1 x n on
their sides, studs out, whose top row holds plates 1 x 2 with a clip on the side (11476) that grab the bars; a plate 2 x
`length` over both rows that crosses their seams; and tiles 2 x n on that. It stands 0.2 studs off the face and covers z
- 5.5 to z - 0.5, so the body under it can be anything. length: 6, 8, 10, or 12. Returns the z of the top of the course.
"""

from emmet_dsl import Plate, Tile, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10300, and the instruction
book 6670152. Not derived from any model file."""

PARTS = [
    "2540",
    "11476",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3795",
    "3034",
    "3832",
    "2445",
    "3068b",
    "87079",
    "69729",
]

# A plate on its side: studs to -y, long along x, its front edge up.
SIDE = frame.rotation(y=-90, z=90)
# A plate with a handle turned three quarters: its handle to -y, half a stud out.
OUT = frame.rotation(z=270)
LENGTHS = (6, 8, 10, 12)
FILL = (8, 6, 4, 3, 2, 1)  # plate 1 x n lengths for the gaps between clips
TILES = (6, 4, 2)


def _put(m, part, color, point, r=None):
    """Places a plate or tile by its origin, the middle of its top face, at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def _split(n, sizes):
    out = []
    while n:
        k = max(s for s in sizes if s <= n)
        out.append(k)
        n -= k
    return out


def _runs(length, clips):
    """The row along a side as (start, length, clip?): a 2-long piece at each clip, plates 1 x n between."""
    out, a = [], 0
    for c in clips + [length]:
        for n in _split(c - a, FILL):
            out.append((a, n, False))
            a += n
        if c < length:
            out.append((c, 2, True))
            a = c + 2
    return out


def build(
    m, x, y, z, length=8, color="Light Bluish Gray", panel="Red", title="Side panel"
):
    """The course with the bars, then the panel; returns the z of the course's top."""
    if length not in LENGTHS:
        raise ValueError(
            f"hang_on_side_panel takes a length of 6, 8, 10, or 12, not {length}"
        )
    clips = [1, length - 2] if length < 8 else [1, length // 2, length - 2]
    runs = _runs(length, clips)
    with m.step(f"{title}: bars"):
        for a, n, clip in runs:
            if clip:
                _put(m, "2540", color, (x + a + 1, y + 0.5, z + 1), OUT)
            else:
                m.add(Plate(1, n), color, at=(x + a, y, z), rot=1)
    # The bars are 0.25 plates under the course's top; a clip's axis is 1.25 plates over its plate's edge and 2 LDU in
    # from its stud face, so the inner layer's stud face is 0.6 studs out and its top edge 0.5 plates under z.
    face, top = y - 0.6, z - 0.5
    rows = (top - 3.75, top - 1.25)  # the two rows' middles
    with m.step(f"{title}: inner layer"):
        _put(m, Plate(1, length), panel, (x + length / 2, face, rows[0]), SIDE)
        for a, n, clip in runs:
            _put(
                m,
                "11476" if clip else Plate(1, n),
                panel,
                (x + a + n / 2, face, rows[1]),
                SIDE,
            )
    with m.step(f"{title}: plate and tiles"):
        _put(m, Plate(2, length), panel, (x + length / 2, face - 0.4, top - 2.5), SIDE)
        a = 0
        for n in _split(length, TILES):
            _put(m, Tile(2, n), panel, (x + a + n / 2, face - 0.8, top - 2.5), SIDE)
            a += n
    return z + 1


def example(m):
    """A panel 8 long hung on a wall two bricks tall."""
    with m.step("Body side"):
        m.wall(0, 0, 0, length=8, rows=2, color="Light Bluish Gray")
    m.use("hang_on_side_panel", x=0, y=0, z=6)
