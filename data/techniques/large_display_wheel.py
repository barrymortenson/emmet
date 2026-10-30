# SPDX-License-Identifier: Apache-2.0
"""Large display wheel: a road wheel for a large display car, a rim and tire on a Technic pin out of the chassis, with a
hubcap pinned into the rim's center.

m.use("large_display_wheel", x, y, z, side="left", rim="Metallic Silver", cap="Light Bluish Gray", title="Wheel") pushes
a friction pin (2780) into the hole of a Technic brick whose hole runs along y, whose bottom is at z and whose hole is
at x (Technic bricks' holes sit on the stud lines between their studs), its outer face at y, and puts a wheel on the
pin: a rim 14 x 18 with holes on both sides (55981) in a tire 14/ 50 x 17 (30391), 3.8 studs across and 2 wide, its
inner face against the brick. A Technic pin 3/4 (32002) in the rim's outer hole carries a round tile 2 x 2 with a hole
(15535) as the hubcap. side: "left" (the wheel on -y of y) or "right" (on +y). The wheel is a submodel, built once
whatever the count. The axle line is 1.75 plates above z and the tire reaches 3 plates below z, so a chassis brick at z
= 3 puts the tire on the ground. Returns the z of the tire's bottom.
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10300, and the instruction
book 6670152. Not derived from any model file."""

PARTS = ["2780", "55981", "30391", "32002", "15535"]

# A rim or tire turned a quarter clockwise: its axle runs along y, its open face to -y.
AXIS = frame.rotation(z=-90)
CAP = Rot(x=90)  # a round tile on its back, its face to -y
OUT = frame.rotation(z=180)  # a pin 3/4 turned half round: its short end to -y
RADIUS = 4.75  # the tire's radius, in plates


def _put(m, part, color, point, r=None):
    """Places a part whose origin is its center (a rim, a tire, a pin) at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def _wheel(m, rim, cap):
    """The wheel in its own frame: the hub's center at the origin, its outer face to -y."""
    with m.step("Wheel: rim and tire"):
        _put(m, "55981", rim, (0, 0, 0), AXIS)
        _put(m, "30391", "Black", (0, 0, 0), AXIS)
    with m.step("Wheel: hubcap"):
        _put(m, "32002", "Dark Bluish Gray", (0, -1, 0), OUT)
        m.add("15535", cap, at=(-1, -1.5, -2.5), rot=CAP)


def build(
    m,
    x,
    y,
    z,
    side="left",
    rim="Metallic Silver",
    cap="Light Bluish Gray",
    title="Wheel",
):
    """The pin and the wheel on it; returns the z of the tire's bottom."""
    if side not in ("left", "right"):
        raise ValueError(
            f"large_display_wheel takes a side of 'left' or 'right', not {side!r}"
        )
    out = -1 if side == "left" else 1
    zc = z + 1.75  # the hole's center line
    wheel = m.submodel("display_wheel", _wheel, rim=rim, cap=cap)
    (x0, y0, z0), (x1, y1, _) = wheel.box
    # The hub's center sits a stud out from the face; turned half round for the right side, the box turns with it.
    hub = (x, y + out, zc)
    corner = (
        (hub[0] + x0, hub[1] + y0, hub[2] + z0)
        if out < 0
        else (hub[0] - x1, hub[1] - y1, hub[2] + z0)
    )
    with m.step(f"{title}: pin"):
        _put(m, "2780", "Black", (x, y, zc))
    with m.step(title):
        m.place(wheel, at=corner, rot=0 if out < 0 else 2)
    return zc - RADIUS


def example(m):
    """Four wheels on a small frame of two Technic bricks 1 x 8 under a plate."""
    with m.step("Frame"):
        m.add("3702", "Light Bluish Gray", at=(0, 0, 3), rot=1)
        m.add("3702", "Light Bluish Gray", at=(0, 5, 3), rot=1)
    with m.step("Floor"):
        m.add("3036", "Light Bluish Gray", at=(0, 0, 6), rot=1)
    for hx in (1, 7):
        m.use("large_display_wheel", x=hx, y=0, z=3, side="left", title=f"Wheel {hx}L")
        m.use("large_display_wheel", x=hx, y=6, z=3, side="right", title=f"Wheel {hx}R")
