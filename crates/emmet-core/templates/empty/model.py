# SPDX-License-Identifier: Apache-2.0
from emmet_dsl import Model, model


@model
def build():
    """An empty green baseplate, ready for a model."""
    m = Model()
    with m.step("Baseplate"):
        m.add("3811", "Green", at=(0, 0, 0), id="base")
    return m
