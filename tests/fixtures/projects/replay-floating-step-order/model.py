# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def shelter():
    """A plate with a tower and a lamp (validate: the tower's top brick floats, and the lamp's step comes before its
    post's). The agent replay in tests/it fixes both from validate's summary."""
    m = Model()
    with m.step("Base"):
        m.add("3958", "Green", at=(0, 0, 0), id="base")
    with m.step("Tower"):
        m.add("3001", "Red", at=(0, 0, 1), id="tower/1")
        m.add("3001", "Red", at=(0, 0, 5), id="tower/2")
    with m.step("Lamp"):
        m.add("3024", "Yellow", at=(5, 5, 4), id="lamp")
    with m.step("Post"):
        m.add("3005", "Black", at=(5, 5, 1), id="post")
    return m
