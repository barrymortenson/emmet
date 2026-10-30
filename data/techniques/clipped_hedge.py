# SPDX-License-Identifier: Apache-2.0
"""Clipped hedge: a low hedge along a wall or path that reads as a row of rounded, clipped bushes: round 2 x 2 tiles
standing on the side studs of bricks, every other bush a plate higher.

m.use("clipped_hedge", x, y, z, length=8, facing="front", colors=("Green", "Bright Green"), title="Hedge") builds, on
the studs at height z (a baseplate is z = 0.5), a hedge one stud deep and `length` studs long (even) from (x, y), along
x facing front (-y) or back (+y), or along y facing left (-x) or right (+x). It is a row of plates under a bush every 2
studs: a Brick 1 x 2 x 1 2/3 with Studs on 1 Side (22885) with its studs to the face, carrying a Tile 2 x 2 Round
(14769) standing on them, so the face is a row of round bushes; every other bush stands on a 1 x 2 plate, a plate higher
than its neighbors, and a 1 x 2 plate caps each, its studs a leafy top. The bushes take `colors` in turn. length: 2 to
32, even. Returns the z of the taller bushes' top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10352. Not derived from any
model file."""

PARTS = ["22885", "14769", "3023b", "3710", "3666", "3460", "3008b"]

# By facing: the side-stud brick's turn, the round tile's turn (standing up, its face out), where the tile goes from
# the bush's corner, and the way the row runs.
FACING = {
    "front": (3, Rot(x=90), (0, -0.4), (1, 0)),
    "back": (1, Rot(x=90, z=180), (0, 1), (1, 0)),
    "left": (2, Rot(x=90, z=270), (-0.4, 0), (0, 1)),
    "right": (0, Rot(x=90, z=90), (1, 0), (0, 1)),
}


def build(
    m,
    x,
    y,
    z,
    length=8,
    facing="front",
    colors=("Green", "Bright Green"),
    title="Hedge",
):
    """The plates, the bushes, and their caps; returns the z of the taller bushes' top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if not 2 <= length <= 32 or length % 2:
        raise ValueError(f"clipped_hedge takes an even length of 2 to 32, not {length}")
    brick_rot, tile_rot, (ox, oy), (ax, ay) = FACING[facing]
    with m.step(f"{title}, base"):
        m.row(Plate, colors[0], at=(x, y, z), length=length, axis="x" if ax else "y")
    z += 1
    bushes = range(length // 2)
    for s in range(0, len(bushes), 2):
        with m.step(f"{title}, bushes {s + 1} to {min(s + 2, len(bushes))}"):
            for k in bushes[s : s + 2]:
                bx, by, raised = x + 2 * k * ax, y + 2 * k * ay, k % 2
                color = colors[k % len(colors)]
                if raised:
                    m.add("3023b", color, at=(bx, by, z), rot=1 if ax else 0)
                zb = z + raised
                m.add("22885", color, at=(bx, by, zb), rot=brick_rot)
                m.add("14769", color, at=(bx + ox, by + oy, zb), rot=tile_rot)
                m.add("3023b", color, at=(bx, by, zb + 5), rot=1 if ax else 0)
    return z + 1 + 6


def example(m):
    """A hedge 10 studs long facing front, on a baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("clipped_hedge", x=3, y=6, z=0.5, length=10)
