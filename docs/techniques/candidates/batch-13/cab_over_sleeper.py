# SPDX-License-Identifier: Apache-2.0
"""Cab over sleeper: a semi-truck's tall sleeper behind its cab, smooth sides built as flat panels stood on side
studs, and a rounded roof fairing that lifts off.

m.use("cab_over_sleeper", x, y, z, color="Yellow", trim="Black", fairing="Yellow", title="Sleeper") builds a sleeper
4 studs long and 6 wide on the floor whose top is at (x, y, z), behind a cab whose back is at x (its front to -x),
4 studs (10 plates) tall: each side wall is two bricks 1 x 4 with studs on the side (30414), facing out, with two
plates between them so their studs are 2 studs apart, and on those studs a 4 x 4 plate on its side tiled smooth (the
panel); the back wall is bricks and plates the same heights. On top a 4 x 6 plate carries three curved slopes 4 x 2
(93606) that fall to the front, the fairing that rounds the sleeper over the cab roof; the plate and slopes lift off
as one. Returns the z of the fairing's top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 60440. Not derived from any
model file."""

PARTS = ["30414", "3710", "3031", "87079", "3032", "93606"]

# A plate on its side, its top (and its tiles) facing out: to -y on the left wall, to +y on the right.
LEFT = Rot(y=-90, z=90)
RIGHT = Rot(y=-90, z=-90)


def build(m, x, y, z, color="Yellow", trim="Black", fairing="Yellow", title="Sleeper"):
    """The walls, the panels, and the fairing; returns the z of the fairing's top."""
    for r, h in enumerate((0, 5)):
        with m.step(f"{title}: walls, row {r + 1}"):
            m.add("30414", trim, at=(x, y, z + h), rot=3)
            m.add("30414", trim, at=(x, y + 5, z + h), rot=1)
            m.add("3010", trim, at=(x + 3, y + 1, z + h), rot=0)
        with m.step(f"{title}: walls, plates {r + 1}"):
            for k in range(2):
                for side in (y, y + 5):
                    m.add("3710", trim, at=(x, side, z + h + 3 + k), rot=1)
                m.add("3710", trim, at=(x + 3, y + 1, z + h + 3 + k), rot=0)
    with m.step(f"{title}: side panels"):
        m.add("3031", color, at=(x, y - 0.4, z), rot=LEFT)
        m.add("3031", color, at=(x, y + 6, z), rot=RIGHT)
        for k in range(2):
            m.add("87079", color, at=(x, y - 0.8, z + 5 * k), rot=LEFT)
            m.add("87079", color, at=(x, y + 6.4, z + 5 * k), rot=RIGHT)
    with m.step(f"{title}: fairing"):
        m.add("3032", fairing, at=(x, y, z + 10), rot=0)
        for k in range(3):
            m.add("93606", fairing, at=(x, y + 2 * k, z + 11), rot=2)
    return z + 14


def example(m):
    """A sleeper behind a short cab on a 6-wide floor."""
    with m.step("Floor"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    with m.step("Cab"):
        m.add("2456", "Yellow", at=(0, 0, 1), rot=0)
        m.add("4176", "Trans-Clear", at=(0, 0, 4), rot=0)
    for r in range(3):
        with m.step(f"Cab, sides {r + 1}"):
            m.add("3004", "Yellow", at=(2, 0, 1 + 3 * r), rot=1)
            m.add("3004", "Yellow", at=(2, 5, 1 + 3 * r), rot=1)
    with m.step("Cab, roof"):
        m.add("3032", "Yellow", at=(0, 0, 10), rot=0)
    m.use("cab_over_sleeper", x=4, y=0, z=1)
