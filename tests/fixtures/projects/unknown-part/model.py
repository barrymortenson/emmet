# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def wrong_part():
    """A model that asks for a part id that doesn't exist, on line 10."""
    m = Model()
    m.add("3001", "Red", at=(0, 0, 0))
    m.add("3001x", "Red", at=(0, 0, 3))
    return m
