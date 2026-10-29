# SPDX-License-Identifier: Apache-2.0
"""Angled hull panel: a smooth panel hung on click hinges along a deck's edge and set at an angle, the sloped side or
nose of a spaceship's hull.

m.use("angled_hull_panel", x, y, z, length, depth=4, angle=30, facing="front", color="Light Bluish Gray",
hinge_color="Light Bluish Gray", title="Hull panel") puts hinge plates 1 x 2 with two fingers (44302b) on the edge row
of a deck at z, `length` studs from (x, y), at each end and every 4 studs between, each across the edge row and the
row inside it, its fingers past the edge. The panel, `length` × `depth` studs, is built flat as a sub-assembly: a
hinge plate 1 x 2 with one finger (44301b) at each of those places, plates over them in bands along the edge, and
tiles over the plates running front to back, so they tie the bands together; then it goes on, its fingers between the
deck's, turned down by `angle` degrees (0 is flat, 90 hangs straight down; a negative angle turns it up). The click
hinges lock at set steps; any angle is placed as asked. facing is the edge the panel hangs from: "front" (-y) and
"back" (+y) edges run along x, "left" (-x) and "right" (+x) ones along y; (x, y) is the first stud of the edge row.
length is 2 to 12 and depth 2 to 8. It brings its own steps and returns the lowest z the panel reaches.
"""

from emmet_dsl import Plate, Rot, Tile, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = [
    "44302b",
    "44301b",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "3024",
    "3070b",
    "3069b",
    "63864",
    "2431",
    "6636",
    "4162",
    "3068b",
    "26603",
    "87079",
    "69729",
]

# Body boxes (LDU, LDraw's frame) and the hinge axis, which both hinge plates have at the same point of their own frame.
BASE, BASE_BODY = "44302b", [-20, -4, -10, 36, 8, 10]
PANEL, PANEL_BODY = "44301b", [-20, -5.38, -10, 35.35, 8, 10]
AXIS = (30, 2, 0)
# By facing: the deck hinge plate's quarter turn, where it goes relative to its cell (its fingers 0.8 studs past the
# edge), and the turn about z that takes the panel, built facing front, round to that side.
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


def _panel(m, length, depth, hinges, color, hinge_color):
    """The panel lying flat, facing front: its hinge plates along its back edge, fingers toward +y, the panel toward
    -y from y = 0. The plates run along the edge in bands 2 wide, the first over the hinge plates; the tiles run the
    other way, front to back, so each one ties every band to the first."""
    with m.step("Hinge plates"):
        for c in hinges:
            m.add(PANEL, hinge_color, at=(c, -2, 0))
    for k, back in enumerate(range(0, -depth, -2)):
        width = min(2, back + depth)
        with m.step(f"Plates, band {k + 1}"):
            m.row(Plate, color, at=(0, back - width, 1), length=length, width=width)
    for c in range(0, length, 4):
        with m.step(f"Tiles, from {c + 1}"):
            for cc in range(c, min(c + 4, length), 2):
                width = min(2, length - cc)
                m.row(
                    Tile, color, at=(cc, -depth, 2), length=depth, axis="y", width=width
                )


def build(
    m,
    x,
    y,
    z,
    length,
    depth=4,
    angle=30,
    facing="front",
    color="Light Bluish Gray",
    hinge_color="Light Bluish Gray",
    title="Hull panel",
):
    """The deck's hinge plates and the panel; returns the lowest z the panel reaches."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if not 2 <= length <= 12 or not 2 <= depth <= 8:
        raise ValueError(
            f"angled_hull_panel takes a length of 2 to 12 and a depth of 2 to 8, not {length} × {depth}"
        )
    if not -90 <= angle <= 90:
        raise ValueError(
            f"angled_hull_panel takes an angle from -90 to 90, not {angle}"
        )
    turn, (dx, dy), spin = FACING[facing]
    hinges = sorted({0, length - 1, *range(4, length - 1, 4)})
    along = (1, 0) if facing in ("front", "back") else (0, 1)
    # The panel's hinges run the other way along a back or left edge, where it's turned round.
    flip = facing in ("back", "left")

    with m.step(f"{title}: hinge plates"):
        for c in hinges:
            a = c if not flip else length - 1 - c
            m.add(
                BASE,
                hinge_color,
                at=(x + along[0] * a + dx, y + along[1] * a + dy, z),
                rot=turn,
            )
    a0 = hinges[0] if not flip else length - 1 - hinges[0]
    hinge = _axis(BASE_BODY, (x + along[0] * a0 + dx, y + along[1] * a0 + dy, z), turn)
    sub = m.submodel(
        f"{title}: panel",
        _panel,
        length=length,
        depth=depth,
        hinges=tuple(hinges),
        color=color,
        hinge_color=hinge_color,
    )
    rot = Rot(x=angle, z=spin)
    landed = _world(sub, _axis(PANEL_BODY, (hinges[0], -2, 0), 0), rot)
    with m.step(f"{title}: panel"):
        placed = m.place(sub, at=tuple(hinge[i] - landed[i] for i in range(3)), rot=rot)
    return placed.min[2]


def example(m):
    """A deck with a panel sloping down from its front edge."""
    with m.step("Deck"):
        m.add("3958", "Dark Bluish Gray", at=(0, 0, 3))
        m.add("3003", "Dark Bluish Gray", at=(2, 2, 0))
    m.use("angled_hull_panel", x=0, y=0, z=4, length=6, depth=3, angle=30)
