# SPDX-License-Identifier: Apache-2.0
"""Solar panel: a pair of solar panels on a boom, a bar out of a station's side with each panel clipped onto it, so
they turn together to face the sun.

m.use("solar_panel", x, y, z, width=4, tilt=0, color="Dark Bluish Gray", cells="Trans-Dark Blue", boom_color="Light
Bluish Gray", title="Solar array") puts a brick 1 x 1 with a stud on one side (87087) at (x, y, z), its stud facing
right (+x), a round brick 1 x 1 (3062b) on its side on that stud, and a bar 6L with a stop (63965a) in the round
brick's hollow stud as the boom, running along x and reaching to x + 7.6, 1.75 plates above z. Two panels, each built
apart as a sub-assembly, hang on the boom from x + 3 to x + 7: a plate `width` × 4 with 1 x 2 tiles in `cells` over
it, on plates 1 x 1 with a clip (4081b) under its edge, the clips of one panel between the other's. One panel reaches
`width` studs (plus 1 for the clips) to the back of the boom and the other the same to the front; both turn `tilt`
degrees about the boom (0 is flat, 90 stands them on edge; a positive tilt lowers the far edge of the back panel and
raises the front one's), the clips holding them there. width is 2, 4, or 6. It brings its own steps and returns the
(x, y, z) of the boom's far end.
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["87087", "3062b", "63965a", "4081b", "3020", "3031", "3032", "3069b"]

# A panel's plate by its width, and the quarter turn that runs its 4-stud side along x.
PLATES = {2: ("3020", 1), 4: ("3031", 0), 6: ("3032", 0)}
# Body box of the clip (LDU, LDraw's frame) and where the bar it holds runs.
CLIP_BODY = [-10, -6, -28, 10, 10, 10]
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


def _clip(side, c):
    """Where a panel's clip goes in its own frame, and its quarter turn: under its row nearest the boom, which runs
    along x at y = 0 (a back panel reaches +y from it, a front one -y), the clip pointing at the boom."""
    return ((c, -0.9, -0.25), 3) if side == "back" else ((c, -1, -0.25), 1)


def _panel(m, width, side, first, color, cells):
    """A panel lying flat in its own frame, its cells up: clips at x = first, first + 2, then the plate and its
    tiles."""
    y0 = 0 if side == "back" else -width
    with m.step("Clips"):
        for c in range(first, 4, 2):
            at, rot = _clip(side, c)
            m.add("4081b", color, at=at, rot=rot)
    with m.step("Plate"):
        part, rot = PLATES[width]
        m.add(part, color, at=(0, y0, 1), rot=rot)
    for row in range(0, width, 4):
        with m.step(f"Cells, rows {row + 1} to {min(row + 4, width)}"):
            for r in range(row, min(row + 4, width)):
                for c in (0, 2):
                    m.add("3069b", cells, at=(c, y0 + r, 2), rot=1)


def build(
    m,
    x,
    y,
    z,
    width=4,
    tilt=0,
    color="Dark Bluish Gray",
    cells="Trans-Dark Blue",
    boom_color="Light Bluish Gray",
    title="Solar array",
):
    """The mount, the boom, and the two panels; returns the boom's far end."""
    if width not in (2, 4, 6):
        raise ValueError(f"solar_panel takes a width of 2, 4, or 6, not {width}")
    if not -90 <= tilt <= 90:
        raise ValueError(f"solar_panel takes a tilt from -90 to 90, not {tilt}")
    boom = (y + 0.5, z + 1.75)  # the boom's line (y, z)
    with m.step(f"{title}: boom"):
        m.add("87087", color, at=(x, y, z), rot=0)
        m.add("3062b", boom_color, at=(x + 1, y, z + 0.5), rot=Rot(y=90))
        # The bar's short end goes in the hollow stud, which ends at x + 2.4, up to its stop.
        r = frame.to_ldraw_rotation(frame.rotation(0, 90, 0))
        p = frame.to_ldraw_point((x + 2.4, *boom))
        m.add_raw("63965a", boom_color, [*p, *r[0], *r[1], *r[2]])
    # The two panels are mirror images, not one turned over, so both keep their cells up; they turn together.
    rot = Rot(x=-tilt)
    for side, first in (("back", 0), ("front", 1)):
        sub = m.submodel(
            f"{title}: {side} panel",
            _panel,
            width=width,
            side=side,
            first=first,
            color=color,
            cells=cells,
        )
        landed = _world(sub, _axis(CLIP_BODY, *_clip(side, first)), rot)
        hinge = (x + 3 + first, *boom)
        with m.step(f"{title}: {side} panel"):
            m.place(sub, at=tuple(hinge[i] - landed[i] for i in range(3)), rot=rot)
    return (x + 7.6, *boom)


def example(m):
    """A solar array on the side of a station's column, tilted toward the sun."""
    with m.step("Column"):
        m.add("3003", "Light Bluish Gray", at=(0, 0, 0))
        m.add("3003", "Light Bluish Gray", at=(0, 0, 3))
    m.use("solar_panel", x=1, y=0, z=6, tilt=-25)
