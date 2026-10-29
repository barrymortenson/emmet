# SPDX-License-Identifier: Apache-2.0
"""Arrow slit: a narrow slit through a thick wall, a stud wide on the outside and opening into a wide embrasure with a
corbelled head on the inside, where the archer stands.

m.use("arrow_slit", x, y, z, length=9, offset=None, color="Light Bluish Gray", title="Arrow slit") builds a wall 2 studs
thick and `length` studs long along x from (x, y), 5 bricks tall from z, with a slit `offset` studs from its start (by
default in the middle). The outer leaf (row y) is whole on the first course, leaves the slit open for the second and
third, and closes it on the fourth with an Arch 1 x 3 (4490), which gives the slit its rounded head; the inner leaf (row
y + 1) leaves the embrasure, 3 studs wide, open for three courses, and closes it on the fourth with a corbelled head: an
inverted slope 2 x 1 (3665b) from each side, stepping in a stud, with the bricks of the course above spanning the stud
between. A course of 2-wide plates on top ties the two leaves. The courses are a running bond. Returns the z of the top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "4490",
    "3665b",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "91988",
    "4282",
    "3023b",
]

BRICKS = {8: "3008", 6: "3009", 4: "3010", 3: "3622", 2: "3004", 1: "3005"}
ROWS = 5


def _run(m, color, x, y, z, n, shift):
    """A run of n studs of 1 x k bricks along x, the first one shorter when shift (the running bond)."""
    sizes, left = [], n
    if shift and n > 2:
        first = max(k for k in BRICKS if k <= n) // 2
        sizes.append(first)
        left -= first
    for k in sorted(BRICKS, reverse=True):
        while left >= k:
            sizes.append(k)
            left -= k
    for k in sizes:
        m.add(BRICKS[k], color, at=(x, y, z), rot=1)
        x += k


def build(
    m, x, y, z, length=9, offset=None, color="Light Bluish Gray", title="Arrow slit"
):
    """The two leaves, the slit's arch and the embrasure's head, and the tie course; returns the z of the top."""
    c = (length - 1) // 2 if offset is None else offset
    if c < 3 or length - c - 1 < 3:
        raise ValueError(
            f"arrow_slit needs 3 studs of wall each side of the slit, not {c} and {length - c - 1}"
        )
    for r in range(ROWS):
        zr = z + 3 * r
        # Each leaf's gaps this course, as (start, end) in studs from x.
        outer = [(c, c + 1)] if r in (1, 2) else [(c - 1, c + 2)] if r == 3 else []
        inner = [(c - 1, c + 2)] if r < 3 else [(c - 2, c + 3)] if r == 3 else []
        for row, gaps, shift, leaf in (
            (y, outer, r % 2, "outside"),
            (y + 1, inner, 1 - r % 2, "inside"),
        ):
            with m.step(f"{title}, row {r + 1}: {leaf}"):
                if r == 3 and leaf == "outside":
                    m.add("4490", color, at=(x + c - 1, y, zr), rot=1)
                if r == 3 and leaf == "inside":
                    # The corbels, their slopes facing each other across the slit.
                    m.add("3665b", color, at=(x + c - 2, row, zr), rot=0)
                    m.add("3665b", color, at=(x + c + 1, row, zr), rot=2)
                u = 0
                for start, end in gaps + [(length, length)]:
                    if start > u:
                        _run(m, color, x + u, row, zr, start - u, shift)
                    u = end
    with m.step(f"{title}: tie course"):
        top = z + 3 * ROWS
        m.row(Plate, color, at=(x, y, top), length=length, width=2, first=2)
    return top + 1


def example(m):
    """A stretch of thick wall with an arrow slit in the middle."""
    m.use("arrow_slit", x=0, y=0, z=0, length=7)
