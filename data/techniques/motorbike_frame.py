# SPDX-License-Identifier: Apache-2.0
"""Motorbike frame: a display motorbike's frame of Technic beams on edge, laid by their holes on whole-stud
triangles, so every joint is a pin or an axle in a hole and the frame carries the swingarm, the fork, the chain, and
a stand.

m.use("motorbike_frame", x, y, z, color="Black", pin_color="Black", axle_color="Light Bluish Gray", on=None,
title="Frame")
builds it around the swingarm's pivot at (x, z), the bike running along +x and its two sides 8 studs apart across the
center plane y. Each side is a triangle of beams, their holes along y: a top spar (a 15-hole beam, 32278) rising 5
studs over 12 from the pivot to the steering head, past the head by a hole; a bottom spar (7 holes, 32524) falling 5
studs over 12 from the pivot for 6 of them; and a down tube (9 holes, 40490) from the bottom spar's end up 8 studs to
the top spar's hole 12, pinned at both ends (friction pins, 2780): sides 11, 6, and 8, so the triangle can't rack.
The right side (-y) goes together first, then an axle 10 long (3737) through its pivot, the left side onto that axle,
and another through the head: each a stud out past each side, for the swingarm and the fork. On a stand, `on` is the
hole that holds the bike at its pivot (`display_stand_nameplate` returns it): the pivot axle goes through it first,
and both sides onto the axle. From the outside in, each side's planes are the top spar's, the down tube's, and the bottom spar's; the
swingarm and the fork go outside them, the chain inside the swingarm. It brings its own steps and returns the frame:
a dict of "pivot" and "head" (the steering head, the top spar's hole 15), "y" (the center plane), "width",
"planes" (the y of the outside, top, down, and bottom planes on the right, -y side), and the right side's "top",
"bottom", and "down" Handles, whose holes the other display-bike techniques use (`swingarm_shock`,
`suspension_fork`, `chain_drive`, `display_stand_nameplate`).
"""

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic frames. Not derived from
any model file."""

PARTS = ["32278", "32524", "40490", "2780", "3737"]

WIDTH = 8


def _side(m, x, y, z, s, color, pin_color, title):
    """One side, all of it, before the other (its pins go in from inside the frame): s is -1 for the right side (-y,
    the bike facing +x), 1 for the left."""
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
        top = m.beam("32278", color, at=(x, planes["top"], z), run=12, rise=5)
        bottom = m.beam(
            "32524",
            color,
            at=(x, planes["bottom"], z),
            run=12 * 6 / 13,
            rise=-5 * 6 / 13,
        )
    with m.step(f"{title}, {side} side: down tube"):
        b, t = bottom.hole(7).at, top.hole(12).at
        down = m.beam(
            "40490",
            color,
            at=(b[0], planes["down"], b[2]),
            run=t[0] - b[0],
            rise=(t[2] - b[2]) / 2.5,
        )
        m.insert("2780", pin_color, top.hole(12), out=inward)
        m.insert("2780", pin_color, bottom.hole(7), out=outward)
    return planes, top, bottom, down


def build(
    m,
    x,
    y,
    z,
    color="Black",
    pin_color="Black",
    axle_color="Light Bluish Gray",
    on=None,
    title="Frame",
):
    """The two sides and the axles between them; returns the frame (see the module's doc)."""
    # The right side, the pivot axle through it, the left side onto the axle, and the head axle: each axle a stud out
    # past each side, for the swingarm and the fork. On a stand, the pivot axle goes through it first, and both sides
    # onto the axle.
    if on is not None:
        with m.step(f"{title}: pivot axle"):
            reach = (
                on.at[1] - on.length / 2 - (y - WIDTH / 2 - 1)
            )  # from the hole's -y face to the axle's end
            m.insert("3737", axle_color, on, out="-y", out_by=reach)
    planes, top, bottom, down = _side(m, x, y, z, -1, color, pin_color, title)
    if on is None:
        with m.step(f"{title}: pivot axle"):
            m.insert("3737", axle_color, top.hole(1), out="+y", out_by=WIDTH)
    _side(m, x, y, z, 1, color, pin_color, title)
    with m.step(f"{title}: head axle"):
        m.insert("3737", axle_color, top.hole(15), out="+y", out_by=WIDTH)
    head = top.hole(15).at
    return {
        "pivot": (x, y, z),
        "head": (head[0], y, head[2]),
        "y": y,
        "width": WIDTH,
        "planes": planes,
        "top": top,
        "bottom": bottom,
        "down": down,
    }


def example(m):
    """The frame on its own, its lowest point on the table."""
    m.use("motorbike_frame", x=0, y=4, z=8)
