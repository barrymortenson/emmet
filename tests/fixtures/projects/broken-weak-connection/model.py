# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def weak_connection():
    """A 1 x 8 plate held by one stud at its end (validate: one weak_connection warning)."""
    m = Model()
    with m.step("Base"):
        m.add("3958", "Green", at=(0, 0, 0), id="base")
    with m.step("Post"):
        m.add("3005", "Black", at=(0, 0, 1), id="post")
    with m.step("Arm"):
        m.add("3460", "Red", at=(0, 0, 4), rot=1, id="arm")
    return m
