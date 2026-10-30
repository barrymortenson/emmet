# SPDX-License-Identifier: Apache-2.0
"""Lean-to awning: a shallow shed roof over a porch or a window, a panel of plates hinged to the wall on clips and
tipped down to rest on two posts.

m.use("lean_to_awning", x, y, z, length, depth=4, angle=20, ground=0, color="White", posts="Reddish Brown",
clip_color="Black", title="Awning") hangs an awning from the front wall (facing -y) whose row of studs is at y, at
the height z of that wall's top course so far: it lays a course of plates on the wall from x to x + length with clips
(4081b) every other stud, their clips reaching out past the wall's face, and returns that course's top for the wall
to go on above. The panel, built flat and turned about the handle plates (2540) along its back edge, is a base of
plates, a 2-deep row of tiles along the wall (it ties the base), plates studs up (the snow lies on them), and a row
of tiles along its front edge; it is depth + 1 rows deep and length + 1 long (half a stud past each end) and tips down
by `angle` degrees. Two posts of round bricks 1 x 1 and round plates stand from `ground` at its front corners, as
tall as fits under it. length must be 4 or more, and depth 2, 3, 4, 6, 8, 10, or 12 (the plates that run up the
panel come in those lengths). It brings its own steps and returns the z of the clip course's top.
"""

import math

from emmet_dsl import Brick, Plate, Rot, Tile, frame
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387. Not derived from any
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
    "3062b",
    "6141",
]

RUNS = (
    2,
    3,
    4,
    6,
    8,
    10,
    12,
)  # plate lengths 1 and 2 wide both come in, for the columns up the panel

# Body boxes (LDU, LDraw's frame) and where each part's bar or clip runs: the hinge's axis.
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


def _panel(m, length, rows, color):
    """The panel lying flat, its handles along its back edge (toward -y here): the hinge row and columns of plates up
    the panel (their joints fall between the handles'), then the tiles along the hinge edge, which tie them, the
    plates, and the front edge's tiles."""
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
    in_steps(
        m,
        "Cover",
        row_actions(m, Tile, color, (0, 0, 1), length, width=2)
        + [
            a
            for ry in range(2, rows - 1)
            for a in row_actions(m, Plate, color, (0, ry, 1), length)
        ]
        + row_actions(m, Tile, color, (0, rows - 1, 1), length),
    )


def build(
    m,
    x,
    y,
    z,
    length,
    depth=4,
    angle=20,
    ground=0,
    color="White",
    posts="Reddish Brown",
    clip_color="Black",
    title="Awning",
):
    """The clip course, the panel, and the posts; returns the z of the clip course's top."""
    rows = depth + 1
    if length < 4 or rows - 1 not in RUNS:
        raise ValueError(
            f"lean_to_awning needs a length of 4 or more and a depth of 2, 3, 4, 6, 8, 10 or 12, not {depth}"
        )
    panel_length = (
        length + 1
    )  # from x - 0.5, so the handles (at panel x 1, 3, ...) center on studs
    handles = list(range(1, panel_length - 2, 2))
    # The clips on the wall's studs, mirrored like hinge_roof's back ones (the panel goes on turned around), reaching
    # out over the wall's face; plates on the studs between.
    spots = sorted(x + length - 1 - hx for hx in handles)
    course = [
        (1, lambda c=c: m.add(CLIP, clip_color, at=(c, y - 0.9, z - 0.25), rot=3))
        for c in spots
    ]
    edges = [x] + [c + 1 for c in spots]
    for a, b in zip(edges, spots + [x + length], strict=True):
        if b > a:
            course += row_actions(m, Plate, color, (a, y, z), b - a)
    in_steps(m, f"{title}, clips", course)
    hinge = _axis(CLIP_BODY, (spots[-1], y - 0.9, z - 0.25), 3)
    sub = m.submodel(
        f"{title}: panel", _panel, length=panel_length, rows=rows, color=color
    )
    turn = Rot(x=-angle, z=180)
    handle = _axis(HANDLE_BODY, (1, -0.8, 0), 3)
    landed = _world(sub, handle, turn)
    at = tuple(hinge[i] - landed[i] for i in range(3))
    with m.step(f"{title}, panel"):
        m.place(sub, at=at, rot=turn)

    # The posts: under the panel's front row, on the first whole stud row inside its front edge, as tall as fits.
    def under(fy):
        p = _world(sub, (0, fy, 0), turn)
        return (at[1] + p[1], at[2] + p[2])

    front_y = under(rows)[0]
    py = math.ceil(front_y - 1e-6)
    (ya, za), (yb, zb) = under(rows), under(rows - 1)
    lowest = za + (zb - za) * (py - ya) / (yb - ya)
    tall = math.floor(lowest - ground - 0.1)
    if tall < 3:
        raise ValueError(
            f"lean_to_awning's posts would be {tall} plates: raise the awning or lower `ground`"
        )
    with m.step(f"{title}, posts"):
        for px in (x, x + length - 1):
            m.stack("3062b", posts, at=(px, py, ground), count=tall // 3)
            if tall % 3:
                m.stack(
                    "6141", posts, at=(px, py, ground + 3 * (tall // 3)), count=tall % 3
                )
    return z + 1


def example(m):
    """A front wall three bricks tall with an awning along it, and the wall going on above."""
    with m.step("Ground"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, -5, 0), size=(8, 6))
    for r in range(3):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Tan",
                at=(0, 0, 1 + 3 * r),
                length=8,
                first=None if r % 2 == 0 else 2,
            )
    top = m.use("lean_to_awning", x=0, y=0, z=10, length=8, depth=3, ground=1)
    with m.step("Wall, row 4"):
        m.row(Brick, "Tan", at=(0, 0, top), length=8)
