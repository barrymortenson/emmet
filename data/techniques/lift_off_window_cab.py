# SPDX-License-Identifier: Apache-2.0
"""Lift-off window cab: a shunting loco's cab with glass on all four sides, built as one box with its roof that lifts
off the cab floor to put the driver in.

m.use("lift_off_window_cab", x, y, z, color="Black", roof="Yellow", lamp="Trans Neon Orange", title="Window cab") builds
a box 4 studs long (along x) and 6 wide and places it on the studs at (x, y, z): a frame of plates around its edge;
a train window 1 x 4 x 3 with its glass (4033c01) in the front and the back, between corner posts of 1 x 1 bricks;
a train window 1 x 2 x 3 (4035c01) in each side; and a 4 x 6 roof plate, tiled, with a round beacon on top. The box is
a submodel, so the steps build it apart and then set it on; it only sits on the studs, so it lifts off whole. Put
the seat and controls on the floor first. Returns the placed copy's Handle (its .max[2] is the top of the beacon).
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 60509. Not derived from any
model file."""

PARTS = ["3666", "3023b", "3005", "4033c01", "4035c01", "3032", "6141", "3062b"]


def _box(m, color, roof, lamp, title):
    """The cab box in its own frame, its corner at the origin."""
    with m.step(f"{title}: frame"):
        for fx in (0, 3):
            m.add("3666", color, at=(fx, 0, 0), rot=0)
        for side in (0, 5):
            m.add("3023b", color, at=(1, side, 0), rot=1)
    for fx, end in ((0, "front"), (3, "back")):
        with m.step(f"{title}: {end} window"):
            m.add("4033c01", color, at=(fx, 1, 1), rot=0)
            for side in (0, 5):
                m.stack("3005", color, at=(fx, side, 1), count=3)
    with m.step(f"{title}: side windows"):
        for side in (0, 5):
            m.add("4035c01", color, at=(1, side, 1), rot=1)
    with m.step(f"{title}: roof"):
        m.add("3032", roof, at=(0, 0, 10), rot=0)
        m.fill(Tile, roof, at=(0, 0, 11), size=(4, 2))
        m.fill(Tile, roof, at=(0, 4, 11), size=(4, 2))
    with m.step(f"{title}: beacon"):
        m.add("3022", roof, at=(1, 2, 11))
        m.add("6141", "Light Bluish Gray", at=(1, 2, 12))
        m.add("3062b", lamp, at=(1, 2, 13))


def build(
    m,
    x,
    y,
    z,
    color="Black",
    roof="Yellow",
    lamp="Trans Neon Orange",
    title="Window cab",
):
    """The box as a submodel, set on the floor; returns the placed copy's Handle."""
    box = m.submodel("window_cab", _box, color=color, roof=roof, lamp=lamp, title=title)
    with m.step(title):
        return m.place(box, at=(x, y, z))


def example(m):
    """A shunter's cab floor with the seat on it, and the window cab set over it."""
    with m.step("Floor"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    with m.step("Seat"):
        m.add("4079", "Black", at=(2, 2, 1), rot=3)
    m.use("lift_off_window_cab", x=1, y=0, z=1)
