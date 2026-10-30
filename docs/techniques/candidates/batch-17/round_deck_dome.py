# SPDX-License-Identifier: Apache-2.0
"""Round deck dome: a low stepped dome or bridge on top of a disc or hull, round plates of shrinking size stacked on
one center.

m.use("round_deck_dome", x, y, z, size=6, tower=True, color="Light Bluish Gray", trim="Dark Bluish Gray",
lights="Trans-Light Blue", title="Dome") stacks, on the studs at z, round plates centered in the `size` x `size`
square from (x, y): with size=8, a Plate 8 x 8 Round (74611); then a Plate 6 x 6 Round (11213) and a Plate 4 x 4 Round
(60474), each in `color`, so the edges step in like a contour map; a ring of eight round 1 x 1 tiles in `lights` on
the 6 x 6's open studs; a round plate 2 x 2 in `trim`; with tower, a round brick 2 x 2 (3941) as the bridge; and a
dome (Dish 2 x 2 Inverted, 4740) on top. Put it where its square's studs are free. Returns the z of the dome's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10356. Not derived from any
model file."""

PARTS = ["74611", "11213", "60474", "98138", "4032b", "3941", "4740"]

# The 6 x 6 round plate's open studs round the 4 x 4: the middle two of each side.
RING = [(0, 2), (0, 3), (5, 2), (5, 3), (2, 0), (3, 0), (2, 5), (3, 5)]


def build(
    m,
    x,
    y,
    z,
    size=6,
    tower=True,
    color="Light Bluish Gray",
    trim="Dark Bluish Gray",
    lights="Trans-Light Blue",
    title="Dome",
):
    """The stepped plates, the lights, and the top; returns the z of the top."""
    if size not in (6, 8):
        raise ValueError(f"round_deck_dome takes a size of 6 or 8, not {size}")
    c = (size - 6) // 2  # the 6 x 6's corner, from (x, y)
    zl = z
    with m.step(f"{title}, plates"):
        if size == 8:
            m.add("74611", color, at=(x, y, zl))
            zl += 1
        m.add("11213", color, at=(x + c, y + c, zl))
        m.add("60474", color, at=(x + c + 1, y + c + 1, zl + 1))
    with m.step(f"{title}, lights"):
        for i, j in RING:
            m.add("98138", lights, at=(x + c + i, y + c + j, zl + 1))
    zl += 2
    with m.step(f"{title}, top"):
        m.add("4032b", trim, at=(x + c + 2, y + c + 2, zl))
        zl += 1
        if tower:
            m.add("3941", color, at=(x + c + 2, y + c + 2, zl))
            zl += 3
        m.add("4740", color, at=(x + c + 2, y + c + 2, zl))
    return zl + 1


def example(m):
    """A dome on a hull deck 8 x 8."""
    with m.step("Deck"):
        m.add("41539", "Light Bluish Gray", at=(0, 0, 0))
    m.use("round_deck_dome", x=0, y=0, z=1, size=8)
