# SPDX-License-Identifier: AGPL-3.0-or-later
from typing import Annotated

from emmet_dsl import Brick, Model, Range, model


def tower(m, rows=3):
    """A tower: a 2 x 2 base and a column of bricks with a cap."""
    with m.step("Base"):
        m.add(Brick(2, 2), "Light Bluish Gray", at=(0, 0, 0), id="base")
    with m.step("Column"):
        m.stack(Brick(1, 2), "Tan", at=(0, 0, 3), count=rows, id="wall")
    with m.step("Cap"):
        m.add("3022", "Dark Red", at=(0, 0, 3 + 3 * rows), id="cap")


@model
def towers(rows: Annotated[int, Range(1, 6)] = 3):
    """One tower built once and placed four times, at the corners of a plate.
    rows: how many bricks tall each tower's column is
    """
    m = Model()
    with m.step("Plate"):
        m.add("3958", "Green", at=(0, 0, 0), id="plate")
    unit = m.submodel("Tower", tower, rows=rows)
    with m.step("Towers"):
        for x, y in ((0, 0), (4, 0), (0, 4), (4, 4)):
            m.place(unit, at=(x, y, 1))
    return m
