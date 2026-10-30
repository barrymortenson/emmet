# SPDX-License-Identifier: Apache-2.0
"""Muzzle and nose: an animal's face, a muzzle standing out from the head with its top corners rounded, a round dark
nose on its tip, and a tongue poking out under it: a dog, a cat, a bear.

m.use("muzzle_and_nose", x, y, z, width=4, color="Tan", nose="Black", tongue="Dark Pink", title="Muzzle") builds a
muzzle `width` studs across (4 or 6) and 2 deep from (x, y), its front at y, on the studs at height z (the head's chin
plate reaching under it): a layer of plates with, in its middle, a plate 2 x 2 in the tongue color that reaches a stud
past the front, two tiles 1 x 1 with a rounded end on that stud row (the tongue); a course of bricks; on top, a cylinder
2 x 2 with a dome in the nose color at the front middle and curved slopes 2 x 1 falling forward beside it. tongue=None
leaves the tongue out (plain plates). Returns the z of the muzzle's top, under the nose (the nose stands 3 plates above
it).
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 43290 and 21368. Not derived
from any model file."""

PARTS = [
    "3022",
    "24246",
    "3023b",
    "3001",
    "2456",
    "30367a",
    "11477",
    "3710",
    "3666",
    "3024",
]


def build(
    m, x, y, z, width=4, color="Tan", nose="Black", tongue="Dark Pink", title="Muzzle"
):
    """The plates and tongue, the bricks, then the nose and the rounded corners; returns the z of the top."""
    if width not in (4, 6):
        raise ValueError(f"muzzle_and_nose takes a width of 4 or 6, not {width}")
    mid = x + width // 2 - 1  # the nose's and tongue's left column
    with m.step(f"{title}: plates"):
        if tongue:
            m.add("3022", tongue, at=(mid, y - 1, z))
            m.row(Plate, color, at=(x, y, z), length=mid - x, axis="x", width=2)
            m.row(
                Plate,
                color,
                at=(mid + 2, y, z),
                length=x + width - mid - 2,
                axis="x",
                width=2,
            )
            m.add(
                "3023b", color, at=(mid, y + 1, z), rot=1
            )  # Plate 1 x 2 behind the tongue
        else:
            m.row(Plate, color, at=(x, y, z), length=width, axis="x", width=2)
    if tongue:
        with m.step(f"{title}: tongue"):
            for i in range(2):
                m.add(
                    "24246", tongue, at=(mid + i, y - 1, z + 1), rot=3
                )  # rounded end to the front
    with m.step(f"{title}: bricks"):
        m.row(Brick, color, at=(x, y, z + 1), length=width, axis="x", width=2)
    top = z + 4
    with m.step(f"{title}: nose"):
        m.add("30367a", nose, at=(mid, y, top))
        for cx in (x, x + width - 1):
            m.add("11477", color, at=(cx, y, top), rot=3)  # falling forward
        if width == 6:
            m.add("11477", color, at=(x + 1, y, top), rot=3)
            m.add("11477", color, at=(x + 4, y, top), rot=3)
    return top


def example(m):
    """A dog's head: a block 6 studs wide with the muzzle on its chin plate."""
    from emmet_dsl import Tile

    with m.step("Chin plate"):
        m.add("3032", "Tan", at=(0, -2, 0), rot=1)  # Plate 4 x 6, along x
        m.add("3795", "Tan", at=(0, 2, 0), rot=1)  # Plate 2 x 6
    m.use("muzzle_and_nose", x=1, y=-2, z=1, width=4)
    for c in range(3):
        with m.step(f"Head, course {c + 1}"):
            m.row(
                Brick,
                "Tan",
                at=(0, 0, 1 + 3 * c),
                length=6,
                axis="x",
                width=2,
                first=2 if c % 2 else None,
            )
            m.row(
                Brick,
                "Tan",
                at=(0, 2, 1 + 3 * c),
                length=6,
                axis="x",
                width=2,
                first=None if c % 2 else 2,
            )
    with m.step("Head, top"):
        m.fill(Plate, "Tan", at=(0, 0, 10), size=(6, 4))
        m.fill(Tile, "Tan", at=(0, 0, 11), size=(6, 4))
