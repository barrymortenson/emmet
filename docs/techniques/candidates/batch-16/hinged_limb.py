# SPDX-License-Identifier: Apache-2.0
"""Hinged limb: an arm or a wing built flat as a sub-assembly and hung on click hinges at a body's shoulder edge, so it
swings out from the body or folds flush against it: a robot's arm, a bird's wing, a flipper.

m.use("hinged_limb", x, y, z, length=4, angle=60, style="arm", facing="left", color="White", hinge_color="Light Bluish
Gray", title="Arm") puts two hinge plates 1 x 2 with two fingers (44302b) side by side on the edge row of the body's top
at z, from (x, y), their fingers past the edge. The limb, built flat, is two hinge plates with one finger (44301b) and
over them a plate 2 x `length` (3 to 8). style "arm" covers it with tiles and ends it in a curved slope 2 x 2 falling to
the tip, a rounded hand; "wing" sets a wing plate 2 x 4 under its end, tapering a further 2 studs to a point, and tiles
the plate. It goes on with its fingers between the body's, turned down by `angle` degrees: 90 folds it flush down the
body's side, 0 holds it straight out; the hinges click and hold the angle. facing is the edge it hangs from: "left" (-x)
and "right" (+x) edges run along y, "front" (-y) and "back" (+y) along x; (x, y) is the first stud of the edge row. It
brings its own steps and returns the lowest z the limb reaches.
"""

from emmet_dsl import Plate, Rot, Tile, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 43279 and 43290 (the arms and
wings there hang on clips and pins; click hinges hold them the same way). Not derived from any model file."""

PARTS = [
    "44302b",
    "44301b",
    "3020",
    "3021",
    "3795",
    "3034",
    "15068",
    "41769a",
    "41770a",
    "3069b",
    "63864",
    "2431",
]

# Body boxes (LDU, LDraw's frame) and the hinge axis, which both hinge plates have at the same point of their own frame.
BASE, BASE_BODY = "44302b", [-20, -4, -10, 36, 8, 10]
LIMB, LIMB_BODY = "44301b", [-20, -5.38, -10, 35.35, 8, 10]
AXIS = (30, 2, 0)
# By facing: the head's hinge plate's quarter turn, where it goes relative to its cell (its fingers 0.8 studs past the
# edge), and the turn about z that takes the limb, built hanging off a front edge, round to that side.
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


WING = (
    "41770a",
    1,
)  # Wing 2 x 4 Left, turned to point along -y with its square side out


def _limb(m, length, style, color, hinge_color):
    """The limb lying flat, reaching toward -y from its hinges."""
    with m.step("Hinge plates" if style == "arm" else "Hinge plates and wing tip"):
        for c in (0, 1):
            m.add(LIMB, hinge_color, at=(c, -2, 0))
        if style == "wing":
            # Under the plate's last 2 rows and 2 studs past them.
            m.add(WING[0], color, at=(0, -length - 2, 0), rot=WING[1])
    with m.step("Plate"):
        m.row(Plate, color, at=(0, -length, 1), length=length, axis="y", width=2)
    with m.step("Tiles" if style == "wing" else "Tiles and hand"):
        if style == "arm":
            m.add("15068", color, at=(0, -length, 2), rot=3)  # falling to the tip
            m.row(
                Tile,
                color,
                at=(0, -length + 2, 2),
                length=length - 2,
                axis="y",
                width=2,
            )
        else:
            m.row(Tile, color, at=(0, -length, 2), length=length, axis="y", width=2)


def build(
    m,
    x,
    y,
    z,
    length=4,
    angle=60,
    style="arm",
    facing="left",
    color="White",
    hinge_color="Light Bluish Gray",
    title="Arm",
):
    """The body's hinge plates and the limb; returns the lowest z the limb reaches."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if style not in ("arm", "wing"):
        raise ValueError(f"hinged_limb takes style='arm' or 'wing', not {style!r}")
    if not 3 <= length <= 8:
        raise ValueError(f"hinged_limb takes a length of 3 to 8, not {length}")
    if not 0 <= angle <= 90:
        raise ValueError(f"hinged_limb takes an angle from 0 to 90, not {angle}")
    turn, (dx, dy), spin = FACING[facing]
    along = (1, 0) if facing in ("front", "back") else (0, 1)
    # The limb's hinges run the other way along a back or left edge, where it's turned round.
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
        "hinged_limb",
        _limb,
        length=length,
        style=style,
        color=color,
        hinge_color=hinge_color,
    )
    rot = Rot(x=angle, z=spin)
    landed = _world(sub, _axis(LIMB_BODY, (0, -2, 0), 0), rot)
    with m.step(f"{title}: {style}"):
        placed = m.place(sub, at=tuple(hinge[i] - landed[i] for i in range(3)), rot=rot)
    return placed.min[2]


def example(m):
    """A robot's body 6 studs wide, one arm raised and one folded down its side."""
    from emmet_dsl import Brick

    with m.step("Base"):
        m.add("3032", "Dark Bluish Gray", at=(0, -1, 0), rot=1)  # Plate 4 x 6, along x
    for c in range(3):
        with m.step(f"Body, course {c + 1}"):
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
    with m.step("Body, top plates round the shoulders"):
        m.add(
            "3020", "White", at=(2, -1, 10)
        )  # Plate 2 x 4, between the shoulder hinges
        for cx in (0, 4):
            for cy in (-1, 2):
                m.add("3023b", "White", at=(cx, cy, 10), rot=1)  # Plate 1 x 2, along x
    m.use(
        "hinged_limb",
        x=0,
        y=0,
        z=10,
        length=4,
        angle=30,
        facing="left",
        title="Left arm",
    )
    m.use(
        "hinged_limb",
        x=5,
        y=0,
        z=10,
        length=4,
        angle=90,
        facing="right",
        title="Right arm",
    )
