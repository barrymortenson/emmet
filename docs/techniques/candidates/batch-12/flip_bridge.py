# SPDX-License-Identifier: Apache-2.0
"""Flip bridge: a plank bridge on hinges between two pillars, with a push rod through the far pillar under its free
end, the game trap that flips the bridge over when the rod is shoved.

m.use("flip_bridge", x, y, z, span=4, levels=3, color="Light Bluish Gray", planks="Reddish Brown", rod="Red",
title="Bridge") stands, on the studs at height z, two pillars 2 x 4 (along x) `span` studs apart: the near one over
(x, y) to (x + 2, y + 4), `levels` bricks tall; the far one with its top course a pair of Technic bricks 1 x 2 with a
hole (3700) and two plates and a layer of tiles over it, so its top is smooth and level with the hinges. On the near
pillar two hinge bricks (3937) carry hinge tops 2 x 2 (6134), and on them the bridge: two bands of plates 2 wide out
over the far pillar, each starting with a 2 x 1, tied by tiles across them as the planks. The free end only rests on
the far pillar's tiles. A Technic axle 6 (3706) through the Technic bricks pokes under the free end and out the far
side as the push rod. Shut is the only pose modeled. span is 2 to 8. Returns the z of the bridge's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72040. Not derived from any
model file."""

PARTS = [
    "3001",
    "3010",
    "3937",
    "6134",
    "3700",
    "3004",
    "3020",
    "87079",
    "2431",
    "3706",
]

AXLE_BOTTOM = 1  # an axle through a Technic brick's hole starts this far above the brick's bottom, in plates
AXLE = 0.6  # an axle's thickness, in studs
POKE = (
    1.5  # how far the rod pokes past the far pillar's face, under the bridge, in studs
)


def build(
    m,
    x,
    y,
    z,
    span=4,
    levels=3,
    color="Light Bluish Gray",
    planks="Reddish Brown",
    rod="Red",
    title="Bridge",
):
    """The pillars, the rod, the hinges, and the bridge; returns the z of the bridge's top."""
    if not 2 <= span <= 8:
        raise ValueError(f"flip_bridge takes a span of 2 to 8, not {span}")
    if levels < 2:
        raise ValueError(f"flip_bridge needs 2 or more levels, not {levels}")
    fx = x + 2 + span  # the far pillar's left side
    top = z + 3 * levels
    for k in range(levels - 1):
        with m.step(f"{title}: pillars, level {k + 1}"):
            m.add("3001", color, at=(x, y, z + 3 * k))
            m.add("3001", color, at=(fx, y, z + 3 * k))
    with m.step(f"{title}: pillars, level {levels}"):
        m.add("3001", color, at=(x, y, top - 3))
        m.add("3700", color, at=(fx, y + 1, top - 3))
        m.add("3700", color, at=(fx + 1, y + 1, top - 3))
        m.add("3004", color, at=(fx, y, top - 3), rot=1)
        m.add("3004", color, at=(fx, y + 3, top - 3), rot=1)
    with m.step(f"{title}: push rod"):
        m.add(
            "3706", rod, at=(fx - POKE, y + 2 - AXLE / 2, top - 3 + AXLE_BOTTOM), rot=1
        )
    with m.step(f"{title}: far pillar top"):
        m.add("3020", color, at=(fx, y, top))
        m.add("3020", color, at=(fx, y, top + 1))
        m.add("87079", color, at=(fx, y, top + 2))
    with m.step(f"{title}: hinges"):
        m.add("3010", color, at=(x, y, top))
        m.add("3937", color, at=(x + 1, y, top), rot=2)
        m.add("3937", color, at=(x + 1, y + 2, top), rot=2)
        m.add("6134", color, at=(x + 1, y, top + 1), rot=2)
        m.add("6134", color, at=(x + 1, y + 2, top + 1), rot=2)
    length = fx + 2 - (x + 1)
    with m.step(f"{title}: bridge plates"):
        # A short plate first, so every joint between plates falls under the middle of a plank.
        for row in (y, y + 2):
            m.row(
                Plate, planks, at=(x + 1, row, top + 3), length=length, width=2, first=1
            )
    with m.step(f"{title}: planks"):
        for c in range(0, length - 1, 2):
            m.add("87079", planks, at=(x + 1 + c, y, top + 4))
        if length % 2:
            m.add("2431", planks, at=(x + length, y, top + 4))
    return top + 5


def example(m):
    """A flip bridge over a gap, on a strip of ground."""
    with m.step("Ground"):
        m.fill(Plate, "Green", at=(0, 0, 0), size=(10, 4))
    m.use("flip_bridge", x=0, y=0, z=1, span=4)
