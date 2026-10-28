# SPDX-License-Identifier: Apache-2.0
"""Jumper offset: jumper plates that move a stud half a stud over, so a part stands centered on an even-width base.

m.use("jumper_offset", x, y, z, length=2, depth=2, color="Light Bluish Gray", title="Jumpers") centers one stud on
the length × depth area whose front-left corner is (x, y), on the studs at height z (the top of the base). An even
length takes a jumper plate (1 x 2 with one center stud) along x, an even depth one along y; both even take two side
by side along x and one across them along y, a plate higher. At least one of length and depth must be even (an odd
one already has a center stud). It brings its own step, titled "<title>", and returns the (x, y, z) to put a 1 x 1
part at so it stands on the centered stud: m.add("3062b", "White", at=m.use("jumper_offset", ...)).
"""

PARTS = ["3794b", "15573"]


def build(m, x, y, z, length=2, depth=2, color="Light Bluish Gray", title="Jumpers"):
    """The jumpers; returns the (x, y, z) a 1 x 1 part goes at to stand on the centered stud."""
    if length < 1 or depth < 1 or (length % 2 and depth % 2):
        raise ValueError(
            f"jumper_offset needs an even length or depth, not {length} × {depth}: "
            "an odd side already has a stud in the middle"
        )
    cx, cy = x + length / 2, y + depth / 2  # the center of the area
    with m.step(title):
        if length % 2 == 0 and depth % 2 == 0:
            # Two jumpers along x put studs at the center's x, half a stud either side of its y; one along y on
            # them (without an understud, so it sits between studs) brings its stud to the center.
            for row in (cy - 1, cy):
                m.add("3794b", color, at=(cx - 1, row, z), rot=1)
            m.add("15573", color, at=(cx - 0.5, cy - 1, z + 1))
            return (cx - 0.5, cy - 0.5, z + 2)
        if length % 2 == 0:
            m.add("3794b", color, at=(cx - 1, cy - 0.5, z), rot=1)
        else:
            m.add("3794b", color, at=(cx - 0.5, cy - 1, z))
        return (cx - 0.5, cy - 0.5, z + 1)


def example(m):
    """A lamp post centered on a 2 × 2 base."""
    with m.step("Base"):
        m.add("3022", "Dark Bluish Gray", at=(0, 0, 0))
    at = m.use("jumper_offset", x=0, y=0, z=1, length=2, depth=2)
    with m.step("Lamp post"):
        m.stack("3062b", "Black", at=at, count=3)
        m.add("4589", "Yellow", at=(at[0], at[1], at[2] + 9))
