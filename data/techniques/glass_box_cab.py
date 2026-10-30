# SPDX-License-Identifier: Apache-2.0
"""Glass box cab: the all-glass operator's cab of a crane, reach stacker, or digger, clear panels on every side of a
small plate, a seat inside, and a tiled roof, built apart and set on the machine.

m.use("glass_box_cab", x, y, z, color="Yellow", glass="Trans-Clear", roof="White", title="Operator cab") builds a cab
6 studs long (along x, its front to -x) and 4 wide as a submodel and places it with its plate's corner at (x, y, z):
a 4 x 6 plate; a seat 2 x 2 facing the front; clear panels 1 x 4 x 3 (60581) across the front and the back and
along both sides, so there is glass all around, three bricks tall; and a 4 x 6 roof plate with tiles. Set it on two
or more studs of the machine anywhere along its plate, so it can sit off to one side of a boom the way the books'
cabs do. Returns the placed copy's Handle (its .max[2] is the roof's top).
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 60509. Not derived from any
model file."""

PARTS = ["3032", "4079", "60581", "87079"]


def _cab(m, color, glass, roof, title):
    """The cab in its own frame, its plate's corner at the origin."""
    with m.step(f"{title}: floor and seat"):
        m.add("3032", color, at=(0, 0, 0), rot=1)
        m.add("4079", "Black", at=(2, 1, 1), rot=0)
    with m.step(f"{title}: glass"):
        for gx in (0, 5):
            m.add("60581", glass, at=(gx, 0, 1), rot=0)
        for side in (0, 3):
            m.add("60581", glass, at=(1, side, 1), rot=1)
    with m.step(f"{title}: roof"):
        m.add("3032", roof, at=(0, 0, 10), rot=1)
        m.fill(Tile, roof, at=(0, 0, 11), size=(6, 4))


def build(
    m, x, y, z, color="Yellow", glass="Trans-Clear", roof="White", title="Operator cab"
):
    """The cab as a submodel, placed; returns the placed copy's Handle."""
    cab = m.submodel(
        "glass_box_cab", _cab, color=color, glass=glass, roof=roof, title=title
    )
    with m.step(title):
        return m.place(cab, at=(x, y, z))


def example(m):
    """A machine's deck with a boom foot on one side and the glass cab set beside it, hanging over the deck's edge."""
    with m.step("Deck"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    with m.step("Boom foot"):
        m.stack("3003", "Yellow", at=(5, 4, 1), count=2)
    m.use("glass_box_cab", x=1, y=-1, z=1)
