# SPDX-License-Identifier: Apache-2.0
"""Roof hatch: a small hatch box on a flat roof, a curb of bricks a brick or two tall around a 2 x 2 opening, with
a lid on hinges that stands open.

m.use("roof_hatch", x, y, z, rows=2, angle=60, color="Dark Bluish Gray", lid="Light Bluish Gray",
title="Roof hatch") builds, on the studs at height z, a curb 4 x 4 whose corner is (x - 1, y - 1), around the
opening x to x + 2, y to y + 2: `rows` courses of bricks, each laid so its corners belong to the other sides than the
course below's; the top course leaves its back middle for hatch_on_hinge's hinge bases, and the lid goes on with
m.use("hatch_on_hinge", ..., angle=angle); then a plate and tile frame round the lid, level with it when shut. It
brings its own steps and returns the z of the frame's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["3010", "3622", "3004", "3710", "3623", "2431", "63864", "3937", "6134"]


def build(
    m,
    x,
    y,
    z,
    rows=2,
    angle=60,
    color="Dark Bluish Gray",
    lid="Light Bluish Gray",
    title="Roof hatch",
):
    """The curb, the hinged lid, and the frame; returns the z of the frame's top."""
    if rows < 1:
        raise ValueError(f"roof_hatch needs 1 or more rows, not {rows}")
    for r in range(rows):
        top = r == rows - 1
        with m.step(f"{title}, curb row {r + 1}"):
            if (rows - 1 - r) % 2 == 0:
                # Front and back own the corners... except the top course's back middle, which holds the hinges.
                m.add("3010", color, at=(x - 1, y - 1, z + 3 * r), rot=1)
                m.add("3622", color, at=(x - 1, y, z + 3 * r))
                m.add("3622", color, at=(x + 2, y, z + 3 * r))
                if not top:
                    m.add("3004", color, at=(x, y + 2, z + 3 * r), rot=1)
            else:
                m.add("3010", color, at=(x - 1, y - 1, z + 3 * r))
                m.add("3010", color, at=(x + 2, y - 1, z + 3 * r))
                m.add("3004", color, at=(x, y - 1, z + 3 * r), rot=1)
                m.add("3004", color, at=(x, y + 2, z + 3 * r), rot=1)
    zt = z + 3 * (rows - 1)
    m.use(
        "hatch_on_hinge",
        x=x,
        y=y,
        z=zt,
        width=2,
        depth=2,
        angle=angle,
        color=lid,
        hinge_color=color,
        title=title,
    )
    with m.step(f"{title}: frame"):
        m.add("3710", color, at=(x - 1, y - 1, zt + 3), rot=1)
        m.add("3623", color, at=(x - 1, y, zt + 3))
        m.add("3623", color, at=(x + 2, y, zt + 3))
    with m.step(f"{title}: frame tiles"):
        m.add("2431", lid, at=(x - 1, y - 1, zt + 4), rot=1)
        m.add("63864", lid, at=(x - 1, y, zt + 4))
        m.add("63864", lid, at=(x + 2, y, zt + 4))
    return zt + 5


def example(m):
    """A patch of flat roof with a hatch standing open beside a chimney."""
    from emmet_dsl import Plate

    with m.step("Roof"):
        m.fill(Plate, "Light Bluish Gray", at=(0, 0, 0), size=(8, 6))
    m.use("roof_hatch", x=2, y=2, z=1)
    m.use("chimney_stack", x=6, y=3, z=1, rows=3)
