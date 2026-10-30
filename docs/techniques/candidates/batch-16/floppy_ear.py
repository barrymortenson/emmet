# SPDX-License-Identifier: Apache-2.0
"""Floppy ear: a long flat ear built as a sub-assembly and hung on click hinges at the top edge of a head, so it hangs
beside the head and swings out: a dog's, a rabbit's, an elephant's.

m.use("floppy_ear", x, y, z, length=4, angle=90, facing="left", color="Black", hinge_color="Black", title="Ear") puts
two hinge plates 1 x 2 with two fingers (44302b) side by side on the edge row of the head's top at z, from (x, y), their
fingers past the edge. The ear, built flat, is two hinge plates with one finger (44301b), a plate 2 x `length` over them
(length 3 to 8), a round plate 2 x 2 under its far end that rounds the tip a stud past it, and tiles over the plate, so
its outer face is smooth. It goes on with its fingers between the head's, turned down by `angle` degrees: 90 hangs
straight down against the head's side, less splays it out (0 sticks it straight out, like a wing). The hinges click and
hold the angle. facing is the edge it hangs from: "left" (-x) and "right" (+x) edges run along y, "front" (-y) and
"back" (+y) along x; (x, y) is the first stud of the edge row. It brings its own steps and returns the lowest z the ear
reaches.
"""

from emmet_dsl import Plate, Rot, Tile, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21368 and 43290 (the ears
there hang on clips; click hinges hold them the same way). Not derived from any model file."""

PARTS = [
    "44302b",
    "44301b",
    "3623",
    "3020",
    "3021",
    "3795",
    "3666",
    "4032a",
    "3069b",
    "63864",
    "2431",
    "6636",
]

# Body boxes (LDU, LDraw's frame) and the hinge axis, which both hinge plates have at the same point of their own frame.
BASE, BASE_BODY = "44302b", [-20, -4, -10, 36, 8, 10]
EAR, EAR_BODY = "44301b", [-20, -5.38, -10, 35.35, 8, 10]
AXIS = (30, 2, 0)
# By facing: the head's hinge plate's quarter turn, where it goes relative to its cell (its fingers 0.8 studs past the
# edge), and the turn about z that takes the ear, built hanging off a front edge, round to that side.
FACING = {
    "front": (2, (0, -0.8), 0),
    "back": (0, (0, -1), 180),
    "left": (1, (-0.8, 0), 270),
    "right": (3, (-1, 0), 90),
}


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


def _ear(m, length, color, hinge_color):
    """The ear lying flat, reaching toward -y from its hinges: the hinge plates and the round tip under the plate,
    then the plate, then its tiles."""
    with m.step("Hinge plates and tip"):
        for c in (0, 1):
            m.add(EAR, hinge_color, at=(c, -2, 0))
        m.add(
            "4032a", color, at=(0, -length - 1, 0)
        )  # Round plate 2 x 2, half past the end
    with m.step("Plate"):
        m.row(Plate, color, at=(0, -length, 1), length=length, axis="y", width=2)
    with m.step("Tiles"):
        m.row(Tile, color, at=(0, -length, 2), length=length, axis="y", width=2)


def build(
    m,
    x,
    y,
    z,
    length=4,
    angle=90,
    facing="left",
    color="Black",
    hinge_color="Black",
    title="Ear",
):
    """The head's hinge plates and the ear; returns the lowest z the ear reaches."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if not 3 <= length <= 8:
        raise ValueError(f"floppy_ear takes a length of 3 to 8, not {length}")
    if not 0 <= angle <= 90:
        raise ValueError(f"floppy_ear takes an angle from 0 to 90, not {angle}")
    turn, (dx, dy), spin = FACING[facing]
    along = (1, 0) if facing in ("front", "back") else (0, 1)
    # The ear's hinges run the other way along a back or left edge, where it's turned round.
    flip = facing in ("back", "left")
    with m.step(f"{title}: hinge plates"):
        for c in (0, 1):
            a = c if not flip else 1 - c
            m.add(
                BASE,
                hinge_color,
                at=(x + along[0] * a + dx, y + along[1] * a + dy, z),
                rot=turn,
            )
    a0 = 0 if not flip else 1
    hinge = _axis(BASE_BODY, (x + along[0] * a0 + dx, y + along[1] * a0 + dy, z), turn)
    sub = m.submodel(
        "floppy_ear", _ear, length=length, color=color, hinge_color=hinge_color
    )
    rot = Rot(x=angle, z=spin)
    landed = _world(sub, _axis(EAR_BODY, (0, -2, 0), 0), rot)
    with m.step(f"{title}: ear"):
        placed = m.place(sub, at=tuple(hinge[i] - landed[i] for i in range(3)), rot=rot)
    return placed.min[2]


def example(m):
    """A white dog's head, 6 studs wide and 4 deep, with a black ear hanging at each side."""
    from emmet_dsl import Brick

    with m.step("Neck plate"):
        m.add("3032", "Dark Bluish Gray", at=(0, -1, 0), rot=1)  # Plate 4 x 6, along x
    for c in range(3):
        with m.step(f"Head, course {c + 1}"):
            m.row(
                Brick,
                "White",
                at=(0, -1, 1 + 3 * c),
                length=6,
                axis="x",
                width=2,
                first=2 if c % 2 else None,
            )
            m.row(
                Brick,
                "White",
                at=(0, 1, 1 + 3 * c),
                length=6,
                axis="x",
                width=2,
                first=None if c % 2 else 2,
            )
    with m.step("Head, top plates round the ear hinges"):
        m.add("3020", "White", at=(2, -1, 10))  # Plate 2 x 4, between the ear hinges
        for cx in (0, 4):
            for cy in (-1, 2):
                m.add("3023b", "White", at=(cx, cy, 10), rot=1)  # Plate 1 x 2, along x
    m.use(
        "floppy_ear",
        x=0,
        y=0,
        z=10,
        length=5,
        angle=90,
        facing="left",
        title="Left ear",
    )
    m.use(
        "floppy_ear",
        x=5,
        y=0,
        z=10,
        length=5,
        angle=90,
        facing="right",
        title="Right ear",
    )
