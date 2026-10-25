# SPDX-License-Identifier: AGPL-3.0-or-later
from typing import Annotated

from emmet_dsl import Color, Model, Range, model


@model
def sized(
    width: Annotated[int, Range(2, 16)] = 4,
    color: Color = "Tan",
):
    """A model whose params.json sets width out of range.
    width: how many studs wide
    """
    m = Model()
    m.add("3001", color, at=(0, 0, 0))
    return m
