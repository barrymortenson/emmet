# SPDX-License-Identifier: Apache-2.0
"""Chain drive: a display motorbike's or e-bike's chain, a loop of Technic chain links from a chainring on the
frame's crank axle to the sprocket on the rear axle, with crank arms and pedals.

m.use("chain_drive", frame=frame, swingarm=swingarm, pedals=True, ring_color="Black", chain_color="Black",
crank_color="Black", title="Chain") takes what `motorbike_frame` and `swingarm_shock` returned, the swingarm made with
its sprocket (`sprocket=True`, the default: a gear 16 tooth on the rear axle inside the right arm, which goes on with
the wheel, before the axle closes). A gear 24 tooth (3648b), the chainring, goes on the frame's crank axle in the same
plane, outside the frame and the tire, before the crank arms close the axle's ends. Technic chain links (3711) run
round them: two straight runs, each tangent to both, and round each gear, the loop's length split into whole links
(about 0.8 studs each), each link turned toward the next, so each one clips to the next and the gears hold the loop.
With pedals, a crank arm on each end of the crank axle (a Technic beam 3 x 0.5, 6632) holds the axle's end and carries
an axle pin (3749) and a pedal (another 6632 across it); the right arm points forward and down, the left back and up.
It brings its own steps (at most 8 links a step) and returns the number of links.
"""

import math

from emmet_dsl import frame as dsl_frame
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes. Not derived
from any model file."""

PARTS = ["3648b", "3711", "6632", "3749"]

RING, SPROCKET = 1.5, 1.0  # the 24- and 16-tooth gears' pitch radii, in studs
PITCH = 0.8  # a chain link's, in studs


def _loop(rear, front):
    """Points round the loop, PITCH apart (give or take what its length needs), in studs (x, up): the top run forward
    from the rear sprocket, round the chainring's front, the bottom run back, round the sprocket's back."""
    dx, dz = front[0] - rear[0], front[1] - rear[1]
    d = math.hypot(dx, dz)
    ex, ez = dx / d, dz / d
    nx, nz = -ez, ex  # a quarter turn up from the line between the centers
    sb = (RING - SPROCKET) / d
    cb = math.sqrt(1 - sb * sb)
    up = (
        nx * cb - ex * sb,
        nz * cb - ez * sb,
    )  # the top run's side, from either center
    low = (-nx * cb - ex * sb, -nz * cb - ez * sb)
    run = d * cb
    beta = math.asin(sb)
    front_arc, rear_arc = RING * (math.pi + 2 * beta), SPROCKET * (math.pi - 2 * beta)
    total = 2 * run + front_arc + rear_arc
    n = round(total / PITCH)
    step = total / n
    a_up, a_low = math.atan2(up[1], up[0]), math.atan2(low[1], low[0])
    out = []
    for i in range(n):
        s = i * step
        if s < run:  # the top run, rear to front
            p = (rear[0] + SPROCKET * up[0], rear[1] + SPROCKET * up[1])
            q = (front[0] + RING * up[0], front[1] + RING * up[1])
            out.append((p[0] + s * (q[0] - p[0]) / run, p[1] + s * (q[1] - p[1]) / run))
            continue
        s -= run
        if s < front_arc:  # round the chainring's front, from the top to the bottom
            a = a_up - s / RING
            out.append((front[0] + RING * math.cos(a), front[1] + RING * math.sin(a)))
            continue
        s -= front_arc
        if s < run:  # the bottom run, front to rear
            p = (front[0] + RING * low[0], front[1] + RING * low[1])
            q = (rear[0] + SPROCKET * low[0], rear[1] + SPROCKET * low[1])
            out.append((p[0] + s * (q[0] - p[0]) / run, p[1] + s * (q[1] - p[1]) / run))
            continue
        s -= run  # round the sprocket's back, from the bottom to the top
        a = a_low - s / SPROCKET
        out.append((rear[0] + SPROCKET * math.cos(a), rear[1] + SPROCKET * math.sin(a)))
    return out


def build(
    m,
    frame,
    swingarm,
    pedals=True,
    ring_color="Black",
    chain_color="Black",
    crank_color="Black",
    title="Chain",
):
    """The chainring, the chain round it and the swingarm's sprocket, and the cranks and pedals; returns how many links
    it took."""
    if swingarm.get("sprocket") is None:
        raise ValueError(
            "chain_drive: the swingarm has no sprocket; make it with swingarm_shock(..., sprocket=True), which puts the "
            "sprocket on with the rear wheel, before the axle closes"
        )
    y = frame["y"]
    cx, _, cz = frame["crank"]
    rx, plane, rz = swingarm["sprocket"]
    with m.step(f"{title}: chainring"):
        m.add("3648b", ring_color, at=(cx, plane, cz), rot=1, hole=3)  # its axle hole
    points = _loop((rx, rz / 2.5), (cx, cz / 2.5))

    def link(i):
        (px, pz), (qx, qz) = points[i], points[(i + 1) % len(points)]
        turn = math.atan2(-(qz - pz), -(qx - px))
        c, s = math.cos(turn), math.sin(turn)
        at = dsl_frame.to_ldraw_point((px, plane, 2.5 * pz))
        return lambda: m.add_raw("3711", chain_color, [*at, 1, 0, 0, 0, c, s, 0, -s, c])

    in_steps(m, f"{title}: links", [(1, link(i)) for i in range(len(points))])
    if pedals:
        with m.step(f"{title}: cranks and pedals"):
            for s in (-1, 1):
                # A crank arm on each end of the crank axle: the right one forward and down, the left back and up.
                at = (cx, y + s * 4.75, cz)
                arm = m.beam(
                    "6632", crank_color, at=at, run=-1.6 * s, rise=1.2 * s, holes=(1, 3)
                )
                pin = m.insert(
                    "3749",
                    crank_color,
                    arm.hole(3),
                    out="-y" if s < 0 else "+y",
                    out_by=1.5,
                )
                p = pin.hole(1).at
                foot = (p[0], y + s * 6.25, p[2])
                m.beam(
                    "6632",
                    crank_color,
                    at=foot,
                    to=(foot[0] + 1, foot[1], foot[2]),
                    holes=(2, 3),
                )
    return len(points)


def example(m):
    """A chain from the crank to the rear wheel, on a frame and swingarm."""
    frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
    swingarm = m.use("swingarm_shock", frame=frame)
    m.use("chain_drive", frame=frame, swingarm=swingarm)
