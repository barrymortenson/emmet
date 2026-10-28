# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def step_order():
    """A shelter on a plate (validate: two step_order findings). The lamp goes on in step 2, but the post it stands on
    only in step 3; and the crate goes in under the roof after the roof is on, with no way in from above."""
    m = Model()
    with m.step("Base"):
        m.add("3958", "Green", at=(0, 0, 0), id="base")
    with m.step("Lamp"):
        m.add("3024", "Yellow", at=(5, 5, 4), id="lamp")
    with m.step("Lamp post"):
        m.add("3005", "Black", at=(5, 5, 1), id="post")
    with m.step("Corner posts"):
        for i, (x, y) in enumerate([(0, 0), (3, 0), (0, 3), (3, 3)]):
            m.add("3005", "Tan", at=(x, y, 1), id=f"corner/{i + 1}")
    with m.step("Roof"):
        m.add("3031", "Dark Red", at=(0, 0, 4), id="roof")
    with m.step("Crate"):
        m.add("3005", "Reddish Brown", at=(1, 1, 1), id="crate")
    return m
