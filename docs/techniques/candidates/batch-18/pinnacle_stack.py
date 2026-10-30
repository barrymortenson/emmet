# SPDX-License-Identifier: Apache-2.0
"""Pinnacle stack: small pointed Gothic pinnacles, a stack of round plates under a cone and a slim spike, set in rows on
buttresses and parapets so the roofline bristles.

m.use("pinnacle_stack", x, y, z, count=4, spacing=2, axis="x", color="Tan", spike=True, finial="Dark Tan",
title="Pinnacles") stands `count` pinnacles on the studs at height z, the first on the stud at (x, y) and the rest every
`spacing` studs along `axis` ("x" or "y"). Each is a submodel built once: a 1 x 1 square plate, two 1 x 1 round plates,
a cone 1 x 1, and, with spike, a slim wand in `finial` in the cone's hollow stud as the finial, about 3.5 bricks tall in
all. Put them on every buttress top and along a parapet, last, in quantity. Returns the z of the cones' top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file."""

PARTS = ["3024", "6141", "4589", "36752a"]

WAND = 0.4  # the wand's thickness, in studs


def _pinnacle(m, color, spike, finial, title):
    """One pinnacle in its own frame, on the stud at the origin."""
    with m.step(f"{title}, pinnacle"):
        m.add("3024", color, at=(0, 0, 0))
        m.stack("6141", color, at=(0, 0, 1), count=2)
        m.add("4589", color, at=(0, 0, 3))
        if spike:
            m.add("36752a", finial, at=(0.5 - WAND / 2, 0.5 - WAND / 2, 5.5))


def build(
    m,
    x,
    y,
    z,
    count=4,
    spacing=2,
    axis="x",
    color="Tan",
    spike=True,
    finial="Dark Tan",
    title="Pinnacles",
):
    """The pinnacles; returns the z of the cones' top."""
    if count < 1 or spacing < 1 or axis not in ("x", "y"):
        raise ValueError(
            f"pinnacle_stack takes a count and spacing of 1 or more and an axis of x or y, not {count}, {spacing}, "
            f"{axis!r}"
        )
    sub = m.submodel(
        "pinnacle", _pinnacle, color=color, spike=spike, finial=finial, title=title
    )
    for i0 in range(0, count, 4):
        with m.step(f"{title} {i0 + 1}-{min(count, i0 + 4)}"):
            for i in range(i0, min(count, i0 + 4)):
                d = i * spacing
                m.place(sub, at=(x + d, y, z) if axis == "x" else (x, y + d, z))
    return z + 6


def example(m):
    """A parapet with four pinnacles."""
    from emmet_dsl import Brick

    with m.step("Parapet"):
        m.row(Brick, "Tan", at=(0, 0, 0), length=8)
    with m.step("Parapet, coping"):
        for k in range(4):
            m.add("3005", "Tan", at=(2 * k, 0, 3))
            m.add("3070b", "Dark Tan", at=(2 * k + 1, 0, 3))
    m.use("pinnacle_stack", x=0, y=0, z=6)
