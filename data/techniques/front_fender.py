# SPDX-License-Identifier: Apache-2.0
"""Front fender: a display e-bike's or dirt bike's high front fender, a blade over the front tire that hangs from the
fork tubes, so it turns with the fork, and stays put while the wheel springs up and down under it.

m.use("front_fender", fork=fork, length=8, color="Black", top_color="Black", bracket_color="Black", title="Front
fender") takes what `suspension_fork` returned (its front axle at least 10 studs down the fork, so the fender has room
above the tire). On each fork tube, just under the lower clamp's bushes, a Technic cross block 2 x 2 (32291) is fixed by
its axle hole; two friction pins (2780) from its twin pin holes hold a Technic L-liftarm 2 x 4 (32140) inside it, one
leg along the tube and the other reaching 3 studs forward. Two axles cross between the liftarms' forward holes: an axle
7 (44294) through their end axle holes, which grip it, and an axle 8 (3707) a stud behind, with a half bush (32123a) on
each end. On them, between the liftarms, four Technic bricks 1 x 4 with holes (3701) sit with their studs up the fork;
on those, a blade of plates 4 wide and `length` studs long (6 or 8), from a stud behind the fork tubes forward, and
tiles on top. On the narrower "motorcycle" fork the axles are a 5 (32073) and a 6 (3706). It brings its own steps and returns nothing.
"""

import math

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of dirt bikes and the high front
fender on the e-bike Barry builds. Not derived from any model file."""

PARTS = [
    "32291",
    "2780",
    "32140",
    "44294",
    "3707",
    "32073",
    "3706",
    "32123a",
    "3701",
    "3020",
    "3022",
    "87079",
    "3068b",
]

PLATES2 = {4: "3020", 2: "3022"}
GRIPPED = {4: "44294", 3: "32073"}  # by the fork's half width: through both liftarms
LOOSE = {4: "3707", 3: "3706"}  # and a half bush past each
TILES = {4: "87079", 2: "3068b"}


def _matrix(rot):
    """The rotation Rot(x, y, z) makes (x first, then y, then z, about the model's axes), as rows."""
    a, b, c = (math.radians(v) for v in rot)
    rx = ((1, 0, 0), (0, math.cos(a), -math.sin(a)), (0, math.sin(a), math.cos(a)))
    ry = ((math.cos(b), 0, math.sin(b)), (0, 1, 0), (-math.sin(b), 0, math.cos(b)))
    rz = ((math.cos(c), -math.sin(c), 0), (math.sin(c), math.cos(c), 0), (0, 0, 1))
    mul = lambda p, q: tuple(
        tuple(sum(p[i][k] * q[k][j] for k in range(3)) for j in range(3))
        for i in range(3)
    )
    return mul(rz, mul(ry, rx))


def build(
    m,
    fork,
    length=8,
    color="Black",
    top_color="Black",
    bracket_color="Black",
    title="Front fender",
):
    """The brackets, liftarms, axles, bricks, plates, and tiles."""
    if length not in (6, 8):
        raise ValueError(f"front_fender: length {length} isn't 6 or 8 studs")
    along, half, travel = fork["along"], fork["half"], fork["travel"]
    if travel < 10:
        raise ValueError(
            f"front_fender: the front axle is {travel:.1f} studs down the fork; the fender needs 10 or more, for room "
            "between the lower clamp and the tire"
        )
    rx, rz = fork["rake"]
    r = math.degrees(math.atan2(rx, -rz))  # the rake
    t0 = 3.0  # the brackets' axle holes, studs down the fork: under the lower clamp's bushes
    pin = t0 + 0.5  # the liftarms' corner holes, on the brackets' lower pins
    bracket_rot, arm_rot, brick_rot = (
        Rot(x=-(90 + r), z=90),
        Rot(x=90, y=-r),
        Rot(x=r, z=-90),
    )
    with m.step(f"{title}: brackets"):
        for s in (-1, 1):
            block = m.add(
                "32291",
                bracket_color,
                at=along(t0, side=s * half),
                rot=bracket_rot,
                hole="2",
            )
            arm = m.add(
                "32140",
                bracket_color,
                at=along(pin, f=1, side=s * (half - 1)),
                rot=arm_rot,
                hole="1",
            )
            for n in (1, 3):
                m.insert(
                    "2780",
                    "Black",
                    block.hole(n),
                    out="+y" if s < 0 else "-y",
                    out_by=1,
                )
            if s < 0:
                right = arm
    with m.step(f"{title}: axles and bricks"):
        # The liftarms' outside faces are half - 0.5 from the center: the gripped axle reaches them, the loose one
        # half a stud past, for its bushes.
        m.insert(GRIPPED[half], "Black", right.hole(5), out="+y", out_by=2 * half - 2)
        m.insert(LOOSE[half], "Black", right.hole(4), out="+y", out_by=2 * half - 1.5)
        for s in (-1, 1):
            m.add(
                "32123a",
                "Light Bluish Gray",
                at=along(pin, f=3, side=s * (half - 0.25)),
                rot=1,
                hole="1",
            )
        for k in range(4):
            m.add(
                "3701",
                color,
                at=along(pin, f=3, side=-1.5 + k),
                rot=brick_rot,
                hole="2",
            )
    # The blade: plates on the bricks' studs (up the fork, at t0), from a stud behind the tubes forward.
    R = _matrix((r, 0, -90))
    # The plates' corner: their underside, their back end, their left edge.
    o_t, o_f, o_y = t0, -1.0, 2.0

    def put(part, size, f, y_edge, t, part_color):
        """A part w x l (studs) x h (plates) whose corner (underside, back end, left edge) is at f, y_edge, t."""
        w, l, h = size
        corners = [(a, b, c) for a in (0, w) for b in (0, l) for c in (0, 0.4 * h)]
        world = [
            tuple(sum(R[i][j] * p[j] for j in range(3)) for i in range(3))
            for p in corners
        ]
        lo = [min(v[i] for v in world) for i in range(3)]
        x0, y0, z0 = along(t, f=f, side=y_edge)
        m.add(
            part,
            part_color,
            at=(x0 + lo[0], y0 + lo[1], z0 + 2.5 * lo[2]),
            rot=Rot(x=r, z=-90),
        )

    with m.step(f"{title}: blade"):
        for row in range(2):
            f, left = o_f, length
            while left > 0:
                k = max(k for k in PLATES2 if k <= left)
                put(PLATES2[k], (2, k, 1), f, o_y - 2 * row, o_t, color)
                f, left = f + k, left - k
    with m.step(f"{title}: tiles"):
        for row in range(2):
            f, left = o_f, length
            while left > 0:
                k = max(k for k in TILES if k <= left)
                put(TILES[k], (2, k, 1), f, o_y - 2 * row, o_t - 0.4, top_color)
                f, left = f + k, left - k


def example(m):
    """A front fender on a bike's fork."""
    frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
    swingarm = m.use("swingarm_shock", frame=frame)
    fork = m.use("suspension_fork", frame=frame, swingarm=swingarm)
    m.use("front_fender", fork=fork)
