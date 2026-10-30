# SPDX-License-Identifier: Apache-2.0
"""Studs-up snow roof: a roof deep in snow, two hinged panels of white plates laid studs up on a dark underlayer,
with a rounded lip of white curved slopes along each eave.

m.use("studs_up_snow_roof", x, y, z, length, depth, color="Dark Green", snow="White", clip_color="Black",
title="Roof") roofs the length x depth rectangle whose front-left corner is (x, y), on walls whose top is at z, on
hinge_roof's frame: clips (4081b) on the front and back walls' top course, and two panels built flat, each turned up
about the handle plates (2540) along its bottom edge until the front one meets the back one, which is wider and
covers the ridge. A panel is a layer of `color` plates (the roof itself, showing as a dark line along the edges),
then a layer of `snow` plates laid studs up (the studs make it soft), with tiles on the top row so the panels close
at the ridge, and a row of curved slopes 2 x 1 (11477) on the eave, their rounded faces down the roof: a lip of
snow two plates thick. length
must be 4 or more and depth 6 or more. It brings its own steps: "<title>, clips", then each panel built apart; and
returns the z of the ridge's top.
"""

import math

from emmet_dsl import Plate, Rot, Tile, frame
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387. Not derived from any
model file."""

PARTS = [
    "4081b",
    "2540",
    "11477",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "3020",
    "3795",
    "3034",
    "3832",
    "3069b",
    "63864",
    "2431",
    "3070b",
]


# Body boxes (LDU, LDraw's frame) and where each part's bar or clip runs: the hinge's axis.
CLIP, CLIP_BODY = "4081b", [-10, -6, -28, 10, 10, 10]
HANDLE, HANDLE_BODY = "2540", [-20, -4, -26, 20, 8, 10]
AXIS = (0, 2, -20)
RUNS = (
    2,
    3,
    4,
    6,
    8,
    10,
    12,
)  # plate lengths 1 and 2 wide both come in, for the columns up a panel
BELOW = 6  # LDU from the hinge's axis down to a panel's underside (its top is 2 + 8 a plate above the axis)


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


def _base(m, length, rows, color):
    """A panel's base, lying flat, in two steps: the bottom row (a 1 x 1 plate, handle plates with their bars toward
    -y at x 1, 3, ..., and plates to the end), then a column of plates 2 wide (one plate each) running up the panel
    from the next row.
    The handles' joints fall on odd x and the columns' on even x, so any cover whose bottom course spans the first
    two rows ties the whole base together, whatever its own joints."""
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


def _angle(span, front_end, above):
    """The tilt (radians) at which a panel reaching `front_end` LDU from its hinge, `above` LDU thick over the axis at
    its top edge, touches the underside of the opposite panel, whose hinge is `span` LDU away at the same height."""

    def gap(t):
        s, c = math.sin(t), math.cos(t)
        top = above if c * c >= s * s else -BELOW
        return -span * s + front_end * math.sin(2 * t) + top * math.cos(2 * t) + BELOW

    lo, hi = math.radians(10), math.radians(70)
    if gap(lo) * gap(hi) > 0:
        return None
    for _ in range(60):
        mid = (lo + hi) / 2
        if gap(lo) * gap(mid) <= 0:
            hi = mid
        else:
            lo = mid
    return (lo + hi) / 2


def _reach(span, front_end, above, t):
    """How far up the back panel, from its hinge, the front panel's top edge touches it (LDU)."""
    s, c = math.sin(t), math.cos(t)
    top = above if c * c >= s * s else -BELOW
    return span * c - front_end * math.cos(2 * t) + top * math.sin(2 * t)


def _gable(m, x, y, z, length, depth, panel, above, clip_color, title, **kw):
    """hinge_roof's frame with another panel: clips on the front and back walls' top course, and two panels built
    flat by panel(s, length, rows, side, **kw) and turned up about their handles until the front one's top edge
    meets the back one, which is wide enough to cover the ridge. `above` is how many LDU the front panel's top edge
    stands over the hinge axis. Returns the z of the ridge's top."""
    if length < 4 or depth < 8:
        raise ValueError(
            f"needs a length of 4 or more and a depth of 8 or more, not {length} x {depth}"
        )
    panel_length = (
        length + 1
    )  # from x - 0.5, so the handles (at panel x 1, 3, ...) center on studs
    handles = range(1, panel_length - 2, 2)
    front = [(x + hx, y, z - 0.25) for hx in handles]
    back = [(x + length - 1 - hx, y + depth - 1.9, z - 0.25) for hx in handles]
    hinge_f, hinge_b = _axis(CLIP_BODY, front[0], 1), _axis(CLIP_BODY, back[0], 3)
    span = (hinge_b[1] - hinge_f[1]) * frame.STUD
    # As hinge_roof: the fewest front rows that meet the back panel at 45 degrees or more (one fewer past 55), and a
    # back panel wide enough to cover the ridge.
    # Only row counts whose columns are each one plate (1 x 3, 2 x 4, ... 2 x 16 run up the panel).
    fits = [
        (n, _angle(span, 10 + frame.STUD * n, above(n)))
        for n in range(3, depth)
        if n - 1 in RUNS
    ]
    fits = [(n, t) for n, t in fits if t is not None]
    if not fits:
        raise ValueError(f"can't fit hinged panels over a depth of {depth}")
    k = next(
        (i for i, (_, t) in enumerate(fits) if t >= math.radians(45)), len(fits) - 1
    )
    if k and fits[k][1] > math.radians(55):
        k -= 1
    rows, t = fits[k]
    back_rows = rows + 1
    while (
        10 + frame.STUD * back_rows
        < _reach(span, 10 + frame.STUD * rows, above(rows), t) + 4
    ):
        back_rows += 1
    while back_rows - 1 not in RUNS:
        back_rows += 1
    degrees = math.degrees(t)
    in_steps(
        m,
        f"{title}, clips",
        [(1, lambda c=c: m.add(CLIP, clip_color, at=c, rot=3)) for c in back]
        + [(1, lambda c=c: m.add(CLIP, clip_color, at=c, rot=1)) for c in front],
    )
    handle = _axis(
        HANDLE_BODY, (1, -0.8, 0), 3
    )  # the first handle's axis, in a panel's own frame
    top = z
    for side, n, hinge, turn in (
        ("front", rows, hinge_f, Rot(x=degrees)),
        ("back", back_rows, hinge_b, Rot(x=degrees, z=180)),
    ):
        sub = m.submodel(
            f"{title}: {side} panel",
            panel,
            length=panel_length,
            rows=n,
            side=side,
            **kw,
        )
        landed = _world(sub, handle, turn)
        at = tuple(hinge[i] - landed[i] for i in range(3))
        with m.step(f"{title}, {side} panel"):
            placed = m.place(sub, at=at, rot=turn)
        top = max(top, placed.max[2])
    return top


def _panel(m, length, rows, side, color, snow):
    """A panel lying flat: the base in the roof color, then the snow: a 2-deep row of plates along the eave (it ties
    the base together), plates up the panel, tiles on the top row, and the curved slopes on the eave row."""
    _base(m, length, rows, color)
    in_steps(
        m,
        "Snow",
        row_actions(m, Plate, snow, (0, 0, 1), length, width=2)
        + [
            a
            for ry in range(2, rows - 1)
            for a in row_actions(m, Plate, snow, (0, ry, 1), length)
        ]
        + row_actions(m, Tile, snow, (0, rows - 1, 1), length),
    )
    # rot 3 points a slope down toward -y: the eave.
    in_steps(
        m,
        "Snow at the eave",
        [
            (1, lambda ex=ex: m.add("11477", snow, at=(ex, 0, 2), rot=3))
            for ex in range(length)
        ],
    )


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    color="Dark Green",
    snow="White",
    clip_color="Black",
    title="Roof",
):
    """The clips and the two snowy panels; returns the z of the ridge's top."""
    return _gable(
        m,
        x,
        y,
        z,
        length,
        depth,
        _panel,
        lambda rows: 10,
        clip_color,
        title,
        color=color,
        snow=snow,
    )


def example(m):
    """A snowy roof on four walls, 8 x 8 and two bricks tall."""
    for row in range(2):
        for side in ("back", "sides", "front"):
            with m.step(f"Walls, row {row + 1}: {side}"):
                m.wall(
                    0, 0, 3 * row, length=8, depth=8, sides=side, color="Reddish Brown"
                )
    m.use("studs_up_snow_roof", x=0, y=0, z=6, length=8, depth=8)
