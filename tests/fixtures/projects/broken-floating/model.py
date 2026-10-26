# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def floating():
    """A brick on a plate, and one a plate above it (validate: one floating group of two)."""
    m = Model()
    m.add("3031", "Green", at=(0, 0, 0), id="base")
    m.add("3001", "Red", at=(0, 0, 1), id="tower/1")
    m.add("3001", "Red", at=(0, 0, 5), id="tower/2")
    m.add("3020", "Red", at=(0, 0, 8), id="tower/cap")
    return m
