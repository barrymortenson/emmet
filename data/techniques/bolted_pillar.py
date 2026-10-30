# SPDX-License-Identifier: Apache-2.0
"""Bolted pillar: a gray game pillar 1 x 2 with handholds sticking out both ends at every brick, topped by a round
platform to land on.

m.use("bolted_pillar", x, y, z, levels=4, color="Light Bluish Gray", handles="Yellow", platform="Green",
title="Pillar") stands, on the studs at height z over (x, y) to (x + 1, y + 2), `levels` bricks 1 x 2, each on a plate
1 x 2 with handles on opposite ends (18649) whose bars stick out 0.8 studs past each end, along y, then a 1 x 2 jumper
whose one stud is the pillar's middle and on it a round plate 4 x 4 (60474), centered over the pillar: from (x - 1.5,
y - 1) to (x + 2.5, y + 3). Returns the z of the platform's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72040. Not derived from any
model file."""

PARTS = ["3004", "18649", "3794b", "60474"]

REACH = 0.8  # how far a handle sticks out past the end of its plate, in studs


def build(
    m,
    x,
    y,
    z,
    levels=4,
    color="Light Bluish Gray",
    handles="Yellow",
    platform="Green",
    title="Pillar",
):
    """The pillar and its platform; returns the z of the platform's top."""
    if levels < 1:
        raise ValueError(f"bolted_pillar needs 1 or more levels, not {levels}")
    zt = z
    for k in range(levels):
        with m.step(f"{title}, level {k + 1}"):
            m.add("18649", handles, at=(x, y - REACH, zt))
            m.add("3004", color, at=(x, y, zt + 1))
        zt += 4
    with m.step(f"{title}: platform"):
        m.add("3794b", color, at=(x, y, zt))
        m.add("60474", platform, at=(x - 1.5, y - 1, zt + 1))
    return zt + 2


def example(m):
    """A short pillar and a tall one on a strip of ground."""
    from emmet_dsl import Plate

    with m.step("Ground"):
        m.fill(Plate, "Bright Green", at=(0, 0, 0), size=(10, 4))
    m.use("bolted_pillar", x=2, y=1, z=1, levels=2)
    m.use("bolted_pillar", x=7, y=1, z=1, levels=4, title="Tall pillar")
