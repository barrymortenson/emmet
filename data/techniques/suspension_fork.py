# SPDX-License-Identifier: Apache-2.0
"""Suspension fork: a display motorbike's front fork that steers, a triple tree on a steerer turning in the frame's
head bearing, two fork tubes through it, a shock absorber on each, and the front wheel between them.

m.use("suspension_fork", frame=frame, color="Black", tube_color="Light Bluish Gray", shock_color="Yellow",
axle_color="Light Bluish Gray", wheel=True, drop=4.15, title="Fork") takes the frame `motorbike_frame` returned. An
axle 3 (4519), the steerer, goes through the frame's head bearing along the raked steering axis; on it, a stud above
and a stud below the bearing, the upper and lower triple clamps, each a 7-hole beam (32524) across the bike with the
steerer through its middle hole. The fork is held to the frame by the steerer alone, turning in the bearing's round
hole, so it steers. The fork tubes, an axle 10 (3737) through each clamp's end holes, run down parallel to the
steering axis, 3 studs either side of the center, a stud above the upper clamp for `handlebars`. Low on each tube a
Technic cross block 1 x 2 (6536), the slider, turns the line to a pin hole across the bike a stud forward of the tube;
a shock absorber 6.5L (73129) hangs from it outside the tube (a friction pin, 2780), and an axle 10 through the
shocks' lower holes carries the front wheel (88517c01) when wheel is True. The sliders sit so the front axle is `drop`
studs below the pivot, level with `swingarm_shock`'s rear axle at the defaults. It brings its own steps and returns
a dict of "front_axle" (the axle's middle, in the center plane), "tubes" (the fork tubes' Handles, the right one
first), "shocks" (likewise), "clamps" (upper, lower), "rake" (a stud down the steering axis, as run and rise),
"forward" (a stud square to it, forward), and "half" (the tubes' studs from the center).
"""

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes and Barry's
note that the bars turn on a triple tree and a pivot. Not derived from any model file."""

PARTS = ["4519", "32524", "3737", "6536", "73129", "2780", "88517c01"]

SHOCK = 5.5  # studs between the shock's holes
HALF = 3  # the fork tubes, studs from the center (a 7-hole clamp's end holes)


def build(
    m,
    frame,
    color="Black",
    tube_color="Light Bluish Gray",
    shock_color="Yellow",
    axle_color="Light Bluish Gray",
    wheel=True,
    drop=4.15,
    title="Fork",
):
    """The steerer, the triple clamps, the tubes, the sliders and shocks, and the front axle (with the wheel); returns
    where the axle is."""
    steering, y = frame["steering"], frame["y"]
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
                "32524",
                color,
                at=along(t, side=-HALF),
                to=along(t, side=HALF),
                axis=down,
            )
            for t in (-1, 1)
        ]
    with m.step(f"{title}: fork tubes"):
        tubes = [
            m.insert("3737", tube_color, clamps[0].hole(n), out=down, out_by=8)
            for n in (1, 7)
        ]
    # The sliders' depth on the tubes: the front axle `drop` studs below the pivot.
    pivot = frame["pivot"]
    slide = ((c[2] - pivot[2]) / 2.5 + fz + drop) / -rz - SHOCK
    shocks = []
    with m.step(f"{title}: sliders and shocks"):
        for s, out in ((-1, "-y"), (1, "+y")):
            block = m.beam(
                "6536",
                color,
                at=along(slide, side=s * HALF),
                to=along(slide, forward=1, side=s * HALF),
                axis=down,
                holes=(2, 1),
            )
            top = block.hole(1).at
            at = (top[0], y + s * (HALF + 1), top[2])
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
            m.add("88517c01", "Black", at=(end[0], y, end[2]), rot=1, hole=3)
        m.insert("3737", axle_color, shocks[0].hole(1), out="+y", out_by=HALF + 5.5)
    return {
        "front_axle": (end[0], y, end[2]),
        "tubes": tubes,
        "shocks": shocks,
        "clamps": clamps,
        "rake": frame["rake"],
        "forward": frame["forward"],
        "half": HALF,
    }


def example(m):
    """The fork on a frame, its front wheel on the table."""
    frame = m.use("motorbike_frame", x=0, y=6, z=27)
    m.use("suspension_fork", frame=frame)
