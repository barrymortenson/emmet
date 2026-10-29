# SPDX-License-Identifier: Apache-2.0
"""Radar dish: a dish on a mast, tilted on a clip at the mast's top that holds its carrier plate's handle.

m.use("radar_dish", x, y, z, tilt=45, height=2, color="Light Bluish Gray", dish_color="Light Bluish Gray",
title="Radar") stands a mast of `height` round bricks 1 x 1 (3062b) on the stud at (x, y, z), puts a plate 1 x 1 with
a clip (4081b) on its top, the clip facing front (-y), and hangs the dish in it, built apart as a sub-assembly: a
plate 1 x 2 with a handle (2540), its handle in the clip, a jumper plate on it, and a dish 4 x 4 (3960) on the
jumper's stud. The carrier is turned down by `tilt` degrees about the clip (0 would lie flat in front of the mast with
the dish facing up; 90 hangs it straight down, the dish facing the front); the clip holds any angle. tilt is 20 to 90
and height 1 to 6; the dish's lower edge reaches about 4 plates below the mast's top at 30°, 6 at 60°, and 7 at 90°,
so keep the mast taller than that. It brings its own steps and returns the z of the mast's top.
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3062b", "4081b", "2540", "3794b", "3960"]

# Body boxes (LDU, LDraw's frame) and the hinge axis, the clip's and the handle's, which is at the same point of both.
CLIP_BODY = [-10, -6, -28, 10, 10, 10]
HANDLE_BODY = [-20, -4, -26, 20, 8, 10]
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


def _carrier(m, color, dish_color):
    """The carrier lying flat in its own frame: the handle plate along x, its handle toward +y, the dish over it."""
    with m.step("Carrier"):
        m.add("2540", color, at=(0, 0, 0), rot=1)
        m.add("3794b", color, at=(0, 0, 1), rot=1)
    with m.step("Dish"):
        m.add("3960", dish_color, at=(-1, -1.5, 2))


def build(
    m,
    x,
    y,
    z,
    tilt=45,
    height=2,
    color="Light Bluish Gray",
    dish_color="Light Bluish Gray",
    title="Radar",
):
    """The mast, the clip, and the tilted dish; returns the z of the mast's top."""
    if not 20 <= tilt <= 90:
        raise ValueError(f"radar_dish takes a tilt of 20 to 90 degrees, not {tilt}")
    if not 1 <= height <= 6:
        raise ValueError(f"radar_dish takes a height of 1 to 6 bricks, not {height}")
    top = z + 3 * height
    with m.step(f"{title}: mast"):
        m.stack("3062b", color, at=(x, y, z), count=height)
        clip_at = (x, y - 0.9, top - 0.25)
        m.add("4081b", color, at=clip_at, rot=3)
    hinge = _axis(CLIP_BODY, clip_at, 3)
    sub = m.submodel(f"{title}: dish", _carrier, color=color, dish_color=dish_color)
    turn = Rot(x=tilt)
    landed = _world(sub, _axis(HANDLE_BODY, (0, 0, 0), 1), turn)
    with m.step(f"{title}: dish"):
        m.place(sub, at=tuple(hinge[i] - landed[i] for i in range(3)), rot=turn)
    return top


def example(m):
    """A radar dish on a short mast on a deck."""
    with m.step("Deck"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("radar_dish", x=1, y=2, z=1, tilt=60)
