# SPDX-License-Identifier: Apache-2.0
"""Side stud eyes: two round eyes on the face of a small head, each on the stud of a brick with a stud on its side, laid
in the course where the eyes go.

m.use("side_stud_eyes", x, y, z, width=4, spacing=0, facing="front", color="Yellow", eye="plain", title="Eyes") builds
one course (3 plates) of a head's face row, `width` studs long (3 to 8) from (x, y) on the studs at height z: a brick 1
x 1 with a stud on the side at each eye, `spacing` studs apart (0 to 4) and centered (exactly when width and spacing are
both even or both odd), and plain bricks in the rest of the row, in the head's color; lay the rest of the course behind
it. On each stud goes an eye: "plain" is a white round plate 1 x 1 with a black round tile 1 x 1 on its stud as the
pupil (it stands 2 plates out), "printed" a round tile 1 x 1 printed with an eye (1 plate out). facing is the face:
"front" (-y) and "back" (+y) rows run along x, "left" (-x) and "right" (+x) ones along y; (x, y) is the row's first
cell. Returns the z of the course's top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43290. Not derived from any
model file."""

PARTS = ["87087", "6141", "98138", "98138p07", "3005", "3004", "3622", "3010"]

# By facing: the way the row runs, the quarter turn that points a side stud out, the turn that stands a round plate or
# tile on its side facing out, and how far past the row's cell its outer face is (in studs, along the face's normal).
FACING = {
    "front": ("x", 3, Rot(x=90), (0, -1)),
    "back": ("x", 1, Rot(x=-90), (0, 1)),
    "left": ("y", 2, Rot(y=-90), (-1, 0)),
    "right": ("y", 0, Rot(y=90), (1, 0)),
}
PLATE = 0.4  # a plate's thickness, in studs


def build(
    m,
    x,
    y,
    z,
    width=4,
    spacing=0,
    facing="front",
    color="Yellow",
    eye="plain",
    title="Eyes",
):
    """The face row with the eye bricks, then the eyes; returns the z of the course's top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if not 3 <= width <= 8 or not 0 <= spacing <= 4 or spacing + 2 > width:
        raise ValueError(
            f"side_stud_eyes takes a width of 3 to 8 and a spacing of 0 to 4 that fits, not {width}, {spacing}"
        )
    if eye not in ("plain", "printed"):
        raise ValueError(f"side_stud_eyes takes eye='plain' or 'printed', not {eye!r}")
    axis, turn, stand, (nx, ny) = FACING[facing]
    first = (width - 2 - spacing) // 2
    eyes = (first, first + 1 + spacing)

    def cell(a):
        return (x + a, y) if axis == "x" else (x, y + a)

    with m.step(f"{title}: face row"):
        start = 0
        for c in (*eyes, width):
            if c > start:
                m.row(Brick, color, at=(*cell(start), z), length=c - start, axis=axis)
            if c < width:
                m.add("87087", color, at=(*cell(c), z), rot=turn)
            start = c + 1
    # The stud is 1.75 plates up the brick; a round plate or tile 1 x 1 stood on it covers 1.25 plates either side.
    zc = z + 0.5
    with m.step(f"{title}: eyes"):
        for c in eyes:
            cx, cy = cell(c)
            # The face is the cell's outer side; each layer stands a plate further out.
            fx = cx + (1 if nx > 0 else 0) - (PLATE if nx < 0 else 0)
            fy = cy + (1 if ny > 0 else 0) - (PLATE if ny < 0 else 0)
            if eye == "printed":
                m.add("98138p07", "White", at=(fx, fy, zc), rot=stand)
            else:
                m.add("6141", "White", at=(fx, fy, zc), rot=stand)
                m.add(
                    "98138",
                    "Black",
                    at=(fx + nx * PLATE, fy + ny * PLATE, zc),
                    rot=stand,
                )
    return z + 3


def example(m):
    """A small yellow head, 4 studs wide and 3 deep, with plain eyes in its middle course."""
    from emmet_dsl import Plate, Tile

    with m.step("Neck plate"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))  # Plate 4 x 4
    with m.step("Head, course 1"):
        m.add("3010", "Yellow", at=(0, 0, 1), rot=1)  # Brick 1 x 4, along x
        m.add("3001", "Yellow", at=(0, 1, 1), rot=1)  # Brick 2 x 4, along x
    m.use("side_stud_eyes", x=0, y=0, z=4, width=4, spacing=0)
    with m.step("Head, course 2: behind the eyes"):
        m.add("3001", "Yellow", at=(0, 1, 4), rot=1)
    with m.step("Head, course 3"):
        m.add("3010", "Yellow", at=(0, 0, 7), rot=1)
        m.add("3001", "Yellow", at=(0, 1, 7), rot=1)
    with m.step("Top"):
        m.fill(Plate, "Yellow", at=(0, 0, 10), size=(4, 3))
        m.fill(Tile, "Yellow", at=(0, 0, 11), size=(4, 3))
