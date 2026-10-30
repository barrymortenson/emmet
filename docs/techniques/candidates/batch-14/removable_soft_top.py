# SPDX-License-Identifier: Apache-2.0
"""Removable soft top: a roadster's rear deck with its folded soft top on it, built as one unit that sits on two
locating studs behind the cabin and lifts off.

m.use("removable_soft_top", x, y, z, width=8, color="Dark Green", top="Tan", title="Soft top") puts two round plates 1 x
1 (6141) on the chassis's studs at z (tile the rest), a stud in from the front corners of a deck 6 studs long along x
from x and `width` studs along y from y, and sets on their studs the deck, a submodel: a plate 6 x `width`; the folded
top across its front 4 studs, plates in `top` with curved slopes 2 x 1 (11477) rolling down to the front and to the
back, a soft round bundle; and behind it a row of curved slopes in the body color rolling down to the tail. The deck is
held by the two studs only, so it lifts off. width: 6 or 8. Returns the z of the bundle's top (z + 5).
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11381. Not derived from any
model file."""

PARTS = ["6141", "3958", "3036", "3795", "3034", "11477"]


def _deck(m, width, color, top):
    """The deck in its own frame: its front-left corner at the origin, its front to -x."""
    with m.step("Soft top: deck"):
        m.add(Plate(6, width), color, at=(0, 0, 0))
    with m.step("Soft top: tail"):
        for k in range(width):
            m.add("11477", color, at=(4, k, 1), rot=0)  # rolls down to the tail
    with m.step("Soft top: folded top, plates"):
        for a in (0, 2):
            m.add(Plate(2, width), top, at=(a, 0, 1))
    for a, turn, side in ((0, 2, "front"), (2, 0, "back")):
        with m.step(f"Soft top: folded top, {side}"):
            for k in range(width):
                m.add("11477", top, at=(a, k, 2), rot=turn)


def build(m, x, y, z, width=8, color="Dark Green", top="Tan", title="Soft top"):
    """The locating studs, then the deck set on them; returns the z of the bundle's top."""
    if width not in (6, 8):
        raise ValueError(f"removable_soft_top takes a width of 6 or 8, not {width}")
    with m.step(f"{title}: locating studs"):
        for k in (1, width - 2):
            m.add("6141", "Black", at=(x + 1, y + k, z))
    deck = m.submodel("soft_top_deck", _deck, width=width, color=color, top=top)
    with m.step(title):
        m.place(deck, at=(x, y, z + 1))
    return z + 5


def example(m):
    """The deck behind a roadster's cabin, on its chassis plate."""
    with m.step("Chassis"):
        m.add("3036", "Black", at=(0, 0, 0))
    m.use("removable_soft_top", x=0, y=0, z=1)
