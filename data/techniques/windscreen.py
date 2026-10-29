# SPDX-License-Identifier: Apache-2.0
"""Windscreen: a sloped windscreen on a vehicle's cab, with a roof of tiles over it.

m.use("windscreen", x, y, z, color="Trans-Clear", roof="Red", title="Windscreen") puts a windscreen 2 x 4 x 2 (3823)
at (x, y, z), 2 deep along x and 4 wide along y, its slope to the front (-x), and a roof on it: a 4 x 4 plate from x
back (over the windscreen's top and two studs of cab behind it) with tiles over it. Build the cab's sides and back
up to z + 6 behind it first. Returns the z of the roof's top.
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3823", "3031", "3068b", "87079"]


def build(m, x, y, z, color="Trans-Clear", roof="Red", title="Windscreen"):
    """The windscreen and its roof; returns the z of the roof's top."""
    with m.step(f"{title}: glass"):
        m.add("3823", color, at=(x, y, z), rot=0)
    with m.step(f"{title}: roof"):
        m.add("3031", roof, at=(x, y, z + 6))
        m.fill(Tile, roof, at=(x, y, z + 7), size=(4, 4))
    return z + 8


def example(m):
    """A small cab: a plate, its back and sides two bricks tall, and the windscreen and roof."""
    from emmet_dsl import Brick

    with m.step("Floor"):
        m.add("3031", "Light Bluish Gray", at=(0, 0, 0))
    for r in range(2):
        with m.step(f"Cab, row {r + 1}"):
            m.row(Brick, "Red", at=(3, 0, 1 + 3 * r), length=4, axis="y")
            m.add("3004", "Red", at=(2, 0, 1 + 3 * r))
            m.add("3004", "Red", at=(2, 3, 1 + 3 * r))
    m.use("windscreen", x=0, y=0, z=1)
