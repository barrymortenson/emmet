# SPDX-License-Identifier: Apache-2.0
"""Davit lifeboat: a small double-ended boat hanging outboard beside a ship's deck from two curved davits.

m.use("davit_lifeboat", x, y, z, color="White", cover="Tan", davit="White", title="Lifeboat") hangs a boat 6 x 2
studs along x from x, outboard of a deck whose front edge runs along x at y (the boat takes y - 2 to y) and whose top
is at z. The boat is a submodel, built once however many hang: a round-bilged hull of two inverted curved slopes 2 x 2
(32803) at its ends and two plates 2 x 2 between, a plate 2 x 6 over them as the gunwale, and a cover of tiles in
`cover`, two studs left bare at each davit. Each davit stands on the deck's edge stud at x + 1 and x + 4: two round
bricks 1 x 1 in `davit`, a plate 1 x 3 reaching out over the boat, its underside on the boat's bare studs, and a curved
slope 2 x 1 on it bending down outboard. Returns the z of the davits' top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10335. Not derived from any
model file."""

PARTS = ["32803", "3022", "3795", "3068b", "3069b", "3062b", "3623", "11477"]

KEEL = 3  # the boat's keel, in plates over the deck's top: its bare studs meet the davits' arms


def _boat(m, color, cover):
    """The boat right way up, its keel's corner at the origin, along x."""
    with m.step("Hull"):
        m.add("32803", color, at=(0, 0, 0), rot=2)
        m.add("3022", color, at=(2, 0, 0))
        m.add("3022", color, at=(2, 0, 1))
        m.add("32803", color, at=(4, 0, 0), rot=0)
    with m.step("Gunwale"):
        m.add("3795", color, at=(0, 0, 2), rot=1)
    with m.step("Cover"):
        m.add("3069b", cover, at=(0, 0, 3))
        m.add("3068b", cover, at=(2, 0, 3))
        m.add("3069b", cover, at=(5, 0, 3))


def build(m, x, y, z, color="White", cover="Tan", davit="White", title="Lifeboat"):
    """The davits' posts and arms, and the boat under them; returns the z of the davits' top."""
    boat = m.submodel("lifeboat", _boat, color=color, cover=cover)
    with m.step(f"{title}, davit posts"):
        for dx in (1, 4):
            m.stack("3062b", davit, at=(x + dx, y, z), count=2)
    with m.step(f"{title}, davit arms"):
        for dx in (1, 4):
            m.add("3623", davit, at=(x + dx, y - 2, z + 6))
            m.add("11477", davit, at=(x + dx, y - 2, z + 7), rot=1)
    with m.step(f"{title}, boat"):
        # Pressed up onto the arms' undersides.
        m.place(boat, at=(x, y - 2, z + KEEL))
    return z + 9


def example(m):
    """A lifeboat hung beside a strip of hull and deck."""
    from emmet_dsl import Brick, Plate

    with m.step("Hull side"):
        m.row(Brick, "Black", at=(0, 0, 0), length=8, width=2)
    with m.step("Deck"):
        m.row(Plate, "Tan", at=(0, 0, 3), length=8, width=2)
    m.use("davit_lifeboat", x=1, y=0, z=4)
