# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def wrong_color():
    """A model that misspells a color on line 10."""
    m = Model()
    m.add("3001", "Red", at=(0, 0, 0))
    m.add("3001", "Light Blush Gray", at=(0, 0, 3))
    return m
