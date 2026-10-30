# SPDX-License-Identifier: Apache-2.0
"""Dashboard and wheel: a large car's dashboard, a ledge of bricks across the cabin with dials on brackets facing the
driver, and the steering wheel on an axle through a Technic brick in the ledge.

m.use("dashboard_and_wheel", x, y, z, width=6, driver=1, color="Dark Bluish Gray", dial="White", wheel="Black",
title="Dashboard") lays a ledge one stud deep and two bricks tall across the cabin at x, `width` studs along y from (y,
z), the driver on +x: bricks, the upper course with a Technic brick 1 x 2 with a hole (3700) at y + `driver`, its hole
along x. An axle 3 (4519) goes through the hole, 2 studs out toward the driver, with a small Technic steering wheel
(2819) on its end. On the ledge, a bracket 1 x 2 - 2 x 2 up (99207) over the column and one at the far end face the
driver, each with two round tiles 1 x 1 (98138) as dials and a tile 1 x 2 under them, and tiles cover the rest of the
ledge. driver is 0 to width - 4. Returns the z of the ledge's top (z + 7).
"""

from emmet_dsl import Brick, Rot, Tile, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 42213, set 11381, set 10300.
Not derived from any model file."""

PARTS = [
    "3700",
    "4519",
    "2819",
    "99207",
    "98138",
    "3069b",
    "3004",
    "3010",
    "3622",
    "3005",
    "3070b",
    "2431",
    "63864",
]

FACE = Rot(y=90)  # a tile on its back, its face to +x
COLUMN = frame.rotation(z=90)  # an axle along x
TOWARD = frame.rotation(y=90)  # the steering wheel's face to +x


def _put(m, part, color, point, r=None):
    """Places a part by its LDraw origin at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def _dials(m, x, y, z, dial, color):
    """The dials on a bracket whose base is at (x, y, z): its face is 1.2 studs in front of x."""
    face = x + 1.2
    for k in range(2):
        # The upper row of studs, 3.75 plates up.
        m.add("98138", dial, at=(face, y + k, z + 2.5), rot=FACE)
    m.add("3069b", color, at=(face, y, z), rot=FACE)  # the lower row, 1.25 plates up


def build(
    m,
    x,
    y,
    z,
    width=6,
    driver=1,
    color="Dark Bluish Gray",
    dial="White",
    wheel="Black",
    title="Dashboard",
):
    """The ledge, the column and wheel, and the dials; returns the z of the ledge's top."""
    if width < 4 or not 0 <= driver <= width - 4:
        raise ValueError(
            f"dashboard_and_wheel needs a width of 4 or more and driver from 0 to width - 4, not {width}, {driver}"
        )
    with m.step(f"{title}: ledge, lower course"):
        m.row(Brick, color, at=(x, y, z), length=width, axis="y")
    # The upper course holds the column's Technic brick.
    with m.step(f"{title}: ledge, upper course"):
        if driver:
            m.row(Brick, color, at=(x, y, z + 3), length=driver, axis="y")
        m.add("3700", color, at=(x, y + driver, z + 3), rot=0)
        m.row(
            Brick,
            color,
            at=(x, y + driver + 2, z + 3),
            length=width - driver - 2,
            axis="y",
        )
    z += 3
    zc, yc = z + 1.75, y + driver + 1
    with m.step(f"{title}: steering"):
        _put(m, "4519", "Black", (x + 1.5, yc, zc), COLUMN)
        # The wheel's hub takes the axle a stud deep; its origin is 1.2 studs above the hub's bottom.
        _put(m, "2819", wheel, (x + 3.2, yc, zc), TOWARD)
    brackets = (driver, width - 2)
    with m.step(f"{title}: brackets and dials"):
        for b in brackets:
            m.add("99207", color, at=(x, y + b, z + 3), rot=0)
            _dials(m, x, y + b, z + 3, dial, color)
    with m.step(f"{title}: top"):
        a = 0
        for b in sorted(brackets) + [width]:
            if b > a:
                m.row(Tile, color, at=(x, y + a, z + 3), length=b - a, axis="y")
            a = b + 2
    return z + 4


def example(m):
    """A dashboard across a cabin 6 studs wide, the driver on the left."""
    with m.step("Floor"):
        m.add("3036", "Tan", at=(0, 0, 0))
    m.use("dashboard_and_wheel", x=0, y=0, z=1)
