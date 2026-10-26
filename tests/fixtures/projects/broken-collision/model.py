# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def collision():
    """Two bricks on a plate, the second one stud into the first (validate: one collision)."""
    m = Model()
    m.add("3031", "Green", at=(0, 0, 0), id="base")
    m.add("3001", "Red", at=(0, 0, 1), id="wall/a")
    m.add("3001", "Blue", at=(1, 0, 1), id="wall/b")
    return m
