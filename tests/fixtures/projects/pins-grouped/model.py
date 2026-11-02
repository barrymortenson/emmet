# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def pins_grouped():
    """Two loose bricks far apart and a step of ten (validate: two floating findings and one step_size warning)."""
    m = Model()
    with m.step("Base"):
        m.add("3958", "Green", at=(0, 0, 0), id="base")
    with m.step("Wall"):
        for i in range(5):
            m.add("3004", "Red", at=(0, i, 1), rot=1, id=f"wall/{i + 1}")
            m.add("3004", "Red", at=(0, i, 4), rot=1, id=f"wall/{i + 6}")
    with m.step("Loose"):
        m.add("3001", "Blue", at=(12, 0, 4), id="loose/a")
        m.add("3001", "Blue", at=(-12, 0, 4), id="loose/b")
    return m
