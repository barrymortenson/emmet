# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Brick, Model, model


@model
def figure_clearance():
    """A shop front with the mistakes figure_clearance finds (validate: four figure_clearance warnings): a canopy a
    brick too low beside a door, a bench against a column, a wall across a doorway, and a lamp in a path the model
    keeps clear."""
    m = Model()
    with m.step("Base"):
        m.add("3811", "Green", at=(0, 0, 0), id="base")
    m.use(
        "door_frame",
        x=2,
        y=10,
        z=0.5,
        length=8,
        rows=7,
        offset=1,
        facing="front",
        title="Shop",
    )
    m.use(
        "door_frame",
        x=12,
        y=10,
        z=0.5,
        length=8,
        rows=7,
        offset=2,
        facing="front",
        title="Back room",
    )
    with m.step("Bench"):
        m.use("bench", at=(3, 4, 0.5), id="bench")
    m.use("canopy", x=2, y=4, z=0.5, length=8, depth=6, spacing=7, height=5)
    with m.step("Wall across the back room's door"):
        m.stack(Brick(1, 4), "Tan", at=(14, 8, 0.5), count=6, rot=1, id="wall")
    m.keep_clear(((22, 2, 1.5), (24, 8, 18.5)), "the path to the garden gate")
    with m.step("Lamp"):
        m.use("lamp_post", at=(22, 4, 0.5), id="lamp")
    return m
