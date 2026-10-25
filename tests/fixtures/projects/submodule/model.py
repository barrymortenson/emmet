# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model
from models.walls import wall


@model
def house():
    """A model that keeps its walls in models/walls.py, where line 9 has a bad color."""
    m = Model()
    wall(m, length=4)
    return m
