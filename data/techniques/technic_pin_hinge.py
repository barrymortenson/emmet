# SPDX-License-Identifier: Apache-2.0
"""Technic pin hinge: an arm of Technic brick that swings on a friction pin through the hole of a Technic brick beside
it, and stays at the angle it's set to.

m.use("technic_pin_hinge", x, y, z, length=4, angle=30, color="Light Bluish Gray", arm_color="Light Bluish Gray",
pin_color="Black", title="Pin hinge") puts a Technic brick 1 x 2 with a hole (3700) along x at (x, y, z), its hole
running along y, and beside it (on y + 1) an arm, a Technic brick 1 x `length` with holes (3701, 3894, or 3702 for 4,
6, or 8) whose first hole lines up with the 1 x 2's, joined by a friction pin (2780) through both holes. The arm
reaches along +x from the pin, raised `angle` degrees (0 is level; up to 90, or down to -90 if there's room below).
The pin's friction holds any angle. It brings its own steps and returns the (x, y, z) of the arm's last hole, where
the next pin goes.
"""

import math

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["3700", "3701", "3894", "3702", "2780"]

ARMS = {4: "3701", 6: "3894", 8: "3702"}


def _mul(a, b):
    return tuple(
        tuple(sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3))
        for i in range(3)
    )


def _put(m, part, color, point, local, r_dsl):
    """Places a part turned by r_dsl (DSL axes) so the point `local` of its own frame (LDU) lands on `point` (DSL)."""
    r = frame.to_ldraw_rotation(r_dsl)
    p = frame.to_ldraw_point(point)
    pos = [p[i] - sum(r[i][k] * local[k] for k in range(3)) for i in range(3)]
    return m.add_raw(part, color, [*pos, *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    length=4,
    angle=30,
    color="Light Bluish Gray",
    arm_color="Light Bluish Gray",
    pin_color="Black",
    title="Pin hinge",
):
    """The fixed brick, the arm, and the pin; returns the arm's last hole, (x, y, z)."""
    if length not in ARMS:
        raise ValueError(
            f"technic_pin_hinge takes an arm length of 4, 6, or 8, not {length}"
        )
    if not -90 <= angle <= 90:
        raise ValueError(
            f"technic_pin_hinge takes an angle from -90 to 90, not {angle}"
        )
    pivot = (x + 1, y + 1, z + 1.75)  # the pin's middle, between the two bricks' holes
    # The arm along x (a quarter turn), then raised about y; its first hole is (length/2 - 1) studs from its middle,
    # 10 LDU under its top.
    turn = _mul(frame.rotation(0, -angle, 0), frame.rotation(0, 0, 90))
    first = (20 * (length / 2 - 1), 10, 0)
    with m.step(f"{title}: brick"):
        m.add("3700", color, at=(x, y, z), rot=1)
    with m.step(f"{title}: arm and pin"):
        _put(m, ARMS[length], arm_color, (pivot[0], y + 1.5, pivot[2]), first, turn)
        _put(m, "2780", pin_color, pivot, (0, 0, 0), frame.rotation())
    a, reach = math.radians(angle), length - 2
    return (
        pivot[0] + reach * math.cos(a),
        y + 1.5,
        pivot[2] + 2.5 * reach * math.sin(a),
    )


def example(m):
    """An arm raised on a brick, on a base."""
    with m.step("Base"):
        m.add("3020", "Dark Bluish Gray", at=(0, 0, 0))
        m.add("3004", "Light Bluish Gray", at=(0, 0, 1), rot=1)
    m.use("technic_pin_hinge", x=0, y=0, z=4, length=6, angle=35)
