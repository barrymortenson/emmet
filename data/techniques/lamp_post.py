# SPDX-License-Identifier: Apache-2.0
"""Lamp post: a street or platform lamp, 1 stud square and 7 bricks tall, built apart as a sub-assembly and placed.

m.use("lamp_post", at=(x, y, z), color="Black", light="Trans-Yellow", height=5, id=None) places one lamp post on a
stud at `at`: a round brick for its foot, a post `height` bricks tall (5, one Brick 1 x 1 x 5, or any height built
from 1 x 1 round bricks), and a cone for the lamp. Every lamp with the same colors and height is one sub-assembly,
built once and placed "× n", so place them inside your own step (m.repeat spaces them out). It returns the placed
copy's Handle.
"""

PARTS = ["3062b", "2453b", "4589"]


def _lamp(m, color, light, height):
    with m.step("Post"):
        m.add("3062b", color, at=(0, 0, 0), id="foot")
        if height == 5:
            m.add("2453b", color, at=(0, 0, 3), id="post")
        else:
            m.stack("3062b", color, at=(0, 0, 3), count=height, id="post")
        m.add("4589", light, at=(0, 0, 3 + 3 * height), id="lamp")


def build(m, at, color="Black", light="Trans-Yellow", height=5, id=None):
    lamp = m.submodel("Lamp post", _lamp, color=color, light=light, height=height)
    return m.place(lamp, at=at, id=id)


def example(m):
    """Three lamp posts along a plate."""
    with m.step("Plate"):
        m.add("3034", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    with m.step("Lamps"):
        m.repeat(lambda: m.use("lamp_post", at=(0, 0, 1)), 3, dx=3)
