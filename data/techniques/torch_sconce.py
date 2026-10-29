# SPDX-License-Identifier: Apache-2.0
"""Torch sconce: torches held out from a wall in clips, the clip bricks laid in one of the wall's own courses.

m.use("torch_sconce", x, y, z, length, at=None, color="Light Bluish Gray", torch="Reddish Brown", flame="Trans-Orange",
title="Torches") lays one course of a wall 1 stud thick and `length` studs long along x from (x, y), from z, with a
Brick 1 x 1 with Clip Vertical (60475b) at each of `at` (studs from the wall's start; by default one in the middle), its
clip out front (-y). A minifig torch (3959) stands upright in each clip, its handle gripped by the clip, and a flame
(64647) plugs into its head. Lay the courses below and above as usual; the torch and its flame reach about two bricks
above the course, a stud out from the wall. Returns the z of the course's top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "60475b",
    "3959",
    "64647",
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

CLIP = 0.7  # how far the clip stands out from the brick's face
UPRIGHT = Rot(y=-90)  # the torch on end, its head up


def build(
    m,
    x,
    y,
    z,
    length,
    at=None,
    color="Light Bluish Gray",
    torch="Reddish Brown",
    flame="Trans-Orange",
    title="Torches",
):
    """The course with its clip bricks, then the torches and flames; returns the z of the course's top."""
    spots = [length // 2] if at is None else sorted(at)
    if (
        not spots
        or spots[0] < 0
        or spots[-1] > length - 1
        or len(set(spots)) < len(spots)
    ):
        raise ValueError(
            f"torch_sconce needs its torches at different studs along the wall, not at {at}"
        )
    with m.step(f"{title}: course"):
        u = 0
        for s in spots + [length]:
            if s > u:
                m.row(Brick, color, at=(x + u, y, z), length=s - u)
            if s < length:
                m.add("60475b", color, at=(x + s, y - CLIP, z), rot=3)
            u = s + 1
    with m.step(f"{title}: torches"):
        for s in spots:
            # The torch's handle centered in the clip, half a stud in front of the wall.
            t = m.add("3959", torch, at=(x + s - 0.15, y - 1, z), rot=UPRIGHT)
            m.add("64647", flame, at=(x + s - 0.37, y - 0.7, t.max[2] - 1))
    return z + 3


def example(m):
    """A wall with a course of two torches."""
    with m.step("Wall, row 1"):
        m.row(Brick, "Light Bluish Gray", at=(0, 0, 0), length=6)
    m.use("torch_sconce", x=0, y=0, z=3, length=6, at=[1, 4])
    with m.step("Wall, row 3"):
        m.row(Brick, "Light Bluish Gray", at=(0, 0, 6), length=6, first=2)
