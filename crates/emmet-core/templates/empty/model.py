# SPDX-License-Identifier: Apache-2.0
from emmet_dsl import Model, model


@model
def build(baseplate: bool = True):
    """An empty green baseplate, ready for a model.
    baseplate: start on a green 32 x 32 baseplate (off: on the table, at z = 0)
    """
    m = Model()
    if baseplate:
        with m.step("Baseplate"):
            m.add("3811", "Green", at=(0, 0, 0), id="base")
    return m
