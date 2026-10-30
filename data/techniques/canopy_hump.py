# SPDX-License-Identifier: Apache-2.0
"""Canopy hump: a rounded, bulging cockpit hump on a car's top, layers of round-cornered plates and bricks each smaller
than the one under it, a slit window across its front, and curved slopes rolling it off front and back.

m.use("canopy_hump", x, y, z, color="Black", window="Trans Clear", title="Canopy") sets a hump 6 x 6 studs on the studs
at z, its front-left corner at (x, y), its front to -x: four plates 3 x 3 with a round corner (30357) make a 6 x 6 base
with round corners; four bricks 2 x 2 with a round corner (85080) a 4 x 4 drum on it; a layer of plates whose front row
is two plates 1 x 2 in `window` (3023b), the slit window; and four curved slopes 2 x 2 (15068) rolling down to the front
and back. It's a submodel on studs, so it lifts off to show the seat. Returns the z of its top (z + 7).
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book 6670152. Not derived from any model
file."""

PARTS = ["30357", "85080", "3023b", "3710", "3020", "15068"]

# The corners of a square, (dx, dy) in its own halves, and the quarter turn that puts each part's round corner there.
CORNERS = ((0, 0), (1, 0), (1, 1), (0, 1))
PLATE_TURN = {
    (0, 0): 1,
    (1, 0): 2,
    (1, 1): 3,
    (0, 1): 0,
}  # plate 3 x 3 with a round corner
BRICK_TURN = {
    (0, 0): 2,
    (1, 0): 3,
    (1, 1): 0,
    (0, 1): 1,
}  # brick 2 x 2 with a round corner


def _hump(m, color, window):
    """The hump in its own frame, its front-left corner at the origin."""
    with m.step("Canopy: base"):
        for c in CORNERS:
            m.add("30357", color, at=(3 * c[0], 3 * c[1], 0), rot=PLATE_TURN[c])
    with m.step("Canopy: drum"):
        for c in CORNERS:
            m.add("85080", color, at=(1 + 2 * c[0], 1 + 2 * c[1], 1), rot=BRICK_TURN[c])
    with m.step("Canopy: window"):
        for dy in (1, 3):
            m.add("3023b", window, at=(1, dy, 4), rot=0)
        m.add("3710", color, at=(2, 1, 4), rot=0)
        m.add("3020", color, at=(3, 1, 4), rot=0)
    with m.step("Canopy: top"):
        for dy in (1, 3):
            m.add("15068", color, at=(1, dy, 5), rot=2)  # rolls down to the front
            m.add("15068", color, at=(3, dy, 5), rot=0)  # and to the back


def build(m, x, y, z, color="Black", window="Trans Clear", title="Canopy"):
    """The hump, built apart and set on; returns the z of its top."""
    hump = m.submodel("canopy_hump", _hump, color=color, window=window)
    with m.step(title):
        m.place(hump, at=(x, y, z))
    return z + 7


def example(m):
    """A hump on a car's rear deck."""
    with m.step("Deck"):
        m.add("41539", "Black", at=(0, 0, 0))
    m.use(
        "canopy_hump",
        x=1,
        y=1,
        z=1,
        color="Dark Bluish Gray",
        window="Trans Light Blue",
    )
