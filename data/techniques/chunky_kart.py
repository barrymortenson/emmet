# SPDX-License-Identifier: Apache-2.0
"""Chunky kart: a stubby game kart on four big wheels with round hubcaps, 2 studs of body between the wheels and
fenders 6 wide over them.

m.use("chunky_kart", x, y, z, color="Red", accent="Yellow", frame_color="Black", title="Kart") stands a kart on the
ground at height z, nose to +x, its body over (x, y) to (x + 8, y + 2) and its wheels out to y - 2 and y + 4: two
Technic bricks 1 x 8 (3702) side by side, holes across, with a friction pin (2780) in the first and last hole each
side and on each a wheel 3.8 studs across (rim 55981 in `accent`, tire 30391) with a round tile 2 x 2 hubcap (15535)
on a pin 3/4; deck plates, a seat 2 x 2 (4079) facing the nose, a steering wheel, and a block at each end under a 2 x
6 plate across the kart as the fenders, with a curved slope on the nose and two exhausts on the tail. The wheels are
as tall as the body below the fenders. It brings its own steps; the wheel is a submodel, built once. Returns the z of
the seat's base (z + 7).
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72036. Not derived from any
model file."""

PARTS = [
    "3702",
    "2780",
    "55981",
    "30391",
    "32002",
    "15535",
    "3460",
    "4079",
    "3003",
    "3795",
    "15068",
    "30663",
    "4589",
]

AXIS = frame.rotation(
    z=-90
)  # a rim or tire turned a quarter clockwise: its axle runs along y, its open face to -y
CAP = Rot(x=90)  # a round tile on its back, its face to -y
OUT = frame.rotation(z=180)  # a pin 3/4 turned half round: its short end to -y
CHASSIS = 3  # the Technic bricks' bottom above the ground when the tires stand on it, in plates
HOLE = 1.75  # a Technic brick's hole above its bottom, in plates


def _put(m, part, color, point, r=None):
    """Places a part whose origin is its middle (a rim, a tire, a pin) at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def _wheel(m, rim, cap):
    """A wheel in its own frame: the hub's middle at the origin, its outer face to -y."""
    with m.step("Wheel: rim and tire"):
        _put(m, "55981", rim, (0, 0, 0), AXIS)
        _put(m, "30391", "Black", (0, 0, 0), AXIS)
    with m.step("Wheel: hubcap"):
        _put(m, "32002", "Dark Bluish Gray", (0, -1, 0), OUT)
        m.add("15535", cap, at=(-1, -1.5, -2.5), rot=CAP)


def build(m, x, y, z, color="Red", accent="Yellow", frame_color="Black", title="Kart"):
    """The chassis, the wheels, and the body; returns the z of the seat's base."""
    zc = z + CHASSIS
    with m.step(f"{title}: chassis"):
        m.add("3702", frame_color, at=(x, y, zc), rot=1)
        m.add("3702", frame_color, at=(x, y + 1, zc), rot=1)
    wheel = m.submodel(f"{title} wheel", _wheel, rim=accent, cap=color)
    (x0, y0, z0), (x1, y1, _) = wheel.box
    for end, hx in (("rear", x + 1), ("front", x + 7)):
        with m.step(f"{title}: {end} pins"):
            for py in (y, y + 2):
                _put(m, "2780", "Black", (hx, py, zc + HOLE))
        with m.step(f"{title}: {end} wheels"):
            # The hub's middle a stud out from the brick's face; turned half round on the right, the box turns too.
            hub = (hx, y - 1, zc + HOLE)
            m.place(wheel, at=(hub[0] + x0, hub[1] + y0, hub[2] + z0))
            hub = (hx, y + 3, zc + HOLE)
            m.place(wheel, at=(hub[0] - x1, hub[1] - y1, hub[2] + z0), rot=2)
    with m.step(f"{title}: deck and seat"):
        m.add("3460", color, at=(x, y, zc + 3), rot=1)
        m.add("3460", color, at=(x, y + 1, zc + 3), rot=1)
        m.add("4079", "Black", at=(x + 2, y, zc + 4), rot=0)
    with m.step(f"{title}: tail"):
        m.add("3003", color, at=(x, y, zc + 4))
        m.add("3795", color, at=(x, y - 2, zc + 7))
        m.add("4589", "Dark Bluish Gray", at=(x, y, zc + 8))
        m.add("4589", "Dark Bluish Gray", at=(x, y + 1, zc + 8))
    with m.step(f"{title}: nose"):
        m.add("3003", color, at=(x + 6, y, zc + 4))
        m.add("3795", color, at=(x + 6, y - 2, zc + 7))
        m.add("15068", color, at=(x + 6, y, zc + 8), rot=0)
    with m.step(f"{title}: steering wheel"):
        m.add("30663", "Black", at=(x + 4, y, zc + 4), rot=2)
    return zc + 4


def example(m):
    """A kart on a strip of track."""
    from emmet_dsl import Plate

    with m.step("Track"):
        m.row(Plate, "Dark Bluish Gray", at=(-2, -2, 0), length=12, width=6)
    m.use("chunky_kart", x=0, y=0, z=1)
