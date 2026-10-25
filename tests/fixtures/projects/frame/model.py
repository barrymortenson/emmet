# SPDX-License-Identifier: AGPL-3.0-or-later
# Pins the DSL frame (plan section 08) against expected.ldr, which was worked out by hand.
from emmet_dsl import Model, model


@model
def frame():
    """Six parts that pin the frame: rotations 0 to 3, a plate, a slope, and half-stud offsets."""
    m = Model()
    m.add("3001", "Red", at=(0, 0, 0), id="brick")
    m.add("3001", "Red", at=(0, 0, 3), rot=1, id="brick-turned")
    m.add("3024", "White", at=(2, 1, 0), id="plate")
    m.add("3039", 4, at=(4, 0, 0), rot=3, id="slope")
    m.add("3004", "Blue", at=(0, 4, 0), rot=2, id="half-turn")
    m.add("3024", "Light Bluish Gray", at=(6.5, 0.5, 1), id="offset")
    return m
