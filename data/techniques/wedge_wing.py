# SPDX-License-Identifier: Apache-2.0
"""Wedge wing: a pair of swept wings from left- and right-handed wedge plates, in two layers whose joints cross, so
the wings and the deck between them are one stiff plate.

m.use("wedge_wing", x, y, z, span=4, gap=4, color="Light Bluish Gray", title="Wings") builds, from z, a deck `gap`
studs wide and 4 deep whose front-left corner is (x, y), with a wing `span` studs long each side of it (span 3 or 4,
from wedge plates 3 x 2 or 4 x 2): the right wing reaches to x + gap + span and the left one to x - span, their
leading edges swept back. The bottom layer is a plate 2 studs wide along the back the whole width, and in front of it
a wedge plate for each wing (a right-handed one on the right and a left-handed one, turned round, on the left; a left
wing is not a right one turned over) with plates between. The top layer puts a second pair of wedge plates a stud
further back, so each one bridges the joint under it, a strip of plates 1 wide along the back the whole width, and
plates over the deck. The leading edges step back a stud at the top. It brings its own steps and returns the z of the
top (z + 2).
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = [
    "41769a",
    "41770a",
    "43722a",
    "43723a",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "91988",
    "4282",
    "3024",
    "3004",
]

# Wedge plates by span: (right-handed, left-handed). At rot 0 both run along x from their root, the right-handed one
# with its cut on the front; the left-handed one turned round (rot 2) is the mirror image, its root on the right.
WEDGES = {3: ("43722a", "43723a"), 4: ("41769a", "41770a")}


def build(m, x, y, z, span=4, gap=4, color="Light Bluish Gray", title="Wings"):
    """The deck and the two wings; returns the z of the top."""
    if span not in WEDGES:
        raise ValueError(f"wedge_wing takes a span of 3 or 4, not {span}")
    if gap < 2 or gap % 2 or gap + 2 * span > 16:
        raise ValueError(
            f"wedge_wing needs an even gap of 2 or more, with gap + 2 × span at most 16, not gap={gap}"
        )
    right, left = WEDGES[span]
    x0, width = x - span, gap + 2 * span
    with m.step(f"{title}: bottom layer"):
        m.row(Plate, color, at=(x0, y + 2, z), length=width, width=2)
        m.add(right, color, at=(x + gap, y, z))
        m.add(left, color, at=(x0, y, z), rot=2)
        m.row(Plate, color, at=(x, y, z), length=gap, width=2)
    with m.step(f"{title}: top layer"):
        m.add(right, color, at=(x + gap, y + 1, z + 1))
        m.add(left, color, at=(x0, y + 1, z + 1), rot=2)
        m.row(Plate, color, at=(x, y, z + 1), length=gap)
        m.row(Plate, color, at=(x, y + 1, z + 1), length=gap, width=2)
        m.row(Plate, color, at=(x0, y + 3, z + 1), length=width)
    return z + 2


def example(m):
    """Wings on a short hull."""
    with m.step("Stand"):
        m.add("3004", "Dark Bluish Gray", at=(1, 1, 0), rot=1)
    top = m.use("wedge_wing", x=0, y=0, z=3, span=4, gap=4)
    with m.step("Hull"):
        m.add("3020", "White", at=(0, 0, top), rot=1)
        m.add("3020", "White", at=(0, 2, top), rot=1)
