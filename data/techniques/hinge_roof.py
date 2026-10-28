# SPDX-License-Identifier: Apache-2.0
"""Hinge roof: two smooth roof panels on clip hinges, tilted up from the front and back walls to meet at the ridge.

m.use("hinge_roof", x, y, z, length, depth, color="Dark Red", clip_color="Black", title="Roof") roofs the length ×
depth rectangle whose front-left corner is (x, y), on walls whose top is at z. Clips (plates 1 x 1 with a clip,
4081b) go on the top course of the front and back walls, every other stud, clear of the corners; each holds the
handle (the bar) of a plate 1 x 2 with a handle (2540) along the bottom edge of a panel. A panel is a layer of plates
under a layer of tiles, one stud longer than the walls (half a stud over each end, which centers each handle on its
clip), built flat as a submodel and placed turned up about its hinge. The angle is worked out so the front panel's
edge meets the underside of the back one, which is a row or more wider and covers the ridge: 45° to 55° for most depths
(49.5° for 8; 38.6° for 6). The clips hold the panels at that angle, and the panels brace each other at the ridge.
(The hinge plates 1 x 4, 2429 and 2430, turn about an upright pin, flat: they angle walls, not roofs.)
The clips reach a stud into the room and a quarter plate below the walls' top, so keep a ceiling clear of that strip.
length must be 4 or more and depth 6 or more. It brings its own steps: "<title>, clips", then the two panels, each
built apart; and returns the z of the ridge's top.
"""

import math

from emmet_dsl import Plate, Rot, Tile, frame
from emmet_dsl.core import in_steps

PARTS = [
    "4081b",
    "2540",
    "3024",
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
    "4282",
    "3070b",
    "3069b",
    "63864",
    "2431",
    "6636",
    "4162",
    "3068b",
    "87079",
]

# Body boxes (LDU, LDraw's frame) and where each part's bar or clip runs: the hinge's axis.
CLIP, CLIP_BODY = "4081b", [-10, -6, -28, 10, 10, 10]
HANDLE, HANDLE_BODY = "2540", [-20, -4, -26, 20, 8, 10]
AXIS = (0, 2, -20)
BELOW, ABOVE = (
    6,
    10,
)  # LDU from the hinge's axis down to a panel's underside, and up to its tiles' top


def _axis(body, at, rot):
    """Where the hinge axis of a part placed at `at`, rot (quarter turns) runs through, in DSL units."""
    pos, r, _ = frame.place(body, at, frame.quarter_turns(rot))
    w = [sum(r[i][k] * AXIS[k] for k in range(3)) + pos[i] for i in range(3)]
    return (-w[2] / frame.STUD, w[0] / frame.STUD, -w[1] / frame.PLATE)


def _panel(m, length, rows, color):
    """A panel lying flat, its handles along its front edge (toward −y) at x 1, 3, …: a hinge row of handle plates
    and plates, the rest of the plates, then tiles across them. Built in steps of the step target."""
    handles = list(range(1, length - 2, 2))
    hinge = [(1, lambda: m.add("3024", color, at=(0, 0, 0)))]
    hinge += [
        (1, lambda hx=hx: m.add(HANDLE, color, at=(hx, -0.8, 0), rot=3))
        for hx in handles
    ]
    rest = length - (handles[-1] + 2)
    hinge.append(
        (
            1,
            lambda: m.row(Plate, color, at=(length - rest, 0, 0), length=rest),
        )
    )
    in_steps(m, "Hinge row", hinge)
    if rows > 1:
        in_steps(m, "Plates", _fill(m, Plate, color, (0, 1, 0), (length, rows - 1)))
    in_steps(m, "Tiles", _fill(m, Tile, color, (0, 0, 1), (length, rows)))


def _fill(m, kind, color, at, size):
    """m.fill's strips (two wide along the long side, one wide for an odd rest) as in_steps actions."""
    (x, y, z), (w, d) = at, size
    axis = "x" if w >= d else "y"
    long_side, short_side = (w, d) if axis == "x" else (d, w)
    actions, done = [], 0
    while done < short_side:
        width = 2 if short_side - done >= 2 else 1
        start = (x, y + done, z) if axis == "x" else (x + done, y, z)
        pieces = len(kind.split(long_side, width, None, None, "hinge_roof.py", 0))

        def strip(start=start, width=width):
            m.row(kind, color, at=start, length=long_side, axis=axis, width=width)

        actions.append((pieces, strip))
        done += width
    return actions


def _angle(span, front_end):
    """The tilt (radians) at which a panel reaching `front_end` LDU from its hinge touches, with its top edge, the
    underside of the opposite panel, whose hinge is `span` LDU away at the same height."""

    def gap(t):
        s, c = math.sin(t), math.cos(t)
        top = (
            ABOVE if c * c >= s * s else -BELOW
        )  # the end's corner that reaches furthest
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


def _reach(span, front_end, t):
    """How far up the back panel, from its hinge, the front panel's top edge touches it (LDU)."""
    s, c = math.sin(t), math.cos(t)
    top = ABOVE if c * c >= s * s else -BELOW
    return span * c - front_end * math.cos(2 * t) + top * math.sin(2 * t)


def _world(sub, at_axis_local, turn):
    """Where a point of a submodel (DSL units, its own frame) lands when the submodel is placed at (0, 0, 0)."""
    pos, r, _ = frame.place(
        frame.box_in_ldraw(*sub.box), (0, 0, 0), frame.rotation(turn.x, turn.y, turn.z)
    )
    p = frame.to_ldraw_point(at_axis_local)
    w = [sum(r[i][k] * p[k] for k in range(3)) + pos[i] for i in range(3)]
    return (-w[2] / frame.STUD, w[0] / frame.STUD, -w[1] / frame.PLATE)


def build(
    m, x, y, z, length, depth, color="Dark Red", clip_color="Black", title="Roof"
):
    """The clips and the two panels; returns the z of the ridge's top."""
    if length < 4 or depth < 6:
        raise ValueError(
            f"hinge_roof needs a length of 4 or more and a depth of 6 or more, not {length} × {depth}"
        )
    panel_length = (
        length + 1
    )  # from x − 0.5, so the handles (at panel x 1, 3, …) center on studs
    handles = range(1, panel_length - 2, 2)
    # Each clip on the stud its handle centers on: the front ones as the panel lies, the back ones mirrored (the
    # back panel is the same, turned around). They point into the room, from the walls' top course.
    front = [(x + hx, y, z - 0.25) for hx in handles]
    back = [(x + length - 1 - hx, y + depth - 1.9, z - 0.25) for hx in handles]
    hinge_f, hinge_b = _axis(CLIP_BODY, front[0], 1), _axis(CLIP_BODY, back[0], 3)
    span = (hinge_b[1] - hinge_f[1]) * frame.STUD

    # The front panel: the fewest rows that meet the back panel at 45° or more, where the two panels' undersides
    # meet on the middle line (shallower, they meet short of it); one row fewer if that's steeper than 55°. The back
    # panel is a row wider, or more if the front one's edge would touch it past its end, and covers the ridge.
    fits = [(n, _angle(span, 10 + frame.STUD * n)) for n in range(1, depth)]
    fits = [(n, t) for n, t in fits if t is not None]
    if not fits:
        raise ValueError(f"hinge_roof can't fit panels over a depth of {depth}")
    k = next(
        (i for i, (_, t) in enumerate(fits) if t >= math.radians(45)), len(fits) - 1
    )
    if k and fits[k][1] > math.radians(55):
        k -= 1
    fit = fits[k]
    rows, t = fit
    back_rows = rows + 1
    while 10 + frame.STUD * back_rows < _reach(span, 10 + frame.STUD * rows, t) + 4:
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
            f"{title}: {side} panel", _panel, length=panel_length, rows=n, color=color
        )
        landed = _world(sub, handle, turn)
        at = tuple(hinge[i] - landed[i] for i in range(3))
        with m.step(f"{title}, {side} panel"):
            placed = m.place(sub, at=at, rot=turn)
        top = max(top, placed.max[2])
    return top


def example(m):
    """A hinged roof on four walls, 8 × 8 and two bricks tall."""
    for row in range(2):
        for side in ("back", "sides", "front"):
            with m.step(f"Walls, row {row + 1}: {side}"):
                m.wall(0, 0, 3 * row, length=8, depth=8, sides=side, color="White")
    m.use("hinge_roof", x=0, y=0, z=6, length=8, depth=8)
