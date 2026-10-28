# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def step_size():
    """A wall of ten bricks in one step (validate: one step_size warning; the 6+ target is 8 a step)."""
    m = Model()
    with m.step("Base"):
        m.add("3958", "Green", at=(0, 0, 0), id="base")
    with m.step("Wall"):
        for i in range(5):
            m.add("3004", "Red", at=(0, i, 1), rot=1, id=f"wall/{i + 1}")
            m.add("3004", "Red", at=(0, i, 4), rot=1, id=f"wall/{i + 6}")
    return m
