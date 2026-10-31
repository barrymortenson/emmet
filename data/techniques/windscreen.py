# SPDX-License-Identifier: Apache-2.0
"""Windscreen: a sloped windscreen on a vehicle's cab, with a roof of tiles over it.

m.use("windscreen", x, y, z, color="Trans-Clear", roof="Red", title="Windscreen") puts a windscreen 2 x 4 x 2 (3823)
at (x, y, z), 2 deep along x and 4 wide along y, its slope to the front (-x), and a roof on it: a 4 x 4 plate from x
back (over the windscreen's top and two studs of cab behind it) with tiles over it. Build the cab's sides and back
up to z + 6 behind it first. A figure sitting in the cab needs the roof's underside 9 plates over its seat
(docs/spec/building-rules.md, rule 3), so a seat's top goes at z - 3 or lower: stand the windscreen on a dashboard to
raise it, as the example does. Returns the z of the roof's top, with .underside (z + 6).
"""

from emmet_dsl import Tile
from emmet_dsl.core import fit

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3823", "3031", "3068b", "87079"]


def build(m, x, y, z, color="Trans-Clear", roof="Red", title="Windscreen"):
    """The windscreen and its roof; returns the z of the roof's top, with .underside."""
    with m.step(f"{title}: glass"):
        m.add("3823", color, at=(x, y, z), rot=0)
    with m.step(f"{title}: roof"):
        m.add("3031", roof, at=(x, y, z + 6))
        m.fill(Tile, roof, at=(x, y, z + 7), size=(4, 4))
    return fit(z + 8, underside=z + 6)


def example(m):
    """A small cab a figure sits in: a seat facing the front, a dashboard of a brick and a plate under the windscreen,
    and sides three bricks and a plate tall, so the roof is 9 plates over the seat."""
    with m.step("Floor and seat"):
        m.add("3031", "Light Bluish Gray", at=(0, 0, 0))
        m.add("4079", "Black", at=(2, 1, 1), rot=2)
    # The seat's top is at z = 2: a seated figure needs 3 bricks clear over its front row.
    m.keep_clear(((2, 1, 2), (3, 3, 11)), "a figure sitting in the cab")
    with m.step("Dashboard"):
        m.add("3001", "Red", at=(0, 0, 1))
        m.add("3020", "Red", at=(0, 0, 4))
    for r in range(3):
        with m.step(f"Cab sides, row {r + 1}"):
            for side in (0, 3):
                m.add("3004", "Red", at=(2, side, 1 + 3 * r), rot=1)
    with m.step("Cab sides, top"):
        for side in (0, 3):
            m.add("3023b", "Red", at=(2, side, 10), rot=1)
    m.use("windscreen", x=0, y=0, z=5)
