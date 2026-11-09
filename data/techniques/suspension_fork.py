# SPDX-License-Identifier: Apache-2.0
"""Suspension fork: a display motorbike's front fork that steers, a triple tree on a steerer turning in the frame's head
bearing, two fork tubes through it, a shock absorber on each, and the front wheel between them.

m.use("suspension_fork", frame=frame, drop=4.15, wheel="motorcycle", color="Black", tube_color="Black",
shock_color="Red", axle_color="Black", title="Fork") takes the frame `motorbike_frame` returned. An axle 3 (4519), the
steerer, goes through the frame's head bearing along the raked steering axis; on it, a stud above and a stud below the
bearing, the upper and lower triple clamps, each a 7-hole beam (32524) across the bike with the steerer through its
middle hole. The fork is held to the frame by the steerer alone, turning in the bearing's round hole, so it steers. The
fork tubes, an axle 8 or 10 (3707, 3737: whichever reaches the sliders) through each clamp's end holes, run down
parallel to the steering axis, 3 studs either side of the center, a stud above the upper clamp for `handlebars`. Low on
each tube a Technic cross block 1 x 2 (6536), the slider, turns the line to a pin hole across the bike a stud forward of
the tube; a shock absorber 6.5L (73129) hangs from it outside the tube (a friction pin, 2780), and an axle 10 through
the shocks' lower holes carries the front wheel: "motorcycle", "fat", or None, as `swingarm_shock` has them (the "fat"
wheel's fork is a stud wider each side, on 9-hole clamps and an axle 12). The sliders sit so the front axle is `drop`
studs below the pivot: give it the swingarm's, and the wheels stand level. It brings its own steps and returns a dict of
"front_axle" (the axle's middle, in the center plane), "tubes" (the fork tubes' Handles, the right one first), "shocks"
(likewise), "clamps" (upper, lower), "rake" (a stud down the steering axis, as run and rise), "forward" (a stud square
to it, forward), and "half" (the tubes' studs from the center).
"""

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes and Barry's
note that the bars turn on a triple tree and a pivot. Not derived from any model file."""

PARTS = [
    "4519",
    "32524",
    "40490",
    "3707",
    "3737",
    "3708",
    "6536",
    "73129",
    "2780",
    "88517c01",
    "44772c01",
]

SHOCK = 5.5  # studs between the shock's holes
# A wheel: the part, its hub's axle hole, and how far out the fork tubes go for it (a 7- or 9-hole clamp's end holes).
WHEELS = {"motorcycle": ("88517c01", "3", 3), "fat": ("44772c01", "1", 4)}
CLAMPS = {3: "32524", 4: "40490"}


def build(
    m,
    frame,
    drop=4.15,
    wheel="motorcycle",
    color="Black",
    tube_color="Black",
    shock_color="Red",
    axle_color="Black",
    title="Fork",
):
    """The steerer, the triple clamps, the tubes, the sliders and shocks, and the front axle (with the wheel); returns
    where the axle is."""
    steering, y = frame["steering"], frame["y"]
    half = WHEELS[wheel][2] if wheel else 3  # the fork tubes, studs from the center
    (rx, rz), (fx, fz) = frame["rake"], frame["forward"]
    down = (rx, 0.0, rz)  # a stud down the steering axis, in space along the DSL's axes
    c = steering.at

    def along(t, forward=0.0, side=0.0):
        """The point t studs down the steering axis from the bearing's middle, `forward` studs forward of it, `side`
        studs across (DSL: x and y studs, z plates)."""
        return (
            c[0] + t * rx + forward * fx,
            y + side,
            c[2] + 2.5 * (t * rz + forward * fz),
        )

    with m.step(f"{title}: steerer and triple clamps"):
        m.insert("4519", tube_color, steering, out=down, out_by=1)
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
    # The sliders' depth on the tubes: the front axle `drop` studs below the pivot.
    pivot = frame["pivot"]
    slide = ((c[2] - pivot[2]) / 2.5 + fz + drop) / -rz - SHOCK
    # A tube runs from a stud above the upper clamp (t = -2.5) down past its slider.
    tube = next((n for n in (8, 10) if slide + 0.5 <= n - 2.5), None)
    if tube is None or slide < 2:
        raise ValueError(
            f"suspension_fork: drop {drop} puts the sliders {slide:.1f} studs down the fork; 2 to 7 fit"
        )
    with m.step(f"{title}: fork tubes"):
        part = {8: "3707", 10: "3737"}[tube]
        tubes = [
            m.insert(part, tube_color, clamps[0].hole(n), out=down, out_by=tube - 2)
            for n in (1, 2 * half + 1)
        ]
    shocks = []
    with m.step(f"{title}: sliders and shocks"):
        for s, out in ((-1, "-y"), (1, "+y")):
            block = m.beam(
                "6536",
                color,
                at=along(slide, side=s * half),
                to=along(slide, forward=1, side=s * half),
                axis=down,
                holes=(2, 1),
            )
            top = block.hole(1).at
            at = (top[0], y + s * (half + 1), top[2])
            shocks.append(
                m.beam(
                    "73129",
                    shock_color,
                    at=at,
                    run=SHOCK * rx,
                    rise=SHOCK * rz,
                    holes=(2, 1),
                )
            )
            m.insert("2780", "Black", block.hole(1), out=out)
    end = shocks[0].hole(1).at
    with m.step(f"{title}: front wheel" if wheel else f"{title}: front axle"):
        if wheel:
            part, hub, _ = WHEELS[wheel]
            m.add(part, "Black", at=(end[0], y, end[2]), rot=1, hole=hub)
        axle = (
            "3737" if half == 3 else "3708"
        )  # an axle 10, or 12 for the wider fork: through both shocks
        m.insert(
            axle,
            axle_color,
            shocks[0].hole(1),
            out="+y",
            out_by=half + 5.5 + (half - 3),
        )
    return {
        "front_axle": (end[0], y, end[2]),
        "tubes": tubes,
        "shocks": shocks,
        "clamps": clamps,
        "rake": frame["rake"],
        "forward": frame["forward"],
        "half": half,
    }


def example(m):
    """The fork on a frame, its front wheel on the table."""
    frame = m.use("motorbike_frame", x=0, y=6, z=27)
    m.use("suspension_fork", frame=frame)
