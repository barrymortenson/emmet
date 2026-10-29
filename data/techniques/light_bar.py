# SPDX-License-Identifier: Apache-2.0
"""Light bar: a row of lamps along a vehicle's roof edge, round plates on a bracket whose face hangs over the edge.

m.use("light_bar", x, y, z, color="Black", lamps=("Trans Dark Blue", "Trans Red"), title="Light bar") puts a bracket
1 x 2 - 1 x 4 (2436b) on a roof whose front edge is at x and whose top is z, its 1 x 2 plate on the roof's front row
at y + 1 and its 1 x 4 face hanging over the edge, four studs to the front (-x), and a round 1 x 1 plate on each, the
colors taken in turn from `lamps`. The lamps cover y to y + 4. Returns nothing.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["2436b", "6141"]

LAMP = Rot(y=-90)  # a round plate on its back, its face to -x


def build(
    m, x, y, z, color="Black", lamps=("Trans Dark Blue", "Trans Red"), title="Light bar"
):
    """The bracket and its lamps."""
    with m.step(title):
        # The bracket's face hangs 1.5 plates below its plate, in front of the roof's edge.
        m.add("2436b", color, at=(x - 0.2, y, z - 1.5), rot=2)
        for k in range(4):
            m.add("6141", lamps[k % len(lamps)], at=(x - 0.6, y + k, z - 1.5), rot=LAMP)


def example(m):
    """A light bar on a roof plate."""
    with m.step("Roof"):
        m.add("3031", "White", at=(0, 0, 0))
    m.use("light_bar", x=0, y=0, z=1)
