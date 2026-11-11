# SPDX-License-Identifier: Apache-2.0
"""Swingarm with its shock: a display motorbike's or e-bike's rear swingarm on the frame's pivot axle, one shock
absorber from a cross axle on the swingarm up to the frame, so the wheel can only rise by squeezing the spring, and the
rear wheel on its axle between the arms.

m.use("swingarm_shock", frame=frame, reach=8, wheel="fat", disc=True, sprocket=True, color="Black", shock_color="Red",
axle_color="Black", bush_color="Black", disc_color="Light Bluish Gray", sprocket_color="Light Bluish Gray",
title="Swingarm") takes the frame `motorbike_frame` returned. Two beams, one outside each side of
the frame, hang on the pivot axle and slope down to the rear axle `reach` studs back (6 to 14, even: a 9-hole beam's 8
at the default), at the angle the frame chose so the shock fits: a shock absorber 6.5L (73129, its holes 5.5 studs
apart) in the center plane, from an axle 12 (3708) through the arms' second holes, a stud behind the pivot, to the
frame's shock mount: an axle 6 (3706) pushed in through the right down tube, a half bush (32123a), the shock's eye,
another half bush, and the left down tube. `wheel` is "fat" (an off-road wheel and
tire, 44772c01: 11.84 studs across, 5.5 wide), "motorcycle" (88517c01, 12.6 across, 2.4 wide), or None for the axle
alone; the rear axle is an axle 12 through the arms' end holes and the wheel's hub. With `disc`, a brake disc 6 x 6
(65416) is on it inside the left arm, and with `sprocket`, the chain's gear 16 tooth (4019) inside the right arm, for
`chain_drive`: they go on with the wheel, before the axle, since nothing can slide onto it once its ends are closed.
Every axle here has a bush (3713) on each end, outside the arms, so nothing slides out. It brings its own steps and returns a dict of "rear_axle" (the axle's
middle, in the center plane), "arms" (the right arm's Handle, then the left's), "angle" (degrees below level), "drop"
(studs from the pivot down to the rear axle), "radius" (the wheel's, in studs; 0 with no wheel), and "sprocket" (the
sprocket's middle, or None without one).
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes and Barry's notes
on his e-bike hand sessions: the shock has to be the spring, every axle needs something to retain it, and nothing goes
on an axle after both its ends are closed (the disc and the sprocket from the retired disc_brake and chain_drive's
gear). Not derived from any model file."""

PARTS = [
    "40490",
    "32524",
    "32525",
    "41239",
    "32278",
    "73129",
    "3708",
    "3706",
    "3713",
    "65416",
    "4019",
    "32123a",
    "44772c01",
    "88517c01",
]

BEAMS = {6: "32524", 8: "40490", 10: "32525", 12: "41239", 14: "32278"}
# A wheel: the part, its hub's axle hole, its radius in studs.
WHEELS = {"motorcycle": ("88517c01", "3", 6.29), "fat": ("44772c01", "1", 5.92)}


def wheel_on(m, kind, center, color="Black"):
    """A wheel `kind` (WHEELS) whose hub is on an axle along y through `center`; returns its Handle."""
    part, hub, _ = WHEELS[kind]
    return m.add(part, color, at=center, rot=1, hole=hub)


def build(
    m,
    frame,
    reach=8,
    wheel="fat",
    disc=True,
    sprocket=True,
    color="Black",
    shock_color="Red",
    axle_color="Black",
    bush_color="Black",
    disc_color="Light Bluish Gray",
    sprocket_color="Light Bluish Gray",
    title="Swingarm",
):
    """The arms, the shock, and the rear axle (with the wheel); returns where the rear axle is."""
    if reach not in BEAMS:
        raise ValueError(
            f"swingarm_shock: reach {reach} isn't 6, 8, 10, 12, or 14 (a beam's end hole)"
        )
    if wheel is not None and wheel not in WHEELS:
        raise ValueError(
            f"swingarm_shock: wheel {wheel!r} isn't 'fat', 'motorcycle', or None"
        )
    to_model, y = frame["to_model"], frame["y"]
    th = math.radians(frame["arm_angle"])
    back = (-math.cos(th), -math.sin(th))  # a stud along the arm, from the pivot back
    half = frame["planes"]["arm"]
    shock_low = to_model(*back)
    rear = to_model(reach * back[0], reach * back[1])
    radius = WHEELS[wheel][2] if wheel else 0.0

    def bushes(center, part="3713", at=5.5):
        for s in (-1, 1):
            m.add(
                part, bush_color, at=(center[0], y + s * at, center[2]), rot=1, hole="1"
            )

    arms = []
    with m.step(f"{title}: arms"):
        for s in (-1, 1):
            arms.append(
                m.beam(
                    BEAMS[reach],
                    color,
                    at=to_model(0, 0, s * half),
                    to=to_model(reach * back[0], reach * back[1], s * half),
                    holes=(1, reach + 1),
                )
            )
        bushes(to_model(0, 0))  # on the frame's pivot axle, outside the arms
    with m.step(f"{title}: shock"):
        hole = arms[0].hole(2)
        m.insert(
            "3708",
            axle_color,
            hole,
            out="+y",
            out_by=y + 6 - (hole.at[1] + hole.length / 2),
        )
        bushes(shock_low)
        mount = frame["shock_mount"]
        m.beam("73129", shock_color, at=shock_low, to=mount, holes=(1, 2))
        # Either side of its eye, so it can't slide along the mount axle
        bushes(mount, part="32123a", at=0.75)
        # The mount axle, in last, through the down tubes, the half bushes, and the shock's eye
        hole = frame["shock_mount_hole"]
        m.insert(
            "3706",
            axle_color,
            hole,
            out="+y",
            out_by=y + 3 - (hole.at[1] + hole.length / 2),
        )
    inside = half - 1  # just inside each arm
    gear = (rear[0], y - inside, rear[2]) if sprocket else None
    with m.step(f"{title}: rear wheel" if wheel else f"{title}: rear axle"):
        if wheel:
            wheel_on(m, wheel, rear)
        # On before the axle: once it's through both arms, nothing can slide onto it.
        if disc:
            m.add("65416", disc_color, at=(rear[0], y + inside, rear[2]), rot=1, hole=1)
        if sprocket:
            m.add("4019", sprocket_color, at=gear, rot=1, hole=1)
        hole = arms[0].hole(reach + 1)
        m.insert(
            "3708",
            axle_color,
            hole,
            out="+y",
            out_by=y + 6 - (hole.at[1] + hole.length / 2),
        )
        bushes(rear)
    return {
        "rear_axle": rear,
        "arms": arms,
        "angle": frame["arm_angle"],
        "drop": reach * math.sin(th),
        "radius": radius,
        "reach": reach,
        "sprocket": gear,
    }


def example(m):
    """The swingarm and its shock on a frame, its rear wheel on the table."""
    frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
    m.use("swingarm_shock", frame=frame)
