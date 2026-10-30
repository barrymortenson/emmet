# SPDX-License-Identifier: Apache-2.0
"""Curved wedge nose: the rounded, sloping nose of a high-speed train, curved slopes rising from headlights on brackets
into a curved canopy windscreen.

m.use("curved_wedge_nose", x, y, z, color="White", glass="Trans Light Blue", lens="Trans-Clear", title="Nose") builds
a 6-wide nose on the plate whose front edge is at x and whose top is z (y to y + 6), its front to -x: a row of three
brackets 1 x 2 - 1 x 2 up (99780) along the front edge, their faces to the front with a clear 1 x 1 tile at each
corner as the headlights and tiles between; three curved slopes 4 x 2 (93606) on them rising from the front edge to 4
plates; a dash brick and side bricks behind, each with a plate on top; and on them a curved windscreen 3 x 6 (62360)
that carries the curve on up to 7 plates, so the nose and the cab glass read as one sweep. The nose is 7 studs long.
Returns the z of the windscreen's top, where the roof goes (from x + 7 back).
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 60337. Not derived from any
model file."""

PARTS = [
    "99780",
    "3070b",
    "3069b",
    "3795",
    "3666",
    "93606",
    "3009",
    "3004",
    "3023b",
    "62360",
]

FACE = Rot(y=-90)  # a tile on its back, its face to -x, long along y


def build(
    m,
    x,
    y,
    z,
    color="White",
    glass="Trans Light Blue",
    lens="Trans-Clear",
    title="Nose",
):
    """The lights, the curved slopes, and the canopy; returns the z of the canopy's top."""
    with m.step(f"{title}: brackets"):
        for b in range(0, 6, 2):
            m.add("99780", color, at=(x - 0.2, y + b, z), rot=2)
        m.add("3795", color, at=(x + 1, y, z), rot=0)
        m.add("3666", color, at=(x + 3, y, z), rot=0)
    with m.step(f"{title}: headlights"):
        for k in range(6):
            if k in (2, 3):
                continue
            m.add(
                "3070b",
                lens if k in (0, 5) else color,
                at=(x - 0.6, y + k, z),
                rot=FACE,
            )
        m.add("3069b", color, at=(x - 0.6, y + 2, z), rot=FACE)
    with m.step(f"{title}: curved slopes"):
        for k in range(3):
            m.add("93606", color, at=(x, y + 2 * k, z + 1), rot=2)
    with m.step(f"{title}: dash"):
        m.add("3009", color, at=(x + 4, y, z), rot=0)
        m.add("3666", color, at=(x + 4, y, z + 3), rot=0)
        for side in (y, y + 5):
            m.add("3004", color, at=(x + 5, side, z), rot=1)
            m.add("3023b", color, at=(x + 5, side, z + 3), rot=1)
    with m.step(f"{title}: canopy"):
        m.add("62360", glass, at=(x + 4, y, z + 4), rot=2)
    return z + 7


def example(m):
    """A train's front end: the nose on a 6-wide base, with the cab's walls and roof behind the canopy."""
    with m.step("Base"):
        m.add("3033", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    top = m.use("curved_wedge_nose", x=0, y=0, z=1)
    for r in range(2):
        with m.step(f"Cab, row {r + 1}"):
            for side in (0, 5):
                m.add("3004", "White", at=(7, side, 1 + 3 * r), rot=1)
            m.add("3009", "White", at=(9, 0, 1 + 3 * r), rot=0)
    with m.step("Cab, plates"):
        for side in (0, 5):
            m.add("3023b", "White", at=(7, side, 7), rot=1)
        m.add("3666", "White", at=(9, 0, 7), rot=0)
    with m.step("Cab, roof"):
        m.add("3795", "White", at=(7, 0, top), rot=0)
        m.add("3666", "White", at=(9, 0, top), rot=0)
