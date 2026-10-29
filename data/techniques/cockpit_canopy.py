# SPDX-License-Identifier: Apache-2.0
"""Cockpit canopy: a glass canopy hinged at its top to the wall behind the cockpit, by a hinge tile standing on the
wall's side studs, so it swings up to let the pilot in.

m.use("cockpit_canopy", x, y, z, angle=0, color="Dark Bluish Gray", glass="Trans-Light Blue", hinge_color="Dark Bluish
Gray", title="Canopy") covers a cockpit 4 studs wide from x and 4 deep from y, on a deck whose top is at z, facing
front (-y). Behind it, on the row y + 4, it builds a wall: two courses of bricks 1 x 4 and a brick 1 x 4 with studs on
the side (30414), its studs facing the cockpit; a hinge tile 1 x 4 (4625) stands on those studs, its fingers along its
top edge. The canopy, a windscreen 4 x 4 x 3 with hinge (2620), hangs from those fingers: shut (angle 0) its rim rests
on the deck from y - 0.4, or swung up by `angle` degrees, up to 90. It brings its own steps and returns the z of the
wall's top (z + 9), where the hull behind the cockpit carries on.
"""

from emmet_dsl import Rot, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3010", "30414", "4625", "2620"]

# LDU, LDraw's frame; its hinge runs along X through its origin.
CANOPY_BODY = [-40, -4, -76, 40, 68, 4]
# The hinge tile standing on its long edge, its face to -y, its fingers along the top.
TILE = Rot(y=90, z=-90)


def _canopy_at(axis, angle):
    """The footprint corner that puts the canopy's hinge on `axis` (DSL), facing front and swung up by `angle`."""
    pos, _, _ = frame.place(CANOPY_BODY, (0, 0, 0), frame.rotation(0, -angle, 270))
    landed = (-pos[2] / frame.STUD, pos[0] / frame.STUD, -pos[1] / frame.PLATE)
    return tuple(axis[i] - landed[i] for i in range(3))


def build(
    m,
    x,
    y,
    z,
    angle=0,
    color="Dark Bluish Gray",
    glass="Trans-Light Blue",
    hinge_color="Dark Bluish Gray",
    title="Canopy",
):
    """The wall, the hinge tile, and the canopy; returns the z of the wall's top."""
    if not 0 <= angle <= 90:
        raise ValueError(f"cockpit_canopy opens from 0 to 90 degrees, not {angle}")
    wall = y + 4
    with m.step(f"{title}: wall"):
        m.add("3010", color, at=(x, wall, z), rot=1)
        m.add("3010", color, at=(x, wall, z + 3), rot=1)
        m.add("30414", color, at=(x, wall, z + 6), rot=3)
    # The tile and the canopy go on together: the canopy's fingers close over the tile's from above.
    with m.step(f"{title}: hinge and canopy"):
        # On the side studs, its face 0.8 studs in front of the wall (fingers included) and centered on them.
        m.add("4625", hinge_color, at=(x, wall - 0.8, z + 6.5), rot=TILE)
        # The fingers' axis: 0.6 studs in front of the wall, 8.5 plates above the deck, which is where the rim of
        # the shut canopy rests.
        m.add(
            "2620",
            glass,
            at=_canopy_at((x + 2, wall - 0.6, z + 8.5), angle),
            rot=Rot(y=-angle, z=270),
        )
    return z + 9


def example(m):
    """A cockpit on a small deck, its canopy part way open."""
    with m.step("Deck"):
        m.add("3958", "Dark Bluish Gray", at=(0, 0, 0))
    top = m.use("cockpit_canopy", x=1, y=0, z=1, angle=35)
    with m.step("Hull behind"):
        m.add("3020", "Light Bluish Gray", at=(1, 4, top), rot=1)
