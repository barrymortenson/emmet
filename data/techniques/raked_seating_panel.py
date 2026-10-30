# SPDX-License-Identifier: Apache-2.0
"""Raked seating panel: a micro stadium stand, built as an upright wall of grille bricks and plain bricks with white
aisle plates across it, then laid back at the stand's rake on clip hinges, so the grilles read as rows of seats.

m.use("raked_seating_panel", x, y, z, length=8, rows=4, rake=35, color="Red", aisle="White", clip_color=None,
title="Stand") builds, on the studs at height z, a stand `length` studs wide (even, 4 to 16) along x whose foot is the
row of studs at y, rising toward +y. The panel is built apart as a submodel, standing: a hinge row of plates with
handles on their front edge (2540) and plain plates, then `rows` courses (2 to 8) of 1 x 2 bricks, grille bricks (2877)
and plain ones in turn along the course and shifted a brick each course, with a white 1 x n plate across the panel every
two courses as an aisle line. It's half a stud longer than the stand at each end, so its handles center on studs. A
course of bricks along the foot row lifts the hinge, clips (4081b) on it hold the handles, and the panel is placed
turned back about them to `rake` degrees from the ground (20 to 60), its weight on the clips (in the stand's color
unless clip_color is given). Returns the placed panel's Handle; its .max is the top of the stand at the back.
"""

from emmet_dsl import Brick, Plate, Rot, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10272. Not derived from any
model file."""

PARTS = [
    "2540",
    "4081b",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "2877",
    "3004",
    "3005",
]

# Body boxes (LDU, LDraw's frame) and where each part's bar or clip runs: the hinge's axis (as in hinge_roof).
CLIP, CLIP_BODY = "4081b", [-10, -6, -28, 10, 10, 10]
HANDLE, HANDLE_BODY = "2540", [-20, -4, -26, 20, 8, 10]
AXIS = (0, 2, -20)


def _axis(body, at, rot):
    """Where the hinge axis of a part placed at `at`, rot (quarter turns) runs through, in DSL units."""
    pos, r, _ = frame.place(body, at, frame.quarter_turns(rot))
    w = [sum(r[i][k] * AXIS[k] for k in range(3)) + pos[i] for i in range(3)]
    return (-w[2] / frame.STUD, w[0] / frame.STUD, -w[1] / frame.PLATE)


def _world(sub, point, turn):
    """Where a point of a submodel (DSL units, its own frame) lands when the submodel is placed at (0, 0, 0)."""
    pos, r, _ = frame.place(
        frame.box_in_ldraw(*sub.box), (0, 0, 0), frame.rotation(turn.x, turn.y, turn.z)
    )
    p = frame.to_ldraw_point(point)
    w = [sum(r[i][k] * p[k] for k in range(3)) + pos[i] for i in range(3)]
    return (-w[2] / frame.STUD, w[0] / frame.STUD, -w[1] / frame.PLATE)


def _panel(m, length, rows, color, aisle):
    """The panel standing, its handles along the foot of its front face at x 1, 3, …: a hinge row, then the
    courses, an aisle plate over every second one."""
    handles = list(range(1, length - 2, 2))
    with m.step("Hinge row"):
        m.add("3024", color, at=(0, 0, 0))
        for hx in handles:
            m.add(HANDLE, color, at=(hx, -0.8, 0), rot=3)
        rest = length - (handles[-1] + 2)
        m.row(Plate, color, at=(length - rest, 0, 0), length=rest)
    z = 1
    for r in range(rows):
        with m.step(f"Course {r + 1}"):
            # Bricks 2 long from x = 0 or 1 (the half-stud ends are 1 x 1s), grille and plain in turn.
            at, k = 0, r % 2
            if k:
                m.add(Brick(1, 1), color, at=(0, 0, z))
                at = 1
            while at + 2 <= length:
                m.add(
                    "2877" if (at // 2 + r) % 2 == 0 else "3004",
                    color,
                    at=(at, 0, z),
                    rot=1,
                )
                at += 2
            if at < length:
                m.add(Brick(1, 1), color, at=(at, 0, z))
        z += 3
        if r % 2 == 1 and r < rows - 1:
            with m.step(f"Aisle {r // 2 + 1}"):
                m.row(Plate, aisle, at=(0, 0, z), length=length)
            z += 1
    with m.step("Top"):
        m.row(Plate, color, at=(0, 0, z), length=length)


def build(
    m,
    x,
    y,
    z,
    length=8,
    rows=4,
    rake=35,
    color="Red",
    aisle="White",
    clip_color=None,
    title="Stand",
):
    """The clips and the panel; returns the placed panel's Handle."""
    if (
        length % 2
        or not 4 <= length <= 16
        or not 2 <= rows <= 8
        or not 20 <= rake <= 60
    ):
        raise ValueError(
            "raked_seating_panel takes an even length of 4 to 16, 2 to 8 rows, and a rake of 20 to 60 degrees, "
            f"not {length}, {rows}, {rake}"
        )
    panel_length = (
        length + 1
    )  # from x - 0.5, so the handles (at panel x 1, 3, …) center on studs
    handles = range(1, panel_length - 2, 2)
    # The foot: a course of bricks, so the panel's back edge, which swings down as it leans back, clears the ground.
    with m.step(f"{title}: foot"):
        m.row(Brick, color, at=(x, y, z), length=length)
    clips = [(x + hx, y, z + 3 - 0.25) for hx in handles]
    hinge = _axis(CLIP_BODY, clips[0], 1)
    with m.step(f"{title}: clips"):
        for c in clips:
            m.add(CLIP, clip_color or color, at=c, rot=1)
    sub = m.submodel(
        f"{title}: panel",
        _panel,
        length=panel_length,
        rows=rows,
        color=color,
        aisle=aisle,
    )
    # Standing, the panel's face is to -y; turned back about x, its top leans toward +y at `rake` from the ground.
    turn = Rot(x=-(90 - rake))
    landed = _world(sub, _axis(HANDLE_BODY, (1, -0.8, 0), 3), turn)
    with m.step(f"{title}: panel"):
        return m.place(sub, at=tuple(hinge[i] - landed[i] for i in range(3)), rot=turn)


def example(m):
    """A stand 8 studs wide at a 35 degree rake, beside a strip of pitch."""
    from emmet_dsl import Tile

    with m.step("Ground"):
        m.add(Plate(8, 8), "Dark Bluish Gray", at=(0, 0, 0))
    with m.step("Pitch edge"):
        m.fill(Tile, "Green", at=(0, 0, 1), size=(8, 2))
    m.use("raked_seating_panel", x=0, y=2, z=1, length=8, rows=4)
