# SPDX-License-Identifier: Apache-2.0
"""Six-wide truck cab: a city truck's cab for one figure, a full-width windscreen on a dash, side windows, and a roof
with a light bar that lifts off.

m.use("six_wide_truck_cab", x, y, z, color="White", glass="Trans-Clear", roof="White", lamps=("Trans-Orange",),
title="Cab") builds a cab 8 studs long and 6 wide on the floor whose top is at (x, y, z), its front to -x: a 2 x 6
brick across the front as the dash, a windscreen 2 x 6 x 2 (4176) on it, a seat with its steering wheel in front
(cab_seat), side walls a brick of 1 x 4s then a window 1 x 2 x 2 and 1 x 2 bricks, a back wall three bricks tall, and
on the walls a 6 x 8 roof plate tiled smooth with a light bar on its front edge, which lifts off to put the driver
in. The whole cab is 3 bricks tall, about 4 over the chassis with the roof. Returns the z of the roof's top.
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 60509. Not derived from any
model file."""

PARTS = ["2456", "4176", "3010", "3005", "60592c01", "3004", "3009", "3036"]


def build(
    m,
    x,
    y,
    z,
    color="White",
    glass="Trans-Clear",
    roof="White",
    lamps=("Trans-Orange",),
    title="Cab",
):
    """The dash, the seat, the walls, the windscreen, and the roof; returns the z of the roof's top."""
    with m.step(f"{title}: dash and sides"):
        m.add("2456", color, at=(x, y, z), rot=0)
        for side in (y, y + 5):
            m.add("3010", color, at=(x + 2, side, z), rot=1)
            m.add("3005", color, at=(x + 6, side, z))
        m.add("3009", color, at=(x + 7, y, z))
    m.use("cab_seat", x=x + 3, y=y + 2, z=z, title=f"{title}: driver")
    with m.step(f"{title}: windscreen"):
        m.add("4176", glass, at=(x, y, z + 3), rot=0)
    with m.step(f"{title}: side windows"):
        for side in (y, y + 5):
            m.add("60592c01", color, at=(x + 2, side, z + 3), rot=1)
    for r in range(2):
        with m.step(f"{title}: walls, row {r + 2}"):
            for side in (y, y + 5):
                # The joints of each row a stud from the last's.
                m.add("3004", color, at=(x + 4 + r, side, z + 3 + 3 * r), rot=1)
                m.add("3005", color, at=(x + 6 - 2 * r, side, z + 3 + 3 * r))
            m.add("3009", color, at=(x + 7, y, z + 3 + 3 * r))
    with m.step(f"{title}: roof"):
        m.add("3036", roof, at=(x, y, z + 9), rot=1)
        m.fill(Tile, roof, at=(x + 1, y, z + 10), size=(7, 6))
    m.use("light_bar", x=x, y=y + 1, z=z + 10, lamps=lamps, title=f"{title}: light bar")
    return z + 10


def example(m):
    """A cab on the front of a 6-wide truck's floor."""
    with m.step("Floor"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("six_wide_truck_cab", x=0, y=0, z=1)
