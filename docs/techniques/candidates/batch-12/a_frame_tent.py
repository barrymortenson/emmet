# SPDX-License-Identifier: Apache-2.0
"""A-frame tent: a camping tent of two canvas panels leaning together on clip hinges over low side walls, open at the
ends, with a sleeping bag inside.

m.use("a_frame_tent", x, y, z, length=6, canvas="Tan", floor="Dark Tan", bag="Red", pillow="White", title="Tent")
builds, on the studs at height z over (x, y) to (x + length, y + 6), a floor of plates, a wall one brick tall along
the front and back rows, a sleeping bag (a 2 x 4 plate, a 2 x 2 tile, and a curved slope 2 x 2 as the pillow) on the
floor between them, and over it the two canvas panels, each a plate-and-tile panel on clips along a wall top, tilted
up until they meet at the ridge (the library's hinge_roof does the panels and works out the angle). The ends are left
open to see in. length is even, 4 to 10. Returns the z of the ridge's top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72040. Not derived from any
model file."""

PARTS = ["3001", "3010", "3020", "3068b", "15068", "4081b", "2540"]

DEPTH = 6  # the tent's depth, in studs: the narrowest the hinged panels meet over


def build(
    m,
    x,
    y,
    z,
    length=6,
    canvas="Tan",
    floor="Dark Tan",
    bag="Red",
    pillow="White",
    title="Tent",
):
    """The floor, the walls, the bag, and the canvas; returns the z of the ridge's top."""
    if length % 2 or not 4 <= length <= 10:
        raise ValueError(
            f"a_frame_tent takes an even length from 4 to 10, not {length}"
        )
    with m.step(f"{title}: floor"):
        m.fill(Plate, floor, at=(x, y, z), size=(length, DEPTH))
    with m.step(f"{title}: sleeping bag"):
        # Lying along the tent in its middle two rows, clear of the clips, its pillow at the right end.
        m.add("3020", bag, at=(x + length // 2 - 2, y + 2, z + 1), rot=1)
        m.add("3068b", bag, at=(x + length // 2 - 2, y + 2, z + 2))
        m.add("15068", pillow, at=(x + length // 2, y + 2, z + 2), rot=2)
    with m.step(f"{title}: walls"):
        m.row(Brick, canvas, at=(x, y, z + 1), length=length)
        m.row(Brick, canvas, at=(x, y + DEPTH - 1, z + 1), length=length)
    return m.use(
        "hinge_roof",
        x=x,
        y=y,
        z=z + 4,
        length=length,
        depth=DEPTH,
        color=canvas,
        title=f"{title}: canvas",
    )


def example(m):
    """A tent on a patch of grass."""
    with m.step("Grass"):
        m.fill(Plate, "Green", at=(0, 0, 0), size=(8, 8))
    m.use("a_frame_tent", x=1, y=1, z=1)
