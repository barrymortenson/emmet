# SPDX-License-Identifier: Apache-2.0
"""Dock pier: a timber pier on round piles standing in the water, with cross beams, stringers, and planks laid across.

m.use("dock_pier", x, y, z, length, width=4, height=2, color="Reddish Brown", deck="Reddish Brown", title="Pier") builds
a pier `width` studs wide (2 to 8, even) whose front-left corner is (x, y), running `length` studs back (+y) from the
shore, on piles that stand on the studs at z (the water's bed or surface). Piles of round bricks 1 x 1 (3062b), `height`
tall, stand in pairs at the two sides every 3 studs, a pair at each end; a plate 1 x width lies across each pair as a
cross beam; a run of 1-wide plates along each side on the beams is a stringer; and tiles 1 x width laid across the
stringers, one per stud, are the planks. Returns the z of the planks' top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3062b",
    "3023b",
    "3710",
    "3666",
    "3460",
    "3069b",
    "2431",
    "6636",
    "4162",
    "3024",
    "3623",
    "4477",
    "60479",
]

BEAMS = {2: "3023b", 4: "3710", 6: "3666", 8: "3460"}  # Plate 1 x n
PLANKS = {2: "3069b", 4: "2431", 6: "6636", 8: "4162"}  # Tile 1 x n


def build(
    m,
    x,
    y,
    z,
    length,
    width=4,
    height=2,
    color="Reddish Brown",
    deck="Reddish Brown",
    title="Pier",
):
    """The piles, the beams, the stringers, and the planks; returns the z of the planks' top."""
    if width not in BEAMS or length < 4 or not 1 <= height <= 8:
        raise ValueError(
            f"dock_pier takes an even width from 2 to 8, a length of 4 or more and a height from 1 to 8, not "
            f"{width}, {length}, {height}"
        )
    bays = sorted({*range(0, length - 1, 3), length - 1})
    # A pair of piles a step, or one pile a step when they're tall.
    groups = (
        [(v, (0, width - 1)) for v in bays]
        if height <= 4
        else [(v, (u,)) for v in bays for u in (0, width - 1)]
    )
    for v, sides in groups:
        with m.step(f"{title}: piles at {v}"):
            for u in sides:
                m.stack("3062b", color, at=(x + u, y + v, z), count=height)
    zb = z + 3 * height
    with m.step(f"{title}: cross beams"):
        for v in bays:
            m.add(BEAMS[width], deck, at=(x, y + v, zb), rot=1)
    with m.step(f"{title}: stringers"):
        for u in (0, width - 1):
            m.row(Plate, deck, at=(x + u, y, zb + 1), length=length, axis="y")
    for k in range(0, length, 8):
        with m.step(f"{title}: planks" + (f", {k // 8 + 1}" if length > 8 else "")):
            for v in range(k, min(k + 8, length)):
                m.add(PLANKS[width], deck, at=(x, y + v, zb + 2), rot=1)
    return zb + 3


def example(m):
    """A short pier on piles standing on the water."""
    with m.step("Water"):
        m.add("3031", "Dark Blue", at=(0, 0, 0))  # Plate 4 x 4
        m.add("3031", "Dark Blue", at=(0, 4, 0))
    m.use("dock_pier", x=0, y=0, z=1, length=7)
