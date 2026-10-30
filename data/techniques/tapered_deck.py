# SPDX-License-Identifier: Apache-2.0
"""Tapered deck: a wedge-shaped car deck that narrows toward the nose in plan, two layers of plates with wing plates at
the nose whose slanted edges sit a stud apart, so the outline steps in at a slant without visible steps.

m.use("tapered_deck", x, y, z, length=12, width=6, color="Light Bluish Gray", title="Deck") lays, at z, a deck `length`
studs along x from x, its nose at x (to -x), `width` studs along y from y: a first layer of plates with a wing 2 x 4
left (41770a) and right (41769a) at the nose's two sides, narrow end forward, and plates between and behind; then a
second layer the same, a stud further back, its plates across the first layer's seam; then tiles over the second layer
except the wings, whose studs are left for what goes on the nose. The outline narrows by a stud each side over the first
4 studs. width: 6 to 12, even. Returns the z of the tiles' top (z + 3).
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10300, and the instruction
book 6670152. Not derived from any model file."""

PARTS = [
    "41770a",
    "41769a",
    "3020",
    "3021",
    "3795",
    "3034",
    "3032",
    "3035",
    "3068b",
    "87079",
    "69729",
]


def _layer(m, x, y, z, length, width, color, title):
    """A layer: a wing each side at the nose (narrow end to -x), plates between them, then plates behind."""
    with m.step(f"{title}: nose"):
        m.add("41770a", color, at=(x, y, z), rot=2)  # its slant on the -y side
        # Its slant on the +y side.
        m.add("41769a", color, at=(x, y + width - 2, z), rot=2)
        m.fill(Plate, color, at=(x, y + 2, z), size=(4, width - 4))
    with m.step(f"{title}: behind"):
        m.fill(Plate, color, at=(x + 4, y, z), size=(length - 4, width))


def build(m, x, y, z, length=12, width=6, color="Light Bluish Gray", title="Deck"):
    """The two layers and the tiles; returns the z of the top."""
    if width % 2 or not 6 <= width <= 12:
        raise ValueError(f"tapered_deck takes an even width from 6 to 12, not {width}")
    if length < 8:
        raise ValueError(f"tapered_deck takes a length of 8 or more, not {length}")
    _layer(m, x, y, z, length, width, color, f"{title}, first layer")
    _layer(m, x + 1, y, z + 1, length - 1, width, color, f"{title}, second layer")
    with m.step(f"{title}: tiles, nose"):
        m.fill(Tile, color, at=(x + 1, y + 2, z + 2), size=(4, width - 4))
    with m.step(f"{title}: tiles"):
        m.fill(Tile, color, at=(x + 5, y, z + 2), size=(length - 5, width))
    return z + 3


def example(m):
    """A deck on a short body, its nose standing out 4 studs past it."""
    with m.step("Body"):
        m.add("3036", "Dark Bluish Gray", at=(4, 0, 0), rot=1)
        m.add("3036", "Dark Bluish Gray", at=(4, 0, 1), rot=1)
    m.use("tapered_deck", x=0, y=0, z=2)
