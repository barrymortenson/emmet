# SPDX-License-Identifier: Apache-2.0
"""Teeth row: a jaw's edge set with sharp white teeth, plates 1 x 1 with a tooth standing on the side studs of the jaw's
front course, pointing up from the lower jaw or down from the upper: a shark, a dinosaur, a monster.

m.use("teeth_row", x, y, z, length=6, jaw="lower", facing="front", color="Light Bluish Gray", teeth="White",
title="Jaw") builds the front row of a jaw's course, `length` studs long (2 to 8) from (x, y) at height z: bricks 1 x 1
with a stud on the side pointing out of the face, and on each stud a plate 1 x 1 with a tooth (49668), turned so the
tooth points up (jaw="lower"; the tips stand 2.5 plates above the course) or down (jaw="upper"; they hang 2 plates below
it). A lower jaw stands on the studs at z. An upper jaw hangs over an open mouth, so it comes with a plate 2 x `length`
on top that reaches over the row behind it: lay a brick under that row in the same course first (the head's wall behind
the mouth), and the plate ties the jaw to it. facing is the face: "front" (-y) and "back" (+y) rows run along x, "left"
(-x) and "right" (+x) ones along y; (x, y) is the row's first cell. Returns the z of the course's top (the plate's top
for an upper jaw).
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21350, 75936, and 31109. Not
derived from any model file."""

PARTS = ["87087", "49668", "3795", "3020", "3021", "3022", "3034", "3666", "3710"]

# By facing: the way the row runs, the side stud's quarter turn, the turns that stand a tooth plate on the stud with
# its tooth up or down, and which way the face's normal points.
FACING = {
    "front": ("x", 3, Rot(y=-90, z=90), Rot(y=90, z=-90), (0, -1)),
    "back": ("x", 1, Rot(y=-90, z=-90), Rot(y=90, z=90), (0, 1)),
    "left": ("y", 2, Rot(y=-90, z=180), Rot(y=90), (-1, 0)),
    "right": ("y", 0, Rot(y=-90), Rot(y=90, z=180), (1, 0)),
}
PLATE = 0.4  # a plate's thickness, in studs


def build(
    m,
    x,
    y,
    z,
    length=6,
    jaw="lower",
    facing="front",
    color="Light Bluish Gray",
    teeth="White",
    title="Jaw",
):
    """The studded row, the upper jaw's plate, and the teeth; returns the z of the top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if jaw not in ("lower", "upper"):
        raise ValueError(f"teeth_row takes jaw='lower' or 'upper', not {jaw!r}")
    if not 2 <= length <= 8:
        raise ValueError(f"teeth_row takes a length of 2 to 8, not {length}")
    axis, turn, up, down, (nx, ny) = FACING[facing]

    def cell(a):
        return (x + a, y) if axis == "x" else (x, y + a)

    with m.step(f"{title}: studded row"):
        for a in range(length):
            m.add("87087", color, at=(*cell(a), z), rot=turn)
        if jaw == "upper":
            # The plate over the row and the row behind it, which holds the hanging jaw to the head.
            bx, by = (
                (x, y + (0 if ny < 0 else -1))
                if axis == "x"
                else (x + (0 if nx < 0 else -1), y)
            )
            m.row(Plate, color, at=(bx, by, z + 3), length=length, axis=axis, width=2)
    # The plate is centered on the side stud (1.75 plates up the brick) and the tooth reaches a stud past it.
    zt = z + 0.5 if jaw == "lower" else z - 2
    with m.step(f"{title}: teeth"):
        for a in range(length):
            cx, cy = cell(a)
            fx = cx + (1 if nx > 0 else 0) - (PLATE if nx < 0 else 0)
            fy = cy + (1 if ny > 0 else 0) - (PLATE if ny < 0 else 0)
            m.add("49668", teeth, at=(fx, fy, zt), rot=up if jaw == "lower" else down)
    return z + 3 if jaw == "lower" else z + 4


def example(m):
    """An open mouth: a lower and an upper jaw of teeth, a dark red throat behind, in a gray head 6 studs wide."""
    from emmet_dsl import Brick, Tile

    with m.step("Base"):
        m.add("3032", "Light Bluish Gray", at=(0, 0, 0), rot=1)  # Plate 4 x 6, along x
    m.use("teeth_row", x=0, y=0, z=1, length=6, jaw="lower", title="Lower jaw")
    with m.step("Head, course 1"):
        m.row(Brick, "Light Bluish Gray", at=(0, 1, 1), length=6, axis="x")
        m.row(Brick, "Light Bluish Gray", at=(0, 2, 1), length=6, axis="x", width=2)
    for c in (2, 3):
        with m.step(f"Head, course {c}: the throat"):
            m.row(
                Brick,
                "Dark Red",
                at=(0, 1, 3 * c - 2),
                length=6,
                axis="x",
                first=2 if c == 3 else None,
            )
            m.row(
                Brick,
                "Light Bluish Gray",
                at=(0, 2, 3 * c - 2),
                length=6,
                axis="x",
                width=2,
            )
    with m.step("Head, course 4"):
        m.row(Brick, "Light Bluish Gray", at=(0, 1, 10), length=6, axis="x")
        m.row(Brick, "Light Bluish Gray", at=(0, 2, 10), length=6, axis="x", width=2)
    m.use("teeth_row", x=0, y=0, z=10, length=6, jaw="upper", title="Upper jaw")
    with m.step("Head, top"):
        m.row(Plate, "Light Bluish Gray", at=(0, 2, 13), length=6, axis="x", width=2)
        m.row(Tile, "Light Bluish Gray", at=(0, 0, 14), length=6, axis="x", width=2)
        m.row(Tile, "Light Bluish Gray", at=(0, 2, 14), length=6, axis="x", width=2)
