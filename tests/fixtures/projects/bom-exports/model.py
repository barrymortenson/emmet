# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def exports():
    """A parts list for the BOM export tests: lines Rebrickable and BrickLink know, a brick in a color it was never
    made in, and a window rebrickable-mini doesn't know (unknown, and left out of both exports)."""
    m = Model()
    m.add("3958", "Neon Yellow", at=(0, 0, 0), id="base")
    m.add("3001", "Red", at=(0, 0, 1), id="red/1")
    m.add("3001", "Red", at=(2, 0, 1), id="red/2")
    m.add("3001", "Violet", at=(4, 0, 1), id="violet")
    m.add("3023", "Medium Tan", at=(0, 4, 1), id="moved")
    m.add("3853", "White", at=(0, 5, 1), id="window")
    return m
