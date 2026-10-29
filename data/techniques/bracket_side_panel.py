# SPDX-License-Identifier: Apache-2.0
"""Bracket side panel: a vehicle's smooth side, a panel of plate and tiles built lying flat and pressed onto brackets
whose studs face out.

m.use("bracket_side_panel", x, y, z, length=4, color="Light Bluish Gray", panel="White", title="Side panel") puts
brackets 1 x 2 - 2 x 2 up (99207) along the left side (-y) of a chassis at z, `length` studs along x from x (2 each),
their studs out, and on them a 2-tall panel: a 2 x length plate on its side, tiled. Returns nothing.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["99207", "3022", "3020", "3795", "3068b", "87079"]

PLATES = {2: "3022", 4: "3020", 6: "3795"}  # Plate 2 x n
SIDE = Rot(y=-90, z=90)  # on its back, its face to -y, long along x


def build(
    m, x, y, z, length=4, color="Light Bluish Gray", panel="White", title="Side panel"
):
    """The brackets, then the panel on them."""
    if length not in PLATES:
        raise ValueError(
            f"bracket_side_panel takes a length of 2, 4, or 6, not {length}"
        )
    with m.step(f"{title}: brackets"):
        for a in range(0, length, 2):
            m.add("99207", color, at=(x + a, y, z), rot=1)
    with m.step(f"{title}: panel"):
        m.add(PLATES[length], panel, at=(x, y - 0.4, z), rot=SIDE)
        for a in range(0, length, 2):
            m.add("3068b", panel, at=(x + a, y - 0.8, z), rot=SIDE)


def example(m):
    """A panel on the side of a 4-wide chassis."""
    with m.step("Chassis"):
        m.add("3031", "Red", at=(0, 0, 0))
    m.use("bracket_side_panel", x=0, y=0, z=1)
