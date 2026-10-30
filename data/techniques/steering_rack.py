# SPDX-License-Identifier: Apache-2.0
"""Steering rack: working rack-and-pinion steering, a gear rack that slides crosswise when the column turns, with a tie
rod pinned to each end that swings a steering arm on its pivot, so both front wheels steer together.

m.use("steering_rack", x, y, z, color="Light Bluish Gray", rack="Black", pin="Black", title="Steering") builds, on a
chassis whose top is at z, across y from y + 0.5 to y + 15.5: at each end a round plate 1 x 1 (6141) and on its stud,
through its middle hole, a beam 3 (32523) along x, the steering arm, which turns on the stud; a gear rack 1 x 8 with
holes (6630) along y, its middle at y + 8, on the stud column x, its teeth up; a beam 5 (32316) from each end of the
rack to the front hole of its steering arm, the tie rod, pinned at both ends with friction pins (2780), built apart as a
submodel and set on the arms; and the column: a Technic brick 1 x 2 with a hole (3700) on a brick behind the rack's
middle, an axle 3 (4519) through it along x, and a gear 8 tooth (3647) on its front end over the rack. The arms' back
holes, at x + 2.5, take the wheel hubs. Returns the z of the tie rods' top (z + 6).
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11381, set 42213, set 42212.
Not derived from any model file."""

PARTS = ["6141", "32523", "2780", "6630", "32316", "3004", "3700", "4519", "3647"]

UPRIGHT = frame.rotation(x=90)  # a pin stood up
# A flat beam or an axle turned to run along the other way.
ACROSS = frame.rotation(z=90)


def _put(m, part, color, point, r=None):
    """Places a part whose origin is its center (a beam, a pin, a gear, a rack) at `point` (DSL), turned by r."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def _linkage(m, rack, color, pin):
    """The rack and its tie rods, built apart: the rack's middle on the line x = 0, y = 0, with z = 0 the chassis top.
    The rack sits a quarter plate under the tie rods so its teeth just clear the pinion's."""
    with m.step("Steering: rack"):
        _put(m, "6630", rack, (0, 0, 2.75))
        for s in (-1, 1):
            _put(m, "2780", pin, (0, 3.5 * s, 3.5), UPRIGHT)
    with m.step("Steering: tie rods"):
        for s in (-1, 1):
            _put(m, "32316", color, (0, 5.5 * s, 4.75), ACROSS)
    with m.step("Steering: pins for the arms"):
        for s in (-1, 1):
            _put(m, "2780", pin, (0, 7.5 * s, 3.5), UPRIGHT)


def build(
    m, x, y, z, color="Light Bluish Gray", rack="Black", pin="Black", title="Steering"
):
    """The arms on their pivots, the rack and tie rods (built apart), then the column; returns the z of the top."""
    # The rack's middle, and the line it and the tie rods run on.
    mid, line = (y + 8, x + 0.5)
    sides = (-1, 1)
    # Each arm pivots on the single stud of a round plate: a stud turns freely in a beam's hole.
    with m.step(f"{title}: steering arms"):
        for s in sides:
            m.add("6141", color, at=(x + 1, mid + 7.5 * s - 0.5, z))
            _put(m, "32523", color, (x + 1.5, mid + 7.5 * s, z + 2.25))
    linkage = m.submodel("steering_linkage", _linkage, rack=rack, color=color, pin=pin)
    (bx, by, bz), _ = linkage.box
    with m.step(f"{title}: rack and tie rods"):
        m.place(linkage, at=(line + bx, mid + by, z + bz))
    with m.step(f"{title}: column"):
        m.add("3004", color, at=(x + 1, mid - 1, z), rot=0)
        m.add("3700", color, at=(x + 1, mid - 1, z + 3), rot=0)  # its hole along x
        _put(m, "4519", "Black", (x + 1.5, mid, z + 4.75), ACROSS)
        _put(m, "3647", color, (line, mid, z + 4.75))
    return z + 6


def example(m):
    """The steering on a chassis plate."""
    with m.step("Chassis"):
        m.add("3036", "Dark Bluish Gray", at=(-2, 0, 0), rot=0)
        m.add("3036", "Dark Bluish Gray", at=(-2, 8, 0), rot=0)
        m.add("3795", "Dark Bluish Gray", at=(-2, 16, 0), rot=1)
    m.use("steering_rack", x=1, y=0, z=1)
