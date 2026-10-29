# SPDX-License-Identifier: Apache-2.0
"""Crow's nest: a round lookout platform on a mast's top, a ring of curved bricks for its rail, the topmast going on up
through it with a pennant.

m.use("crows_nest", x, y, z, top=2, color="Reddish Brown", flag="Red", title="Crow's nest") builds on the top (at z) of
a mast of 2 x 2 round bricks whose front-left corner is (x, y): a Plate 8 x 8 Round (74611) centered on the mast, its
middle four anti-studs on the mast's studs; a rail of four Brick 4 x 4 Corner Round (48092) around its edge, which
leaves a lookout 2 studs wide around the mast for a figure; then the topmast, `top` more 2 x 2 round bricks on the
plate's middle studs, a cone, a bar 4L in the cone's hollow stud, and a flag 2 x 2 (2335) clipped to the bar, streaming
aft (-x). Returns the z of the platform's top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["74611", "48092", "3941", "3942c", "30374", "2335"]

RING = [
    ((0, 0), 0),
    ((-4, 0), 1),
    ((-4, -4), 2),
    ((0, -4), 3),
]  # each quarter's corner from the mast's center, its turn


def build(m, x, y, z, top=2, color="Reddish Brown", flag="Red", title="Crow's nest"):
    """The platform, the rail, the topmast and the pennant; returns the z of the platform's top."""
    if top < 1:
        raise ValueError(f"crows_nest needs a topmast of 1 brick or more, not {top}")
    cx, cy = x + 1, y + 1  # the mast's center
    with m.step(f"{title}: platform"):
        m.add("74611", color, at=(cx - 4, cy - 4, z))
    with m.step(f"{title}: rail"):
        for (dx, dy), turn in RING:
            m.add("48092", color, at=(cx + dx, cy + dy, z + 1), rot=turn)
    with m.step(f"{title}: topmast"):
        m.stack("3941", color, at=(x, y, z + 1), count=top)
        zc = z + 1 + 3 * top
        m.add("3942c", color, at=(x, y, zc))
    with m.step(f"{title}: pennant"):
        # The bar goes half a plate into the cone's hollow stud; the flag clips to its top half.
        zb = zc + 6 - 0.5
        m.add("30374", "Black", at=(cx - 0.2, cy - 0.2, zb))
        m.add("2335", flag, at=(cx - 2.5, cy - 0.33, zb + 5))
    return z + 1


def example(m):
    """The top of a mast with its crow's nest."""
    with m.step("Mast"):
        m.add("3031", "Tan", at=(-1, -1, 0))  # Plate 4 x 4, the deck
        m.stack("3941", "Reddish Brown", at=(0, 0, 1), count=2)
    m.use("crows_nest", x=0, y=0, z=7)
