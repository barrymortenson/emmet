# SPDX-License-Identifier: Apache-2.0
"""Lift arm linkage: two parallel Technic beams pinned to a post and to a head, a parallelogram, so the head swings up
and down on the arms and stays upright.

m.use("lift_arm_linkage", x, y, z, length=7, angle=30, color="Light Bluish Gray", beam_color="Light Bluish Gray",
pin_color="Black", title="Linkage") builds a post at (x, y, z) of two Technic bricks 1 x 2 with a hole (3700) along x
with two plates between, which puts their holes 2 studs apart, one over the other, running along y. Two Technic beams
`length` holes long (32316, 32524, or 40490 for 5, 7, or 9) lie in front of the post on y - 1, one from each hole,
pinned (2780), and raised `angle` degrees along +x (0 is level, up to 60, or down to -60 if there's room below). The
head at their far ends, built apart the same way as the post, goes onto pins in their far holes, so it stays upright
whatever the angle. It brings its own steps and returns the (x, y, z) of the head's footprint corner, its lower
brick's, to build on.
"""

import math

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3700", "3023b", "2780", "32316", "32524", "40490"]

BEAMS = {5: "32316", 7: "32524", 9: "40490"}


def _put(m, part, color, point, local, r_dsl):
    """Places a part turned by r_dsl (DSL axes) so the point `local` of its own frame (LDU) lands on `point` (DSL)."""
    r = frame.to_ldraw_rotation(r_dsl)
    p = frame.to_ldraw_point(point)
    pos = [p[i] - sum(r[i][k] * local[k] for k in range(3)) for i in range(3)]
    return m.add_raw(part, color, [*pos, *r[0], *r[1], *r[2]])


def _block(m, color, x=0, y=0, z=0, title=""):
    """A post or head: a Technic brick 1 x 2, two plates, and another, from (x, y, z), its holes running along y."""
    with m.step(f"{title}lower brick and plates"):
        m.add("3700", color, at=(x, y, z), rot=1)
        m.add("3023b", color, at=(x, y, z + 3), rot=1)
        m.add("3023b", color, at=(x, y, z + 4), rot=1)
    with m.step(f"{title}upper brick"):
        m.add("3700", color, at=(x, y, z + 5), rot=1)


def build(
    m,
    x,
    y,
    z,
    length=7,
    angle=30,
    color="Light Bluish Gray",
    beam_color="Light Bluish Gray",
    pin_color="Black",
    title="Linkage",
):
    """The post, the two beams, and the head; returns the head's footprint corner."""
    if length not in BEAMS:
        raise ValueError(f"lift_arm_linkage takes beams 5, 7, or 9 long, not {length}")
    if not -60 <= angle <= 60:
        raise ValueError(f"lift_arm_linkage takes an angle from -60 to 60, not {angle}")
    a = math.radians(angle)
    reach = length - 1  # studs between a beam's end holes
    dx, dz = reach * math.cos(a), 2.5 * reach * math.sin(a)
    holes = [(x + 1, z + 1.75), (x + 1, z + 6.75)]  # the post's two holes (x, z)
    # A beam's holes run along LDraw Y; turned a quarter about x they run along y, then tipped about y by the angle.
    tip = frame.rotation(90, -angle, 0)
    end = (0, 0, 10 * (length - 1))  # its end hole, in its own frame
    _block(m, color, x, y, z, f"{title}, post: ")
    with m.step(f"{title}: pins in the post"):
        for hx, hz in holes:
            _put(m, "2780", pin_color, (hx, y, hz), (0, 0, 0), frame.rotation())
    with m.step(f"{title}: beams"):
        for hx, hz in holes:
            _put(m, BEAMS[length], beam_color, (hx, y - 0.5, hz), end, tip)
    # The pins go into the beams' far holes, and the head, built apart, goes onto them.
    with m.step(f"{title}: pins in the beams"):
        for hx, hz in holes:
            _put(
                m, "2780", pin_color, (hx + dx, y, hz + dz), (0, 0, 0), frame.rotation()
            )
    head = (x + dx, y, z + dz)
    sub = m.submodel(f"{title}: head", _block, color=color)
    with m.step(f"{title}: head"):
        m.place(sub, at=head)
    return head


def example(m):
    """A linkage on a base, its head raised."""
    with m.step("Base"):
        m.add("3020", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("lift_arm_linkage", x=0, y=0, z=1, length=7, angle=35)
