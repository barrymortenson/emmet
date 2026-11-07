# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Brick, Model, model


def chair(m):
    """A chair: a seat on a leg, with a back."""
    with m.step("Seat"):
        m.add(Brick(1, 2), "Reddish Brown", at=(0, 0, 0), id="leg")
        m.add("3022", "Reddish Brown", at=(0, 0, 3), id="seat")
    with m.step("Back"):
        m.add(Brick(1, 2), "Reddish Brown", at=(0, 1, 4), id="back")


def table_set(m):
    """A table with two chairs, each chair built apart."""
    with m.step("Table"):
        m.add(Brick(2, 2), "Tan", at=(2, 0, 0), id="table")
    seat = m.submodel("Chair", chair)
    with m.step("Chairs"):
        m.place(seat, at=(0, 0, 0))
        m.place(seat, at=(5, 0, 0))


@model
def picnic():
    """A table set built apart, with its chairs built apart inside it."""
    m = Model()
    with m.step("Plate"):
        m.add("3958", "Green", at=(0, 0, 0), id="plate")
    unit = m.submodel("Table set", table_set)
    with m.step("Table set"):
        m.place(unit, at=(0, 0, 1))
    return m
