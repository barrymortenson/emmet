# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Brick, Model, model


@model
def turn_wall():
    """A tall wall along the right edge of a plate, which faces the instructions' camera, then bricks behind it:
    out of sight from there, so the instructions turn the model around to show them."""
    m = Model()
    with m.step("Plate"):
        m.add("3958", "Green", at=(0, 0, 0), id="plate")
    with m.step("Wall"):
        m.stack(Brick(1, 6), "Red", at=(5, 0, 1), count=6, id="wall")
    with m.step("Behind the wall"):
        m.add(Brick(2, 2), "Yellow", at=(1, 1, 1), id="behind/1")
        m.add(Brick(2, 2), "Yellow", at=(1, 4, 1), id="behind/2")
    return m
