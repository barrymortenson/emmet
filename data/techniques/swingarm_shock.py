# SPDX-License-Identifier: Apache-2.0
"""Swingarm with its shocks: a display motorbike's rear swingarm on the frame's pivot axle, a shock absorber outside
each arm from the arm up to the frame, and the rear wheel on its axle between the arms.

m.use("swingarm_shock", frame=frame, reach=10, drop=4.15, wheel="motorcycle", color="Black", shock_color="Yellow",
axle_color="Light Bluish Gray", title="Swingarm") takes the frame `motorbike_frame` returned. Two beams, one outside
each side of the frame, hang on the pivot axle and slope down to the rear axle, `reach` studs back (6 to 14, even: an
11-hole beam's 10 at the default), `drop` studs down there. Twin shock absorbers 6.5L (73129, their holes 5.5 studs
apart), one outside each arm, are pinned (friction pins, 2780) to an arm hole and, at the top, to a shock mount: a
3-hole beam (32523) pinned to a hole of the top spar, turned so the shock's 5.5 studs reach it. Of the arm's holes and
the spar's, the pair that hangs the shocks steepest over the middle of the arm. Measure the photo: `reach` is the pivot to the
rear axle, `drop` how far the axle is below the pivot. `wheel` is "motorcycle" (88517c01, 12.6 studs across, 2.4
wide), "fat" (an off-road wheel and tire, 44772c01: 11.8 across, 5.5 wide), or None for the axle alone.
It brings its own steps and returns a dict of "rear_axle" (the axle's middle, in the center plane), "arm" (the right
arm's Handle), "angle" (degrees below level), "drop" (the axle's, below the pivot: give it to `suspension_fork`), and
"radius" (the wheel's, in studs).
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes. Not derived
from any model file."""

PARTS = [
    "32523",
    "32316",
    "32524",
    "40490",
    "32525",
    "41239",
    "32278",
    "73129",
    "2780",
    "3737",
]
PARTS += ["88517c01", "44772c01"]

SHOCK = 5.5  # studs between the shock's holes
BEAMS = {
    3: "32523",
    5: "32316",
    7: "32524",
    9: "40490",
    11: "32525",
    13: "41239",
    15: "32278",
}
# A wheel: the part, its hub's axle hole, its radius in studs.
WHEELS = {"motorcycle": ("88517c01", "3", 6.29), "fat": ("44772c01", "1", 5.92)}


def wheel_on(m, kind, center, color="Black"):
    """A wheel `kind` (WHEELS) whose hub is on an axle along y through `center`; returns its Handle."""
    part, hub, _ = WHEELS[kind]
    return m.add(part, color, at=center, rot=1, hole=hub)


MOUNT = 2  # studs between a shock mount's end holes (a 3-hole beam)


def _choose(frame, reach, drop):
    """The arm's angle (radians, from +x toward +z: `drop` studs down at the rear axle), and the shocks' layout: the arm
    hole (studs from the pivot), the top spar's hole the mount hangs from, and the shock's top (run, rise in studs from
    the pivot), 5.5 studs from the arm hole and a mount's length from the spar's hole."""
    reach_, t, _, _ = frame["lengths"]
    # Back from the pivot and `drop` down at the rear axle.
    t_arm = math.pi + math.asin(max(-1.0, min(1.0, drop / reach)))
    best = None
    x, _, z = frame["pivot"]
    for i in range(3, reach_ + 1):
        if i == t + 1:
            continue  # the down tube's pinned joint
        hole = frame["top"].hole(i - 1 + frame["first"])
        f = (hole.at[0] - x, (hole.at[2] - z) / 2.5)
        for a in range(2, reach):
            arm = (a * math.cos(t_arm), a * math.sin(t_arm))
            d = math.hypot(f[0] - arm[0], f[1] - arm[1])
            if not SHOCK - MOUNT + 0.2 < d < SHOCK + MOUNT - 0.2:
                continue
            # The shock's top: SHOCK from the arm hole, MOUNT from the spar's hole; the one above the arm.
            ux, uz = (f[0] - arm[0]) / d, (f[1] - arm[1]) / d
            along = (SHOCK * SHOCK - MOUNT * MOUNT + d * d) / (2 * d)
            h = math.sqrt(max(SHOCK * SHOCK - along * along, 0.0))
            for side in (1, -1):
                top = (
                    arm[0] + along * ux - side * h * uz,
                    arm[1] + along * uz + side * h * ux,
                )
                # Above the arm's line by more than the beams' half widths, and the shock steep.
                if top[1] - top[0] * math.tan(t_arm) < 1.2:
                    continue
                tilt = abs(
                    math.degrees(math.atan2(top[1] - arm[1], top[0] - arm[0])) - 70
                )
                score = abs(a - 0.55 * reach) + tilt / 30
                if best is None or score < best[0]:
                    best = (score, a, i, top)
    if best is None:
        raise ValueError(
            "swingarm_shock: no top spar hole hangs a shock over the swingarm; try another drop or reach"
        )
    return t_arm, best[1:]


def build(
    m,
    frame,
    reach=10,
    drop=4.15,
    wheel="motorcycle",
    color="Black",
    shock_color="Yellow",
    axle_color="Light Bluish Gray",
    title="Swingarm",
):
    """The arms, the shocks and their mounts, and the rear axle (with the wheel); returns where the rear axle is."""
    if reach not in (6, 8, 10, 12, 14):
        raise ValueError(
            f"swingarm_shock: reach {reach} isn't 6, 8, 10, 12, or 14 (a beam's end hole)"
        )
    x, y, z = frame["pivot"]
    radius = WHEELS[wheel][2] if wheel else 0.0
    t, (a, spar, top) = _choose(frame, reach, drop)
    half = frame["width"] / 2 + 0.5  # the arms' planes, outside the frame's sides
    run, rise = reach * math.cos(t), reach * math.sin(t)
    beam = BEAMS[reach + 1]
    arms = []
    with m.step(f"{title}: arms"):
        for s in (-1, 1):
            arms.append(
                m.beam(
                    beam,
                    color,
                    at=(x, y + s * half, z),
                    run=run,
                    rise=rise,
                    holes=(1, reach + 1),
                )
            )
    hole = frame["top"].hole(spar - 1 + frame["first"])
    peak = (x + top[0], z + 2.5 * top[1])
    for s, arm in zip((-1, 1), arms):
        side = "right" if s < 0 else "left"
        out = "-y" if s < 0 else "+y"
        with m.step(f"{title}: {side} shock"):
            # The mount beside the top spar (where the arm's plane is clear), the shock outside it and the arm.
            mount_at, mount_to = (
                (hole.at[0], y + s * half, hole.at[2]),
                (peak[0], y + s * half, peak[1]),
            )
            mount = m.beam("32523", color, at=mount_at, to=mount_to, holes=(1, 3))
            m.insert(
                "2780", "Black", mount.hole(1), out="+y" if s < 0 else "-y"
            )  # into the top spar
            low, plane = arm.hole(a + 1).at, y + s * (half + 1)
            m.beam(
                "73129",
                shock_color,
                at=(low[0], plane, low[2]),
                to=(peak[0], plane, peak[1]),
                holes=(1, 2),
            )
            m.insert("2780", "Black", arm.hole(a + 1), out=out)
            m.insert("2780", "Black", mount.hole(3), out=out)
    end = arms[0].hole(reach + 1).at
    with m.step(f"{title}: rear wheel" if wheel else f"{title}: rear axle"):
        if wheel:
            wheel_on(m, wheel, (end[0], y, end[2]))
        m.insert("3737", axle_color, arms[0].hole(reach + 1), out="+y", out_by=2 * half)
    angle = -math.degrees(math.asin(math.sin(t)))
    return {
        "rear_axle": (end[0], y, end[2]),
        "arm": arms[0],
        "angle": angle,
        "drop": -reach * math.sin(t),
        "radius": radius,
    }


def example(m):
    """The swingarm on a frame, its rear wheel on the table."""
    frame = m.use("motorbike_frame", x=10, y=6, z=27)
    m.use("swingarm_shock", frame=frame)
