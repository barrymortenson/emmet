# SPDX-License-Identifier: Apache-2.0
"""Hatch on hinge: a lid over an opening in a raised floor, on hinge bricks along its back edge, shut flush or swung
open.

m.use("hatch_on_hinge", x, y, z, width=2, depth=2, angle=0, color="Light Bluish Gray", hinge_color="Dark Bluish Gray",
title="Hatch") covers the opening `width` × `depth` studs whose front-left corner is (x, y), in a floor whose bricks
start at z, with a hatch hinged along the row behind it (y + depth): hinge bases 1 x 2 (3937) on that row, a brick
tall, and on each a hinge top 2 x 2 (6134) reaching forward over the opening's back row. The lid, built flat as a
sub-assembly, is the hinge tops, plates over them and the opening in bands along x, and tiles over the plates running
front to back, which tie the bands together; it goes on turned up by `angle` degrees (0 is shut, 90 stands straight
up). Shut, its top is 2 plates above the floor's bricks, so tile the floor around it on a plate to match. width is
even, 2 to 8, and depth 1 to 6. It brings its own steps and returns the z of the lid's top when shut (z + 5).
"""

from emmet_dsl import Plate, Rot, Tile, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = [
    "3937",
    "6134",
    "3023b",
    "3710",
    "3666",
    "3460",
    "3022",
    "3020",
    "3795",
    "3034",
    "3069b",
    "3068b",
    "26603",
    "87079",
    "69729",
]

# Body boxes (LDU, LDraw's frame); both halves turn about the line through (0, 10, 0) along their own X.
BASE_BODY = [-20, 2, -10, 20, 24, 10]
AXIS = (0, 10, 0)


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


def _lid(m, width, depth, color, hinge_color):
    """The lid lying shut, in its own frame: the hinge tops on the back rows (y = depth - 1 to depth + 1), then plates
    over y = 0 to depth + 1 in bands along x, the first over the hinge tops, and tiles running front to back, so each
    one ties every band to the first."""
    with m.step("Hinge tops"):
        for c in range(0, width, 2):
            m.add("6134", hinge_color, at=(c, depth - 1, 0), rot=1)
    for k, back in enumerate(range(depth + 1, 0, -2)):
        rows = min(2, back)
        with m.step(f"Plates, band {k + 1}"):
            m.row(Plate, color, at=(0, back - rows, 2), length=width, width=rows)
    for c in range(0, width, 4):
        with m.step(f"Tiles, from {c + 1}"):
            for cc in range(c, min(c + 4, width), 2):
                m.row(Tile, color, at=(cc, 0, 3), length=depth + 1, axis="y", width=2)


def build(
    m,
    x,
    y,
    z,
    width=2,
    depth=2,
    angle=0,
    color="Light Bluish Gray",
    hinge_color="Dark Bluish Gray",
    title="Hatch",
):
    """The hinge bases and the lid; returns the z of the lid's top when shut."""
    if width < 2 or width > 8 or width % 2:
        raise ValueError(f"hatch_on_hinge takes an even width from 2 to 8, not {width}")
    if not 1 <= depth <= 6:
        raise ValueError(f"hatch_on_hinge takes a depth of 1 to 6, not {depth}")
    if not 0 <= angle <= 90:
        raise ValueError(f"hatch_on_hinge opens from 0 to 90 degrees, not {angle}")
    with m.step(f"{title}: hinge bases"):
        for c in range(0, width, 2):
            m.add("3937", hinge_color, at=(x + c, y + depth, z), rot=1)
    hinge = _axis(BASE_BODY, (x, y + depth, z), 1)
    sub = m.submodel(
        f"{title}: lid",
        _lid,
        width=width,
        depth=depth,
        color=color,
        hinge_color=hinge_color,
    )
    turn = Rot(x=-angle)
    # The lid's own hinge axis: the first hinge top's, the same point as the base's when shut.
    landed = _world(sub, (hinge[0] - x, hinge[1] - y, hinge[2] - z - 1), turn)
    with m.step(f"{title}: lid"):
        m.place(sub, at=tuple(hinge[i] - landed[i] for i in range(3)), rot=turn)
    return z + 5


def example(m):
    """A hatch in a raised floor, standing open."""
    from emmet_dsl import Brick

    with m.step("Base"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
    with m.step("Floor bricks"):
        m.row(Brick, "Dark Bluish Gray", at=(0, 0, 1), length=4)
        m.add("3004", "Dark Bluish Gray", at=(0, 1, 1), rot=0)
        m.add("3004", "Dark Bluish Gray", at=(3, 1, 1), rot=0)
        m.add("3005", "Dark Bluish Gray", at=(0, 3, 1))
        m.add("3005", "Dark Bluish Gray", at=(3, 3, 1))
    m.use("hatch_on_hinge", x=1, y=1, z=1, angle=60)
    with m.step("Floor top"):
        m.add("3710", "Dark Bluish Gray", at=(0, 0, 4), rot=1)
        m.add("3623", "Dark Bluish Gray", at=(0, 1, 4), rot=0)
        m.add("3623", "Dark Bluish Gray", at=(3, 1, 4), rot=0)
    with m.step("Floor tiles"):
        m.add("2431", "Light Bluish Gray", at=(0, 0, 5), rot=1)
        m.add("63864", "Light Bluish Gray", at=(0, 1, 5), rot=0)
        m.add("63864", "Light Bluish Gray", at=(3, 1, 5), rot=0)
