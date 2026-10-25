# SPDX-License-Identifier: AGPL-3.0-or-later
from typing import Annotated

from emmet_dsl import Model, Range, model


@model
def typo(width: Annotated[int, Range(2, 8)] = 4):
    """A model that misspells its own parameter on line 12."""
    m = Model()
    for x in range(0, width, 2):
        m.add("3001", "Red", at=(x, 0, widht))
    return m
