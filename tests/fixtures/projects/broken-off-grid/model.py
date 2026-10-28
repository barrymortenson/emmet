# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def off_grid():
    """A 2 x 2 plate a fifth of a stud off the studs it should sit on (validate: off_grid, with how far to move it,
    and floating, since nothing holds it)."""
    m = Model()
    with m.step("Base"):
        m.add("3958", "Green", at=(0, 0, 0), id="base")
    with m.step("Block"):
        m.add("3003", "Red", at=(2, 2, 1), id="block")
    with m.step("Cap"):
        m.add("3022", "Blue", at=(2.2, 2, 4), id="cap")
    return m
