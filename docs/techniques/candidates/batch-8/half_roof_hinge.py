# SPDX-License-Identifier: Apache-2.0
"""Half roof hinge: a roof of two panels whose back half lifts like a lid on a hinge at the ridge, to show the attic,
while the front half stays fixed on the front wall.

m.use("half_roof_hinge", x, y, z, length, depth, open=0, color="Dark Bluish Gray", clip_color="Black", title="Roof")
roofs the length x depth rectangle whose front-left corner is (x, y), on walls whose top is at z. The front panel
hangs on clips (4081b) on the front wall's top course by the handle plates (2540) along its bottom edge, as in
hinge_roof, tipped up until its top edge reaches the middle of the rectangle; along that top edge it carries clips of
its own, reaching past it. The back panel hangs from those by the handle plates along its top edge and, shut
(open=0), rests on the back wall's top; `open` lifts it that many degrees about the ridge. Nothing holds it to the
back wall, so it swings up and back down. Each panel is a base of plates and a cover of tiles; the back panel sits
half a stud along from the front one, so its handles center on the front one's clips. length must be 4 or more and
depth 8 or more. It brings its own steps: "<title>, clips", then each panel built apart; and returns the z of the
ridge's top.
"""

import math

from emmet_dsl import Plate, Rot, Tile, frame
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217. Not derived from any
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
    "3068b",
    "87079",
    "3069b",
    "3070b",
    "63864",
    "2431",
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


def _tilt(origin, point, about, degrees):
    """Where `point` (a panel's own frame) lands when the panel turns `degrees` about x (its +y end up) with `about`
    (its own frame) on `origin`: (y, z) in DSL units."""
    t = math.radians(degrees)
    dy, dz = (point[1] - about[1]) * frame.STUD, (point[2] - about[2]) * frame.PLATE
    return (
        origin[1] + (dy * math.cos(t) - dz * math.sin(t)) / frame.STUD,
        origin[2] + (dy * math.sin(t) + dz * math.cos(t)) / frame.PLATE,
    )


def _solve(f, lo, hi):
    """The angle in degrees between lo and hi where f crosses zero, or None."""
    if f(lo) * f(hi) > 0:
        return None
    for _ in range(60):
        mid = (lo + hi) / 2
        if f(lo) * f(mid) <= 0:
            hi = mid
        else:
            lo = mid
    return (lo + hi) / 2


def _panel(m, length, rows, color, clips, clip_color):
    """A panel lying flat: the hinge row (a 1 x 1 plate, handle plates with their bars toward -y at x 1, 3, ..., and
    plates), a column of plates 2 wide up the panel from the next row, a 2-deep row of tiles along the hinge edge that
    ties them, tiles over the rest, and on the top row, with `clips`, clips reaching past the top edge at those x
    and plates between."""
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
    top = rows - 1 if clips else rows
    in_steps(
        m,
        "Tiles",
        row_actions(m, Tile, color, (0, 0, 1), length, width=2)
        + [
            a
            for ry in range(2, top)
            for a in row_actions(m, Tile, color, (0, ry, 1), length)
        ],
    )
    if clips:
        actions = [
            (1, lambda c=c: m.add(CLIP, clip_color, at=(c, rows - 1, 0.75), rot=1))
            for c in clips
        ]
        for a, b in zip([0] + [c + 1 for c in clips], clips + [length], strict=True):
            if b > a:
                actions += row_actions(m, Plate, color, (a, rows - 1, 1), b - a)
        in_steps(m, "Ridge clips", actions)


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    open=0,
    color="Dark Bluish Gray",
    clip_color="Black",
    title="Roof",
):
    """The front clips and the two panels; returns the z of the ridge's top."""
    if length < 4 or depth < 8:
        raise ValueError(
            f"half_roof_hinge needs a length of 4 or more and a depth of 8 or more, not {length} x {depth}"
        )
    panel_length = (
        length + 1
    )  # from x - 0.5, so the handles (at panel x 1, 3, ...) center on studs
    handles = list(range(1, panel_length - 2, 2))
    hinge = _axis(CLIP_BODY, (x + handles[0], y, z - 0.25), 1)
    handle = _axis(
        HANDLE_BODY, (1, -0.8, 0), 3
    )  # the first handle's axis, in a panel's own frame
    mid = y + depth / 2

    # The front panel: the rows whose ridge clips reach the middle nearest 45 degrees.
    fits = []
    for n in (r + 1 for r in RUNS):
        ridge = _axis(CLIP_BODY, (handles[0], n - 1, 0.75), 1)
        t = _solve(
            lambda d, ridge=ridge: _tilt(hinge, ridge, handle, d)[0] - mid, 15, 75
        )
        if t is not None:
            fits.append((abs(t - 45), n, t, ridge))
    if not fits:
        raise ValueError(
            f"half_roof_hinge can't fit a front panel over a depth of {depth}"
        )
    _, rows, tilt, ridge = min(fits)
    axis = (hinge[0], *_tilt(hinge, ridge, handle, tilt))

    # The back panel, hung from the ridge clips and turned down until its far edge's underside meets the back wall's
    # top: the fewest rows that reach the back wall's middle.
    back = None
    for n in (r + 1 for r in RUNS):
        far = (0, n, 0)
        b = _solve(
            lambda d, far=far: _tilt(axis, far, handle, -d)[1] - (z + 0.05), 0, 89
        )
        if b is not None and _tilt(axis, far, handle, -b)[0] >= y + depth - 0.5:
            back = (n, b)
            break
    if back is None:
        raise ValueError(
            f"half_roof_hinge can't fit a back panel over a depth of {depth}"
        )
    back_rows, shut = back

    in_steps(
        m,
        f"{title}, clips",
        [
            (1, lambda hx=hx: m.add(CLIP, clip_color, at=(x + hx, y, z - 0.25), rot=1))
            for hx in handles
        ],
    )
    top = z
    for side, n, clips, turn, at_axis in (
        ("front", rows, handles, Rot(x=tilt), hinge),
        ("back", back_rows, None, Rot(x=-(shut - open)), axis),
    ):
        sub = m.submodel(
            f"{title}: {side} panel",
            _panel,
            length=panel_length,
            rows=n,
            color=color,
            clips=clips,
            clip_color=clip_color,
        )
        landed = _world(sub, handle, turn)
        # The back panel sits half a stud along, so its handles center on the ridge clips.
        shift = -0.5 if side == "back" else 0
        at = (
            at_axis[0] - landed[0] + shift,
            at_axis[1] - landed[1],
            at_axis[2] - landed[2],
        )
        with m.step(f"{title}, {side} panel"):
            placed = m.place(sub, at=at, rot=turn)
        top = max(top, placed.max[2])
    return top


def example(m):
    """Four walls 8 x 8, two bricks tall, with the back half of the roof lifted open."""
    for row in range(2):
        for side in ("back", "sides", "front"):
            with m.step(f"Walls, row {row + 1}: {side}"):
                m.wall(0, 0, 3 * row, length=8, depth=8, sides=side, color="Yellow")
    m.use("half_roof_hinge", x=0, y=0, z=6, length=8, depth=8, open=50)
