# SPDX-License-Identifier: Apache-2.0
"""Curved wall end: the open end of a cutaway wall stepping down in curved slopes instead of stopping square, so the
cut reads as designed.

m.use("curved_wall_end", x, y, z, length, rows=5, sweep=2, end="right", color="Medium Nougat", title="Wall") lays a
wall `length` studs long along x from (x, y, z), facing front (-y), `rows` courses tall in running bond, whose top
`sweep` courses each end in a 3 x 1 curved slope (50950), every course 3 studs shorter than the one below at that
end, so the wall's top corner sweeps down to the end in a curve that flares out toward the bottom. Each slope's
curved part sits on the bricks of the course below (its high end has no stud to hold by). end is "right" (at
x + length) or "left" (at x). Returns the z of the top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387. Not derived from any
model file."""

PARTS = ["50950", "3005", "3004", "3622", "3010", "3009", "3008"]

TURN = {"right": 0, "left": 2}  # the slope's low end toward +x or -x


def build(
    m,
    x,
    y,
    z,
    length,
    rows=5,
    sweep=2,
    end="right",
    color="Medium Nougat",
    title="Wall",
):
    """The courses, each ending at the curved end; returns the z of the top."""
    if end not in TURN:
        raise ValueError(f"end={end!r}: use right or left")
    if not 1 <= sweep <= rows or length < 3 * sweep + 2:
        raise ValueError(
            "curved_wall_end needs 1 <= sweep <= rows and a length of 3 x sweep + 2 or more, "
            f"not {sweep}, {rows}, {length}"
        )
    for r in range(rows):
        zr = z + 3 * r
        cut = 3 * max(0, r - (rows - 1 - sweep))
        with m.step(f"{title}, row {r + 1}"):
            bricks = length - cut
            # Bricks from the far end of the wall; the curve at the near end.
            bx = x if end == "right" else x + length - bricks
            first = 1 if r % 2 and bricks > 2 else None
            m.row(
                Brick,
                color,
                at=(bx, y, zr),
                length=bricks,
                first=first,
                sizes=(8, 6, 4, 3, 2, 1),
            )
            if cut:
                sx = x + bricks if end == "right" else x + cut - 3
                m.add("50950", color, at=(sx, y, zr), rot=TURN[end])
    return z + 3 * rows


def example(m):
    """A cutaway wall whose right end sweeps down, on a floor."""
    with m.step("Floor"):
        m.row(Plate, "Reddish Brown", at=(0, 0, 0), length=10, first=4)
    m.use("curved_wall_end", x=0, y=0, z=1, length=10)
