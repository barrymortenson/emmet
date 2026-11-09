# SPDX-License-Identifier: Apache-2.0
"""Motorbike frame: a display motorbike's frame of Technic beams on edge, a pinned triangle a side sized to the bike in
the photo, so every joint is a pin or an axle in a hole and the frame carries the swingarm, the fork, the chain, and a
kickstand.

m.use("motorbike_frame", x, y, z, head=(13.85, 5.77), bottom=(5.54, -2.31), rake=22.6, color="Black", pin_color="Black",
axle_color="Black", title="Frame") builds it around the swingarm's pivot at (x, z), the bike running along +x and its
two sides 8 studs apart across the center plane y. Measure the photo first: `head` is where the steering head (the head
tube's middle, the steering axis) is from the pivot and `bottom` where the frame's lower front corner is (the bottom
bracket's, on a bicycle), each (run, rise) in studs; `rake` is the steering axis's lean from upright, in degrees.

Each side is a triangle of beams, their holes along y: a top spar from the pivot to a stud behind the head (its length
rounded to a whole stud, 6 to 14; a beam's spare hole goes behind the pivot, never past the head), a bottom spar from
the pivot to the lower corner, and a down tube from there up to a hole of the top spar, pinned at both ends (friction
pins, 2780). The bottom spar's and down tube's lengths are the whole studs that put the corner nearest `bottom`: a
triangle with whole-stud sides closes on its holes at any angle, so it can't rack. The defaults are a long, low
motorbike (sides 14, 6, and 8 to the top spar's hole 12); an upright e-bike has its head higher and nearer, like
head=(12, 7.5). The right side (-y) goes together first, then an axle 10 long (3737) through its pivot, a stud out past
each side for the swingarm, and the left side onto that axle. The steering head is an axle 8 (3707) through the top
spars' last holes and, on it between the sides, a Technic cross block 1 x 2 (6536) whose pin hole, a stud forward, runs
along the steering axis: the bearing the fork's steerer turns in (`suspension_fork`). From the outside in, each side's
planes are the top spar's, the down tube's, and the bottom spar's; the swingarm and the fork go outside them.

It brings its own steps and returns the frame: a dict of "pivot", "steering" (the bearing's pin hole) and "head" (its
middle), "rake" (a stud down the steering axis, as run and rise) and "forward" (a stud square to it, forward), "y" (the
center plane), "width", "planes" (the y of the outside, top, down, and bottom planes on the right, -y side), and the
right side's "top", "bottom", and "down" Handles, "first" (the top spar's hole at the pivot, 1 or 2), and "lengths" (the
top spar's, where the down tube meets it, the bottom spar's, and the down tube's, in studs), whose holes the other
display-bike techniques use (`swingarm_shock`, `suspension_fork`, `chain_drive`, `kickstand`).
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic frames. Not derived from
any model file."""

PARTS = [
    "32523",
    "32316",
    "32524",
    "40490",
    "32525",
    "41239",
    "32278",
    "2780",
    "3737",
    "3707",
    "6536",
]

WIDTH = 8
BEAMS = {
    3: "32523",
    5: "32316",
    7: "32524",
    9: "40490",
    11: "32525",
    13: "41239",
    15: "32278",
}


def _first(reach):
    """The top spar's hole at the pivot: 1, or 2 when no beam ends at the head (an odd reach), so its spare hole is
    behind the pivot, not past the head where the fork turns."""
    return 1 if reach % 2 == 0 else 2


def _beam(span):
    """The shortest beam with a hole `span` studs from its hole 1."""
    return BEAMS[min(n for n in BEAMS if n >= span + 1)]


def _triangle(head, bottom):
    """The top spar's length to the head, the head (run, rise) at that length, and (t, n, k, corner): the top spar's
    hole t studs from the pivot where the down tube's top goes, the bottom spar's length n to the lower corner, and the
    down tube's length k, all whole studs, with the lower corner (run, rise from the pivot) nearest `bottom`."""
    far = math.hypot(*head)
    reach = round(far)
    if not 6 <= reach <= 14:
        raise ValueError(
            f"motorbike_frame: the head is {far:.1f} studs from the pivot; 6 to 14 fit"
        )
    ux, uz = head[0] / far, head[1] / far
    best = None
    for t in range(4, reach - 1):
        for n in range(2, 15):
            for k in range(3, 15):
                if not abs(n - k) < t < n + k:
                    continue
                # The corner: n from the pivot and k from the top spar's hole t, on the spar's lower side.
                a = (n * n - k * k + t * t) / (2 * t)
                h = math.sqrt(max(n * n - a * a, 0.0))
                corner = (a * ux + h * uz, a * uz - h * ux)
                score = math.hypot(corner[0] - bottom[0], corner[1] - bottom[1])
                if best is None or score < best[0] - 1e-9:
                    best = (score, t, n, k, corner)
    return reach, (ux * reach, uz * reach), best[1:]


def _side(m, x, y, z, s, shape, color, pin_color, title):
    """One side, all of it, before the other (its pins go in from inside the frame): s is -1 for the right side (-y,
    the bike facing +x), 1 for the left."""
    reach, head, (t, n, k, corner) = shape
    first = _first(reach)
    edge = y + s * WIDTH / 2
    planes = {
        "outside": edge + s * 0.5,
        "top": edge - s * 0.5,
        "down": edge - s * 1.5,
        "bottom": edge - s * 2.5,
    }
    inward, outward = ("+y", "-y") if s < 0 else ("-y", "+y")
    side = "right" if s < 0 else "left"
    with m.step(f"{title}, {side} side: spars"):
        top = m.beam(
            _beam(reach),
            color,
            at=(x, planes["top"], z),
            run=head[0],
            rise=head[1],
            holes=(first, reach + first),
        )
        bottom = m.beam(
            _beam(n),
            color,
            at=(x, planes["bottom"], z),
            run=corner[0],
            rise=corner[1],
            holes=(1, n + 1),
        )
    with m.step(f"{title}, {side} side: down tube"):
        b, up = bottom.hole(n + 1).at, top.hole(t + first).at
        rise = (up[2] - b[2]) / 2.5
        down = m.beam(
            _beam(k),
            color,
            at=(b[0], planes["down"], b[2]),
            run=up[0] - b[0],
            rise=rise,
            holes=(1, k + 1),
        )
        m.insert("2780", pin_color, top.hole(t + first), out=inward)
        m.insert("2780", pin_color, bottom.hole(n + 1), out=outward)
    return planes, top, bottom, down


def build(
    m,
    x,
    y,
    z,
    head=(13.846, 5.77),
    bottom=(5.538, -2.308),
    rake=22.6,
    color="Black",
    pin_color="Black",
    axle_color="Black",
    title="Frame",
):
    """The two sides, the pivot axle, and the steering head; returns the frame (see the module's doc)."""
    r = math.radians(rake)
    forward, down_axis = (math.cos(r), math.sin(r)), (math.sin(r), -math.cos(r))
    # The bearing (the steering axis) is at `head`, a stud forward of the top spar's end.
    shape = _triangle((head[0] - forward[0], head[1] - forward[1]), bottom)
    reach = shape[0]
    # The right side, the pivot axle through it (a stud out past each side, for the swingarm), the left side onto the
    # axle, and the steering head.
    planes, top, bottom_spar, down = _side(
        m, x, y, z, -1, shape, color, pin_color, title
    )
    with m.step(f"{title}: pivot axle"):
        m.insert("3737", axle_color, top.hole(_first(reach)), out="+y", out_by=WIDTH)
    _side(m, x, y, z, 1, shape, color, pin_color, title)
    first = _first(reach)
    at = top.hole(reach + first).at
    with m.step(f"{title}: steering head"):
        # A cross block on the head axle between the sides: its pin hole, a stud forward, runs along the steering axis,
        # the bearing the fork's steerer turns in.
        to = (at[0] + forward[0], y, at[2] + 2.5 * forward[1])
        bearing = m.beam(
            "6536", color, at=(at[0], y, at[2]), to=to, axis="y", holes=(2, 1)
        )
        m.insert(
            "3707", axle_color, top.hole(reach + first), out="+y", out_by=WIDTH - 1
        )
    steering = bearing.hole(1)
    return {
        "pivot": (x, y, z),
        "head": steering.at,
        "steering": steering,
        "rake": down_axis,
        "forward": forward,
        "y": y,
        "width": WIDTH,
        "planes": planes,
        "top": top,
        "bottom": bottom_spar,
        "down": down,
        "lengths": (reach, *shape[2][:3]),
        "first": first,
    }


def example(m):
    """The frame on its own, its lowest point on the table."""
    m.use("motorbike_frame", x=0, y=4, z=8)
