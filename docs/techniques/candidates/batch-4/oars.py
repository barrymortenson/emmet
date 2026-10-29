# SPDX-License-Identifier: Apache-2.0
"""Oars: a row of oars out through notches in a boat's gunwale, each held in a clip beside its notch as a rowlock.

m.use("oars", x, y, z, length, spacing=3, color="Reddish Brown", oar="Reddish Brown", lock="Black", title="Oars") lays
the top course of a boat's side, 1 stud thick and `length` studs long along x from (x, y), from z, with a notch a stud
wide every `spacing` studs (the first 2 studs in, and 2 studs of side after the last). On the course, just aft of each
notch, a plate 1 x 1 with a clip (4081b) points its clip forward (+x) over the notch, the clip's bar across the side; a
minifig oar (2542) lies in it, its shaft over the notch, its blade out past the side (-y) and its handle inboard, raised
level. Returns the z of the course's top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "4081b",
    "2542",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
]

OUTBOARD = Rot(x=-90)  # the oar lying across the side, its blade out to -y
INBOARD = 2.5  # how far the oar's handle end reaches in from the clip's axis
OAR = 7.2  # an oar's length, in studs; its blade is 2.5 plates wide, centered on the shaft


def build(
    m,
    x,
    y,
    z,
    length,
    spacing=3,
    color="Reddish Brown",
    oar="Reddish Brown",
    lock="Black",
    title="Oars",
):
    """The course with its notches, the rowlocks, and the oars; returns the z of the course's top."""
    if spacing < 2 or length < 5:
        raise ValueError(
            f"oars needs a spacing of 2 or more and a length of 5 or more, not {spacing}, {length}"
        )
    notches = list(range(2, length - 2, spacing))
    zt = z + 3
    with m.step(f"{title}: gunwale"):
        u = 0
        for n in notches + [length]:
            if n > u:
                m.row(Brick, color, at=(x + u, y, z), length=n - u)
            u = n + 1
    with m.step(f"{title}: rowlocks"):
        for n in notches:
            m.add("4081b", lock, at=(x + n - 1, y, zt - 0.25), rot=0)
    with m.step(f"{title}: oars"):
        for n in notches:
            # The clip's axis runs across the side a stud forward of its plate's center, over the notch.
            ax, az = x + n + 0.5, zt + 0.75
            m.add(
                "2542",
                oar,
                at=(ax - 0.2, y + 0.5 + INBOARD - OAR, az - 1.25),
                rot=OUTBOARD,
            )
    return zt


def example(m):
    """A short boat side with three oars."""
    with m.step("Side"):
        m.row(Brick, "Reddish Brown", at=(0, 0, 0), length=12)
    m.use("oars", x=0, y=0, z=3, length=12)
