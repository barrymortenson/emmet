# SPDX-License-Identifier: Apache-2.0
"""Ribbed hip roof: a roadside hip roof, four panels of chunky ribs hanging from the edges of a raised flat center
and sloping down past the walls on every side.

m.use("ribbed_hip_roof", x, y, z, length, depth, inset=2, rows=5, angle=45, color="Orange", base="Light Bluish Gray",
clip_color="Black", title="Roof") roofs the walls of the length x depth rectangle whose front-left corner is (x, y),
whose top is at z: a ceiling of `base` plates over them, a core of bricks inset `inset` studs from the walls on every
side, as many courses as the panels need to clear the walls, a deck of plates on it, and round the deck's edge a
course of clips (4081b) pointing out, every other stud, with plates between. From each side hangs a panel, built flat
and turned down `angle` degrees about the handle plates (2540) along its top edge: a base of `color` plates, a
2-deep row of tiles along the hinge (it ties the base), ribs of 1-wide tiles on every other row below it with bare
studs between, and tiles along the eave. The panels are half a stud longer than the deck's edges at each end, so
they clear each other at the corners, where the hips stay open. rows must be 3, 4, 5, 7, 9, 11 or 13 (the base's
plates run up the panel in one piece); length and depth 2 x inset + 4 or more. It brings its own steps and returns
the z of the deck's top.
"""

import math

from emmet_dsl import Plate, Rot, Tile, frame
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10352. Not derived from any
model file."""

PARTS = [
    "4081b",
    "2540",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "87079",
    "3068b",
    "2431",
    "3069b",
    "3070b",
    "63864",
    "3001",
    "3010",
    "3009",
    "3004",
    "3005",
]

RUNS = (
    2,
    3,
    4,
    6,
    8,
    10,
    12,
)  # plate lengths 1 and 2 wide both come in, for the columns up a panel
# Body boxes (LDU, LDraw's frame) and where each part's bar or clip runs: the hinge's axis.
CLIP, CLIP_BODY = "4081b", [-10, -6, -28, 10, 10, 10]
HANDLE, HANDLE_BODY = "2540", [-20, -4, -26, 20, 8, 10]
AXIS = (0, 2, -20)
# For each side: the clip's quarter turn (pointing out), the panel's turn about z after its tilt, and whether the
# panel's x runs against the edge's.
SIDES = {
    "front": (3, 180, True),
    "right": (0, -90, True),
    "back": (1, 0, False),
    "left": (2, 90, False),
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


def _panel(m, length, rows, color):
    """A panel lying flat, its handles along its hinge edge (toward -y): the hinge row, a column of plates 2 wide up
    the panel from the next row, the 2-deep tiles along the hinge, the ribs on every other row, and the eave's
    tiles."""
    handles = list(range(1, length - 2, 2))
    rest = length - (handles[-1] + 2)
    in_steps(
        m,
        "Hinge row",
        [(1, lambda: m.add("3024", color, at=(0, 0, 0)))]
        + [
            (1, lambda hx=hx: m.add(HANDLE, color, at=(hx, -0.8, 0), rot=3))
            for hx in handles
        ]
        + [(1, lambda: m.row(Plate, color, at=(length - rest, 0, 0), length=rest))],
    )
    in_steps(
        m,
        "Base plates",
        [
            (
                1,
                lambda cx=cx: m.add(
                    Plate(min(2, length - cx), rows - 1), color, at=(cx, 1, 0)
                ),
            )
            for cx in range(0, length, 2)
        ],
    )
    ribs = sorted(set(range(3, rows, 2)) | {rows - 1})
    in_steps(
        m,
        "Ribs",
        row_actions(m, Tile, color, (0, 0, 1), length, width=2)
        + [a for ry in ribs for a in row_actions(m, Tile, color, (0, ry, 1), length)],
    )


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    inset=2,
    rows=5,
    angle=45,
    color="Orange",
    base="Light Bluish Gray",
    clip_color="Black",
    title="Roof",
):
    """The ceiling, the core, the deck with its clips, and the four panels; returns the z of the deck's top."""
    if rows - 1 not in RUNS or min(length, depth) < 2 * inset + 4:
        raise ValueError(
            "ribbed_hip_roof takes rows 3, 4, 5, 7, 9, 11 or 13 and walls 2 x inset + 4 or more"
        )
    dx, dy, dl, dd = x + inset, y + inset, length - 2 * inset, depth - 2 * inset
    handle = _axis(
        HANDLE_BODY, (1, -0.8, 0), 3
    )  # the first handle's axis, in a panel's own frame
    # How far below its clip's axis a panel's eave hangs, in plates: its far bottom edge, turned down `angle`.
    t = math.radians(angle)
    fy, fz = (rows - handle[1]) * frame.STUD, (0 - handle[2]) * frame.PLATE
    drop = -(fy * -math.sin(t) + fz * math.cos(t)) / frame.PLATE
    clip_above = _axis(CLIP_BODY, (0, 0, -0.25), 1)[
        2
    ]  # the clip's axis over the course it sits on
    courses = max(1, math.ceil((drop - clip_above - 2 + 1) / 3))

    with m.step(f"{title}: ceiling"):
        m.fill(Plate, base, at=(x, y, z), size=(length, depth))
    for k in range(courses):
        with m.step(f"{title}: core, row {k + 1}"):
            m.wall(dx, dy, z + 1 + 3 * k, length=dl, depth=dd, color=base)
    zd = z + 1 + 3 * courses
    with m.step(f"{title}: deck"):
        m.fill(Plate, base, at=(dx, dy, zd), size=(dl, dd))
    zc = zd + 1

    # The clips: along each edge of the deck, where the panel's handles fall; then plates on the rest of the deck.
    spots = {}
    for side, (turn, _, against) in SIDES.items():
        n = (dl if side in ("front", "back") else dd) + 1
        hxs = range(1, n - 2, 2)
        lo = dx if side in ("front", "back") else dy
        along = [lo + n - 2 - hx if against else lo + hx for hx in hxs]
        if side == "front":
            cells = [(c, dy - 0.9) for c in along]
        elif side == "back":
            cells = [(c, dy + dd - 1) for c in along]
        elif side == "right":
            cells = [(dx + dl - 1, c) for c in along]
        else:
            cells = [(dx - 0.9, c) for c in along]
        spots[side] = (cells, turn)
    taken = set()
    actions = []
    for cells, turn in spots.values():
        for cx, cy in cells:
            actions.append(
                (
                    1,
                    lambda cx=cx, cy=cy, turn=turn: m.add(
                        CLIP, clip_color, at=(cx, cy, zc - 0.25), rot=turn
                    ),
                )
            )
            taken.add((math.ceil(cx), math.ceil(cy)))
    for cy in range(dy, dy + dd):
        run = None
        for cx in range(dx, dx + dl + 1):
            free = cx < dx + dl and (cx, cy) not in taken
            if free and run is None:
                run = cx
            elif not free and run is not None:
                actions += row_actions(m, Plate, base, (run, cy, zc), cx - run)
                run = None
    in_steps(m, f"{title}: clips", actions)

    for side, (_, spin, _) in SIDES.items():
        n = (dl if side in ("front", "back") else dd) + 1
        sub = m.submodel(
            f"{title}: {side} panel", _panel, length=n, rows=rows, color=color
        )
        cells, turn = spots[side]
        hinge = _axis(CLIP_BODY, (cells[0][0], cells[0][1], zc - 0.25), turn)
        rot = Rot(x=-angle, z=spin)
        landed = _world(sub, handle, rot)
        with m.step(f"{title}, {side} panel"):
            m.place(sub, at=tuple(hinge[i] - landed[i] for i in range(3)), rot=rot)
    return zc + 1


def example(m):
    """Walls 12 x 10, two bricks tall, under a ribbed hip roof at 35 degrees."""
    for row in range(2):
        with m.step(f"Walls, row {row + 1}"):
            m.wall(0, 0, 3 * row, length=12, depth=10, color="White")
    m.use(
        "ribbed_hip_roof", x=0, y=0, z=6, length=12, depth=10, inset=3, rows=7, angle=35
    )
