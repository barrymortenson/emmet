# SPDX-License-Identifier: Apache-2.0
"""Suspension fork: a display motorbike's or e-bike's front fork that steers and springs, a triple tree on a steerer
turning in the frame's head bearing, a fork tube through it each side, a slider on each tube carrying the front axle,
and a shock absorber each side between the tube and the axle, so the wheel can only move along the fork, against the
springs.

m.use("suspension_fork", frame=frame, swingarm=swingarm, wheel="fat", discs=True, fender=False, color="Black",
tube_color="Black", shock_color="Red", axle_color="Black", bush_color="Light Bluish Gray", disc_color="Light Bluish
Gray", bracket_color="Black", title="Fork") takes what `motorbike_frame` and `swingarm_shock` returned. An axle 4 (3705), the steerer, goes through the frame's head bearing along the steering axis,
with a half bush (32123a) on each end; on it, a stud above and a stud below the bearing, the upper and lower triple
clamps, a 9-hole beam (40490; 7-hole, 32524, for the "motorcycle" wheel) across the bike with the steerer through its
middle hole. The fork turns on the steerer alone. The fork tubes, an axle 16 (50451) through each clamp's end holes,
run down along the steering axis, locked to the clamps by a half bush above the upper one and one below the lower one;
their tops stand above the upper clamp for `handlebars`. Low on each tube a Technic cross block 1 x 2 (6536), the
slider, slides on the tube by its pin hole; the front axle, an axle 12 (3708; 10, 3737, for the narrower fork), goes
through the sliders' axle holes a stud forward of the tubes, so they carry it, and through the wheel's hub. Higher up
each tube another cross block is fixed by its axle hole, and a shock absorber 6.5L (73129) runs from a pin in it down
to the front axle outside the slider, so the shocks are the springs that hold the wheel down the fork. The sliders sit
so the front axle is level with the rear one. With `discs`, a brake disc 6 x 6 (65416) is on the front axle against the
inside of each slider, which stands in for the caliper; with `fender`, a Technic cross block 2 x 2 (32291) is on each
fork tube just under the lower clamp's bushes, for `front_fender` to hang its blade on. Both go on with their axle or
tube, before it closes: nothing can slide onto it once both its ends are shut. `wheel` is "fat" (44772c01), "motorcycle" (88517c01), or None for the
axle alone. It brings its own steps and returns a dict of "front_axle" (the axle's middle, in the center plane),
"tubes" (the fork tubes' Handles, the right one first), "sliders", "shocks", "clamps" (upper, lower), "brackets"
(the fender's cross blocks, the right one first, or None), "bracket_at" (their axle holes, in studs down the fork),
"rake" (a stud
down the steering axis, as run and rise), "forward" (a stud square to it, forward), "half" (the tubes' studs from the
center), "travel" (studs down the steering axis from the bearing to the front axle), and "along" (a function from t
studs down the steering axis, f studs forward, and studs across to a point in the model).
"""

import math

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes and Barry's notes
on his e-bike hand sessions: the bars turn on a triple tree and a pivot, the wheel must slide on the fork tubes rather
than hang on the shocks, the axles need something to retain them, and nothing goes on an axle or a tube after both its
ends are closed (the discs from the retired disc_brake, the fender's brackets). Not derived from any model file."""

PARTS = [
    "3705",
    "32123a",
    "40490",
    "32524",
    "50451",
    "6536",
    "73129",
    "2780",
    "3708",
    "3737",
    "65416",
    "32291",
    "44772c01",
    "88517c01",
]

SHOCK = 5.5  # studs between a shock absorber 6.5L's holes
# A wheel: the part, its hub's axle hole, its radius, and the fork tubes' studs from the center.
WHEELS = {"motorcycle": ("88517c01", "3", 6.29, 3), "fat": ("44772c01", "1", 5.92, 4)}
CLAMPS = {3: "32524", 4: "40490"}
TUBE = 16  # the fork tubes' length, an axle 16
BRACKET = 3.0  # the fender's brackets' axle holes, studs down the fork: under the lower clamp's bushes


def build(
    m,
    frame,
    swingarm,
    wheel="fat",
    discs=True,
    fender=False,
    color="Black",
    tube_color="Black",
    shock_color="Red",
    axle_color="Black",
    bush_color="Light Bluish Gray",
    disc_color="Light Bluish Gray",
    bracket_color="Black",
    title="Fork",
):
    """The steerer, the triple clamps, the tubes, the sliders and shocks, and the front axle (with the wheel); returns
    where the axle is (see the module's doc)."""
    if wheel is not None and wheel not in WHEELS:
        raise ValueError(
            f"suspension_fork: wheel {wheel!r} isn't 'fat', 'motorcycle', or None"
        )
    steering, y = frame["steering"], frame["y"]
    half = WHEELS[wheel][3] if wheel else 4
    radius = WHEELS[wheel][2] if wheel else swingarm["radius"]
    (rx, rz), (fx, fz) = frame["rake"], frame["forward"]
    down = (rx, 0.0, rz)
    # A bush's hole turned onto the steering axis
    lean = Rot(y=math.degrees(math.atan2(-rz, rx)))
    c = steering.at

    def along(t, f=0.0, side=0.0):
        return (c[0] + t * rx + f * fx, y + side, c[2] + 2.5 * (t * rz + f * fz))

    # The front axle, a stud forward of the steering axis, as high above the table as the wheel's radius.
    rear = swingarm["rear_axle"]
    level = rear[2] - 2.5 * swingarm["radius"] + 2.5 * radius
    travel = ((level - c[2]) / 2.5 - fz) / rz
    if not 8 <= travel <= 12.5:
        raise ValueError(
            f"suspension_fork: the front axle is {travel:.1f} studs down the fork from the head; 8 to 12.5 fit. Move "
            "the frame's head up or down, or change the rake"
        )
    if fender and travel < 10:
        raise ValueError(
            f"suspension_fork: the front axle is {travel:.1f} studs down the fork; a fender needs 10 or more, for room "
            "between the lower clamp and the tire"
        )

    def bush(t, side):
        return m.add("32123a", bush_color, at=along(t, side=side), rot=lean, hole="1")

    with m.step(f"{title}: steerer and triple clamps"):
        m.insert("3705", axle_color, steering, out=down, out_by=1.5)
        for t in (-1.75, 1.75):
            bush(t, 0)
        clamps = [
            m.beam(
                CLAMPS[half],
                color,
                at=along(t, side=-half),
                to=along(t, side=half),
                axis=down,
            )
            for t in (-1, 1)
        ]
    brackets = None
    with m.step(f"{title}: fork tubes"):
        if fender:
            # On before the tubes: once a tube's through both clamps with its bushes on, nothing slides onto it.
            r = math.degrees(math.atan2(rx, -rz))
            brackets = [
                m.add(
                    "32291",
                    bracket_color,
                    at=along(BRACKET, side=s * half),
                    rot=Rot(x=-(90 + r), z=90),
                    hole="2",
                )
                for s in (-1, 1)
            ]
        tubes = [
            m.insert(
                "50451", tube_color, clamps[0].hole(n), out=down, out_by=travel + 1.0
            )
            for n in (1, 2 * half + 1)
        ]
        for s in (-1, 1):
            # Above the upper clamp and below the lower one: the tubes can't slide
            for t in (-1.75, 1.75):
                bush(t, s * half)
    sliders, shocks = [], []
    with m.step(f"{title}: sliders and shocks"):
        for s in (-1, 1):
            sliders.append(
                m.beam(
                    "6536",
                    color,
                    at=along(travel, side=s * half),
                    to=along(travel, f=1, side=s * half),
                    axis=down,
                    holes=(1, 2),
                )
            )
            top = m.beam(
                "6536",
                color,
                at=along(travel - SHOCK, side=s * half),
                to=along(travel - SHOCK, f=1, side=s * half),
                axis=down,
                holes=(2, 1),
            )
            shocks.append(
                m.beam(
                    "73129",
                    shock_color,
                    at=along(travel, f=1, side=s * (half + 1)),
                    to=along(travel - SHOCK, f=1, side=s * (half + 1)),
                    holes=(1, 2),
                )
            )
            m.insert("2780", "Black", top.hole(1), out="-y" if s < 0 else "+y")
    axle = along(travel, f=1)
    with m.step(f"{title}: front wheel" if wheel else f"{title}: front axle"):
        if wheel:
            part, hub = WHEELS[wheel][:2]
            m.add(part, "Black", at=axle, rot=1, hole=hub)
        if discs:
            # On before the axle: once it's through both sliders, nothing can slide onto it.
            for s in (-1, 1):
                m.add(
                    "65416",
                    disc_color,
                    at=(axle[0], y + s * (half - 1), axle[2]),
                    rot=1,
                    hole=1,
                )
        # Through the sliders and the shocks, and a half bush past each
        span = 2 * half + 4
        hole = sliders[0].hole(2)
        m.insert(
            {12: "3708", 10: "3737"}[span],
            axle_color,
            hole,
            out="+y",
            out_by=y + span / 2 - (hole.at[1] + 0.5),
        )
        for s in (-1, 1):
            m.add(
                "32123a",
                bush_color,
                at=along(travel, f=1, side=s * (half + 1.75)),
                rot=1,
                hole="1",
            )
    return {
        "front_axle": axle,
        "tubes": tubes,
        "sliders": sliders,
        "shocks": shocks,
        "clamps": clamps,
        "brackets": brackets,
        "bracket_at": BRACKET,
        "rake": frame["rake"],
        "forward": frame["forward"],
        "half": half,
        "travel": travel,
        "along": along,
    }


def example(m):
    """The fork on a frame and swingarm, both wheels on the table."""
    frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
    swingarm = m.use("swingarm_shock", frame=frame)
    m.use("suspension_fork", frame=frame, swingarm=swingarm)
