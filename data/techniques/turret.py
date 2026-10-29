# SPDX-License-Identifier: Apache-2.0
"""Turret: a gun turret on a 2 x 2 turntable, its head built as one unit so it turns to aim, with barrels on side
studs.

m.use("turret", x, y, z, aim=0, guns=2, color="Light Bluish Gray", gun_color="Dark Bluish Gray", base_color="Dark
Bluish Gray", title="Turret") puts a turntable base 2 x 2 (3680) on the studs at z from (x, y), and on it the head,
built apart as a sub-assembly: the turntable's top (3679), a plate 2 x 2, a course of two bricks with a stud on one
side (87087) at the front and a brick 1 x 2 behind them, and a 2 x 2 slope on top. Each gun is a 1 x 1 round plate
with an open stud (85861) on a side stud, with a bar 3L (87994) in it as the barrel; guns is 1 or 2. The head is
placed turned by `aim` degrees about the turntable's center, counterclockwise from facing front (-y); the barrels
reach 3.2 studs out from the head. It brings its own steps and returns the z of the head's top (z + 6).
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3680", "3679", "3022", "87087", "3004", "3005", "85861", "87994", "3039"]

OUT = Rot(x=90)  # a round plate or a bar lying along y, its top to -y


def _head(m, guns, color, gun_color):
    """The head facing front, in its own frame: the turntable's top at (0.125, 0.125, 0), the guns toward -y."""
    with m.step("Turntable top and plate"):
        m.add("3679", color, at=(0.125, 0.125, 0))
        m.add("3022", color, at=(0, 0, 1))
    with m.step("Gun mounts"):
        for c in range(2):
            if c < guns:
                m.add("87087", color, at=(c, 0, 2), rot=3)
            else:
                m.add("3005", color, at=(c, 0, 2))
        m.add("3004", color, at=(0, 1, 2), rot=1)
    with m.step("Guns"):
        for c in range(guns):
            m.add("85861", gun_color, at=(c, -0.4, 2.5), rot=OUT)
            m.add("87994", gun_color, at=(c + 0.3, -3.2, 3.25), rot=OUT)
    with m.step("Top"):
        m.add("3039", color, at=(0, 0, 5), rot=3)


def build(
    m,
    x,
    y,
    z,
    aim=0,
    guns=2,
    color="Light Bluish Gray",
    gun_color="Dark Bluish Gray",
    base_color="Dark Bluish Gray",
    title="Turret",
):
    """The turntable base and the head on it; returns the z of the head's top."""
    if guns not in (1, 2):
        raise ValueError(f"turret takes guns=1 or 2, not {guns}")
    with m.step(f"{title}: turntable base"):
        m.add("3680", base_color, at=(x, y, z))
    sub = m.submodel(
        f"{title}: head", _head, guns=guns, color=color, gun_color=gun_color
    )
    turn = Rot(z=aim)
    # Put the head's own turntable center (1, 1, 0) on the base's.
    pos, r, _ = frame.place(
        frame.box_in_ldraw(*sub.box), (0, 0, 0), frame.rotation(0, 0, aim)
    )
    p = frame.to_ldraw_point((1, 1, 0))
    w = [sum(r[i][k] * p[k] for k in range(3)) + pos[i] for i in range(3)]
    landed = (-w[2] / frame.STUD, w[0] / frame.STUD, -w[1] / frame.PLATE)
    with m.step(f"{title}: head"):
        m.place(
            sub,
            at=(x + 1 - landed[0], y + 1 - landed[1], z - landed[2]),
            rot=turn,
        )
    return z + 6


def example(m):
    """A turret on a deck, aimed a little to the right."""
    with m.step("Deck"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("turret", x=1, y=1, z=1, aim=30)
