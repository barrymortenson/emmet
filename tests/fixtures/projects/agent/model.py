# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model

# A spike project.
WIDTH = 8


@model
def row():
    """A row of WIDTH 1 x 1 bricks: the agent replay tests change WIDTH and count the parts."""
    m = Model()
    with m.step("Row"):
        for i in range(WIDTH):
            m.add("3005", "Red", at=(i, 0, 0), id=f"b{i}")
    return m
