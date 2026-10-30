# SPDX-License-Identifier: Apache-2.0
"""Slope hood small window: a small upper-floor window under a tiny hood, a round plate and a slope sloping out over it.

m.use("slope_hood_small_window", x, y, z, color="Sand Blue", frame="Lavender", trim="White", hood="Dark Blue",
title="Small window") sets, in a wall one stud thick along x facing front (-y), a Plate 1 x 2 in the wall color at (x,
y, z), a Window 1 x 2 x 2 with its glass (60592, 60601) on it, a Plate 2 x 2 Round in the trim color over the window
that stands a stud out of the wall, a Slope 45 2 x 2 on it sloping out over the window, and a Plate 1 x 2 on the slope's
back row. It takes the wall's cells x and x + 1 from z to z + 12, four bricks' height; repeat it every four studs along
an upper floor. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["3023b", "60592", "60601", "4032b", "3039"]


def build(
    m,
    x,
    y,
    z,
    color="Sand Blue",
    frame="Lavender",
    trim="White",
    hood="Dark Blue",
    title="Small window",
):
    """The plate, the window, and the hood; returns the z of the top."""
    with m.step(f"{title}: window"):
        m.add("3023b", color, at=(x, y, z), rot=1)
        f = m.add("60592", frame, at=(x, y, z + 1), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
    with m.step(f"{title}: hood"):
        m.add("4032b", trim, at=(x, y - 1, z + 7))
        m.add("3039", hood, at=(x, y - 1, z + 8), rot=3)
        m.add("3023b", color, at=(x, y, z + 11), rot=1)
    return z + 12


def example(m):
    """Two small windows in an upper-floor wall, four studs apart."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Sand Blue", at=(0, 0, 0), length=10)
    for x in (2, 6):
        m.use("slope_hood_small_window", x=x, y=0, z=3)
    for r in range(4):
        with m.step(f"Wall between the windows, row {r + 2}"):
            for cx, n in ((0, 2), (4, 2), (8, 2)):
                m.row(Brick, "Sand Blue", at=(cx, 0, 3 + 3 * r), length=n)
    with m.step("Wall over the windows"):
        m.row(Brick, "Sand Blue", at=(0, 0, 15), length=10)
