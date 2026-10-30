# SPDX-License-Identifier: Apache-2.0
"""Ring light grille: an off-roader's flat front at 1:8, a dark tiled grille between two round headlights 2 studs
across, each a ring tile on a Technic pin in a brick's hole.

m.use("ring_light_grille", x, y, z, width=8, color="Black", grille="Black", ring="White", title="Grille") lays a course
across a vehicle's front at x, `width` studs along y from (y, z), facing -x: a Technic brick 1 x 2 with a hole (3700) at
each end, its hole along x, and bricks 1 x 4 with studs on the side (30414) between, their studs to the front. A Technic
pin 3/4 (32002) goes into each hole with a round tile 2 x 2 with a hole (15535) in `ring` on its short end, the
headlight; grille tiles 1 x 2 (2412b) in `grille` on the side studs make the grille, level with the lights' centers. The
lights reach 0.75 plates below z and 1.25 above the course, and stand 0.5 studs out in front of x. width: 8, 12, or 16.
Returns the z of the course's top (z + 3).
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 42213. Not derived from any
model file."""

PARTS = ["3700", "30414", "32002", "15535", "2412b"]

FACE = Rot(y=-90)  # a tile on its back, its face to -x
PIN = frame.rotation(z=90)  # a pin 3/4 along x, its long end to +x


def _put(m, part, color, point, r=None):
    """Places a part whose origin is its center (a pin) at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def build(
    m, x, y, z, width=8, color="Black", grille="Black", ring="White", title="Grille"
):
    """The course, the lights, and the grille; returns the z of the course's top."""
    if width not in (8, 12, 16):
        raise ValueError(
            f"ring_light_grille takes a width of 8, 12, or 16, not {width}"
        )
    lights = (y, y + width - 2)
    zc = z + 1.75  # the holes' and side studs' center line
    with m.step(f"{title}: course"):
        for ly in lights:
            m.add("3700", color, at=(x, ly, z), rot=0)  # its hole along x
        for k in range(2, width - 2, 4):
            # Its side studs to the front.
            m.add("30414", color, at=(x, y + k, z), rot=2)
    with m.step(f"{title}: headlights"):
        for ly in lights:
            _put(m, "32002", "Tan", (x, ly + 1, zc), PIN)
            m.add("15535", ring, at=(x - 0.5, ly, zc - 2.5), rot=FACE)
    with m.step(f"{title}: grille"):
        for k in range(2, width - 2, 2):
            m.add("2412b", grille, at=(x - 0.4, y + k, zc - 1.25), rot=FACE)
    return z + 3


def example(m):
    """The front of an off-roader 8 studs wide."""
    with m.step("Floor"):
        m.add("3035", "Dark Bluish Gray", at=(0, 0, 0))
    with m.step("Bumper"):
        m.add("3008", "Black", at=(0, 0, 1))
    m.use("ring_light_grille", x=0, y=0, z=4, color="Red")
