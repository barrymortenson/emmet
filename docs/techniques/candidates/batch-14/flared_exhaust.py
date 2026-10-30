# SPDX-License-Identifier: Apache-2.0
"""Flared exhaust: a big round rear exhaust or jet nozzle with a flared, petaled rim, eight curved slopes standing out
round a core on bricks with studs on four sides, the second four turned 45° on a single stud.

m.use("flared_exhaust", x, y, z, color="Dark Bluish Gray", petal="Pearl Silver", core="Black", title="Exhaust") builds a
nozzle as a submodel along its own axis: a round plate 2 x 2 and two round bricks 2 x 2 (3941), a plate with one stud
(87580), then a brick 1 x 1 with studs on four sides (4733) with a curved slope 2 x 1 (11477) on each side stud, its
thick end out to the mouth, a round brick 1 x 1, and a second brick with studs on four sides turned 45° on that one stud
with four more slopes, capped with a round tile. Then it stands it on a bracket 1 x 2 - 2 x 2 up (99207) at (x, y, z),
the bracket's face to +x, so the nozzle points out the rear (+x) from the bracket's 2 x 2 studs. The nozzle reaches
about 8 studs out from x + 1.2 and 1.3 studs round its axis, which is 1 stud in from y and 2.5 plates above z. Returns
the placed nozzle's Handle.
"""

import math

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book 6670152. Not derived from any model
file."""

PARTS = ["4032a", "3941", "87580", "4733", "11477", "3062b", "98138", "99207"]

OUT = Rot(y=90)  # the nozzle built upright, laid on its side: its mouth to +x


def _put(m, part, color, point, r):
    """Places a part by its LDraw origin at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r)
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def _petals(m, z, turn, color):
    """Four curved slopes on the side studs of a brick 1 x 1 with studs on four sides whose bottom is at z, turned
    `turn` degrees: each slope's base on its side stud, 0.5 studs out from the axis, its thin end down."""
    for k in range(4):
        a = turn + 90 * k
        # A curved slope's origin is the middle of its base; its thin end's anti-stud is 1.25 plates under it.
        at = (
            1 + 0.5 * math.cos(math.radians(a)),
            1 + 0.5 * math.sin(math.radians(a)),
            z + 1.75 + 1.25,
        )
        _put(m, "11477", color, at, frame.rotation(0, 90, a))


def _nozzle(m, color, petal, core):
    """The nozzle upright in its own frame: its base's corner at the origin, its axis at (1, 1)."""
    with m.step("Exhaust: core"):
        m.add("4032a", color, at=(0, 0, 0))
        m.stack("3941", color, at=(0, 0, 1), count=2)
        m.add("87580", color, at=(0, 0, 7))
    with m.step("Exhaust: first petals"):
        m.add("4733", core, at=(0.5, 0.5, 8))
        _petals(m, 8, 0, petal)
    with m.step("Exhaust: second petals"):
        m.add("3062b", core, at=(0.5, 0.5, 11))
        # By its top: turned 45° on the one stud.
        _put(m, "4733", core, (1, 1, 17), frame.rotation(z=45))
        _petals(m, 14, 45, petal)
        m.add("98138", core, at=(0.5, 0.5, 17))


def build(
    m,
    x,
    y,
    z,
    color="Dark Bluish Gray",
    petal="Pearl Silver",
    core="Black",
    title="Exhaust",
):
    """The bracket, then the nozzle laid on its side on it; returns the nozzle's Handle."""
    nozzle = m.submodel("flared_exhaust", _nozzle, color=color, petal=petal, core=core)
    (x0, y0, z0), (x1, y1, z1) = nozzle.box
    with m.step(f"{title}: bracket"):
        m.add("99207", color, at=(x, y, z), rot=0)
    # Laid on its side (a turn about y), the nozzle's z becomes +x (plates to studs) and its x becomes -z.
    face, axis_y, axis_z = x + 1.2, y + 1, z + 2.5
    corner = (face + z0 / 2.5, axis_y - (1 - y0), axis_z - 2.5 * (x1 - 1))
    with m.step(title):
        return m.place(nozzle, at=corner, rot=OUT)


def example(m):
    """A nozzle out of the rear of a car's body."""
    with m.step("Body"):
        m.add("3031", "Black", at=(0, 0, 0))
        m.add("3003", "Black", at=(0, 1, 1))
    m.use("flared_exhaust", x=1, y=0, z=4)
