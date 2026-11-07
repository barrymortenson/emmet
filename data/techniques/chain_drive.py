# SPDX-License-Identifier: Apache-2.0
"""Chain drive: a display motorbike's chain, a loop of Technic chain links around a sprocket on the swingarm's pivot
and another on the rear wheel's axle.

m.use("chain_drive", frame=frame, swingarm=swingarm, sprocket_color="Light Bluish Gray", chain_color="Black",
title="Chain") takes what `motorbike_frame` and `swingarm_shock` returned. A gear 16 tooth (4019) goes on the pivot
axle and another on the rear axle, both in the plane just inside the right arm (between the frame's top and bottom
spars at the pivot, and between the arm and the wheel at the rear). Technic chain links (3711) run around them: two
straight runs a stud above and below the swingarm's line, and half a turn round each sprocket, the loop's length split
into whole links (about 0.8 studs each), each link turned toward the next so each one clips to the next and the
sprockets hold the loop. The sprockets go on after the axles here; in a set's instructions they go on as each axle
goes in. It brings its own steps (at most 8 links a step) and returns the number of links.
"""

import math

from emmet_dsl import frame as dsl_frame
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes. Not derived
from any model file."""

PARTS = ["4019", "3711"]

RADIUS = 1.0  # a 16-tooth gear's pitch radius, in studs
PITCH = 0.8  # a chain link's, in studs


def _loop(rear, front):
    """Points around the loop, PITCH apart (give or take what the loop's length needs), in studs (x, up) from the rear
    sprocket's center: the top run forward, round the front sprocket, the bottom run back, round the rear."""
    dx, dz = front[0] - rear[0], front[1] - rear[1]
    run = math.hypot(dx, dz)
    ex, ez = dx / run, dz / run
    nx, nz = -ez, ex  # a quarter turn up from the run
    total = 2 * run + 2 * math.pi * RADIUS
    n = round(total / PITCH)
    step = total / n
    out = []
    for i in range(n):
        s = i * step
        if s < run:  # the top run, rear to front
            out.append((RADIUS * nx + s * ex, RADIUS * nz + s * ez))
            continue
        s -= run
        arc = math.pi * RADIUS
        # Round the front sprocket, from the top over its front to the bottom.
        if s < arc:
            a = math.atan2(nz, nx) - s / RADIUS
            out.append((dx + RADIUS * math.cos(a), dz + RADIUS * math.sin(a)))
            continue
        s -= arc
        if s < run:  # the bottom run, front to rear
            out.append((dx - RADIUS * nx - s * ex, dz - RADIUS * nz - s * ez))
            continue
        s -= run  # round the rear sprocket, from the bottom over its back to the top
        a = math.atan2(-nz, -nx) - s / RADIUS
        out.append((RADIUS * math.cos(a), RADIUS * math.sin(a)))
    return out


def build(
    m,
    frame,
    swingarm,
    sprocket_color="Light Bluish Gray",
    chain_color="Black",
    title="Chain",
):
    """The two sprockets and the chain round them; returns how many links it took."""
    plane = frame["planes"]["down"]  # inside the right arm
    x, _, z = frame["pivot"]
    rx, _, rz = swingarm["rear_axle"]
    with m.step(f"{title}: sprockets"):
        for at in ((x, plane, z), (rx, plane, rz)):
            m.add("4019", sprocket_color, at=at, rot=1, hole=1)
    rear = (rx, rz / 2.5)
    points = _loop(rear, (x, z / 2.5))

    def link(i):
        (px, pz), (qx, qz) = points[i], points[(i + 1) % len(points)]
        turn = math.atan2(-(qz - pz), -(qx - px))
        c, s = math.cos(turn), math.sin(turn)
        at = dsl_frame.to_ldraw_point((rear[0] + px, plane, 2.5 * (rear[1] + pz)))
        return lambda: m.add_raw("3711", chain_color, [*at, 1, 0, 0, 0, c, s, 0, -s, c])

    in_steps(m, f"{title}: links", [(1, link(i)) for i in range(len(points))])
    return len(points)


def example(m):
    """A chain from the pivot to the rear wheel, on a frame and swingarm."""
    frame = m.use("motorbike_frame", x=10, y=6, z=27)
    swingarm = m.use("swingarm_shock", frame=frame)
    m.use("chain_drive", frame=frame, swingarm=swingarm)
